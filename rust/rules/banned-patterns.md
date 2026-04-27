# Banned Patterns

Language-agnostic structural patterns for AI slop detection.
Loaded by the Rust binary at runtime. Only universal (language-agnostic) patterns
live here — language-specific patterns are defined in `patterns-agent.md` and
evaluated by an LLM agent, not regex.

## Universal Structural Patterns

| Name | Lang Scope | Severity | Weight | Regex | Description |
|------|------------|----------|--------|-------|-------------|
| bullet_block | universal | medium | 1.0 | `(?m)(?:^\s*[-*+]\s+.+\n?){3,}` | Excessive bullet block (3+ consecutive items). |
| emoji_decoration | universal | high | 2.0 | `[✅🚀💡⚠📊✨🔥👉👍🎯]` | Emoji decoration in prose. |
| colon_heading | universal | high | 2.0 | `(?m)^\s{0,3}#{1,6}\s*(?:\*\*)?[^:\n]{1,80}(?:\*\*)?\s*:\s+\S.*$` | Colon subtitle heading formula (## Title: Subtitle). |
| bold_emphasis | universal | medium | 0.5 | `(?:\*\*|__)[^*_\n]{1,80}(?:\*\*|__)` | Bold emphasis decoration. |
| em_dash | universal | low | 0.5 | `—` | Em dash decoration. |
| plus_conjunction | universal | medium | 1.5 | `(?i)\b[A-Za-z][A-Za-z0-9]*(?:\s+[A-Za-z][A-Za-z0-9]*){0,2}\s*\+\s*[A-Za-z][A-Za-z0-9]*(?:\s+[A-Za-z][A-Za-z0-9]*){0,2}\b` | Plus-sign conjunction between terms (e.g. "English + Korean", "speed + reliability"). Common in Codex/AI output. |
