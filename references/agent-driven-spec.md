# AI Slop Cleaner v2 Agent-Driven Architecture

Version: 2.0.0  
Status: implemented architecture spec

## Goal

AI Slop Cleaner v2 follows the Ouroboros package pattern: a Python package exposes
a FastMCP server, MCP tools delegate analysis to external AI coding agents, and a
regex fallback keeps the tool useful when no agent runtime is available.

## Package layout

```text
ai-slop-cleaner/
├── pyproject.toml
├── .mcp.json
├── .claude-plugin/.mcp.json
├── .claude/CLAUDE.md
├── .codex/instructions.md
├── references/
│   ├── banned-words.md
│   ├── banned-patterns.md
│   ├── agent-driven-spec.md
│   ├── agent-prompt-template.md
│   └── agent-response-schema.json
└── src/ai_slop_cleaner/
    ├── cli.py
    ├── mcp/server.py
    ├── mcp/tools.py
    └── core/
        ├── detector.py
        ├── scorer.py
        ├── fallback.py
        ├── banned_words.py
        └── banned_patterns.py
```

## Runtime flow

1. MCP client calls `ai_slop_score`, `ai_slop_analyze`, or `ai_slop_check`.
2. `mcp/tools.py` resolves `text` or `file` input.
3. `core/detector.py` builds a prompt from `agent-prompt-template.md`, embedding:
   - `references/banned-words.md`
   - `references/banned-patterns.md`
   - `references/agent-response-schema.json`
   - the target text
4. The detector tries `claude --print` first.
5. If Claude is unavailable, times out, or returns invalid JSON, the detector tries `codex exec`.
6. If no agent returns a valid response, `core/fallback.py` runs deterministic regex detection.
7. `core/scorer.py` normalizes components and computes the final 0-100 score.

## Agent contract

Agents must return only JSON:

```json
{
  "findings": [],
  "components": {"BWD": 0, "SPV": 0, "RHY": 0, "META": 0, "MD": 0}
}
```

Findings should include the exact span where possible, line number where useful,
category, severity, source text, and a suggested fix. Component values are floats
in `0.0..1.0`.

## Component score formula

```text
Score = round(100 * (
  0.25 * BWD +
  0.25 * SPV +
  0.20 * RHY +
  0.15 * META +
  0.15 * MD
))
```

Components:

- `BWD`: banned word and banned phrase density.
- `SPV`: structural pattern violations.
- `RHY`: rhythm monotony.
- `META`: meta commentary density.
- `MD`: markdown overuse.

## Fallback contract

Fallback does not claim authorship. It only reports editing signals. It skips
fenced code blocks, strips inline code, honors `.slopignore`, and uses the same
component keys as agent responses.

## Environment controls

- `AI_SLOP_CLEANER_DISABLE_AGENTS=1`: force regex fallback.
- `AI_SLOP_CLEANER_AGENT_TIMEOUT=<seconds>`: override agent subprocess timeout.

## CLI

```bash
ai-slop-cleaner mcp serve
ai-slop-cleaner score draft.md
ai-slop-cleaner analyze draft.md
```

The MCP server defaults to stdio transport, matching Claude/Codex MCP launch
expectations. SSE is available with `--transport sse` for local debugging.
