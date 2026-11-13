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

/// what a step did: count, and whether the user quit the step
struct Step {
    count: usize,
    quit: bool,
}

/// open questions of one store by score; a letter picks an option line
fn questions(skin: &MadSkin, cake: &Cake, header: &dyn Fn()) -> Result<Step, Box<dyn Error>> {
    let mut step = Step { count: 0, quit: false };
    let asks = cake.call(&["ready", "-t", "ask", "-l"])?;
    let all = cake.call(&["ls"])?;
    let title_of = |id: &str| {
        all.iter().find(|i| str_of(i, "id") == id).map(|i| str_of(i, "title").to_string()).unwrap_or_default()
    };
    if !asks.is_empty() {
        header();
    }
    for ask in &asks {
        let id = str_of(ask, "id");
        let body = body_of(ask);
        skin.print_text(&format!(
            "\n---\n## {}\n*{}* · about *{}*\n\n{}",
            str_of(ask, "title"),
            id,
            title_of(str_of(ask, "parent")),
            body
        ));
        loop {
            let Some(input) = read_line("> ")? else {
                step.quit = true;
                return Ok(step);
            };
            match input.as_str() {
                "" => break,
                "q" => {
                    step.quit = true;
                    return Ok(step);
                }
                _ => match cake.call(&["done", id, "-m", &answer_text(&body, &input)]) {
                    Ok(_) => {
                        step.count += 1;
                        break;
                    }
                    Err(e) => println!("  {}", e),
                },
            }
        }
    }
    Ok(step)
}

/// agents' notes with `done when` and no open question below
fn planned(skin: &MadSkin, cake: &Cake, user: &str, header: &dyn Fn()) -> Result<Step, Box<dyn Error>> {
    let mut step = Step { count: 0, quit: false };
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
    if !planned.is_empty() {
        header();
    }
    for item in planned {
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
            "q" => {
                step.quit = true;
                return Ok(step);
            }
            _ => continue,
        };
        match result {
            Ok(v) if !v.is_empty() => step.count += 1,
            Ok(_) => (),
            Err(e) => println!("  {}", e),
        }
    }
    Ok(step)
}

/// paper todos go to `root`; questions and planned items come from all
/// known stores (stores.json, global), `root` first
pub fn run(skin: &MadSkin, root: &Path, user: &str) -> Result<(), Box<dyn Error>> {
    let same = |a: &Path, b: &Path| std::fs::canonicalize(a).ok() == std::fs::canonicalize(b).ok();
    let mut stores: Vec<(String, std::path::PathBuf)> = vec![("here".to_string(), root.to_path_buf())];
    if let Some((name, _)) = crate::stores::all()?.into_iter().find(|(_, s)| same(s, root)) {
        stores[0].0 = name;
    }
    stores.extend(crate::stores::all()?.into_iter().filter(|(_, s)| !same(s, root)));
    let header = |name: &str| {
        if stores.len() > 1 {
            skin.print_text(&format!("\n# {}", name));
        }
    };

    skin.print_text(&format!("# Morning\n**1 · Paper todos** for *{}*, one per line, empty line ends:", stores[0].0));
    let here = Cake { root };
    let mut added = 0;
    while let Some(line) = read_line("+ ")? {
        if line.is_empty() {
            break;
        }
        match here.call(&["add", "--todo", "-m", &line]) {
            Ok(_) => added += 1,
            Err(e) => println!("  {}", e),
        }
    }

    skin.print_text("\n**2 · Questions**: letter or text, empty skips, `q` ends the step");
    let mut answered = 0;
    for (name, root) in &stores {
        let step = questions(skin, &Cake { root }, &|| header(name))?;
        answered += step.count;
        if step.quit {
            break;
        }
    }

    skin.print_text("\n**3 · Planned by agents**");
    let mut approved = 0;
    for (name, root) in &stores {
        let step = planned(skin, &Cake { root }, user, &|| header(name))?;
        approved += step.count;
        if step.quit {
            break;
        }
    }

    let mut summary = format!("\n---\n**Done:** {} added · {} answered · {} approved\n", added, answered, approved);
    for (name, root) in &stores {
        let ready = Cake { root }.call(&["ready", "-t", "~ask"])?;
        let agents = ready.iter().filter(|i| str_of(i, "by") != user).count();
        if !ready.is_empty() {
            summary += &format!("* *{}*: {} ready for agents · {} for you\n", name, agents, ready.len() - agents);
        }
    }
    skin.print_text(&summary);
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
