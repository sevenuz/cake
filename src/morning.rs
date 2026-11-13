//! `cake morning`: a guided pass over what needs you. Every change runs as
//! its own `cake` call, so agents keep working while you think.

use crate::error;
use serde_json::Value;
use std::error::Error;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::Command;
use termimad::crossterm::style::Stylize;

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
        if let Some(l) = body.lines().find(|l| l.trim_start().starts_with(&tag)) {
            return l.trim_start().trim_start_matches("- ").to_string();
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

/// an item: title, where it is in the tree, body indented
fn show(item: &Value, all: &[Value], label: &str) {
    let mut path = vec![];
    let mut cur = str_of(item, "parent").to_string();
    while let Some(p) = all.iter().find(|i| str_of(i, "id") == cur) {
        path.push(str_of(p, "title").to_string());
        cur = str_of(p, "parent").to_string();
        if path.len() > 3 {
            break;
        }
    }
    path.reverse();
    println!();
    println!("{} {}", label.dark_grey(), str_of(item, "title").bold());
    let mut meta = format!("id {}", str_of(item, "id"));
    if !path.is_empty() {
        meta += &format!(", in: {}", path.join(" > "));
    }
    println!("{}", meta.dark_grey());
    for l in body_of(item).lines() {
        println!("    {}", l);
    }
}

fn done(text: &str) {
    println!("{}", format!("  -> {}", text).green());
}

fn failed(e: Box<dyn Error>) {
    println!("{}", format!("  !! {}", e).red());
}

/// open questions of one store by score; a letter picks an option line
fn questions(cake: &Cake, header: &dyn Fn()) -> Result<Step, Box<dyn Error>> {
    let mut step = Step { count: 0, shown: 0, quit: false };
    let asks = cake.call(&["ready", "-t", "ask", "-l"])?;
    let all = cake.call(&["ls"])?;
    step.shown = asks.len();
    if !asks.is_empty() {
        header();
    }
    for (n, ask) in asks.iter().enumerate() {
        let id = str_of(ask, "id");
        let body = body_of(ask);
        show(ask, &all, &format!("[{}/{}]", n + 1, asks.len()));
        println!("{}", "  answer: option letter or your own words | Enter skip | q quit".dark_grey());
        loop {
            let Some(input) = read_line("  > ")? else {
                step.quit = true;
                return Ok(step);
            };
            match input.as_str() {
                "" => break,
                "q" => {
                    step.quit = true;
                    return Ok(step);
                }
                _ => {
                    let answer = answer_text(&body, &input);
                    match cake.call(&["done", id, "-m", &answer]) {
                        Ok(_) => {
                            done(&format!("answered: {}", answer));
                            step.count += 1;
                            break;
                        }
                        Err(e) => failed(e),
                    }
                }
            }
        }
    }
    Ok(step)
}

/// plans an agent wrote for you to approve: its notes with `done when`,
/// without children of their own (parents are overviews) and without an
/// open question
fn planned(cake: &Cake, user: &str, header: &dyn Fn()) -> Result<Step, Box<dyn Error>> {
    let mut step = Step { count: 0, shown: 0, quit: false };
    let all = cake.call(&["ls", "-l"])?;
    let side = |a: &Value| a["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "ask" || t == "reply"));
    let open_side = |a: &Value| side(a) && matches!(str_of(a, "status"), "todo" | "doing");
    let kids = |i: &Value| -> Vec<&Value> {
        let ids: Vec<&str> = i["children"].as_array().into_iter().flatten().filter_map(|c| c.as_str()).collect();
        all.iter().filter(|a| ids.contains(&str_of(a, "id"))).collect()
    };
    let planned: Vec<&Value> = all
        .iter()
        .filter(|i| i["status"].is_null() && str_of(i, "by") != user)
        .filter(|i| !i["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "doc")))
        .filter(|i| str_of(i, "body").contains("done when"))
        .filter(|i| kids(i).iter().all(|k| side(k)) && !kids(i).iter().any(|k| open_side(k)))
        .collect();
    step.shown = planned.len();
    if !planned.is_empty() {
        header();
    }
    for (n, item) in planned.iter().enumerate() {
        let id = str_of(item, "id");
        show(item, &all, &format!("[{}/{}]", n + 1, planned.len()));
        println!(
            "{}",
            "  d approve | t approve with dates | x drop | Enter skip | q quit\n  anything else: a comment for the agent, the plan stays open"
                .dark_grey()
        );
        let Some(input) = read_line("  > ")? else {
            step.quit = true;
            return Ok(step);
        };
        let result = match input.as_str() {
            "" => continue,
            "q" => {
                step.quit = true;
                return Ok(step);
            }
            "d" => cake.call(&["do", id]).map(|_| "approved, agents may start it".to_string()),
            "t" => {
                let due = read_line("  due, soft target (2026-11-30, Enter none): ")?.unwrap_or_default();
                let deadline = read_line("  deadline, hard limit (Enter none): ")?.unwrap_or_default();
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
            "x" => cake.call(&["drop", id]).map(|_| "dropped".to_string()),
            comment => cake
                .call(&["add", "--todo", "-p", id, "-t", "reply", "-m", comment])
                .map(|_| "comment saved, the agent reworks the plan".to_string()),
        };
        match result {
            Ok(text) => {
                if text.starts_with("approved") {
                    step.count += 1;
                }
                done(&text);
            }
            Err(e) => failed(e),
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

    println!("{}", "Ctrl-C ends any time, every change is saved right away.".dark_grey());
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
        let step = questions(&Cake { root }, &|| header(name))?;
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
        let step = planned(&Cake { root }, user, &|| header(name))?;
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
        assert_eq!(answer_text(body, "c"), "c");
        assert_eq!(answer_text(body, "files, but plain"), "files, but plain");
    }
}
