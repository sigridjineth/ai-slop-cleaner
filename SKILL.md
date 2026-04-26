---
title: AI Slop Cleaner
description: Score prose for AI-slop signals using a Rust CLI (regex fast-path) plus agent-readable pattern definitions for multilingual LLM judgment.
category: writing
tags: [ai, writing, rust, cli, quality, multilingual]
---

# AI Slop Cleaner

Two detection modes work together: a compiled Rust binary gives fast regex-based scoring, while a plain-markdown pattern catalog lets any LLM agent judge slop across languages without touching regex.

Repo: `~/.hermes/skills/ai-slop-cleaner` (also at `sigridjineth/ai-slop-cleaner` on GitHub).

## When to Use

- Scoring a draft before publishing (CLI or CI).
- Feeding analysis results into an agent rewrite loop (OMX, Claude, Codex).
- Adding slop detection to any LLM pipeline that reads markdown.

## Quick Start

```bash
cd ~/.hermes/skills/ai-slop-cleaner/rust
cargo build --release

# score a file
./target/release/ai-slop-cleaner score draft.md --rules-dir rules/

# pipe from stdin
cat draft.md | ./target/release/ai-slop-cleaner stdin --rules-dir rules/

# list loaded rules
./target/release/ai-slop-cleaner rules --rules-dir rules/
```

A pre-built aarch64 Linux binary lives in `releases/`.

## Two Detection Modes

### Mode 1 — Regex (Rust binary)

The binary loads two markdown tables at runtime:

- `rules/banned-patterns.md` — 70 structural regex patterns with `Lang Scope` column (`universal`, `english`, `korean`). Universal patterns (bold, em dash, bullet block, emoji, colon heading, buzzwords) apply to all languages. Regex cells are backtick-wrapped so `|` inside alternation groups parses correctly.
- `rules/banned-words.md` — 93 banned words/phrases with suggested replacements.

Language filtering via `--lang auto|en|ko|all`:
- `auto`: detects language from Hangul ratio (≥5% = Korean)
- `en`: applies `english` + `universal` patterns, skips `korean`
- `ko`: applies `korean` + `universal` patterns, skips `english`
- `all`: applies all patterns

Output: a 0–100 score (sum of weighted matches, capped). Formats: `text`, `json`, `markdown`.

### Mode 2 — Agent-Readable Patterns (multilingual)

`rules/patterns-agent.md` (649 lines) describes every pattern in plain prose with severity, weight, examples, and multilingual notes. No regex knowledge needed.

An LLM agent reads this file, then judges whether a given text matches each pattern by intent — not by string match. This covers languages and nuances that regex cannot reach (Japanese, Chinese, mixed-code prose, cultural idioms).

The agent prompt template lives in `references/agent-prompt-template.md`. It embeds the banned-words list, the pattern catalog, and a JSON response schema with five scoring components:

```
Score = round(100 * (0.25*BWD + 0.25*SPV + 0.20*RHY + 0.15*META + 0.15*MD))
```

BWD = banned word density, SPV = structural pattern violations, RHY = rhythm monotony, META = meta commentary, MD = markdown overuse.

## Architecture

Rust source is in `rust/src/`:

- `pattern_loader.rs` — parses markdown tables with backtick-aware column splitting.
- `scorer.rs` — matches text line-by-line against loaded rules, accumulates weighted score.
- `main.rs` — CLI with `score`, `stdin`, `rules` subcommands.

## Adding or Editing Rules

Edit the markdown files in `rules/`. The binary reloads them on every run. For agent-mode patterns, add a new `###` section to `patterns-agent.md` with severity, weight, description, examples, and a multilingual note.

No recompilation needed for rule changes.

## OMX Delegation

`scripts/omx-delegate.sh` orchestrates iterative cleanup:

```bash
./scripts/omx-delegate.sh <input-file> [output-dir]
```

It runs the Rust binary, generates context for Codex, and delegates to `$team` / `$ralph` / `$ultrawork` roles in a single `omx exec` call. The loop stops when the score drops below 15 or after 3 rounds.

## Pitfalls

- The prompt document itself scores 100/100 because it lists banned words as examples. Self-referential documents are expected to score high.
- Regex cells in `banned-patterns.md` must stay inside backticks. Removing backticks breaks alternation parsing.
- `banned_words.json` is a legacy artifact from the Go/Python era; the Rust binary reads `banned-words.md` instead.
