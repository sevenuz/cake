//! `cake migrate`: from the tag based workflow (todo, ask/reply/review/
//! triaged/ai tags) to inbox/note/task/doing/review with assignee and owner

use crate::commands::Report;
use crate::item::Status;
use crate::store::Store;
use std::error::Error;

const OLD_TAGS: &[&str] = &["ask", "reply", "review", "triaged", "sorted", "ai"];

/// what changes for one item, as readable lines
struct Change {
    id: String,
    what: Vec<String>,
}

pub fn run(store: &mut Store, user: &str, apply: bool) -> Result<Report, Box<dyn Error>> {
    let mut ids: Vec<String> = store.get().keys().cloned().collect();
    ids.sort();
    let mut changes = vec![];
    let mut removed = vec![];
    for id in &ids {
        let Some(item) = store.get_item(id).cloned() else { continue };
        let has = |t: &str| item.tags().iter().any(|x| x == t);
        // without `by` it was written by hand (or in Obsidian): the user's
        let mine = item.by().map_or(true, |b| b == user);
        let open = matches!(item.status(), Some(Status::Task) | Some(Status::Doing));
        let mut what = vec![];
        let mut next = item.clone();

        if has("reply") && open {
            // a comment becomes text of the item it was about
            if let Some(parent) = item.parents().first().cloned() {
                what.push(format!("comment appended to {}, item removed", parent));
                if apply {
                    let date: String = crate::util::format_timestamp(item.timestamp()).chars().take(10).collect();
                    let text: String = item.content().lines().filter(|l| !l.starts_with("# ")).collect::<Vec<_>>().join(" ");
                    let by = item.by().cloned().unwrap_or_else(|| user.to_string());
                    let comment = format!("comment ({}, {}): {} {}", by, date, item.title(), text.trim());
                    store.get_item_mut(&parent).unwrap().append_content(comment.trim());
                    removed.push(id.clone());
                }
                changes.push(Change { id: id.clone(), what });
                continue;
            }
        }
        if has("ask") && open {
            what.push(format!("question: task for {}", user));
            next.set_status(Status::Task).ok();
            next.set_assignee(Some(user.to_string()));
            if next.owner().is_none() {
                next.set_owner(Some(user.to_string()));
            }
        } else if has("review") && open {
            what.push("status review".into());
            next.set_status(Status::Review).ok();
            if next.owner().is_none() {
                next.set_owner(Some(user.to_string()));
            }
        } else if open {
            let who = if has("ai") { "ai".to_string() } else { item.by().cloned().unwrap_or_else(|| user.to_string()) };
            if item.assignee().is_none() {
                what.push(format!("assignee {}", who));
                next.set_assignee(Some(who));
            }
            if item.owner().is_none() {
                what.push(format!("owner {}", user));
                next.set_owner(Some(user.to_string()));
            }
        } else if item.status().is_none() {
            let unapproved_plan = !mine && item.content().contains("done when") && !has("doc");
            let old_inbox = mine && item.parents().is_empty() && !has("triaged") && item.children().is_empty();
            if unapproved_plan || old_inbox {
                what.push(if old_inbox { "inbox (not triaged yet)".into() } else { "inbox (plan not approved yet)".into() });
                next.set_status(Status::Inbox).ok();
            }
        }
        // tokens per model (the old `usage` key) become the free fields an
        // adapter writes: tokens_in, tokens_out, tokens_cache_read, tokens_cache_write
        if let Some(serde_yaml_ng::Value::Mapping(models)) = item.field("usage") {
            let mut sums = [0i64; 4];
            for (_, u) in models {
                for (n, k) in ["in", "out", "cache_read", "cache_write"].iter().enumerate() {
                    sums[n] += u.get(*k).and_then(|v| v.as_i64()).unwrap_or(0);
                }
            }
            let keys = ["tokens_in", "tokens_out", "tokens_cache_read", "tokens_cache_write"];
            what.push(format!("usage → {}", keys.iter().zip(sums).map(|(k, v)| format!("{} {}", k, v)).collect::<Vec<_>>().join(", ")));
            for (k, v) in keys.iter().zip(sums) {
                if v > 0 {
                    next.add_field(k, serde_yaml_ng::Value::Number(v.into()), false).ok();
                }
            }
            next.unset_field("usage").ok();
        }
        let tags: Vec<String> = item.tags().iter().filter(|t| OLD_TAGS.contains(&t.as_str())).cloned().collect();
        if !tags.is_empty() {
            what.push(format!("tags {} removed", tags.join(", ")));
            next.remove_tags(tags);
        }
        if what.is_empty() {
            continue;
        }
        if apply {
            store.get_item_mut(id).unwrap().set(next);
        }
        changes.push(Change { id: id.clone(), what });
    }
    for id in &removed {
        store.remove(id)?;
    }
    let out: Vec<serde_json::Value> =
        changes.iter().map(|c| serde_json::json!({ "id": c.id, "changes": c.what })).collect();
    if !crate::out::json() {
        for c in &changes {
            println!("{}: {}", c.id, c.what.join("; "));
        }
        match (changes.is_empty(), apply) {
            (true, _) => println!("Nothing to migrate."),
            (false, true) => println!("{} items migrated.", changes.len()),
            (false, false) => println!("{} items would change, run cake migrate --apply.", changes.len()),
        }
    }
    Ok(Report::Value(serde_json::json!({ "changes": out, "applied": apply })))
}
