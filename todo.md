● Git Workflow Analysis

  - Stash → checkout cake branch → work → checkout back → stash pop, used in 3 places (init, pre-command, post-command in lib.rs/commands.rs), none guarded by cleanup-on-error (Rust has no try/finally; needs an
    RAII Drop guard).
  - Any error between stash and restore (bad flag, write failure, missing upstream) strands the user on the cake branch with real work stashed — main.rs just prints and exits.
  - git stash (no -u, whole tree) stashes unrelated uncommitted work in the repo, not just the todo file.
  - No git exit-code checks anywhere in git.rs — failed checkout/commit/rebase/push is silently treated as success, which can misdirect commits/pushes onto the wrong branch.
  - Branch existence checks (check_if_branch_exists, check_if_remote_branch_exists) use substring match on git branch output — false positives possible (e.g. cake matches cake-experiment).
  - stash pop result/conflicts are never inspected.
  - is_repo() only checks .git in CWD, inconsistent with how git itself resolves repos from subdirectories.
  - Fix direction: RAII cleanup guard, check .status.success() everywhere, exact branch-name matching, --porcelain status parsing.

  Obsidian Vault Format (as a third save format)

  - Additive, not a migration — same pattern as existing new_from_md/new_from_json + write_md/write_json; selector.rs/view.rs/commands.rs stay untouched since they only see Store/Item.
  - New pieces needed: new_from_vault/write_vault in store.rs, YAML frontmatter (de)serialization in item.rs (new dep, e.g. serde_yaml — only JSON/HJSON present today), [[wikilink]] rendering for parent/child ids
    so Obsidian's graph/backlinks view works natively.
  - Main wrinkle: format dispatch today is by file extension (FILETYPE_MD/FILETYPE_JSON in lib.rs); a vault is a directory, so needs a different detection signal (e.g. path is an existing dir / has no extension).
  - find_save_file (util.rs) needs a sibling that searches upward for a vault directory instead of a filename.
  - git::add currently targets a single filename; vault mode adds a directory instead (git add <dir> already works, small change).
  - Estimate: a few hours to a day, mostly new code, low risk to existing formats.

  Agent-Facing Use Case: Designs, Plans, Todos, Tickets

  - Data model already fits: id + parents/children maps to epics/subtasks, tags maps to labels, content holds freeform design/plan text, timestamp/last_modified gives natural change tracking — no new fields needed
    for basic ticketing.
  - Goal: agents write structured cake items instead of scattering plan/design docs as loose markdown in the repo; keeps the codebase clean while still git-tracked and diffable.
  - Agents can already drive this via the CLI directly (cake add/tag/list ...); an MCP server wrapping the CLI would be a nicer integration point if multiple agents/tools need structured access, but isn't required
    to start.

  GitHub Issues Sync (bidirectional)

  - Would need a new github.rs module using GitHub's REST/GraphQL Issues API, token-based auth.
  - Needs a stable cake-id ↔ GitHub-issue-number mapping — either a new Item field (e.g. external_id) or an encoded tag (e.g. gh:123); a new field is cleaner and consistent with the existing explicit-fields style.
  - Conflict detection: timestamp/last_modified already exist and are the right foundation — sync would compare cake's last_modified against GitHub's updated_at to decide which side wins, same class of problem as
    the git-branch sync conflicts already found above, so worth reusing whatever conflict-handling gets fixed there.
  - Bidirectional sync is the highest-effort item of the three — new API integration, auth/config surface, and a real conflict-resolution policy (not just "abort on conflict" like the current git workflow).

