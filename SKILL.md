---
name: ai-slop-cleaner
title: AI Slop Cleaner
description: Score prose for AI-slop signals using a Rust CLI (regex fast-path) plus agent-readable pattern definitions for multilingual LLM judgment.
category: writing
tags: [ai, writing, rust, cli, quality, multilingual]
---

# AI Slop Cleaner

Two detection modes work together: a compiled Rust binary gives fast regex-based scoring, while a plain-markdown pattern catalog lets any LLM agent judge slop across languages without touching regex.

Repo: `~/.hermes/skills/ai-slop-cleaner` (also at `sigridjineth/ai-slop-cleaner` on GitHub).

## When to Use

Use this skill to score a draft before publishing, feed analysis results into an
agent rewrite loop (OMX, Claude, Codex), or add slop detection to any LLM
pipeline that reads markdown.

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

### Mode 1 Regex (Rust binary)

The binary loads two markdown tables at runtime. `rules/banned-patterns.md`
contains 70 structural regex patterns with a `Lang Scope` column
(`universal`, `english`, `korean`). Universal patterns such as bold, em dash,
bullet block, emoji, colon heading, and buzzword checks apply to all languages.
Regex cells are backtick-wrapped so `|` inside alternation groups parses
correctly. `rules/banned-words.md` contains 93 banned words or phrases with
suggested replacements.

Language filtering via `--lang auto|en|ko|all`:

| Mode | Behavior |
|------|----------|
| `auto` | Detects language from Hangul ratio (≥5% = Korean). |
| `en` | Applies `english` and `universal` patterns; skips `korean`. |
| `ko` | Applies `korean` and `universal` patterns; skips `english`. |
| `all` | Applies all patterns. |

Output: a 0–100 score (sum of weighted matches, capped). Formats: `text`, `json`, `markdown`.

### Mode 2 Agent-Readable Patterns (multilingual)

`rules/patterns-agent.md` (649 lines) describes every pattern in plain prose with severity, weight, examples, and multilingual notes. No regex knowledge needed.

An LLM agent reads this file, then judges whether a given text matches each pattern by intent rather than by string match. This covers languages and nuances that regex cannot reach (Japanese, Chinese, mixed-code prose, cultural idioms).

The agent prompt template lives in `references/agent-prompt-template.md`. It embeds the banned-words list, the pattern catalog, and a JSON response schema with five scoring components:

```
Score = round(100 * (0.25*BWD + 0.25*SPV + 0.20*RHY + 0.15*META + 0.15*MD))
```

BWD = banned word density, SPV = structural pattern violations, RHY = rhythm monotony, META = meta commentary, MD = markdown overuse.

## Architecture

Rust source is in `rust/src/`. `pattern_loader.rs` parses markdown tables with
backtick-aware column splitting, `scorer.rs` matches text line by line against
loaded rules and accumulates weighted score, and `main.rs` provides the
`score`, `stdin`, and `rules` CLI subcommands.

## Adding or Editing Rules

Edit the markdown files in `rules/`. The binary reloads them on every run. For agent-mode patterns, add a new `###` section to `patterns-agent.md` with severity, weight, description, examples, and a multilingual note.

No recompilation needed for rule changes.

## Meta: Auditing Your Own README

A project that detects AI slop should not ship a slop-heavy README. Use the binary on itself:

```bash
cd ~/.hermes/skills/ai-slop-cleaner/rust
./target/release/ai-slop-cleaner --rules-dir rules/ score ../../README.md --format json
```

### Scoring anatomy (what actually drives the number)

Understanding the weight of each pattern is the fastest way to lower a score:

| Pattern | Weight each | Typical README hit | Point impact |
|---------|------------|--------------------|--------------|
| `em_dash` | 0.5 | 10–15 dashes | 5.0–7.5 |
| `bold_emphasis` | 0.5 | 8–12 bold spans | 4.0–6.0 |
| `colon_heading` | 2.0 | 1–2 headings | 2.0–4.0 |
| Banned words | 1.0 | 2–5 words | 2.0–5.0 |
| `bullet_block` | 1.0 | 1 block | 1.0 |

**Em dashes are the single biggest contributor** in most markdown READMEs. If you need a score < 5, remove every `—` and replace with periods, commas, semicolons, or parentheses. Bold inside table cells and headings is also counted; strip `**` from sentences where structure already provides hierarchy.

### Checklist for score < 5

1. **Zero em dashes** — replace all `—` with `.`, `;`, `,`, or `()`.
2. **Minimal bold** — remove `**` from sentences; keep only in nav pills or badges if necessary.
3. **No colon headings** — avoid `### Title: Subtitle`; use `### Title. Subtitle` or `### Title / Subtitle`.
4. **Avoid banned words** — check `banned-words.md` for words like *harness*, *leverage*, *delve*; use plain alternatives.
5. **Keep tables and code blocks** — these do not trigger structural patterns unless they contain the above.

### Style trade-off note

The ouroboros reference README (Q00/ouroboros) scores ~41/100 because em dashes and bold emphasis are intentional stylistic choices. If a user asks to "follow ouroboros style" **and** "score < 5", you must choose: ouroboros style requires em dashes and bold, which guarantees a score > 15. Prioritize the numerical target over stylistic homage when both are requested.

Target for technical docs: **≤ 30**. Target for pure prose: **< 15**. Target when the user explicitly demands a low-slop README: **< 5** (requires the checklist above).

## One-Command Installer (Rust projects)

A `scripts/install.sh` that clones, builds, and wraps the binary is the Rust equivalent of `pip install`. Pattern:

```bash
#!/bin/bash
set -euo pipefail
REPO_URL="https://github.com/sigridjineth/ai-slop-cleaner"
INSTALL_DIR="${HOME}/.local/share/ai-slop-cleaner"
BIN_DIR="${HOME}/.local/bin"

# 1. Ensure cargo
if ! command -v cargo &>/dev/null; then
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "${HOME}/.cargo/env"
fi

# 2. Clone or update
if [ -d "${INSTALL_DIR}/.git" ]; then
  git -C "${INSTALL_DIR}" pull --quiet
else
  git clone --depth 1 --quiet "${REPO_URL}" "${INSTALL_DIR}"
fi

# 3. Build
BINARY="${INSTALL_DIR}/rust/target/release/ai-slop-cleaner"
if [ ! -x "${BINARY}" ]; then
  (cd "${INSTALL_DIR}/rust" && cargo build --release)
fi

# 4. Wrapper with default rules-dir
mkdir -p "${BIN_DIR}"
cat > "${BIN_DIR}/ai-slop-cleaner" <<EOF
#!/bin/bash
exec "${BINARY}" --rules-dir "${INSTALL_DIR}/rust/rules" "\$@"
EOF
chmod +x "${BIN_DIR}/ai-slop-cleaner"
```

Usage for end users: `curl -fsSL .../install.sh | bash`

## Iterative Cleanup (Ralph)

Two options: OMX delegation or standalone harness.

### OMX Delegation

`scripts/omx-delegate.sh` orchestrates iterative cleanup:

```bash
./scripts/omx-delegate.sh <input-file> [output-dir]
```

It runs the Rust binary, generates context for Codex, and delegates to `$team` / `$ralph` / `$ultrawork` roles in a single `omx exec` call. The loop stops when the score drops below 15 or after 3 rounds.

### Standalone Ralph Harness (no OMX)

`scripts/ralph.py` performs the same evolutionary loop without OMX. It uses only Python 3 + `urllib` (no external packages).

```bash
python3 scripts/ralph.py <input-file> [--max-rounds 3] [--target-score 15]
```

Behavior:
- Detects `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `AICHAT_MODEL` and calls the corresponding LLM backend automatically.
- If no API key is found, it writes prompts to `.ralph/round-N-prompt.md` and waits for you to paste the LLM response into `.ralph/round-N-response.md` before continuing.
- After each round it re-runs the Rust binary and stops when the target score is reached or max rounds are exhausted.

To make all agent patterns language-agnostic:

```bash
sed -i 's/- \*\*Lang Scope:\*\* .*/- **Lang Scope:** universal/' rust/rules/patterns-agent.md
```

## Pitfalls

The prompt document itself scores 100/100 because it lists banned words as
examples; self-referential documents are expected to score high. Regex cells in
`banned-patterns.md` must stay inside backticks because removing backticks breaks
alternation parsing. `banned_words.json` is a legacy artifact from the Go/Python
era; the Rust binary reads `banned-words.md` instead.

Do not chase a score of 0 on technical READMEs. Tables, navigation links, and
styled headers are structural necessities. The ouroboros reference README scores
~41/100. A well-structured technical doc in the 12–30 range is clean enough to
ship. Pure prose (essays, emails, posts) should aim for < 15.

When adding new agent-mode patterns, watch for the `example_list_summarize` trap:
"For example:" + bullet list + "That is," / "즉," is itself a detectable slop
cadence. Describe the pattern in plain prose without demonstrating it.
