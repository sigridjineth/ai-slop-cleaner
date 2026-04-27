---
name: ai-slop-cleaner
title: AI Slop Cleaner
description: Analyze prose for structural AI-slop evidence with a Rust CLI, then use universal LLM pattern categories for language-agnostic judgment and full-text rewrites.
category: writing
tags: [ai, writing, rust, cli, quality, multilingual]
---

# AI Slop Cleaner

AI Slop Cleaner separates detection evidence from rewriting. The Rust binary
reports raw structural matches. LLM agents read those matches, the full text, and
universal pattern categories, then judge slop by intent in any language.

Never clean prose with regex substitutions. Rewrites must be full-text LLM
inference that preserves meaning and grammar.

Repo: `~/.hermes/skills/ai-slop-cleaner` (also at `sigridjineth/ai-slop-cleaner` on GitHub).

## When to use

Use this skill to analyze a draft before publishing, feed structural evidence
into an agent rewrite loop, or add language-agnostic slop detection to a prose
pipeline.

## Quick start

```bash
cd ~/.hermes/skills/ai-slop-cleaner/rust
cargo build --release

# Analyze a file for raw structural matches.
./target/release/ai-slop-cleaner --rules-dir rules analyze ../README.md

# Pipe from stdin.
cat draft.md | ./target/release/ai-slop-cleaner --rules-dir rules stdin --format json

# List structural and agent rules.
./target/release/ai-slop-cleaner --rules-dir rules rules
```

Pre-built binaries are published through GitHub Releases for Linux and macOS
(x86_64 and aarch64). The installer downloads the matching asset when available
and falls back to building from source.

## Mode 1. Rust structural analysis

The binary loads universal structural patterns from `rust/rules/banned-patterns.md`.
Examples include bold emphasis, em dash decoration, bullet blocks, emoji
decoration, colon headings, plus conjunctions, and example-list-summary cadence.

`scorer.rs` returns raw `MatchResult` values. It does not compute an overall
score, match banned words, or make semantic judgments. The `--lang` flag remains
for compatibility but does not filter patterns; all Rust structural patterns are
universal.

Use `analyze` for new workflows. The old `score` subcommand exists only for
compatibility and prints a warning.

## Mode 2. Universal LLM categories

`rust/rules/patterns-agent.md` defines universal categories, not language-specific
pattern IDs. The current categories are:

- Translationese
- Structural Monotony
- Cliche/Formulaic
- Rhythm
- Modifier Abuse
- Hedging
- Meta-Commentary
- Connector Abuse
- Dependency Clause Overuse

Each category has an intent description, severity, weight, and multilingual
examples. Examples are illustrations only. The LLM must judge whether the text
exhibits the category intent in its own language and context.

The prompt template in `references/agent-prompt-template.md` keeps the
five-component formula for LLM calibration:

```text
AI_SLOP_SCORE = round(100 * (
    0.25 * BWD +
    0.25 * SPV +
    0.20 * RHY +
    0.15 * META +
    0.15 * MD
))
```

The Rust binary does not compute this score.

## Iterative cleanup with Ralph

```bash
python3 scripts/ralph.py <input-file> --max-rounds 3 --target-matches 0
python3 scripts/ralph.py <input-file> --force-rewrite  # semantic-only slop
```

Each round:

1. Runs `ai-slop-cleaner analyze` on the current full text.
2. Loads `patterns-agent.md` universal categories.
3. Prompts an LLM with the full text, all structural matches, and the categories.
4. Requires a complete rewritten text, not patches or replacement rules.
5. Re-runs `analyze` on the rewritten text.
6. Stops when structural matches are at or below `--target-matches`, or when the
   round limit is reached. With `--force-rewrite`, Ralph runs at least one LLM
   rewrite even when structural analysis finds zero matches. Use this for
   semantic-only slop such as translationese or connector abuse.

Backend priority is `claude -p`, `AICHAT_MODEL`, `OPENAI_API_KEY`, then
`ANTHROPIC_API_KEY`. The `claude -p` path reuses Claude Code authentication, so
it works inside the skill without separate API keys. Without a configured
backend, Ralph writes `.ralph/round-N-prompt.md` and waits for a complete rewrite
in `.ralph/round-N-response.md`.

## OMX delegation

```bash
./scripts/omx-delegate.sh <input-file> [output-dir]
```

The script runs `analyze`, writes context with raw match JSON and the universal
category catalog, then asks OMX to produce a complete rewritten file. It does not
ask for a numeric score and does not request regex substitutions.

## Safety rules

- Do not add regex-based rewrite rules.
- Do not add language-specific semantic pattern IDs.
- Keep Rust structural patterns language-agnostic.
- Treat examples as illustrations, not trigger lists.
- Preserve meaning, names, facts, code, quotations, and necessary formatting.
- Never cut connector-looking syllables out of compound words.
- For Korean specifically, `즉흥적으로` and `즉흥성` are complete words, not the
  connector `즉`; `느낌입니다` must not be chopped into `다`.

## Development checks

```bash
cd rust
cargo test
cargo build --release
./target/release/ai-slop-cleaner -r rules analyze ../README.md
```

From the repository root:

```bash
python3 -m py_compile scripts/ralph.py
bash -n scripts/omx-delegate.sh
./rust/target/release/ai-slop-cleaner -r rust/rules analyze README.md
```
