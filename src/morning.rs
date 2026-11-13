//! `cake morning`: a guided pass over what needs you. Every change runs as
//! its own `cake` call, so agents keep working while you think.

use crate::{error, style};
use serde_json::Value;
use std::error::Error;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::Command;
use ratatui::crossterm::style::Stylize;

/// set for the form's own cake calls: built-ins without alias overrides
pub const PLAIN_ENV: &str = "CAKE_PLAIN";

struct Cake<'a> {
    root: &'a Path,
}

impl Cake<'_> {
    /// runs `cake --store <root> <args> --json`, returns the items
    fn call(&self, args: &[&str]) -> Result<Vec<Value>, Box<dyn Error>> {
        let out = Command::new(exe())
            .arg("--store")
            .arg(self.root)
            .args(args)
            .arg("--json")
            .env(PLAIN_ENV, "1")
            .output()?;
        let v: Value = serde_json::from_slice(&out.stdout)
            .map_err(|_| error::invalid(String::from_utf8_lossy(&out.stderr).trim().to_string()))?;
        if let Some(e) = v.get("error") {
            return Err(error::invalid(e["message"].as_str().unwrap_or("cake failed").to_string()));
        }
        Ok(v["items"].as_array().cloned().unwrap_or_default())
    }
}

/// this binary; after a rebuild while the form runs, Linux names the old
/// one `… (deleted)`, then the new file at that path (or cake on PATH)
fn exe() -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    if exe.is_file() {
        return exe;
    }
    let path = exe.to_string_lossy();
    let replaced = std::path::PathBuf::from(path.strip_suffix(" (deleted)").unwrap_or(&path));
    if replaced.is_file() {
        replaced
    } else {
        "cake".into()
    }
}

fn read_line(prompt: &str) -> Result<Option<String>, Box<dyn Error>> {
    print!("{}", prompt);
    std::io::stdout().flush()?;
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line)? == 0 {
        return Ok(None); // end of input
    }
    Ok(Some(line.trim().to_string()))
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

/// body without its `# title` line
fn body_of(item: &Value) -> String {
    let body = str_of(item, "body");
    let title = format!("# {}", str_of(item, "title"));
    body.lines().filter(|l| l.trim() != title).collect::<Vec<_>>().join("\n").trim().to_string()
}

/// `a` picks the line `- a) …` of the body, anything else is the answer
fn answer_text(body: &str, input: &str) -> String {
    if input.chars().count() == 1 {
        let tag = format!("- {})", input);
        let lines: Vec<&str> = body.lines().collect();
        if let Some(n) = lines.iter().position(|l| l.trim_start().starts_with(&tag)) {
            // an option may wrap: indented lines below belong to it
            let mut text = lines[n].trim_start().trim_start_matches("- ").to_string();
            for l in &lines[n + 1..] {
                if l.trim().is_empty() || l.trim_start().starts_with("- ") || !l.starts_with(' ') {
                    break;
                }
                text = format!("{} {}", text, l.trim());
            }
            return text;
        }
    }
    input.to_string()
}

/// what a step did: count, and whether the user quit the step
struct Step {
    count: usize,
    shown: usize,
    quit: bool,
}

fn done(text: &str) {
    println!("{}", format!("    {}", text).green());
}

fn failed(e: Box<dyn Error>) {
    println!("{}", format!("    {}", e).red());
}

fn find<'a>(all: &'a [Value], id: &str) -> Option<&'a Value> {
    all.iter().find(|i| str_of(i, "id") == id)
}

/// the top-level item above `item`
fn goal_of<'a>(all: &'a [Value], item: &'a Value) -> &'a Value {
    let mut cur = item;
    for _ in 0..crate::store::MAX_DEPTH {
        match find(all, str_of(cur, "parent")) {
            Some(p) => cur = p,
            None => break,
        }
    }
    cur
}

/// the goal's tree, only branches that lead to `targets` (marked ?);
/// with `hints` each line gets a number to open it; returns the lines' ids
fn overview(all: &[Value], goal: &Value, targets: &[&Value], hints: bool) -> Vec<String> {
    let mut keep: Vec<String> = vec![];
    for t in targets {
        let mut cur = Some(*t);
        while let Some(c) = cur {
            if !keep.iter().any(|k| k == str_of(c, "id")) {
                keep.push(str_of(c, "id").to_string());
            }
            cur = find(all, str_of(c, "parent"));
        }
    }
    let g = style::g();
    let mut out = vec![];
    fn walk(
        all: &[Value],
        keep: &[String],
        targets: &[&Value],
        hints: bool,
        id: &str,
        prefix: (&str, &str),
        out: &mut Vec<String>,
    ) {
        let g = style::g();
        let item = find(all, id).unwrap();
        let target = targets.iter().any(|t| str_of(t, "id") == id);
        let hint = if hints { format!("[{}] ", out.len() + 1) } else { String::new() };
        let title = str_of(item, "title");
        let text = if out.is_empty() {
            title.bold().to_string()
        } else if target {
            format!("? {}", title).yellow().to_string()
        } else {
            title.to_string()
        };
        println!("  {}{}{}", prefix.0.dark_grey(), hint.cyan(), text);
        out.push(id.to_string());
        let children: Vec<&str> = all
            .iter()
            .filter(|i| str_of(i, "parent") == id && keep.iter().any(|k| k == str_of(i, "id")))
            .map(|i| str_of(i, "id"))
            .collect();
        for (n, c) in children.iter().enumerate() {
            let last = n + 1 == children.len();
            let own = format!("{}{}", prefix.1, if last { g.last } else { g.branch });
            let below = format!("{}{}", prefix.1, if last { g.space } else { g.pipe });
            walk(all, keep, targets, hints, c, (&own, &below), out);
        }
    }
    let _ = g;
    walk(all, &keep, targets, hints, str_of(goal, "id"), ("", ""), &mut out);
    out
}

/// prints the body of a numbered overview line
fn open(all: &[Value], goal: &Value, targets: &[&Value]) -> Result<(), Box<dyn Error>> {
    let ids = overview(all, goal, targets, true);
    let Some(input) = read_line("    open: ")? else { return Ok(()) };
    match input.parse::<usize>().ok().and_then(|n| ids.get(n.wrapping_sub(1))) {
        Some(id) => {
            let item = find(all, id).unwrap();
            println!("\n  {}", str_of(item, "title").bold());
            for l in body_of(item).lines() {
                println!("    {}", l);
            }
            println!();
        }
        None => println!("{}", "    no such number".dark_grey()),
    }
    Ok(())
}

#[derive(PartialEq)]
enum Kind {
    /// agents' questions, answered with a letter or own words
    Ask,
    /// agents' plans to approve
    Plan,
}

/// what the user did with one item
enum Outcome {
    Next,
    NextGoal,
    Quit,
}

/// one item of a step: shows it, reads keys until it is decided
fn decide(cake: &Cake, kind: &Kind, item: &Value, all: &[Value], goal: &Value, targets: &[&Value], counter: &str) -> Result<(Outcome, bool), Box<dyn Error>> {
    let id = str_of(item, "id");
    let body = body_of(item);
    println!("\n  {} {}", str_of(item, "title").bold(), counter.dark_grey());
    for l in body.lines() {
        println!("    {}", l);
    }
    let legend = match kind {
        Kind::Ask => "letter or own words | f open | Enter skip | n next goal | q quit",
        Kind::Plan => "d approve | t approve with dates | x drop | own words: comment\n    f open | Enter skip | n next goal | q quit",
    };
    println!("{}", format!("    {}", legend).dark_grey());
    loop {
        let Some(input) = read_line("  > ")? else { return Ok((Outcome::Quit, false)) };
        let result = match (kind, input.as_str()) {
            (_, "") => return Ok((Outcome::Next, false)),
            (_, "q") => return Ok((Outcome::Quit, false)),
            (_, "n") => return Ok((Outcome::NextGoal, false)),
            (_, "f") => {
                open(all, goal, targets)?;
                continue;
            }
            (Kind::Ask, text) => {
                let answer = answer_text(&body, text);
                cake.call(&["done", id, "-m", &answer]).map(|_| format!("answered: {}", answer))
            }
            (Kind::Plan, "d") => cake.call(&["do", id]).map(|_| "approved".to_string()),
            (Kind::Plan, "t") => {
                let due = read_line("    due, soft target (2026-11-30, Enter none): ")?.unwrap_or_default();
                let deadline = read_line("    deadline, hard limit (Enter none): ")?.unwrap_or_default();
                let mut args = vec!["edit", id];
                if !due.is_empty() {
                    args.extend(["--due", &due]);
                }
                if !deadline.is_empty() {
                    args.extend(["--deadline", &deadline]);
                }
                let edit = if args.len() > 2 { cake.call(&args).map(|_| ()) } else { Ok(()) };
                edit.and_then(|_| cake.call(&["do", id])).map(|_| "approved with dates".to_string())
            }
            (Kind::Plan, "x") => cake.call(&["drop", id]).map(|_| "dropped".to_string()),
            (Kind::Plan, comment) => cake
                .call(&["add", "--todo", "-p", id, "-t", "reply", "-m", comment])
                .map(|_| "comment saved, the agent reworks the plan".to_string()),
        };
        match result {
            Ok(text) => {
                let counts = text.starts_with("answered") || text.starts_with("approved");
                done(&text);
                return Ok((Outcome::Next, counts));
            }
            Err(e) => failed(e),
        }
    }
}

/// a step over one store: items grouped by goal, the goal with the most
/// urgent item first, each goal with its overview
fn step(cake: &Cake, kind: Kind, user: &str, header: &dyn Fn()) -> Result<Step, Box<dyn Error>> {
    let mut step = Step { count: 0, shown: 0, quit: false };
    let all = cake.call(&["ls", "-l"])?;
    let targets: Vec<&Value> = match kind {
        Kind::Ask => {
            let ids: Vec<String> =
                cake.call(&["ready", "-t", "ask"])?.iter().map(|i| str_of(i, "id").to_string()).collect();
            all.iter().filter(|i| ids.iter().any(|id| id == str_of(i, "id"))).collect()
        }
        Kind::Plan => {
            let side = |a: &Value| a["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "ask" || t == "reply"));
            let open_side = |a: &Value| side(a) && matches!(str_of(a, "status"), "todo" | "doing");
            let kids = |i: &Value| -> Vec<&Value> { all.iter().filter(|a| str_of(a, "parent") == str_of(i, "id")).collect() };
            all.iter()
                .filter(|i| i["status"].is_null() && str_of(i, "by") != user)
                .filter(|i| !i["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "doc")))
                .filter(|i| str_of(i, "body").contains("done when"))
                .filter(|i| kids(i).iter().all(|k| side(k)) && !kids(i).iter().any(|k| open_side(k)))
                .collect()
        }
    };
    step.shown = targets.len();
    if targets.is_empty() {
        return Ok(step);
    }
    header();
    let mut goals: Vec<(&Value, Vec<&Value>)> = vec![];
    for t in &targets {
        let goal = goal_of(&all, t);
        match goals.iter_mut().find(|(g, _)| str_of(g, "id") == str_of(goal, "id")) {
            Some((_, items)) => items.push(t),
            None => goals.push((goal, vec![t])),
        }
    }
    let score = |v: &&Value| v["score"].as_i64().unwrap_or(0);
    goals.sort_by_key(|(_, items)| std::cmp::Reverse(items.iter().map(score).max().unwrap_or(0)));
    let mut n = 0;
    'goals: for (goal, items) in &goals {
        println!();
        overview(&all, goal, items, false);
        for item in items {
            n += 1;
            let counter = format!("{}/{}", n, targets.len());
            let (outcome, counted) = decide(cake, &kind, item, &all, goal, items, &counter)?;
            if counted {
                step.count += 1;
            }
            match outcome {
                Outcome::Next => (),
                Outcome::NextGoal => {
                    n += items.len().saturating_sub(items.iter().position(|i| str_of(i, "id") == str_of(item, "id")).unwrap_or(0) + 1);
                    continue 'goals;
                }
                Outcome::Quit => {
                    step.quit = true;
                    break 'goals;
                }
            }
        }
    }
    Ok(step)
}

/// new tasks go to `root`; questions and planned items come from `root`,
/// with `all` from every known store (stores.json, global)
pub fn run(root: &Path, user: &str, all: bool) -> Result<(), Box<dyn Error>> {
    let same = |a: &Path, b: &Path| std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok();
    let known = crate::stores::all()?;
    let name = known.iter().find(|(_, s)| same(s, root)).map_or("this store".to_string(), |(n, _)| n.clone());
    let mut stores: Vec<(String, std::path::PathBuf)> = vec![(name, root.to_path_buf())];
    if all {
        stores.extend(known.into_iter().filter(|(_, s)| !same(s, root)));
    }
    let header = |name: &str| {
        if stores.len() > 1 {
            println!("\n{}", format!("== {} ==", name).bold());
        }
    };
    let title = |t: &str| println!("\n{}", t.bold().underlined());

    title(&format!("1/3 New tasks for {}", stores[0].0));
    println!("Tasks you already know (e.g. from a paper list), one per line, empty line ends.");
    let here = Cake { root };
    let mut added = 0;
    while let Some(line) = read_line("  + ")? {
        if line.is_empty() {
            break;
        }
        match here.call(&["add", "--todo", "-m", &line]) {
            Ok(_) => {
                done("added as todo");
                added += 1;
            }
            Err(e) => failed(e),
        }
    }

    title("2/3 Questions from agents");
    println!("They block the work they are about until you answer.");
    let (mut answered, mut shown) = (0, 0);
    for (name, root) in &stores {
        let step = step(&Cake { root }, Kind::Ask, user, &|| header(name))?;
        answered += step.count;
        shown += step.shown;
        if step.quit {
            break;
        }
    }
    if shown == 0 {
        println!("{}", "  none open".dark_grey());
    }

    title("3/3 Plans to approve");
    println!("Plans agents wrote for your notes. Approved ones are worked on, the others wait.");
    let (mut approved, mut shown) = (0, 0);
    for (name, root) in &stores {
        let step = step(&Cake { root }, Kind::Plan, user, &|| header(name))?;
        approved += step.count;
        shown += step.shown;
        if step.quit {
            break;
        }
    }
    if shown == 0 {
        println!("{}", "  none waiting".dark_grey());
    }

    title("Done");
    println!("{} added, {} answered, {} approved", added, answered, approved);
    for (name, root) in &stores {
        let ready = Cake { root }.call(&["ready", "-t", "~ask,~reply"])?;
        let agents = ready.iter().filter(|i| str_of(i, "by") != user).count();
        if !ready.is_empty() {
            println!("  {}: {} ready for agents, {} for you", name, agents, ready.len() - agents);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_letter_picks_the_option_line() {
        let body = "Context.\n\n- a) sqlite (recommended)\n- b) files";
        assert_eq!(answer_text(body, "b"), "b) files");
        let wrapped = "- a) by hand (recommended:\n  no code)\n- b) flag";
        assert_eq!(answer_text(wrapped, "a"), "a) by hand (recommended: no code)");
        assert_eq!(answer_text(body, "c"), "c");
        assert_eq!(answer_text(body, "files, but plain"), "files, but plain");
    }
}
