#!/usr/bin/env python3
"""Plays a day of the cake form with real keys in a pseudo terminal and
checks the store afterwards. Usage: form_scenarios.py <cake binary>.
Run by `cargo test` (tests/form.rs); prints keys per decision."""

import json, os, pty, select, signal, struct, subprocess, sys, tempfile, time, fcntl, termios

CAKE = sys.argv[1]
TMP = tempfile.mkdtemp(prefix="cake-form-")
ENV = dict(os.environ, XDG_CONFIG_HOME=f"{TMP}/config", XDG_DATA_HOME=f"{TMP}/data",
           XDG_STATE_HOME=f"{TMP}/state", CAKE_AUTHOR="jul", CAKE_STYLE="unicode")
ENV.pop("AI_AGENT", None)
STORE = f"{TMP}/p"
os.makedirs(STORE)


def cake(*args, agent=False):
    env = dict(ENV)
    if agent:
        env.pop("CAKE_AUTHOR")
        env["AI_AGENT"] = "claude-code"
    out = subprocess.run([CAKE, *args, "--json"], cwd=STORE, env=env, capture_output=True, text=True)
    return json.loads(out.stdout) if out.stdout.strip() else {}


def item(id_):
    return cake("show", id_)["items"][0]


def form(keys, timeout=30):
    """runs `cake` and types `keys` (each a write, 0.4 s apart); returns the screen text"""
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(STORE)
        os.execve(CAKE, [CAKE], ENV)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    out = b""

    def read(t):
        nonlocal out
        end = time.time() + t
        while time.time() < end:
            r, _, _ = select.select([fd], [], [], 0.05)
            if r:
                try:
                    out += os.read(fd, 65536)
                except OSError:
                    return

    read(1.5)
    for k in keys:
        os.write(fd, k.encode())
        read(0.4)
    deadline = time.time() + timeout
    while time.time() < deadline:
        if os.waitpid(pid, os.WNOHANG)[0]:
            return out.decode(errors="replace")
        read(0.1)
    os.kill(pid, signal.SIGKILL)
    raise SystemExit("FAIL: the form hung after the last key")


failures = []


def check(what, ok):
    print(("ok   " if ok else "FAIL ") + what)
    if not ok:
        failures.append(what)


# a day: two ideas in the inbox, a question from the agent, finished work
# to check, a task of the user's
cake("init")
cake("add", "pass", "-m", "# renew passport")
cake("add", "blog", "-m", "# blog idea")
cake("add", "sync", "--task", "--assignee", "ai", "-m", "# sync errors")
cake("add", "hint", "--task", "-p", "sync", "--assignee", "claude-code", "-m", "# hint per error", agent=True)
cake("add", "raw", "--task", "-p", "hint", "--assignee", "jul", "-m", "# raw message too?\n\n- a) always\n- b) with -d", agent=True)
cake("block", "hint", "--by", "raw", agent=True)
cake("add", "feat", "--task", "-p", "sync", "--assignee", "claude-code", "-m", "# name the step\n\n- try: x", agent=True)
cake("review", "feat", agent=True)
cake("add", "land", "--task", "--assignee", "me", "-m", "# call landlord")

keys = [
    # sort: first row a task for ai, due friday; second row a note; save
    "t", "w", ":", "due fri\r", "j", "n", "\r",
    # answer: start typing, leave with jk (typed fast), then option 2
    "i", "may", "jk", "2", "\r",
    # review: c is done
    "c",
    # mine: hand the task to ai, save; triage (no alias) and the end screen
    "w", "\r", "l", "l", "q",
]
form(keys)
first, second = item("pass"), item("blog")
task, note = (first, second) if first["status"] == "task" else (second, first)
check("sort: a task for ai with a due date", task["status"] == "task" and task["assignee"] == "ai" and bool(task["due"]))
check("sort: the other one a note", note["status"] is None)
check("answer: jk left the text, option 2 saved", item("raw")["status"] == "done" and "answer: b) with -d" in item("raw")["body"])
check("review: c closed it", item("feat")["status"] == "done")
check("mine: handed to ai", item("land")["assignee"] == "ai")
decisions = len(keys) - 3  # the last three only walk to the end
print(f"keys: {decisions} for 6 decisions ({decisions / 6:.1f} per decision)")

# the next morning: what the agent did since, held back to the user
time.sleep(1.1)
cake("add", "new", "--task", "-p", "sync", "--assignee", "claude-code", "-m", "# a new subtask", agent=True)
form(["w", "\r"] + ["l"] * 7 + ["q"])
check("news: hold gives it back to the user", item("new")["assignee"] == "jul")

# h and l: leave sort without deciding, come back over the empty modules
cake("add", "zz", "-m", "# idea for later")
cake("add", "mytask", "--task", "--assignee", "me", "-m", "# my task")
form(["l", "h", "t", "\r"] + ["l"] * 6 + ["q"])
check("h/l: back in sort, the idea became a task", item("zz")["status"] == "task")

# F on a screen: feedback becomes a task for the AI, not an inbox item
cake("add", "yy", "-m", "# another idea")
form(["F", "too many columns\r", "q", "q"])
fb = [i for i in cake("ls", "-t", "form-feedback")["items"]]
check("F: feedback is a task for ai", len(fb) == 1 and fb[0]["status"] == "task" and fb[0]["assignee"] == "ai")

subprocess.run(["rm", "-rf", TMP])
sys.exit(1 if failures else 0)
