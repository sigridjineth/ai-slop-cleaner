# AI Slop Cleaner

AI Slop Cleaner is an agent-driven Python package and FastMCP server for scoring
AI-slop signals in prose. The design mirrors Ouroboros — a Python package with
`pyproject.toml`, a FastMCP server, MCP tools, a Claude plugin config, and a CLI
entry point.

## What it provides

- **MCP server:** `ai-slop-cleaner mcp serve`
- **MCP tools:** `ai_slop_score`, `ai_slop_analyze`, `ai_slop_check`
- **Agent-driven detector:** tries `claude --print`, then `codex exec`
- **Regex fallback:** deterministic detection when no AI agent is available
- **Score aggregation:** five weighted components on a 0-100 scale

## Install for development

```bash
pip install -e '.[dev]'
```

The package depends on `mcp[cli]` and uses FastMCP.

## Quick start

```bash
# Start the MCP server over stdio
ai-slop-cleaner mcp serve

# Score a file; prints one integer by default
ai-slop-cleaner score draft.md

# Full JSON analysis
ai-slop-cleaner analyze draft.md

# Force deterministic fallback instead of spawning agents
AI_SLOP_CLEANER_DISABLE_AGENTS=1 ai-slop-cleaner analyze draft.md
```

## MCP configuration

`.mcp.json` and `.claude-plugin/.mcp.json` both register the server through uvx:

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

## Architecture

```text
src/ai_slop_cleaner/
├── __init__.py
├── cli.py
├── mcp/
│   ├── __init__.py
│   ├── server.py
│   └── tools.py
└── core/
    ├── __init__.py
    ├── detector.py
    ├── scorer.py
    ├── fallback.py
    ├── banned_words.py
    └── banned_patterns.py
```

`core/detector.py` builds an agent prompt from reference files and calls runtimes
in this order:

1. `claude --print`
2. `codex exec`
3. regex fallback

The fallback ports the essential behavior from the old Go CLI: it skips fenced
code blocks, strips inline code, honors `.slopignore`, loads the banned-word and
banned-pattern references, and computes the same five component keys.

## AI Slop Score

```text
Score = round(100 * (0.25*BWD + 0.25*SPV + 0.20*RHY + 0.15*META + 0.15*MD))
```

| Component | Weight | Meaning |
| --- | ---: | --- |
| `BWD` | 25% | Banned word and phrase density |
| `SPV` | 25% | Structural pattern violations |
| `RHY` | 20% | Rhythm monotony |
| `META` | 15% | Meta commentary density |
| `MD` | 15% | Markdown overuse |

Use the score to decide how much editing a draft needs. A low score means the
text already reads naturally. A high score points to sections worth rewriting.

## References

- `references/banned-words.md` — canonical word and phrase taxonomy.
- `references/banned-patterns.md` — structural anti-pattern taxonomy.
- `references/agent-driven-spec.md` — v2 architecture spec.
- `references/agent-prompt-template.md` — prompt sent to subagents.
- `references/agent-response-schema.json` — expected JSON response schema.

## Tests

```bash
pip install -e '.[dev]'
python -m pytest tests/ -v
```

If the `python` executable is not present on your system, use `python3`.

## License

MIT
