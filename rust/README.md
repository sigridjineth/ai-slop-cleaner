# AI-Slop-Cleaner

A fast, multilingual CLI tool that detects AI-generated text patterns ("AI slop") in documents. Built in Rust with support for English and Korean, extensible to any language via markdown-based pattern definitions.

## Features

The CLI ships with 70 regex patterns for structural, lexical, and stylistic AI
slop signals, plus 93 banned words with suggested replacements for English and
Korean. Patterns are defined in markdown rather than hardcoded. Output formats
include human-readable text, JSON, and a markdown report. The Rust binary
processes documents in milliseconds, and its JSON output integrates with agent
pipelines.

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
# Score a document
ai-slop-cleaner score document.md

# JSON output for programmatic use
ai-slop-cleaner score document.md --format json

# Markdown report
ai-slop-cleaner score document.md --format markdown

# List all loaded rules
ai-slop-cleaner rules

# Score from stdin
echo "your text here" | ai-slop-cleaner stdin
```

## Score Interpretation

| Range | Meaning |
|-------|---------|
| `0-30` | Clean, likely human-written |
| `30-60` | Some AI patterns detected, light editing recommended |
| `60-80` | Noticeable AI slop, significant revision needed |
| `80-100` | Heavy AI slop, full rewrite recommended |

## Pattern Categories

| Category | Description | Examples |
|----------|-------------|----------|
| A | Translationese / awkward phrasing | "~에 있어서", "~을 기반으로" |
| B | English term abuse | Parenthesized English, raw English terms |
| C | Structural patterns | Mechanical enumeration, bullet abuse, emoji headings |
| D | Cliché formulas | "In conclusion", "It is important to note" |
| E | Sentence uniformity | Same-length sentences, repeated endings |
| F | Modifier abuse | Degree adverbs, suffix chains, double modifiers |
| G | Hedging | "~할 수 있습니다", "~것입니다" overuse |
| H | Connector patterns | Predictable transitions, meta entries |
| I | Korean-specific | Dependent noun abuse, "geotida" overuse |
| J | Formatting abuse | Bold overuse, quote emphasis, em dashes |

## Adding New Patterns

Edit `rules/banned-patterns.md` and add entries to the markdown tables:

```markdown
| Pattern Name | Regex | Severity | Weight | Description |
|-------------|-------|----------|--------|-------------|
| my_new_pattern | `regex here` | medium | 1.0 | Description |
```

For word lists, add to `rules/banned-words.md`:

```markdown
| Word | Replacement | Severity | Weight | Language |
|------|-------------|----------|--------|----------|
| badword | goodword | medium | 1.0 | en |
```

## Integration with OMX

The JSON output is designed for agent pipelines:

```json
{
  "overall_score": 45.5,
  "max_possible_score": 100.0,
  "summary": {
    "total_patterns_matched": 12,
    "total_words_matched": 3,
    "high_severity_count": 2,
    "medium_severity_count": 8,
    "low_severity_count": 5
  },
  "matches": [...],
  "word_matches": [...]
}
```

## License

MIT
