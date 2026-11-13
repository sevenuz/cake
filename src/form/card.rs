//! card modules: answer (questions for you) and review (finished work
//! you own), one item per card with the goal's tree above it

use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    /// tasks for you that block someone: an option or own words
    Answer,
    /// finished work you own: ok, drop or what should change
    Review,
}

/// how many items wait in `kind`
pub(super) fn count(cake: &Cake, kind: Kind, user: &str) -> Result<usize, Box<dyn Error>> {
    Ok(targets(cake, kind, user)?.1.len())
}

/// one item: tree on top, the card with its options below; keys choose,
/// enter confirms
#[allow(clippy::too_many_arguments)]
fn decide(
    ui: &mut Ui,
    cake: &Cake,
    kind: Kind,
    item: &Value,
    all: &[Value],
    goal: &Value,
    targets: &[&Value],
    decided: &std::collections::HashMap<String, String>,
    store: &str,
) -> Result<Outcome, Box<dyn Error>> {
    let t = theme::t();
    let g = style::g();
    let id = str_of(item, "id").to_string();
    let body = body_of(item);
    let (context, mut options) = match kind {
        Kind::Answer => split_options(&body),
        Kind::Review => {
            // a parent: its parts with their result
            let parts: Vec<String> = all
                .iter()
                .filter(|c| str_of(c, "parent") == str_of(item, "id") && !c["status"].is_null())
                .map(|c| format!("- {} {}", if str_of(c, "status") == "done" { style::g().check } else { style::g().dropped }, str_of(c, "title")))
                .collect();
            let text = if parts.is_empty() { body.clone() } else { format!("{}\n\nparts:\n{}", body, parts.join("\n")) };
            (text, vec![('a', "ok, done".into()), ('b', "drop".into())])
        }
    };
    let free = match kind {
        Kind::Answer => "own words",
        Kind::Review => "change something",
    };
    let free_key = (b'a' + options.len() as u8) as char;
    options.push((free_key, format!("{} {}", g.ellipsis, free)));
    let tree = overview(all, goal, targets);
    let width = column_width(ui.term.size()?.width, ui.max_width);
    // the first option (the recommended one) is preselected: enter takes it
    let (mut sel, mut hints, mut help) = (Some(0usize), false, false);
    let mut opened: Option<String> = None;
    let mut note: Option<(String, bool)> = None;
    loop {
        let mut card = markdown::render(&context, width.saturating_sub(4));
        if !card.is_empty() {
            card.push(Line::default());
        }
        for (n, (k, text)) in options.iter().enumerate() {
            let chosen = sel == Some(n);
            let mark = if chosen { g.select } else { " " };
            let s = if chosen { bold().fg(t.hint) } else { Style::default() };
            card.push(Line::from(vec![
                Span::styled(format!("{} {}) ", mark, k), Style::default().fg(t.hint)),
                Span::styled(text.clone(), s),
            ]));
        }
        let key_line = if hints {
            keys(&[("1-9 a-z", "open"), ("esc", "back")])
        } else {
            let mut parts = vec![("a-".to_string() + &free_key.to_string(), "choose".to_string())];
            if sel.is_some() {
                parts.push(("enter".into(), "confirm".into()));
            }
            parts.push(("f".into(), "open".into()));
            if opened.is_some() {
                parts.push(("esc".into(), "close".into()));
            }
            parts.extend([("s".into(), "skip".into()), ("?".into(), "keys".into())]);
            let parts: Vec<(&str, &str)> = parts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            keys(&parts)
        };
        let mut screen = Screen {
            tree: tree_lines(all, &tree, targets, &id, decided, hints, opened.as_deref(), width),
            card_title: format!("{} · {}", str_of(item, "title"), store),
            card,
            keys: key_line,
            note: note.take(),
            help,
        };
        ui.draw(&screen)?;
        let k = key()?;
        if ctrl_c(&k) {
            return Ok(Outcome::Quit);
        }
        if help {
            help = false;
            continue;
        }
        if hints {
            hints = false;
            if let KeyCode::Char(c) = k.code {
                if let Some((_, nid)) = HINTS.find(c).and_then(|n| tree.get(n)) {
                    opened = Some(nid.clone());
                }
            }
            continue;
        }
        let pick = match k.code {
            KeyCode::Esc => {
                opened = None;
                sel = None;
                continue;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                sel = Some(sel.map_or(0, |s| (s + 1).min(options.len() - 1)));
                continue;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                sel = Some(sel.map_or(0, |s| s.saturating_sub(1)));
                continue;
            }
            KeyCode::Char('q') => return Ok(Outcome::Quit),
            KeyCode::Char('n') => return Ok(Outcome::NextGoal),
            KeyCode::Char('s') => return Ok(Outcome::Next(None, false)),
            KeyCode::Char('?') => {
                help = true;
                continue;
            }
            KeyCode::Char('f') => {
                hints = true;
                continue;
            }
            KeyCode::Char('i') => options.len() - 1,
            KeyCode::Char(c) => {
                if let Some(n) = options.iter().position(|(k, _)| *k == c) {
                    sel = Some(n);
                }
                continue;
            }
            KeyCode::Enter => match sel {
                Some(n) => n,
                None => {
                    note = Some(("choose an option first".to_string(), false));
                    continue;
                }
            },
            _ => continue,
        };
        // the chosen option, with text where it needs some
        let last = pick == options.len() - 1;
        let result: Result<(String, bool), Box<dyn Error>> = match (kind, pick) {
            (_, _) if last => {
                let prompt = match kind {
                    Kind::Answer => "your answer:",
                    Kind::Review => "what should change:",
                };
                let Some(text) = ui.input(&mut screen, prompt)?.filter(|t| !t.is_empty()) else { continue };
                ui.save(&mut screen, || match kind {
                    Kind::Answer => {
                        cake.call(&["done", &id, "-m", &format!("answer: {}", text)]).map(|_| (text.clone(), true))
                    }
                    Kind::Review => cake.call(&["comment", &id, "-m", &text]).map(|_| ("sent back".to_string(), false)),
                })?
            }
            (Kind::Answer, n) => {
                let answer = answer_text(&body, &options[n].0.to_string());
                let note = format!("answer: {}", answer);
                ui.save(&mut screen, || cake.call(&["done", &id, "-m", &note]).map(|_| (answer.clone(), true)))?
            }
            (Kind::Review, 0) => ui.save(&mut screen, || cake.call(&["done", &id]).map(|_| ("done".to_string(), true)))?,
            (Kind::Review, _) => ui.save(&mut screen, || cake.call(&["drop", &id]).map(|_| ("dropped".to_string(), false)))?,
        };
        match result {
            Ok((text, counted)) => {
                let short: String = text.chars().take(40).collect();
                let short = if text.chars().count() > 40 { format!("{}{}", short, g.ellipsis) } else { short };
                return Ok(Outcome::Next(Some(short), counted));
            }
            Err(e) => note = Some((e.to_string(), true)),
        }
    }
}

/// the items of a card module in one store, and all items of the store
fn targets(cake: &Cake, kind: Kind, user: &str) -> Result<(Vec<Value>, Vec<String>), Box<dyn Error>> {
    let all = cake.call(&["ls", "-l"])?;
    let open = |id: &str| find(&all, id).is_some_and(|i| matches!(str_of(i, "status"), "inbox" | "task" | "doing" | "review"));
    let ids = all
        .iter()
        .filter(|i| match kind {
            // a task for you that someone else's work waits on
            Kind::Answer => {
                str_of(i, "status") == "task"
                    && str_of(i, "assignee") == user
                    && i["blocks"].as_array().is_some_and(|b| b.iter().any(|b| open(b.as_str().unwrap_or(""))))
            }
            Kind::Review => {
                str_of(i, "status") == "review" && (str_of(i, "owner") == user || i["owner"].is_null())
            }
        })
        .map(|i| str_of(i, "id").to_string())
        .collect();
    Ok((all, ids))
}

/// a step in one store, grouped by goal, the most urgent goal first;
/// returns (counted, quit)
pub(super) fn run(ui: &mut Ui, cake: &Cake, kind: Kind, user: &str, store: &str) -> Result<(usize, bool), Box<dyn Error>> {
    // reload after each pass: a decision can make new items due (a parent
    // goes to review when its last part is done)
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut counted = 0;
    loop {
        let (all, ids) = targets(cake, kind, user)?;
        let items: Vec<&Value> = ids.iter().filter(|id| !seen.contains(*id)).filter_map(|id| find(&all, id)).collect();
        if items.is_empty() {
            return Ok((counted, false));
        }
        ui.total += items.iter().filter(|_| !seen.is_empty()).count();
        let mut goals: Vec<(&Value, Vec<&Value>)> = vec![];
        for it in &items {
            let goal = goal_of(&all, it);
            match goals.iter_mut().find(|(g, _)| str_of(g, "id") == str_of(goal, "id")) {
                Some((_, list)) => list.push(it),
                None => goals.push((goal, vec![it])),
            }
        }
        let score = |v: &&Value| v["score"].as_i64().unwrap_or(0);
        goals.sort_by_key(|(_, list)| std::cmp::Reverse(list.iter().map(score).max().unwrap_or(0)));
        let mut decided = std::collections::HashMap::new();
        'goals: for (goal, list) in &goals {
            for (i, it) in list.iter().enumerate() {
                seen.insert(str_of(it, "id").to_string());
                match decide(ui, cake, kind, it, &all, goal, list, &decided, store)? {
                    Outcome::Next(result, c) => {
                        ui.handled += 1;
                        counted += c as usize;
                        if let Some(r) = result {
                            decided.insert(str_of(it, "id").to_string(), r);
                        }
                    }
                    Outcome::NextGoal => {
                        for rest in &list[i..] {
                            seen.insert(str_of(rest, "id").to_string());
                        }
                        ui.handled += list.len() - i;
                        continue 'goals;
                    }
                    Outcome::Quit => return Ok((counted, true)),
                }
            }
        }
    }
}

