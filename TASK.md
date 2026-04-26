# AI-Slop-Cleaner Dogfooding: Fix Universal Pattern Detection

## Problem

The `--lang auto` / `--lang en` flag skips ALL `ko_*` patterns on English text. But many `ko_*` patterns are **language-agnostic universal patterns** that should apply to ALL languages:

- `ko_J1_bold` — bold `**text**` decoration (applies to any markdown)
- `ko_J3_em_dash` — em dash `—` overuse (applies to any prose)
- `ko_C5_emoji` — emoji in prose (applies to any language)
- `ko_C10_colon_heading` — colon in markdown headings (applies to any language)
- `ko_C2_bullet_block` — excessive bullet lists (applies to any language)
- `ko_B2_raw_english_term` — English buzzwords like `leverage`, `robust` (applies when English words appear)

Current behavior: hermes-agent README.md scores 0/100 with `--lang auto` because all `ko_*` patterns are skipped. But it has 21 em dashes, 16 bold decorations, and colon headings that SHOULD be flagged.

## Root Cause

`should_include_pattern()` in `scorer.rs` line 79:
```rust
"en" => !pattern_name.starts_with("ko_"),
```

This is too blunt. It treats ALL `ko_*` patterns as Korean-only.

## Solution

Add a `lang_scope` field to each pattern in `banned-patterns.md`:
- `universal` — apply to all languages (ko_J1, ko_J3, ko_C5, ko_C2, ko_C10, ko_B2)
- `korean` — only apply when text is Korean (ko_A1~A15, ko_B1, ko_B3, ko_B4, ko_C1, ko_C3, ko_C4, ko_C6~C9, ko_D1~D7, ko_E1~E3, ko_F1~F5, ko_G1~G2, ko_H1~H4, ko_I1~I6, ko_J2, ko_J4)
- `english` — only apply when text is English (redefinition, closing_summary, etc.)

Then update `should_include_pattern()` to check `lang_scope` instead of just the prefix.

## Deliverable

1. Update `banned-patterns.md` to add `Lang Scope` column
2. Update `pattern_loader.rs` to parse the new column
3. Update `scorer.rs` `should_include_pattern()` to use lang_scope
4. Update `BannedPattern` struct to include `lang_scope: String`
5. Run tests: `cargo test`
6. Dogfood on hermes-agent README.md — should score > 0 (em dash, bold detected)
7. Dogfood on Korean text — should still detect ko_A* patterns
8. Commit and push

## Test Cases

```bash
# English text with em dash and bold — should detect universal patterns
cargo run --release -- score /home/mconcat/work/hermes-agent/README.md --lang auto
# Expected: score > 0, matches include ko_J1_bold, ko_J3_em_dash

# Korean text — should detect Korean-specific patterns
cargo run --release -- score /tmp/korean_test.md --lang auto
# Expected: score > 0, matches include ko_A1, ko_A2, etc.

# All mode — should detect everything
cargo run --release -- score /home/mconcat/work/hermes-agent/README.md --lang all
```
