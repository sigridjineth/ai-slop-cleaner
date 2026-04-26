---
name: ai-slop-cleaner
category: writing
version: 2.2.0
description: >
  Score generated prose for AI-slop signals and provide cleanup guidance with
  the Python/FastMCP server, Rust binary, and deterministic fallback.
---

AI Slop Cleaner

AI Slop Cleaner v2 ships as a Python package, FastMCP server, and Rust binary.
The detector asks Claude first, tries Codex next, then falls back to regex
rules, so the same checks work in local CLIs and MCP clients.

Use

Use it before editing generated prose, when a draft needs a 0-100 AI Slop
Score, or when Claude, Codex, and MCP clients should share one detector. Keep
the pass focused on the supplied text. Preserve facts, names, numbers, quotes,
and code blocks.

Files

The Python entry point is src/ai_slop_cleaner/cli.py. MCP lives in
src/ai_slop_cleaner/mcp/server.py and src/ai_slop_cleaner/mcp/tools.py.
Detection and scoring live under src/ai_slop_cleaner/core with detector.py,
scorer.py, fallback.py, code_smells.py, ralph.py, banned_words.py, and
banned_patterns.py. The Rust crate in rust/ provides score, analyze, ralph,
code-smells, and mcp commands.

Tools

ai_slop_score returns score, components, details, detector, and source.
ai_slop_analyze returns findings, components, document patterns, and score.
ai_slop_check returns pass or fail with default threshold 25.

Score

The final score uses BWD, SPV, RHY, META, and MD weights of 25, 25, 20, 15,
and 15. BWD measures banned word and phrase density. SPV counts structural
patterns. RHY flags flat rhythm. META catches navigation chatter. MD tracks
needless formatting.

Commands

```bash
pip install -e '.[dev]'
ai-slop-cleaner mcp serve
ai-slop-cleaner score draft.md
ai-slop-cleaner analyze draft.md
ai-slop-cleaner code-smells src tests --tests tests
ai-slop-cleaner ralph draft.md --threshold 25 --max-iterations 5 --output clean.md

cd rust
cargo build --release
./target/release/ai-slop-cleaner-rs score ../README.md
./target/release/ai-slop-cleaner-rs analyze ../README.md
./target/release/ai-slop-cleaner-rs ralph ../README.md --threshold 25 --max-iterations 5 --output /tmp/clean.md

AI_SLOP_CLEANER_DISABLE_AGENTS=1 ai-slop-cleaner score draft.md
```

Detection

core/detector.py tries claude --print, then codex exec, then core/fallback.py.
Agent prompts include banned terms, structural patterns, and
references/agent-response-schema.json. Returned component values stay in the
0.0 to 1.0 range; scorer.py computes the final integer.

Korean rules

Korean rules are in banned_patterns.py as ko_* regexes and are checked by
tests/test_im_not_ai_coverage.py. They cover translationese, English term
overuse, mechanical structure, signature phrases, rhythm uniformity, modifier
overload, hedging, connector overload, formal noun endings, and visual
decoration. The audit record is references/im-not-ai-audit.md.

Human cues

doc_patterns reports optional H.U.M.A.N. markers as diagnostics rather than a
sixth score component. The checklist is references/human-checklist.md and looks
at honest flaws, varied structure, specific examples, personal perspective, and
natural flow.

Rule audit

To audit another rule repository, clone it outside this package, extract rules,
map each rule to banned_patterns.py, add missing ko_* regexes with tests, run
uv run pytest tests/ -v, then commit the covered source. For broad audits, run
one omx exec that assigns inventory, gap analysis, and implementation roles.

Editing

Only edit where findings point. Prefer plain replacements for banned terms.
Vary structure and sentence length. Delete meta commentary that does not guide
use. Treat the score as triage, not as proof of authorship. Re-run score,
analyze, and the relevant tests before reporting completion.

Links

Package metadata is pyproject.toml. MCP config lives in .mcp.json and
.claude-plugin/.mcp.json. Agent instructions live in .claude/CLAUDE.md and
.codex/instructions.md. Reference docs include agent-driven-spec.md,
agent-prompt-template.md, banned-words.md, banned-patterns.md,
im-not-ai-audit.md, and human-checklist.md under references.
