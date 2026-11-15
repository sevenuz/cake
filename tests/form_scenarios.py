#!/usr/bin/env python3
"""Plays a day of the cake form with real keys in a pseudo terminal and
checks the store afterwards. Usage: form_scenarios.py <cake binary>.
Run by `cargo test` (tests/form.rs); prints keys per decision."""

import json, os, pty, re, shlex, shutil, select, signal, struct, subprocess, sys, tempfile, time, fcntl, termios

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


def form(keys, timeout=30, args=()):
    """runs `cake` (with `args`) and types `keys` (each a write, 0.4 s apart); returns the screen text"""
    pid, fd = pty.fork()
    if pid == 0:
        os.chdir(STORE)
        os.execve(CAKE, [CAKE, *args], ENV)
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
cake("edit", "hint", "--blocked-by", "raw", agent=True)
cake("add", "feat", "--task", "-p", "sync", "--assignee", "claude-code", "-m", "# name the step\n\n- try: x", agent=True)
cake("review", "feat", agent=True)
cake("add", "land", "--task", "--assignee", "me", "-m", "# call landlord")

keys = [
    # inbox: first row a task (sort_who: ai), due friday; second row a
    # note (N); a module stays after its last item, l goes on
    "t", ":", "due fri\r", "j", "N", "l",
    # answer: start typing, leave with jk (typed fast), then option 2
    "i", "may", "jk", "2", "l",
    # review: c is done
    "c", "l",
    # mine: hand the task to ai; the end screen stays on l
    "w", "l", ":q\r",
]
form(keys)
first, second = item("pass"), item("blog")
task, note = (first, second) if first["status"] == "task" else (second, first)
check("inbox: a task for ai with a due date", task["status"] == "task" and task["assignee"] == "ai" and bool(task["due"]))
check("inbox: the other one a note", note["status"] is None)
check("answer: jk left the text, option 2 saved", item("raw")["status"] == "done" and "answer: b) with -d" in item("raw")["body"])
check("review: c closed it", item("feat")["status"] == "done")
check("mine: handed to ai", item("land")["assignee"] == "ai")
decisions = len(keys) - 5  # l to the next module (4) and :q
print(f"keys: {decisions} for 6 decisions ({decisions / 6:.1f} per decision)")

# the next day: what the agent did since, held back to the user
time.sleep(1.1)
cake("add", "new", "--task", "-p", "sync", "--assignee", "claude-code", "-m", "# a new subtask", agent=True)
form(["s"] + ["l"] * 7 + [":q\r"])
check("news: s stops the agent, back to the user", item("new")["assignee"] == "jul")

# h and l: leave sort without deciding, come back over the empty modules
cake("add", "zz", "-m", "# idea for later")
cake("add", "mytask", "--task", "--assignee", "me", "-m", "# my task")
form(["l", "h", "t", "\r"] + ["l"] * 6 + [":q\r"])
check("h/l: back in inbox, the idea became a task", item("zz")["status"] == "task")

# F on a screen: feedback becomes a task for the AI, not an inbox item
cake("add", "yy", "-m", "# another idea")
form(["F", "too many columns\r", ":q\r"])
fb = [i for i in cake("ls", "-t", "form-feedback")["items"]]
check("F: feedback is a task for ai", len(fb) == 1 and fb[0]["status"] == "task" and fb[0]["assignee"] == "ai")

# sticky: two items sorted, h and l: both still in sort with a check
cake("set", "dropped", "yy")
cake("add", "s1", "-m", "# sticky one")
cake("add", "s2", "-m", "# sticky two")


def tmux_screen(keys, args=()):
    """runs `cake` in a detached tmux session, types `keys`, returns the
    screen as text (the real screen: ratatui sends only changed cells)"""
    if not shutil.which("tmux"):
        return None
    name = f"cake-test-{os.getpid()}"
    env = " ".join(f"{k}={shlex.quote(v)}" for k, v in ENV.items() if k.startswith(("XDG_", "CAKE_")))
    subprocess.run(["tmux", "new-session", "-d", "-s", name, "-x", "100", "-y", "30", "-c", STORE, f"env {env} {CAKE} {' '.join(args)}"], check=True)
    time.sleep(1.5)
    for k in keys:
        subprocess.run(["tmux", "send-keys", "-t", name, "-l", k])
        time.sleep(0.5)
    text = subprocess.run(["tmux", "capture-pane", "-p", "-t", name], capture_output=True, text=True).stdout
    subprocess.run(["tmux", "kill-session", "-t", name], capture_output=True)
    return text


back = tmux_screen(["t", "h", "l"])
check("sticky: saved at once", item("s1")["status"] == "task" and item("s1")["assignee"] == "ai")
if back is None:
    print("skip sticky: both still shown (no tmux)")
else:
    check("sticky: still shown after h/l", "✓ sticky one" in back and "sticky two" in back)

# a stray q does not close the form, it says how
stray = tmux_screen(["q"])
if stray is not None:
    check("q: form stays open with the :q hint", "type :q to close" in stray)

# config sort_who "-": a task from sort gets nobody
os.makedirs(f"{TMP}/config/cake", exist_ok=True)
with open(f"{TMP}/config/cake/config.hjson", "w") as f:
    f.write('{ sort_who: "-" }\n')
cake("set", "dropped", "s2")
cake("add", "s3", "-m", "# nobody's task")
form(["t", ":q\r"])
check("sort_who -: task without assignee", item("s3")["status"] == "task" and item("s3")["assignee"] is None)

# answer on the tree: n to the second question, <cr> card, j option 2,
# <cr> saves; p back to the first, 1 picks from the tree; k o on a parent
cake("add", "work", "--task", "--assignee", "claude-code", "-m", "# work", agent=True)
cake("add", "qa", "--task", "-p", "work", "--assignee", "jul", "-m", "# first?\n\n- a) one\n- b) two", agent=True)
cake("add", "qb", "--task", "-p", "work", "--assignee", "jul", "-m", "# second?\n\n- a) one\n- b) two", agent=True)
cake("edit", "work", "--blocked-by", "qa,qb", agent=True)
pid, fd = pty.fork()
if pid == 0:
    os.chdir(STORE)
    os.execve(CAKE, [CAKE, "form", "answer"], ENV)
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
time.sleep(1.5)
for k in ["n", "\r", "j", "\r", "p", "1", "k", "o", ":q\r"]:
    os.write(fd, k.encode())
    time.sleep(0.5)
os.waitpid(pid, 0)
check("answer tree: <cr> j <cr> gave option 2", "answer: b) two" in item("qb")["body"])
check("answer tree: p and 1 answered the first", "answer: a) one" in item("qa")["body"])

# after the last answer the module stays and says so; l passes the empty
# review (answered items stay in answer only) and lands in mine
cake("add", "w2", "--task", "--assignee", "claude-code", "-m", "# more work", agent=True)
cake("add", "qc", "--task", "-p", "w2", "--assignee", "jul", "-m", "# third?\n\n- a) one\n- b) two", agent=True)
cake("edit", "w2", "--blocked-by", "qc", agent=True)
cake("add", "mine2", "--task", "--assignee", "me", "-m", "# a task of mine")
stays = tmux_screen(["1"], args=("form", "answer", "review", "mine"))
if stays is not None:
    check("answer done: the module stays with all done", "questions for you" in stays and "all done here" in stays)
cake("add", "qd", "--task", "-p", "w2", "--assignee", "jul", "-m", "# fourth?\n\n- a) one\n- b) two", agent=True)
cake("edit", "w2", "--blocked-by", "qd", agent=True)
landed = tmux_screen(["1", "l"], args=("form", "answer", "review", "mine"))
if landed is not None:
    check("answer done: on to mine, not review with the answers", "your tasks" in landed and "fourth?" not in landed)

# the last tab: the whole open tree with the details box
end = tmux_screen(["l"], args=("form", "mine"))
if end is not None:
    check("last tab: everything open as a tree", "everything open" in end and "ALL" in end)
start = tmux_screen([], args=("form", "mine"))
if start is not None:
    check("header: all listed from the start", " · all" in start)

# review: : sets fields of the item under review
cake("add", "rv", "--task", "--assignee", "claude-code", "-m", "# to review", agent=True)
cake("edit", "rv", "--owner", "jul")
cake("review", "rv", agent=True)
form([":", "prio 2\r", ":q\r"], args=("form", "review"))
# q on the last tab quits (form() fails if the form hangs)
form(["l", "q"], args=("form", "review"))
check("last tab: q quits", True)
check("review: :prio 2 set the priority", item("rv")["priority"] == 2)

# nothing waits: the form opens at dump anyway, esc goes on to the last tab
empty = tmux_screen(["\x1b"], args=("form", "answer"))
if empty is not None:
    check("nothing waits: dump, then the last tab", "everything open" in empty and "dump" in empty)

subprocess.run(["rm", "-rf", TMP])
sys.exit(1 if failures else 0)
