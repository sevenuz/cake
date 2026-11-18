---
id: tokens-per-model
parent: "[[fields]]"
created: 2026-11-18T22:30:22+01:00
by: claude-code
sessions: [0282881a-f64f-4b2f-ba4d-30f5d674e657]
owner: sevenuz
---
# Tokens per model: per-family keys from the adapter

Context: since the fields switch the Claude Code adapter writes flat totals (tokens_in, tokens_out, tokens_cache_read, tokens_cache_write); the split by model and the per-model footprint factor are gone.
Why: the footprint of a small model is lower; with totals only, every model counts like the largest.
Change (idea, user 2026-11-18: note only): the adapter also writes keys per model family (tokens_out_opus, tokens_in_sonnet, …); the footprint formula weights them (missing ones are 0); migrate --only usage writes them from the old per-model data.
Not: weighted tokens in the script (factors leave the config); nested fields with path syntax (much more core code).
