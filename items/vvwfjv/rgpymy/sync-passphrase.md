---
id: sync-passphrase
parent: "[[rgpymy]]"
created: 2026-11-13T18:23:28+01:00
by: claude-code
sessions: [81954e9f-9185-4d55-a366-5581de2aacd6]
---
# Sync asks the key passphrase per SSH connection

- `cake sync` = `git fetch` + `git merge` + `git push`: fetch and push
  each open an SSH connection, each unlocks the key
- not cake specific: same for `git pull && git push`
- fix outside cake: ssh-agent (`AddKeysToAgent yes` in ~/.ssh/config),
  passphrase once per login
- fix inside cake: SSH multiplexing for its git calls
  (`ControlMaster auto`, `ControlPersist 60`): one connection, one prompt
- done when: one passphrase prompt per sync
