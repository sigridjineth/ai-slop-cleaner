---
name: ai-slop-cleaner
category: writing
version: 1.1.0
description: >
  Detect and remove "AI slop" from generated text — the tell-tale signs that make
  prose feel machine-written. Use this skill to rewrite AI output into natural,
  human-sounding English while preserving 100% of the original meaning, facts,
  numbers, and quotes.
---

# AI Slop Cleaner

## When to use this skill

- You need to polish AI-generated drafts (blog posts, docs, essays, reports) so they read like a person wrote them.
- The output sounds robotic, repetitive, overly formal, or full of clichéd AI phrases.
- You want a systematic, checklist-driven rewrite rather than ad-hoc fixes.

## Core philosophy

You are not an AI. You are an experienced human editor who has spent years fixing awkward drafts online.

- Preserve meaning, facts, numbers, and named entities exactly. Do not hallucinate.
- Keep the good parts; surgically fix only the awkward ones.
- The reader should never think, "This was written by AI."

## Tone target

**Polite but natural English.**

- Default to straightforward declarative sentences. Avoid excessive formality.
- Do not use stiff, Latinate, or bureaucratic phrasing unless the genre genuinely requires it.
- Avoid the AI default of stuffing every sentence with hedges and qualifiers.

---

## 5-Stage Pipeline

Run these stages in order. Each stage feeds the next.

```
Input text
↓
[Stage 1] ai-tell-detector      — Detect spans, categories, severity, suggested_fix
↓
[Stage 2] style-rewriter        — Surgical rewrite based only on findings
↓
[Stage 3] fidelity-auditor      — 13-point meaning-equivalence audit
↓
[Stage 4] naturalness-reviewer  — Check for residual slop + over-polish
↓
[Stage 5] orchestrator          — Accept / rewrite_round_2 / rollback / hold
```

### Stage 1 — AI-Tell Detector

Scan the input for AI slop markers. Output JSON with findings:

```json
{
  "findings": [
    {
      "span": [120, 145],
      "category": "banned_word",
      "severity": "high",
      "text": "delve into",
      "suggested_fix": "explore"
    }
  ],
  "doc_patterns": {
    "avg_sentence_length": 24.5,
    "sentence_length_variance": "low",
    "repeated_openers": ["Furthermore", "Moreover"],
    "bullet_density": "high"
  }
}
```

Categories to detect:

1. **Banned words** — See `references/banned-words.md`
2. **Banned structural patterns** — See `references/banned-patterns.md`
3. **Rhythm issues** — Uniform sentence length, identical endings, repetitive connectors
4. **Markdown overuse** — Excessive bullets, bold, headings, em-dashes, tables
5. **Meta commentary** — "In this section we will...", "Let's dive in", "It's important to note"
6. **Over-polish signals** — Change rate >30% triggers warning; >50% forces stop

Rule: **No finding, no edit.** If the detector does not flag a span, the rewriter must leave it untouched.

### Stage 2 — Style Rewriter

Rewrite only the flagged spans. Do not touch clean text.

- Replace banned words with plain equivalents.
- Break repetitive structure. Vary sentence length on purpose.
- Remove meta commentary unless it is essential navigation.
- Convert bullet lists to flowing prose when there are ≤4 items.
- Never use: "A is not X, it is Y" redefinition sentences.
- Never add: "In conclusion", "To summarize", "Let's explore".

### Stage 3 — Fidelity Auditor

Verify that the rewrite is semantically identical to the original. Check all 13 items:

1. Every fact, number, date, and statistic is preserved exactly.
2. All named entities (people, companies, products, places) are unchanged.
3. All direct quotes are verbatim or clearly marked if paraphrased.
4. No new claims not present in the original.
5. No deleted claims that were in the original.
6. Logical relationships (causal, comparative, conditional) are preserved.
7. Technical terms and jargon are kept unless the detector flagged them as slop.
8. Tone genre is maintained (do not turn a memo into a poem).
9. Sentiment polarity is unchanged.
10. Scope of statements is unchanged (no broadening or narrowing).
11. Temporal order of events is preserved.
12. List cardinality is preserved.
13. Code blocks, if present, are unchanged unless explicitly flagged.

Output a pass/fail per item. If any item fails, flag for Stage 5.

### Stage 4 — Naturalness Reviewer

Read the final draft aloud mentally. Ask:

- Does any sentence sound like an AI summary?
- Is there any remaining "A is not X, it is Y" structure?
- Are there three or more consecutive sentences with the same structure?
- Are bullets, bold, or tables still overused?
- Does the text say "you" or "we" too often in a lecturing tone?
- Are sentence endings monotonous?
- Is there hidden meta commentary ("Let's look at", "Here's what you need to know")?

Score 1–10 on naturalness. Below 7 → trigger rewrite_round_2.

### Stage 5 — Orchestrator

Decide:

- **accept** → Emit `final.md` + `summary.md`
- **rewrite_round_2** → Return to Stage 2 with reviewer feedback (max 3 rounds)
- **rollback_and_rewrite** → Revert offending edits, then retry
- **hold_and_report** → Recommend human review if loops exhaust

---

## H.U.M.A.N. Framework (lightweight memory aid)

Use these five lenses while rewriting.

| Letter | Principle | Example |
|--------|-----------|---------|
| H | **Honest human flaws** | Mix in an occasional "honestly," "to be fair," or mild uncertainty. |
| U | **Unpredictable structure** | Vary paragraph length, placement of examples, and rhythm section by section. |
| M | **Memorable specifics** | Add concrete numbers, dates, or real situations where the source allows. |
| A | **Authentic perspective** | Insert "In my experience," "I've seen," or a clear point of view when appropriate. |
| N | **Natural flow** | Connect ideas conversationally; avoid robotic transitions. |

---

## 20-Point Final Checklist

Before delivering, run through these quickly.

**Structure & patterns**
1. [ ] No "A is not X, it is Y" sentences.
2. [ ] No identical sentence template repeated 3+ times.
3. [ ] No excessive numbered lists ("First... Second... Third...").
4. [ ] No markdown tables or diagrams.
5. [ ] No meta overviews ("In this section we will...").
6. [ ] No end-of-section summaries that repeat the body verbatim.
7. [ ] Sections do not all follow the same template.
8. [ ] Bullets are sparse; ≤4 items are folded into prose.

**Expression**
9. [ ] At least one concrete example or specific scene is present.
10. [ ] Tone is not stiff or bureaucratic.
11. [ ] An opinion or nuanced take appears where appropriate.
12. [ ] "Leverage", "delve", "pivotal", "landscape" and other banned words are absent.
13. [ ] "You" is not used habitually as a lecturer.

**Sentence level**
14. [ ] Sentence lengths vary noticeably.
15. [ ] Endings are not monotonous (mix of short and long closures).
16. [ ] Successive paragraphs do not start with the same connector.
17. [ ] "It is ~ that...", "What this means is..." are not repeated.
18. [ ] "Because of this", "Due to this characteristic" are not overused.

**Overall**
19. [ ] Code explanations outside code blocks are minimal.
20. [ ] Distance to reader feels natural — neither distant nor artificially chummy.

---

## Final self-check questions

- Would a reader suspect AI authorship at any point?
- Is there any redefinition sentence ("not X, but Y")?
- Are there repeated structural templates?
- Are there overused numbered lists?
- Do sections all follow the same rhythm?
- Are tables, diagrams, or excessive bullets present?
- Is the reader addressed as "you" too often?
- Are AI-formality markers ("It is important to note", "In today's digital age") present?

If any answer is yes, rewrite the offending paragraph before delivery.

---

## Linked files

- `.claude/CLAUDE.md` — Claude-native instructions for full editor workflow guidance.
- `.claude-plugin/.mcp.json` — Claude plugin MCP server config for the compiled `ai-slop-cleaner serve` binary.
- `.codex/instructions.md` — Codex-native concise instructions for agent consumption.
- `.mcp.json` — Skill-root MCP server config for `go run scripts/ai-slop-cleaner.go serve`.
- `.opencode/instructions.md` — OpenCode-native concise directive instructions.
- `references/banned-words.md` — Full banned-word taxonomy with plain-English replacements.
- `references/banned-patterns.md` — Structural patterns to avoid, with before/after examples.
- `references/human-checklist.md` — Printable quick-reference checklist.
- `scripts/ai-slop-cleaner.go` — Minimal Go CLI that runs detection + generates a markdown report from stdin or a file.

## Usage example

```bash
# Run the Go detector on a file
go run scripts/ai-slop-cleaner.go -input draft.md -output report.md

# Or pipe stdin
cat draft.md | go run scripts/ai-slop-cleaner.go -output report.md
```

### `.slopignore` support

Create a `.slopignore` file (local or in `$HOME`) to skip lines that match regex patterns. Useful for reference docs, tables, or code blocks.

Example `.slopignore`:

```
# Ignore markdown tables
^\|.*\|.*\|$

# Ignore code blocks
^```
```

Then use the report to guide Stage 2 rewriting manually, or feed the JSON findings into your own automation.

## CLI notes & gotchas

- **Stdin pipes work.** You can pipe text directly: `cat draft.md | go run scripts/ai-slop-cleaner.go -output report.md`
- **Reference files self-flag.** Running the detector on `references/banned-words.md` or `SKILL.md` itself will produce false positives because those files *list* the banned words by design. Scan only actual drafts.
- **Metric in the report:** The CLI prints `Sentence length std dev` (standard deviation), not variance. A low std-dev (< 8) on a document with average length > 15 words is a strong signal of monotonous rhythm.
- **Build requirements:** Go 1.20+ is sufficient. The script has no external dependencies.
