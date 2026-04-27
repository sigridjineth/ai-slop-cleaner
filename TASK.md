# AI-Slop-Cleaner Dogfooding for Universal Pattern Detection

## Problem

The `--lang auto` / `--lang en` flag skips ALL `ko_*` patterns on English text. But many `ko_*` patterns are language-agnostic universal patterns that should apply to ALL languages:

| Pattern | Why it is universal |
|---------|---------------------|
| `ko_J1_bold` | Bold markdown decoration applies to any markdown. |
| `ko_J3_em_dash` | Em dash overuse applies to any prose. |
| `ko_C5_emoji` | Emoji in prose applies to any language. |
| `ko_C10_colon_heading` | Colon headings can appear in any markdown. |
| `ko_C2_bullet_block` | Excessive bullet lists can appear in any language. |
| `ko_B2_raw_english_term` | English buzzword checks apply whenever English terms appear. |

Current behavior: hermes-agent README.md scores 0/100 with `--lang auto` because all `ko_*` patterns are skipped. But it has 21 em dashes, 16 bold decorations, and colon headings that SHOULD be flagged.

## Root Cause

`should_include_pattern()` in `scorer.rs` line 79:
```rust
"en" => !pattern_name.starts_with("ko_"),
```

This is too blunt. It treats ALL `ko_*` patterns as Korean-only.

## Solution

Add a `lang_scope` field to each pattern in `banned-patterns.md`:

| Scope | Applies to |
|-------|------------|
| `universal` | All languages (`ko_J1`, `ko_J3`, `ko_C5`, `ko_C2`, `ko_C10`, `ko_B2`). |
| `korean` | Korean text only (`ko_A1` through `ko_A15`, `ko_B1`, `ko_B3`, `ko_B4`, `ko_C1`, `ko_C3`, `ko_C4`, `ko_C6` through `ko_C9`, `ko_D1` through `ko_D7`, `ko_E1` through `ko_E3`, `ko_F1` through `ko_F5`, `ko_G1` through `ko_G2`, `ko_H1` through `ko_H4`, `ko_I1` through `ko_I6`, `ko_J2`, `ko_J4`). |
| `english` | English text only (`redefinition`, `closing_summary`, etc.). |

Then update `should_include_pattern()` to check `lang_scope` instead of just the prefix.

## Deliverable

1. Update `banned-patterns.md` to add `Lang Scope` column
2. Update `pattern_loader.rs` to parse the new column
3. Update `scorer.rs` `should_include_pattern()` to use lang_scope
4. Update `BannedPattern` struct to include `lang_scope: String`
5. Run tests: `cargo test`
6. Dogfood on hermes-agent README.md; it should score > 0 (em dash, bold detected)
7. Dogfood on Korean text; it should still detect ko_A* patterns
8. Commit and push

## Test Cases

```bash
# English text with em dash and bold should detect universal patterns
cargo run --release -- score /home/mconcat/work/hermes-agent/README.md --lang auto
# expect score > 0, with matches including ko_J1_bold and ko_J3_em_dash

# Korean text should detect Korean-specific patterns
cargo run --release -- score /tmp/korean_test.md --lang auto
# expect score > 0, with matches including ko_A1 and ko_A2

# All mode should detect everything
cargo run --release -- score /home/mconcat/work/hermes-agent/README.md --lang all
```
