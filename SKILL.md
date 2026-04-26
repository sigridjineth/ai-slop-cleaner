---
title: AI Slop Cleaner
description: Detect and score AI-generated slop patterns in text documents using a Rust CLI tool with dynamically loaded rules.
category: writing
tags: [ai, writing, rust, cli, quality]
---

# AI Slop Cleaner

A Rust-based CLI tool that detects and scores AI-generated "slop" patterns in text documents. It uses dynamically loaded rules from markdown tables, allowing the agent to judge patterns without hardcoding them into the binary.

## Quick Start

```bash
# Build the Rust binary
cd rust && cargo build --release

# Score a file
./target/release/ai-slop-cleaner score my-article.md --rules-dir rules/

# Score from stdin
cat my-article.md | ./target/release/ai-slop-cleaner stdin --rules-dir rules/

# List loaded rules
./target/release/ai-slop-cleaner rules --rules-dir rules/
```

## Architecture

The tool consists of three main components:

1. **Pattern Loader** (`src/pattern_loader.rs`): Loads banned patterns and words from markdown tables at runtime. Uses backtick-aware parsing so `|` inside regex alternation is not treated as a column delimiter.
2. **Scorer** (`src/scorer.rs`): Matches text against loaded rules and calculates an overall slop score. Skips malformed rules with a warning instead of crashing.
3. **CLI** (`src/main.rs`): Provides commands for scoring files, stdin, and listing rules.

## Rule Files

Rules are defined in two markdown files:

### `rules/banned-patterns.md`

Contains structural patterns (regex) that indicate AI slop:

| Name | Severity | Weight | Regex | Description |
|------|----------|--------|-------|-------------|
| redefinition | medium | 2.0 | `(?i)(?:\bnot\s+\w+(?:\s+\w+){0,2}\b\|...` | A is not X, it is Y |
| closing_summary | low | 1.0 | `(?i)\b(to summarize\|in summary\|...)\b` | Closing-summary repetition |

**Important:** Regex cells are wrapped in backticks (`` `...` ``). The parser is **backtick-aware** — it ignores `|` characters inside backticks, so regex alternation like `delve\|elucidate\|underscore` works correctly. Do not remove the backtick wrappers.

### `rules/banned-words.md`

Contains individual words and phrases to flag:

| Word | Replacement | Weight |
|------|-------------|--------|
| delve into | explore | 1.0 |
| furthermore | | 2.0 |

## Adding New Rules

To add a new pattern or word, simply edit the corresponding markdown file. The binary will load it dynamically on the next run — no recompilation needed.

## Scoring

The overall score is a sum of weights from all matched patterns and words, capped at 100. Severity levels (high/medium/low) help prioritize which issues to fix first.

## Output Formats

- `text` (default): Human-readable summary
- `json`: Machine-parseable output
- `markdown`: Report suitable for pasting into issues/PRs

## Key Patterns Detected

- **Redefinition**: "A is not X, it is Y" / "A가 아니라 B다"
- **Closing summaries**: "In conclusion", "To summarize"
- **Progress announcements**: "Let's dive in", "Let's explore"
- **Pre-classification**: "There are three types..."
- **Mechanical enumeration**: "First... Second... Third..."
- **Bullet block abuse**: Excessive bullet lists
- **Generic headings**: "Introduction", "Conclusion"
- **Topic sentence formulas**: "The important thing is..."
- **Emoji decoration**: 🚀, 💡, ✅ in prose
- **Korean translationese**: ~에 대해, ~을 통해, ~에 있어서
- **Hedging**: "It seems that...", "One could argue..."
- **Buzzwords**: leverage, harness, delve, pivotal, etc.

## Oh-My-Codex (OMX) Delegation

For automated cleanup, use `scripts/omx-delegate.sh`:

```bash
./scripts/omx-delegate.sh <input-file> [output-dir]
```

This script:
1. Runs the Rust binary to analyze the text
2. Generates an `omx-context.md` for Codex
3. Delegates to oh-my-codex with three roles (`$team`, `$ralph`, `$ultrawork`)
4. Iteratively cleans the text until the score drops below 15.0 (max 3 rounds)

**Note:** If the global `codex` CLI is outdated and can't use newer models like `gpt-5.5`, the script falls back to `npx @openai/codex@latest exec` which always pulls the latest version.

## Troubleshooting

### Regex compilation errors
If you see `RegexError` warnings, check that:
- Regex cells in `banned-patterns.md` are wrapped in backticks
- `\|` is used for alternation (not bare `|` which would break the markdown table)
- Unicode ranges like `\uac00-\ud7a3` are properly escaped

### OMX delegation fails
If `codex exec` fails with "model requires a newer version":
- The script auto-detects this and uses `npx @openai/codex@latest` instead
- Alternatively, update the global package: `npm install -g @openai/codex@latest`
