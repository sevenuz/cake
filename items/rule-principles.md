---
id: rule-principles
tags: [rule]
created: 2026-11-20T23:09:54+01:00
by: claude-code
sessions: [1ed9be18-ebbd-4e14-8abc-cf08798df82d]
---
# Principles: no AI inside cake, no harness code in the core

Context: holds for every item in this store; the same principles open AGENTS.md in the repo.
Rule: cake never calls an AI API and must be fully useful without AI; agents drive it from outside through the CLI and the skill.
Rule: no agent- or harness-specific code in the core: agent data are free header fields, harness adapters are scripts next to the skill, stats for them are config.
Rule: plans, designs and findings go into this store, never as files in the repo.
