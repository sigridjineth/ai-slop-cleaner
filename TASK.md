# AI-Slop-Cleaner Rust Regex Fix & OMX Delegation

## Task
1. Fix regex syntax in `rust/rules/banned-patterns.md` — the markdown table parser now handles `\|` correctly, but verify all 70 patterns compile in Rust's regex crate.
2. Test Korean pattern detection — verify Unicode ranges (`\uac00-\ud7a3`) work correctly for Korean AI slop patterns.
3. Add the "A가 아니다" / "is not just" patterns that were specifically requested — these are in the markdown and loading correctly after the parser fix.
4. Integration with oh-my-codex — create a wrapper script and TASK.md for orchestration.

## Current State
- `pattern_loader.rs` has been fixed with backtick-aware markdown table parsing.
- `\|` inside backtick-wrapped regex cells is now converted to `|` for proper Rust regex alternation.
- All 70 patterns compile successfully.
- 6 unit tests pass including Korean pattern detection and English redefinition tests.
- Binary builds in release mode successfully.

## Verification Commands
```bash
cd /home/mconcat/.hermes/skills/ai-slop-cleaner/rust
cargo test
cargo build --release
./target/release/ai-slop-cleaner rules
./target/release/ai-slop-cleaner stdin < /tmp/comprehensive_test.txt
```

## Test Results
- 6/6 tests pass
- 70 banned patterns loaded
- 93 banned words loaded
- Korean "A가 아니라 B" detection: working
- English "is not just" / "not X but Y" detection: working
- Unicode Korean ranges (`\uac00-\ud7a3`): working

## Deliverables
- [x] Fixed `pattern_loader.rs` with backtick-aware table parsing
- [x] Verified all 70 regex patterns compile
- [x] Verified Korean Unicode pattern detection
- [x] Verified "A가 아니다" / "is not just" patterns load and match
- [x] Unit tests added to `scorer.rs`
- [ ] OMX delegation wrapper (next step)
