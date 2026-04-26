---
name: ai-slop-cleaner
category: writing
version: 2.0.0
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
- `.claude-plugin/.mcp.json` — Claude plugin MCP config.
- `.mcp.json` — project-level MCP config.
- `.claude/CLAUDE.md` — Claude detection-subagent instructions.
- `.codex/instructions.md` — Codex detection-subagent instructions.
- `references/agent-driven-spec.md` — architecture spec.
- `references/agent-prompt-template.md` — subagent prompt template.
- `references/agent-response-schema.json` — expected subagent JSON schema.
- `references/banned-words.md` — canonical banned-word taxonomy.
- `references/banned-patterns.md` — canonical structural-pattern taxonomy.
