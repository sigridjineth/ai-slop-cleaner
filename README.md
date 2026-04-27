<p align="right">
  <strong>English</strong> | <a href="./README.ko.md">한국어</a>
</p>

<p align="center">
  <br/>
  ◯ ───────────── ◯
  <br/><br/>
  <strong>A I   S L O P   C L E A N E R</strong>
  <br/><br/>
  ◯ ───────────── ◯
  <br/>
</p>

<p align="center">
  <strong>Detect slop. Ship clean prose.</strong>
  <br/>
  <sub>A Rust CLI and an agent-readable pattern catalog for multilingual AI-slop detection</sub>
</p>

<p align="center">
  <a href="https://github.com/sigridjineth/ai-slop-cleaner/actions"><img src="https://img.shields.io/badge/build-rust-orange" alt="Build"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="License"></a>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> ·
  <a href="#why-ai-slop-cleaner">Why</a> ·
  <a href="#what-you-get">Results</a> ·
  <a href="#the-loop">How It Works</a> ·
  <a href="#commands">Commands</a> ·
  <a href="#architecture">Architecture</a>
</p>

Score prose for AI-generated signals in any language. Regex catches the structure, and an LLM agent judges the intent.

AI Slop Cleaner is a dual-mode detection engine. A compiled Rust binary gives you fast, deterministic scoring for universal structural patterns (emoji floods, bullet blocks, bold abuse, em dashes). For semantic and language-specific signals (translationese, hedging, hype vocabulary, rhythm monotony), a plain-markdown pattern catalog lets any LLM agent judge prose without touching regex. No hardcoded language lists. No API keys required for scoring.

---

## Why AI Slop Cleaner?

AI writing fails at the surface, not the facts. The reader trusts the content less before they finish the first paragraph.

| Problem | What Happens | AI Slop Cleaner Fix |
| :------ | :----------- | :------------------ |
| Emoji and bullet blocks | Reader scans, does not read | Universal regex catches formatting slop instantly |
| Translationese | 에 대해, 를 통해, "based on" | 70 agent-readable patterns flag intent, not just keywords |
| "Looks fine to me" | No objective quality gate | 0–100 score with explicit thresholds |

---

## Quick Start

Install with one command, everything auto-built:

```bash
curl -fsSL https://raw.githubusercontent.com/sigridjineth/ai-slop-cleaner/main/scripts/install.sh | bash
```

Score by passing any markdown or plain text:

```bash
ai-slop-cleaner score draft.md
```

> The installer clones the repository, compiles the Rust binary, and drops a wrapper in `~/.local/bin` that auto-points to the built-in rules directory.

<details>
<summary>Other install methods</summary>

|cargo install (requires Rust >= 1.70):
```bash
cargo install --git https://github.com/sigridjineth/ai-slop-cleaner.git
# Running with explicit rules directory
ai-slop-cleaner --rules-dir ~/.cargo/git/checkouts/ai-slop-cleaner-*/rust/rules score draft.md
```

|Build from clone:
```bash
git clone https://github.com/sigridjineth/ai-slop-cleaner.git
cd ai-slop-cleaner/rust
cargo build --release
./target/release/ai-slop-cleaner --rules-dir ../rules score draft.md
```

</details>

<details>
<summary>Uninstall</summary>

```bash
rm -rf ~/.local/share/ai-slop-cleaner
rm ~/.local/bin/ai-slop-cleaner
```

</details>

---

## What You Get

After one command, vague suspicion becomes a measured verdict:

| Step | Before | After |
| :--- | :----- | :---- |
| Score | "This feels AI-generated" | Concrete 0–100 score with weighted match list |
| Diagnose | Manual guessing | Line-by-line pattern and word match report |
| Clean | Rewrite blindly | OMX agent loop targets highest-weight violations first |

<details>
<summary><strong>What just happened?</strong></summary>

```
score    ->  Regex scanned for 6 universal structural patterns
              LLM agent judged 70 semantic patterns across 10 categories
              Overall score: 42/100 (revise flagged sections)

rules    ->  93 banned words/phrases with suggested replacements
              6 regex patterns (universal scope)
              70 agent patterns (multilingual scope)

clean    ->  ralph.py runs iterative cleanup until score < 15
```

</details>

---

## How It Compares

| | Manual Review | Vanilla LLM Rewrite | AI Slop Cleaner |
| :--- | :------------ | :------------------ | :---------------- |
| Speed | Slow; read every line | Fast, but hidden slop remains | Fast regex alongside deep agent scan |
| Objectivity | "Looks fine to me" | No score, no threshold | 0–100 score with explicit gates |
| Language | Native speaker required | Same training bias as the slop | Universal regex for any-language agent |
| Actionable | Vague feedback | Full rewrite, no priorities | Ranked match list: fix top 5 first |

---

## The Loop

AI Slop Cleaner does not just score. It guides revision. Two modes, one pipeline:

```
    Mode 1 (Regex)  ->  Mode 2 (Agent)  ->  Iterate
         |                   |                |
    6 patterns         70 patterns       OMX ralph loop
    0.1 ms             1–2 s             Until score < 15
```

Each pass does not repeat. It evolves. The agent reads the pattern catalog, judges the text by intent, and targets the highest-weight violations first.

| Phase | What Happens |
| :---- | :----------- |
| Regex | Fast universal scan: emoji, bullets, bold, em dashes, colon headings, `+` conjunctions |
| Agent | LLM reads `patterns-agent.md` and judges semantic signals in any language |
| Iterate | `ralph.py` runs the loop: score -> plan -> rewrite -> rescore, max 3 rounds |

### Ralph. The Loop That Cleans

`python3 scripts/ralph.py <file>` runs the evolutionary loop persistently across violation categories until the score drops below 15 or 3 rounds complete. Each round is stateless: the JSON score report reconstructs the full violation lineage, so even if you restart, the loop picks up where it left off.

```
Ralph Cycle 1: score -> plan -> rewrite -> score 42 -> action=CONTINUE
Ralph Cycle 2: score -> plan -> rewrite -> score 28 -> action=CONTINUE
Ralph Cycle 3: score -> plan -> rewrite -> score 12 -> action=STOP
                                                +-- Ralph stops.
                                                    The prose is clean.
```

---

## Score Interpretation

| Range | Meaning | Action |
| :---- | :------ | :----- |
| 0–15 | Clean | Ship it |
| 15–30 | Light signals | Spot-check flagged lines |
| 30–60 | Noticeable slop | Revise flagged sections |
| 60–100 | Heavy slop | Rewrite or run agent cleanup |

---

## Commands

| Command | What It Does |
| :------ | :----------- |
| `ai-slop-cleaner score <file>` | Score a file (text, JSON, or markdown output) |
| `ai-slop-cleaner stdin` | Score from stdin (pipe-friendly) |
| `ai-slop-cleaner rules` | List loaded rules and patterns |
| `ai-slop-cleaner score <file> --format json` | Machine-readable JSON report |
| `ai-slop-cleaner score <file> --format markdown` | Human-readable markdown report |
| `ai-slop-cleaner -l en score <file>` | Force English word filtering |
| `ai-slop-cleaner -l ko score <file>` | Force Korean word filtering |

> The wrapper script installed by the curl installer automatically points `--rules-dir` to the built-in rules. When running the raw binary, pass `--rules-dir rust/rules/` explicitly.

---

## Rules

All rules live in `rust/rules/` as markdown files. The binary reloads them on every run, so no recompilation is needed.

| File | Purpose |
| :--- | :------ |
| `banned-patterns.md` | 6 universal regex patterns (bullet block, emoji, colon heading, bold, em dash, plus conjunction) |
| `banned-words.md` | 93 banned words and phrases with replacements (English buzzwords, English phrases, Korean buzzwords) |
| `patterns-agent.md` | 70 patterns in plain prose for LLM agent evaluation; covers 10 categories across any language |

### Pattern categories (agent mode)

| Category | Signals |
| :------- | :------ |
| A | Translationese (에 대해, 를 통해, based on, by-passive) |
| B | Untranslated terms, parenthesized English, long quotes |
| C | Structural: mechanical enumeration, bullet blocks, emoji, colon headings |
| D | Cliché: conclusion phrases, importance inflation, hype vocabulary |
| E | Rhythm: uniform sentence length, repeated endings, uniform paragraphs |
| F | Modifier abuse: degree adverbs, double modifiers, suffix chains |
| G | Hedging: triple hedges, possibility stacking |
| H | Connectors: sentence-initial transitions, meta-entries |
| I | Korean-specific: 것이다 endings, dependent noun crutches, need-to formulas |
| J | Formatting: bold overuse, quote emphasis, em dashes, parenthetical asides |

### Adding rules

For universal regex patterns, add a row to `banned-patterns.md`:

```markdown
| my_pattern | universal | medium | 1.5 | `regex here` | Description. |
```

For language-specific or semantic patterns, add a `###` section to `patterns-agent.md`:

| Field | Example value |
| :---- | :------------ |
| Heading | `### my_pattern` |
| Severity | `medium` |
| Lang Scope | `english` |
| Weight | `1.5` |
| Description | What the pattern detects and why it matters. |
| Examples | "Example sentence that matches." |
| Multilingual note | How this manifests in other languages. |

---

## Architecture

<details>
<summary>Architecture overview (Rust)</summary>

```
rust/src/
├── main.rs           # CLI entrypoint (score, stdin, rules subcommands)
├── scorer.rs         # Regex matching, language filtering, weighted scoring
└── pattern_loader.rs # Markdown table parser (backtick-aware column splitting)

rust/rules/
├── banned-patterns.md   # 6 universal regex patterns
├── banned-words.md      # 93 banned words/phrases with replacements
└── patterns-agent.md    # 70 agent-readable patterns (718 lines)

references/
├── agent-prompt-template.md  # Prompt template for LLM subagents
├── agent-response-schema.json
└── ...

scripts/
└── ralph.py                  # Standalone iterative cleanup script
```

|Key internals:

| Component | What It Does |
| :-------- | :----------- |
| Language filter | `auto` detects Korean from Hangul ratio (≥5%); `en` applies English and universal; `ko` applies Korean and universal; `all` applies everything. |
| Scoring | Sum of weighted matches, capped at 100. Formats: `text`, `json`, `markdown`. |
| Agent mode | `patterns-agent.md` is pure prose. Any LLM reads it, then judges text by intent. No regex knowledge needed. |
| Loop script | `scripts/ralph.py` runs the Rust binary, generates prompts, and calls an LLM API or writes interactive prompts. No OMX needed. |

</details>

---

## Agent Mode

The agent prompt template (`references/agent-prompt-template.md`) embeds the pattern catalog and a JSON response schema with five scoring components:

```
Score = round(100 × (0.25×BWD + 0.25×SPV + 0.20×RHY + 0.15×META + 0.15×MD))
```

| Component | Weight | Meaning |
| :-------- | -----: | :------ |
| BWD | 25% | Banned word/phrase density |
| SPV | 25% | Structural pattern violations |
| RHY | 20% | Rhythm monotony |
| META | 15% | Meta commentary density |
| MD | 15% | Markdown overuse |

---

## MCP Server

The `.mcp.json` registers a stdio MCP server for Claude / Codex integration:

```json
{
  "mcpServers": {
    "ai-slop-cleaner": {
      "command": "uvx",
      "args": ["--from", "ai-slop-cleaner", "ai-slop-cleaner", "mcp", "serve"]
    }
  }
}
```

---

## Contributing

```bash
git clone https://github.com/sigridjineth/ai-slop-cleaner
cd ai-slop-cleaner/rust
cargo test
cargo build --release
```

[Issues](https://github.com/sigridjineth/ai-slop-cleaner/issues) · [License](./LICENSE)

---

<p align="center">
  <em>"The machine writes. The machine detects. The human decides."</em>
  <br/><br/>
  <strong>Score before you ship.</strong>
  <br/><br/>
  <code>MIT License</code>
</p>
