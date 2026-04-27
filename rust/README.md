# AI-Slop-Cleaner

A fast CLI pattern-matching engine for AI-generated text artifacts ("AI slop").
The Rust binary is intentionally dumb: it emits raw structural matches and
does not compute a holistic score. Agent workflows consume the full text plus
these matches and make the semantic judgment.

## Features

The CLI ships with six universal regex patterns for structural AI slop signals.
Patterns are defined in markdown rather than hardcoded. The `analyze` command
outputs a clean JSON array of match objects for agent pipelines. Language- and
semantic-level judgment lives in `rules/patterns-agent.md` and is performed by
LLM agents, not by the Rust binary.

## Installation

```bash
# From source
cd rust
cargo build --release

# Binary path is
# target/release/ai-slop-cleaner
```

## Usage

```bash
# Analyze a document and emit raw structural matches as JSON
ai-slop-cleaner analyze document.md

# Deprecated compatibility path: delegates to analyze and emits no score
ai-slop-cleaner score document.md --format json

# Human-readable report from the compatibility command
ai-slop-cleaner score document.md --format markdown

# List all loaded rules
ai-slop-cleaner rules

# Analyze from stdin
echo "your text here" | ai-slop-cleaner stdin --format json
```

## Architecture

The Rust binary only does structural regex matching:

1. Load universal structural patterns from `rules/banned-patterns.md`.
2. Match those patterns against the input text.
3. Emit raw matches: pattern name, severity, weight, matched text, and line
   number.

It does **not** calculate an overall score, count banned words as findings, or
judge document quality. Agent workflows should read the source document plus
the JSON matches and assign any holistic score.

## Universal Structural Patterns

Current Rust-matched patterns include:

- Bullet blocks
- Emoji decoration
- Colon subtitle headings
- Bold emphasis decoration
- Em dashes
- Plus-sign conjunctions

## Adding New Patterns

Edit `rules/banned-patterns.md` and add entries to the markdown tables:

```markdown
| Name | Lang Scope | Severity | Weight | Regex | Description |
|------|------------|----------|--------|-------|-------------|
| my_new_pattern | universal | medium | 1.0 | `regex here` | Description |
```

Keep language-specific, semantic, and judgment-oriented patterns in
`rules/patterns-agent.md` for LLM agents. The Rust analyzer should stay a raw
structural matcher.

## Integration with OMX

The JSON output is designed for agent pipelines:

```json
[
  {
    "pattern_name": "emoji_decoration",
    "severity": "high",
    "weight": 2.0,
    "matched_text": "🚀",
    "line_number": 12
  }
]
```

## License

MIT
