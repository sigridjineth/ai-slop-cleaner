---
name: ai-slop-cleaner
category: writing
version: 2.2.0
description: >
  Detect AI-slop signals in prose with an agent-driven Python/FastMCP server.
  Use it to score generated text, identify suspicious spans, and guide surgical
  humanization while preserving the original meaning.
---

# AI Slop Cleaner

AI Slop Cleaner v2 is a Python package and MCP server modeled after the
Ouroboros architecture: MCP tools call a detector, the detector delegates to AI
agent CLIs when available, and a deterministic regex fallback keeps the tool
usable everywhere.

## When to use this skill

- You need to identify AI-output tells before editing prose.
- You want a 0-100 AI Slop Score with the five component subscores.
- You want Claude, Codex, or any MCP client to call the same detector.

## Architecture

```text
src/ai_slop_cleaner/
├── cli.py                         # ai-slop-cleaner mcp serve|score|analyze
├── mcp/server.py                  # FastMCP server
├── mcp/tools.py                   # ai_slop_score/analyze/check
└── core/
    ├── detector.py                # claude --print -> codex exec -> fallback
    ├── scorer.py                  # 5-component weighted score formula
    ├── fallback.py                # regex fallback
    ├── code_smells.py            # Python/JS/Rust code smell detector
    ├── ralph.py                  # iterative cleanup loop
    ├── banned_words.py            # references/banned-words.md loader
    └── banned_patterns.py         # references/banned-patterns.md loader
```

## MCP tools

- `ai_slop_score` — returns `score`, `components`, details, detector, and source.
- `ai_slop_analyze` — returns findings, components, document patterns, and score.
- `ai_slop_check` — returns pass/fail. Passing means score `<= 25` by default.

## Score formula

```text
Score = round(100 * (0.25*BWD + 0.25*SPV + 0.20*RHY + 0.15*META + 0.15*MD))
```

Components:

- `BWD` — banned word and phrase density.
- `SPV` — structural pattern violations.
- `RHY` — rhythm monotony.
- `META` — meta commentary density.
- `MD` — markdown overuse.

## CLI usage

```bash
pip install -e '.[dev]'
ai-slop-cleaner mcp serve
ai-slop-cleaner score draft.md
ai-slop-cleaner analyze draft.md
ai-slop-cleaner code-smells src tests --tests tests
ai-slop-cleaner ralph draft.md --threshold 25 --max-iterations 5 --output clean.md
```

Rust single-binary build:

```bash
cd rust
cargo build --release
./target/release/ai-slop-cleaner-rs score ../README.md
./target/release/ai-slop-cleaner-rs mcp serve
```

For local tests or deterministic runs:

```bash
AI_SLOP_CLEANER_DISABLE_AGENTS=1 ai-slop-cleaner score draft.md
```

## Agent-driven detection

`core/detector.py` tries these paths in order:

1. `claude --print`
2. `codex exec`
3. `core/fallback.py`

The prompt sent to agents includes the banned-word list, structural-pattern list,
and `references/agent-response-schema.json`. Agents return component values in
`0.0..1.0`; the package computes the final score.

## Korean AI slop detection

The package includes extensive Korean-language pattern coverage derived from
external sources (e.g. `epoko77-ai/im-not-ai`). Categories include:

| Category | Count | Examples |
|---|---|---|
| A — Translationese | 15 | `~에 대해(서)`, `~를 통해`, `~에 있어(서)`, `~할 수 있다` |
| B — English term/quote overuse | 4 | Parenthesized English, raw buzzwords, long quotes |
| C — Structural AI patterns | 10 | `첫째/둘째/셋째`, bullet blocks, generic headings, binary parallelism |
| D — Signature Korean AI phrases | 7 | Conclusion formulas, hype words, personified abstract subjects |
| E — Rhythm uniformity | 3 | Low sentence-length variation, repeated endings, uniform paragraphs |
| F — Modifier/abstraction overload | 5 | Degree adverbs, double modifiers, `~적 N` chains |
| G — Hedging | 2 | Hedge endings, double/triple hedges |
| H — Connector overload | 4 | Sentence-initial connectors, `하지만/그러나`, `즉` |
| I — Formal/dependent noun overload | 6 | `~것이다`, dependent nouns, `~할 필요가 있다` |
| J — Visual decoration overload | 4 | Bold emphasis, quote emphasis, em dash, parenthetical asides |

All 60 rules are implemented as `ko_*` regex patterns in `banned_patterns.py`
and validated by `tests/test_im_not_ai_coverage.py`.

## H.U.M.A.N. Framework diagnostics

Positive human-quality signals are reported in `doc_patterns.human_framework`
instead of adding a sixth score component:

| Element | Markers | Coverage |
|---|---|---|
| H — Honest human flaws | `솔직히 말하면`, `to be fair`, `honestly` | RHY/SPV absence |
| U — Unpredictable structure | Varying paragraph/sentence lengths | RHY + SPV |
| M — Memorable specifics | Numbers, dates, `예를 들어` | doc_patterns diagnostic |
| A — Authentic perspective | `제 경험으로는`, `개인적으로` | doc_patterns diagnostic |
| N — Natural flow | Conversational connectors | SPV + RHY opener repetition |

## Bulk external-rule audit with omx

To audit coverage against an external rule repository (e.g. `im-not-ai`):

1. Clone the external repo and extract all detection rules.
2. Map each rule to existing `banned_patterns.py` patterns.
3. Implement missing rules as `ko_*` regexes with test cases.
4. Run `uv run pytest tests/ -v` to verify.
5. Commit as `audit: cover all <source> + HUMAN framework rules`.

Use a **single omx exec** with `$team` (inventory), `$ralph` (gap analysis),
and `$ultrawork` (implementation) roles in one prompt for efficiency.

## Editing discipline

If using findings to rewrite text:

- No finding, no edit.
- Preserve meaning, facts, numbers, named entities, quotes, and code blocks.
- Replace banned words with plain equivalents.
- Break repeated structure and vary rhythm.
- Remove meta commentary unless it is necessary navigation.
- Treat the score as triage, never as proof of authorship.

## Linked files

- `pyproject.toml` — uv/pip Python package metadata.
- `src/ai_slop_cleaner/mcp/server.py` — FastMCP server.
- `src/ai_slop_cleaner/mcp/tools.py` — MCP tool functions.
- `src/ai_slop_cleaner/core/detector.py` — agent delegation.
- `src/ai_slop_cleaner/core/fallback.py` — deterministic fallback.
- `src/ai_slop_cleaner/core/code_smells.py` — code cleanup smell detector.
- `src/ai_slop_cleaner/core/ralph.py` — iterative Ralph cleanup loop.
- `src/ai_slop_cleaner/core/banned_patterns.py` — regex patterns (English + Korean).
- `src/ai_slop_cleaner/core/banned_words.py` — banned word taxonomy loader.
- `src/ai_slop_cleaner/core/scorer.py` — 5-component weighted score formula.
- `.claude-plugin/.mcp.json` — Claude plugin MCP config.
- `.mcp.json` — project-level MCP config.
- `.claude/CLAUDE.md` — Claude detection-subagent instructions.
- `.codex/instructions.md` — Codex detection-subagent instructions.
- `references/agent-driven-spec.md` — architecture spec.
- `references/agent-prompt-template.md` — subagent prompt template.
- `references/agent-response-schema.json` — expected subagent JSON schema.
- `references/banned-words.md` — canonical banned-word taxonomy.
- `references/banned-patterns.md` — canonical structural-pattern taxonomy.
- `references/im-not-ai-audit.md` — external rule coverage audit report.
- `references/human-checklist.md` — H.U.M.A.N. Framework 20-point checklist.
- `tests/test_im_not_ai_coverage.py` — Korean pattern regression tests.
- `tests/test_code_smells.py` — code-smell detector regression tests.
- `tests/test_ralph.py` — Ralph mode regression tests.
- `rust/` — Rust single-binary implementation with score/analyze/ralph/code-smells/MCP commands.
