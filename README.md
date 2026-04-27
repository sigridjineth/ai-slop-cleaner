# AI Slop Cleaner

Detect AI-generated prose patterns ("AI slop") in any language. A Rust CLI handles fast regex scoring for universal structural signals (emoji, bold abuse, em dashes, plus-sign conjunctions). Language-specific and semantic patterns are defined in plain markdown and evaluated by an LLM agent, with no hardcoded language lists.

## Dual-mode architecture

```
┌─────────────────────────────────────────────┐
│  Mode 1: Regex (Rust binary)                │
│  Universal structural patterns only         │
│  banned-patterns.md → 6 patterns            │
│  banned-words.md → 93 word/phrase entries    │
│  Fast, deterministic, 0–100 score           │
└─────────────────────────────────────────────┘
                    ↕
┌─────────────────────────────────────────────┐
│  Mode 2: Agent (LLM reads patterns-agent.md)│
│  70 patterns in plain prose, any language    │
│  Judges intent, not string match            │
│  5-component weighted score (BWD/SPV/RHY/   │
│  META/MD)                                   │
└─────────────────────────────────────────────┘
```

Mode 1 catches language-agnostic formatting signals that regex handles well: bullet blocks, emoji decoration, colon headings, bold overuse, em dashes, and `+` conjunctions.

Mode 2 covers everything else: translationese, hedging, hype vocabulary, closing formulas, and rhythm monotony across English, Korean, Japanese, Chinese, or any language an LLM can read. The pattern catalog (`rules/patterns-agent.md`, 718 lines) describes each signal in plain prose with severity, weight, examples, and multilingual notes.

## Install

```bash
cd rust
cargo build --release
# built binary path is rust/target/release/ai-slop-cleaner
```

## Usage

Global flags (`-r`, `-l`) go before the subcommand.

```bash
# Score a file (auto-detects language for word filtering)
ai-slop-cleaner score document.md

# Force English word filtering
ai-slop-cleaner -l en score document.md

# JSON output
ai-slop-cleaner score document.md --format json

# Markdown report
ai-slop-cleaner score document.md --format markdown

# Score from stdin
cat draft.md | ai-slop-cleaner stdin

# Custom rules directory
ai-slop-cleaner -r /path/to/rules score document.md

# List loaded rules
ai-slop-cleaner rules
```

## Score interpretation

| Range | Meaning | Action |
|-------|---------|--------|
| 0–15 | Clean | Ship it |
| 15–30 | Light signals | Spot-check flagged lines |
| 30–60 | Noticeable slop | Revise flagged sections |
| 60–100 | Heavy slop | Rewrite or run agent cleanup |

## Rules

All rules live in `rust/rules/` as markdown files. The binary reloads them on every run, so no recompilation is needed.

| File | Purpose |
|------|---------|
| `banned-patterns.md` | 6 universal regex patterns (bullet block, emoji, colon heading, bold, em dash, plus conjunction) |
| `banned-words.md` | 93 banned words and phrases with replacements (English buzzwords, English phrases, Korean buzzwords) |
| `patterns-agent.md` | 70 patterns in plain prose for LLM agent evaluation; covers 10 categories across any language |

### Pattern categories (agent mode)

| Category | Signals |
|----------|---------|
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
|-------|---------------|
| Heading | `### my_pattern` |
| Severity | `medium` |
| Lang Scope | `english` |
| Weight | `1.5` |
| Description | What the pattern detects and why it matters. |
| Examples | "Example sentence that matches." |
| Multilingual note | How this manifests in other languages. |

## Agent mode

The agent prompt template (`references/agent-prompt-template.md`) embeds the pattern catalog and a JSON response schema with five scoring components:

```
Score = round(100 × (0.25×BWD + 0.25×SPV + 0.20×RHY + 0.15×META + 0.15×MD))
```

| Component | Weight | Meaning |
|-----------|-------:|---------|
| BWD | 25% | Banned word/phrase density |
| SPV | 25% | Structural pattern violations |
| RHY | 20% | Rhythm monotony |
| META | 15% | Meta commentary density |
| MD | 15% | Markdown overuse |

## MCP server

The `.mcp.json` registers a stdio MCP server for Claude/Codex integration:

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

## OMX delegation

`scripts/omx-delegate.sh` runs iterative cleanup: score → generate context → delegate to `$team`/`$ralph`/`$ultrawork` in a single `omx exec` call. The loop stops when the score drops below 15 or after 3 rounds.

## Project structure

```
ai-slop-cleaner/
├── README.md
├── rust/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # CLI (score, stdin, rules)
│   │   ├── scorer.rs         # Regex matching, language filtering, scoring
│   │   └── pattern_loader.rs # Markdown table parser (backtick-aware)
│   └── rules/
│       ├── banned-patterns.md   # Universal regex patterns
│       ├── banned-words.md      # Banned words with replacements
│       └── patterns-agent.md    # Agent-readable pattern catalog (70 patterns)
├── references/
│   ├── agent-prompt-template.md # Prompt template for LLM subagents
│   ├── agent-response-schema.json
│   ├── banned-patterns.md       # Reference copy
│   ├── banned-words.md          # Reference copy
│   └── ...
├── scripts/
│   └── omx-delegate.sh          # OMX iterative cleanup
├── .mcp.json                    # MCP server config
└── SKILL.md                     # Hermes skill definition
```

## License

MIT
