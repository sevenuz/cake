#!/usr/bin/env python3
"""Claude Code adapter for cake: a Stop hook.

Reads the hook JSON on stdin (session_id, transcript_path, cwd), then the
new lines of the session transcript, and tells cake through its CLI only:

- tokens of each answer go to the items of this session whose timer ran at
  that moment, as free fields (cake edit <id> --add tokens_in=...); items
  with a running timer collect them here until it stops
- the session's activity goes to the session log (cake log add): the
  user's chat stretches (kind chat) and the agent's turns (who claude-code,
  kind agent), split where the session's timers start and stop

Keys written (the contract other adapters follow too): tokens_in,
tokens_out, tokens_cache_read, tokens_cache_write; the items' session ids
are recorded by cake itself (config session_env).

State (offsets, pending tokens, where stretches ended) lives in
$XDG_STATE_HOME/cake-hooks/claude-code/. Settings:

  "hooks": {"Stop": [{"hooks": [{"type": "command",
    "command": "python3 ~/.claude/skills/cake/hooks/claude-code.py >/dev/null 2>&1 || true"}]}]}
"""

import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path

AGENT = "claude-code"
GAP = 15 * 60  # a pause longer than this ends a stretch of activity
KEYS = {
    "input_tokens": "tokens_in",
    "output_tokens": "tokens_out",
    "cache_read_input_tokens": "tokens_cache_read",
    "cache_creation_input_tokens": "tokens_cache_write",
}


def state_dir():
    base = os.environ.get("XDG_STATE_HOME") or os.path.expanduser("~/.local/state")
    d = Path(base) / "cake-hooks" / AGENT
    d.mkdir(parents=True, exist_ok=True)
    return d


def load(name, default):
    try:
        return json.loads((state_dir() / name).read_text())
    except (OSError, ValueError):
        return default


def save(name, value):
    p = state_dir() / name
    tmp = p.with_suffix(".tmp")
    tmp.write_text(json.dumps(value))
    tmp.replace(p)


def ts(text):
    """RFC 3339 (also with Z) -> unix seconds, None if unreadable"""
    try:
        return int(datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp())
    except (AttributeError, ValueError):
        return None


def cake(cwd, *args):
    out = subprocess.run(["cake", *args, "--json"], cwd=cwd, capture_output=True, text=True)
    try:
        return json.loads(out.stdout or "{}")
    except ValueError:
        return {}


def read_lines(path):
    """complete new lines since the last call: answers (time, usage),
    times of user and assistant lines, the agent's turns"""
    offsets = load("offsets.json", {})
    key = str(path)
    with open(path, "rb") as f:
        size = f.seek(0, 2)
        offset = offsets.get(key, 0)
        if offset > size:
            offset = 0  # rewritten transcript
        f.seek(offset)
        data = f.read()
    complete = data.rfind(b"\n") + 1
    answers, times, turns, seen, turn = [], [], [], set(), None
    for raw in data[:complete].splitlines():
        try:
            v = json.loads(raw)
        except ValueError:
            continue
        kind = v.get("type")
        t = ts(v.get("timestamp", ""))
        if kind in ("user", "assistant") and t is not None:
            times.append(t)
            content = (v.get("message") or {}).get("content")
            prompt = kind == "user" and (
                isinstance(content, str)
                or (isinstance(content, list) and not any(isinstance(c, dict) and c.get("type") == "tool_result" for c in content))
            )
            if prompt:
                if turn and turn[1] > turn[0]:
                    turns.append(turn)
                turn = [t, t]
            elif kind == "assistant" and turn:
                turn[1] = t
        m = v.get("message") or {}
        if kind != "assistant" or not m.get("usage") or t is None:
            continue
        mid = m.get("id")
        if mid in seen:
            continue  # one answer may span several lines
        if mid:
            seen.add(mid)
        answers.append((t, {k: int(m["usage"].get(k) or 0) for k in KEYS}))
    if turn and turn[1] > turn[0]:
        turns.append(turn)
    offsets[key] = offset + complete
    save("offsets.json", offsets)
    return answers, times, turns


def intervals(item):
    now = int(datetime.now().timestamp())
    out = []
    for t in item.get("time") or []:
        s, e = ts(t.get("start") or ""), ts(t.get("end") or "") if t.get("end") else now
        if s is not None and e is not None:
            out.append((s, e))
    return out


def stretches(times, after):
    out = []
    for t in sorted(times):
        if out and t - out[-1][1] <= GAP:
            out[-1][1] = max(out[-1][1], t)
        elif not out and after is not None and after <= t <= after + GAP:
            out.append([after, t])
        else:
            out.append([t, t])
    return [(s, e) for s, e in out if e > s]


def split(start, end, timers):
    """[start, end] cut where timers start and stop, each part with the id
    of the timer that runs (or None); neighbours with one id join"""
    cuts = sorted({start, end, *(c for _, s, e in timers for c in (s, e) if start < c < end)})
    out = []
    for a, b in zip(cuts, cuts[1:]):
        mid = a + (b - a) // 2
        iid = next((i for i, s, e in timers if s <= mid < e), None)
        if out and out[-1][2] == iid:
            out[-1][1] = b
        else:
            out.append([a, b, iid])
    return out


def share(usage, n, first):
    """one of n even parts, the first part takes the remainder"""
    return {k: v // n + (v % n if first else 0) for k, v in usage.items()}


def main():
    hook = json.load(sys.stdin)
    session, cwd = hook["session_id"], hook.get("cwd") or os.getcwd()
    answers, times, turns = read_lines(hook["transcript_path"])
    items = [i for i in cake(cwd, "ls", "-A", "-l").get("items", []) if session in (i.get("sessions") or [])]
    timers = [(i["id"], s, e) for i in items for s, e in intervals(i)]
    running = {i["id"] for i in items if i.get("running")}

    # tokens: per item, running ones wait here
    pending = load("pending.json", {})
    for t, usage in answers:
        ids = [iid for iid, s, e in timers if s <= t <= e]
        ids = list(dict.fromkeys(ids))
        for n, iid in enumerate(ids):
            part = share(usage, len(ids), n == 0)
            acc = pending.setdefault(iid, {})
            for k, v in part.items():
                acc[k] = acc.get(k, 0) + v
    for iid in [i for i in pending if i not in running]:
        adds = [f"{KEYS[k]}={v}" for k, v in pending[iid].items() if v]
        if adds:
            args = ["edit", iid]
            for a in adds:
                args += ["--add", a]
            if "error" in cake(cwd, *args) and any(i["id"] == iid for i in items):
                continue  # keep it for the next run
        del pending[iid]
    save("pending.json", pending)

    # the session log: the user's chat stretches and the agent's turns
    ends = load("ends.json", {})
    spans = stretches(times, ends.get(session))
    for s, e in spans:
        for a, b, iid in split(s, e, timers):
            args = ["log", "add", "--start", str(a), "--end", str(b), "--kind", "chat", "--agent", AGENT, "--session", session]
            cake(cwd, *args, *(["--item", iid] if iid else []))
    for s, e in turns:
        for a, b, iid in split(s, e, timers):
            args = ["log", "add", "--start", str(a), "--end", str(b), "--who", AGENT, "--kind", "agent", "--session", session]
            cake(cwd, *args, *(["--item", iid] if iid else []))
    if spans:
        ends[session] = spans[-1][1]
        save("ends.json", ends)


if __name__ == "__main__":
    main()
