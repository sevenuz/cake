//! `cake morning`: a guided pass over what needs you. Every change runs as
//! its own `cake` call, so agents keep working while you think.

use crate::error;
use serde_json::Value;
use std::error::Error;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::Command;
use termimad::{MadSkin, Question};

/// set for the form's own cake calls: built-ins without alias overrides
pub const PLAIN_ENV: &str = "CAKE_PLAIN";

struct Cake<'a> {
    root: &'a Path,
}

impl Cake<'_> {
    /// runs `cake --store <root> <args> --json`, returns the items
    fn call(&self, args: &[&str]) -> Result<Vec<Value>, Box<dyn Error>> {
        let out = Command::new(std::env::current_exe()?)
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

pub fn run(skin: &MadSkin, root: &Path, user: &str) -> Result<(), Box<dyn Error>> {
    let cake = Cake { root };
    let (mut added, mut answered, mut approved) = (0, 0, 0);

    skin.print_text("# Morning\n**1 · Paper todos**, one per line, empty line ends:");
    while let Some(line) = read_line("+ ")? {
        if line.is_empty() {
            break;
        }
        match cake.call(&["add", "--todo", "-m", &line]) {
            Ok(_) => added += 1,
            Err(e) => println!("  {}", e),
        }
    }

    let asks = cake.call(&["ready", "-t", "ask", "-l"])?;
    let all = cake.call(&["ls", "-l"])?;
    let title_of = |id: &str| {
        all.iter().find(|i| str_of(i, "id") == id).map(|i| str_of(i, "title").to_string()).unwrap_or_default()
    };
    skin.print_text(&format!("\n**2 · Questions** ({}), letter or text, empty skips, `q` ends:", asks.len()));
    'asks: for ask in &asks {
        let id = str_of(ask, "id");
        let body = body_of(ask);
        skin.print_text(&format!("\n---\n## {}\n*{}* · about *{}*\n\n{}", str_of(ask, "title"), id, title_of(str_of(ask, "parent")), body));
        loop {
            let Some(input) = read_line("> ")? else { break 'asks };
            match input.as_str() {
                "" => break,
                "q" => break 'asks,
                _ => match cake.call(&["done", id, "-m", &answer_text(&body, &input)]) {
                    Ok(_) => {
                        answered += 1;
                        break;
                    }
                    Err(e) => println!("  {}", e),
                },
            }
        }
    }

    // planned: agents' notes with `done when` and no open question below;
    // read again, answers above may have unblocked some
    let all = cake.call(&["ls", "-l"])?;
    let open_ask = |i: &Value| {
        i["children"].as_array().into_iter().flatten().any(|c| {
            all.iter().any(|a| {
                str_of(a, "id") == c.as_str().unwrap_or("")
                    && a["tags"].as_array().is_some_and(|t| t.iter().any(|t| t == "ask"))
                    && matches!(str_of(a, "status"), "todo" | "doing")
            })
        })
    };
    let planned: Vec<&Value> = all
        .iter()
        .filter(|i| i["status"].is_null() && str_of(i, "by") != user)
        .filter(|i| str_of(i, "body").contains("done when") && !open_ask(i))
        .collect();
    skin.print_text(&format!("\n**3 · Planned by agents** ({}):", planned.len()));
    'planned: for item in planned {
        let id = str_of(item, "id");
        skin.print_text(&format!("\n---\n## {}\n*{}*\n\n{}", str_of(item, "title"), id, body_of(item)));
        let mut q = Question::new("");
        q.add_answer('d', "**d**o: approve");
        q.add_answer('t', "do with **t**imes: due, deadline");
        q.add_answer('x', "drop");
        q.add_answer('s', "**s**kip");
        q.add_answer('q', "**q**uit");
        q.set_default('s');
        let result = match q.ask(skin)?.as_str() {
            "d" => cake.call(&["do", id]),
            "t" => {
                let due = read_line("  due (soft, 2026-11-30, empty none): ")?.unwrap_or_default();
                let deadline = read_line("  deadline (hard, empty none): ")?.unwrap_or_default();
                let mut args = vec!["edit", id];
                if !due.is_empty() {
                    args.extend(["--due", &due]);
                }
                if !deadline.is_empty() {
                    args.extend(["--deadline", &deadline]);
                }
                let edit = if args.len() > 2 { cake.call(&args) } else { Ok(vec![]) };
                edit.and_then(|_| cake.call(&["do", id]))
            }
            "x" => cake.call(&["drop", id]).map(|_| vec![]),
            "q" => break 'planned,
            _ => continue,
        };
        match result {
            Ok(v) if !v.is_empty() => approved += 1,
            Ok(_) => (),
            Err(e) => println!("  {}", e),
        }
    }

    let ready = cake.call(&["ready", "-t", "~ask"])?;
    let for_agents = ready.iter().filter(|i| str_of(i, "by") != user).count();
    skin.print_text(&format!(
        "\n---\n**Done:** {} added · {} answered · {} approved · {} ready for agents · {} for you",
        added,
        answered,
        approved,
        for_agents,
        ready.len() - for_agents
    ));
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
