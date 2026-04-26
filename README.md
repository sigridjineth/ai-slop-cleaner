# AI Slop Cleaner

A Hermes skill that detects and removes the tell-tale signs of AI-generated prose — the repetitive phrasing, stiff vocabulary, uniform rhythm, and meta commentary that make readers suspect a machine wrote the text.

## What it does

- **Detects** AI slop markers in any prose document.
- **Scores** the overall "AI sloppiness" of a text on a 0–100 scale.
- **Guides** rewrite work with a 5-stage pipeline, 20-point checklist, and H.U.M.A.N. framework.

## Installation

No installation required. The skill is self-contained within `~/.hermes/skills/ai-slop-cleaner`.

### CLI (optional)

The Go CLI runs without external dependencies and needs only Go 1.20+:

```bash
cd scripts
go build ai-slop-cleaner.go
```

## Quick start

### 1. Run the detector on a file

```bash
go run scripts/ai-slop-cleaner.go -input draft.md -output report.md
```

### 2. Pipe from stdin

```bash
cat draft.md | go run scripts/ai-slop-cleaner.go -output report.md
```

### 3. Get a single AI Slop Score

```bash
cat draft.md | go run scripts/ai-slop-cleaner.go -score
# Output: 42
```

The score is an integer from 0 (clean) to 100 (dense AI slop). It combines:

| Component | Weight | What it measures |
|-----------|--------|------------------|
| Banned word density | 25% | Overused AI vocabulary (delve, leverage, landscape, etc.) |
| Structural violations | 25% | Repeated templates, redefinition sentences, list cadence |
| Rhythm monotony | 20% | Uniform sentence length, repeated openers/endings |
| Meta commentary | 15% | "In this section we will...", "Let's dive in" |
| Markdown overuse | 15% | Excessive bullets, bold, headings, tables |

### 4. Use `.slopignore`

Create a `.slopignore` file to skip lines matching regex patterns (useful for tables or code blocks):

```
^\|.*\|.*\|$
^```
```

## Skill structure

```
ai-slop-cleaner/
├── SKILL.md                       # Main skill definition
├── README.md                      # This file
├── .claude/
│   └── CLAUDE.md                  # Claude Code agent instructions
├── .codex/
│   └── instructions.md          # Codex CLI agent instructions
├── .opencode/
│   └── instructions.md          # OpenCode agent instructions
├── references/
│   ├── banned-words.md          # Full banned-word taxonomy
│   ├── banned-patterns.md       # Structural anti-patterns
│   ├── human-checklist.md       # Quick-reference checklist
│   └── slop-score-spec.md       # AI Slop Score specification
└── scripts/
    └── ai-slop-cleaner.go       # Go CLI detector + scorer
```

## 5-Stage Pipeline

1. **AI-Tell Detector** — Scan and flag banned words, structural patterns, rhythm issues, markdown overuse, and meta commentary.
2. **Style Rewriter** — Rewrite only flagged spans. Replace AI vocabulary with plain English, vary sentence length, fold short lists into prose.
3. **Fidelity Auditor** — Verify semantic identity: facts, numbers, names, quotes, technical terms, logical relationships, and code blocks must be preserved exactly.
4. **Naturalness Reviewer** — Read aloud mentally. Score 1–10. Trigger a second rewrite round if below 7.
5. **Orchestrator** — Accept, rewrite again, roll back, or hold for human review.

## H.U.M.A.N. Framework

Five lenses to keep the rewrite grounded:

- **H — Honest human flaws.** Mild uncertainty or conversational turns are fine.
- **U — Unpredictable structure.** Vary paragraph length and rhythm section by section.
- **M — Memorable specifics.** Keep dates, numbers, product names, real situations.
- **A — Authentic perspective.** Preserve or clarify point of view.
- **N — Natural flow.** Connect ideas conversationally.

## Core rules

- **No finding, no edit.** If the detector does not flag a span, leave it untouched.
- **Redefinition sentences are forbidden.** Never write "A is not X; it is Y." Say what it is directly.
- **Track rewrite distance.** >30% words changed → warn. >50% → stop and request review.
- **Preserve everything.** Facts, numbers, names, dates, quotes, technical terms, and code blocks stay exactly as they are.

## Agent instructions

Each supported AI agent has its own instruction file tuned to its context-window size and interaction style:

- **Claude Code** → `.claude/CLAUDE.md` (detailed, context-heavy)
- **Codex CLI** → `.codex/instructions.md` (concise, structured)
- **OpenCode** → `.opencode/instructions.md` (directive, compact)

## License

MIT
