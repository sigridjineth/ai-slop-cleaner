# AI Slop Score Specification

Version: 0.1.0  
Status: draft for `ai-slop-cleaner` implementation  
Scale: `0` means no detected AI-slop signals; `100` means dense, repeated, high-confidence slop signals.

## Source model: Ouroboros ambiguity scoring

This score borrows the shape of Ouroboros's ambiguity gate, not its subject matter. Ouroboros measures readiness by scoring several clarity dimensions from `0.0` to `1.0`, weighting them, then computing ambiguity as the inverse of weighted clarity:

```text
Ambiguity = 1 - Sum(clarity_i * weight_i)
```

In Ouroboros, a Seed can be generated only when ambiguity is `<= 0.2`, meaning roughly 80% weighted clarity. The implementation uses fixed weights, clamps component scores to `0.0..1.0`, and rounds the final ambiguity value.

The AI Slop Score adapts that pattern into a deterministic smell score:

```text
AI Slop Score = round(100 * Sum(component_slop_i * weight_i))
```

Each component is a normalized slop signal in `0.0..1.0`. Higher is worse.

Research references:

- Ouroboros README, ambiguity formula, weights, and `<= 0.2` gate: https://github.com/Q00/ouroboros/blob/99de3220ff718724752d0b41c46f372c818b4436/README.md#ambiguity-score-the-gate-between-wonder-and-code
- Ouroboros scoring implementation: https://github.com/Q00/ouroboros/blob/99de3220ff718724752d0b41c46f372c818b4436/src/ouroboros/bigbang/ambiguity.py

## Scope

The score estimates how strongly a prose document matches recurring AI-output tells. It is not an authorship detector and must not be reported as proof that a human or model wrote the text. It is an editing triage score for deciding how much cleanup is needed.

The required components are:

1. Banned word density
2. Structural pattern violations
3. Rhythm monotony
4. Meta commentary density
5. Markdown overuse

## Preprocessing

Before scoring, produce a `clean_text` view and a line-index map back to the original document.

1. Remove fenced code blocks from `clean_text`.
2. Remove inline code spans from `clean_text`, but keep their character ranges for reporting.
3. Remove YAML/TOML front matter from `clean_text`.
4. Keep block quotes in scope unless the caller explicitly marks them as quoted source material.
5. Honor `.slopignore` regular expressions before creating findings.
6. Case-fold for matching. Preserve original text in reported spans.
7. Count words with Unicode-aware word tokens: runs of letters, numbers, or apostrophes inside words.
8. Split sentences on `.`, `!`, `?`, and paragraph boundaries. Do not score rhythm when fewer than 5 sentences remain.

Definitions:

```text
W = max(word_count(clean_text), 1)
S = sentence_count(clean_text)
L = max(nonblank_prose_line_count_outside_fenced_code, 1)
clip01(x) = min(max(x, 0), 1)
```

All per-1,000-word densities use `W`.

## Overall formula

```text
AI_SLOP_SCORE = round(100 * (
    0.25 * BWD +
    0.25 * SPV +
    0.20 * RHY +
    0.15 * META +
    0.15 * MD
))
```

Where:

- `BWD` = banned word density subscore
- `SPV` = structural pattern violation subscore
- `RHY` = rhythm monotony subscore
- `META` = meta commentary density subscore
- `MD` = markdown overuse subscore

Round only once, at the final score. Component values should be stored to 4 decimal places for repeatable reports.

## Component formulas

### 1. Banned word density (`BWD`, weight 25%)

Use `references/banned-words.md` as the canonical list. Treat banned phrases as stronger than single words because they usually mark a full canned construction.

```text
weighted_banned_hits = single_word_hits + (2 * banned_phrase_hits)
banned_density = 1000 * weighted_banned_hits / W
BWD = clip01(banned_density / 16)
```

Thresholds:

- `0` hits: clean
- `>0` and `<2` weighted hits per 1,000 words: trace signal
- `2..4.99`: light signal
- `5..7.99`: moderate signal
- `8..15.99`: high signal
- `>=16`: severe signal, `BWD = 1.0`

Implementation notes:

- Match whole words for single-word entries.
- Match banned phrases case-insensitively after normalizing internal whitespace.
- Do not count a single-word hit inside an already-counted banned phrase span.
- Report replacement suggestions from `references/banned-words.md` when available.

### 2. Structural pattern violations (`SPV`, weight 25%)

Use `references/banned-patterns.md` as the canonical taxonomy. Each detected violation receives a severity weight.

Structural weights:

| Pattern | Weight |
| --- | ---: |
| `not X, it is Y` / `not X but Y` redefinition | 2.0 |
| Three or more consecutive sentences with the same template | 2.0 per streak |
| Three or more sections with the same heading-intro-bullets-summary mold | 2.0 per document |
| Over-numbered list or repeated `First/Second/Third` framing | 1.5 per sequence |
| Broad overview followed by field dump | 1.5 per sequence |
| Closing-summary repetition | 1.0 per occurrence |
| Connector monotony across paragraphs | 1.0 per repeated connector group |
| Progress announcement (`let's unpack`, `we'll examine`, etc.) | 1.0 per occurrence |
| Pre-classification framing (`there are three types...`) | 1.0 per occurrence |
| Imagine/picture/visualize prompt | 1.0 per occurrence |
| Post-code textbook narration | 1.0 per occurrence |

Formula:

```text
weighted_structural_hits = sum(pattern_weight for each structural finding)
structural_density = 1000 * weighted_structural_hits / W
SPV = clip01(structural_density / 10)
```

Thresholds:

- `<1` weighted hit per 1,000 words: clean/trace
- `1..2.99`: light
- `3..4.99`: moderate
- `5..9.99`: high
- `>=10`: severe, `SPV = 1.0`

Implementation notes:

- A sentence-template streak starts at 3 consecutive sentences with the same normalized skeleton. A practical skeleton is the sequence of coarse token classes for the first 8 tokens: pronoun, noun, verb, adjective, adverb, determiner, preposition, conjunction, number, other.
- A connector group is repeated when the same paragraph opener appears 3 or more times in one document.
- Avoid double-counting: if a phrase is counted as meta commentary and structural progress announcement, keep both component findings but link them to the same source span.

### 3. Rhythm monotony (`RHY`, weight 20%)

Rhythm monotony captures prose that moves in flat, repeating units.

If `S < 5`, set `RHY = 0` and include `rhythm_sample_confidence = "low"` in the report.

For `S >= 5`:

```text
sentence_lengths = word counts per sentence
mean_len = mean(sentence_lengths)
sd_len = population_standard_deviation(sentence_lengths)
cv = sd_len / max(mean_len, 1)

cv_penalty = clip01((0.55 - cv) / 0.35)
if mean_len < 8:
    cv_penalty = 0.5 * cv_penalty

opener_repeat_rate = sentences_with_an_opener_used_3plus_times / S
opener_penalty = clip01((opener_repeat_rate - 0.15) / 0.25)

ending_repeat_rate = sentences_with_a_repeated_last_two_content_words / S
ending_penalty = clip01((ending_repeat_rate - 0.15) / 0.25)

max_template_streak = longest run of adjacent sentences with the same skeleton
template_streak_penalty = clip01((max_template_streak - 2) / 4)

RHY = (0.50 * cv_penalty) +
      (0.20 * opener_penalty) +
      (0.20 * ending_penalty) +
      (0.10 * template_streak_penalty)
```

Thresholds:

- `cv >= 0.55`: sentence lengths are varied enough; no CV penalty
- `cv ~= 0.375`: moderate CV penalty
- `cv <= 0.20`: severe CV penalty
- opener or ending repeat rate `<=15%`: no penalty
- opener or ending repeat rate `>=40%`: severe penalty
- max same-template streak `<=2`: no streak penalty
- max same-template streak `>=6`: severe streak penalty

Implementation notes:

- Content words exclude common function words: `a`, `an`, `the`, `of`, `to`, `in`, `for`, `with`, `and`, `or`, `but`, `is`, `are`, `was`, `were`.
- Repeated endings should use the last two content words when available; otherwise use the final word.
- Rhythm findings are document-level findings. They may not map to a single span.

### 4. Meta commentary density (`META`, weight 15%)

Meta commentary tells the reader what the text is about to do instead of doing it.

Canonical examples include:

- `In this section...`
- `Let's dive in`
- `Let's explore`
- `Let's unpack this`
- `Here's what you need to know`
- `The key takeaway here is`
- `To summarize`
- `In conclusion`
- `It is important to note`

Formula:

```text
meta_density = 1000 * meta_commentary_hits / W
META = clip01(meta_density / 6)
```

Thresholds:

- `0` hits: clean
- `<1` hit per 1,000 words: trace
- `1..1.99`: light
- `2..3.32`: moderate
- `3.33..5.99`: high; this matches the existing red flag of more than 1 hit per 300 words
- `>=6`: severe, `META = 1.0`

Implementation notes:

- Keep a canonical regular-expression list in code, backed by `references/banned-patterns.md`.
- Do not count useful navigation once at the very top of long technical docs unless the phrase appears again later. Mark this exemption in the finding record as `allowed_roadmap = true`.
- Count repeated end-of-section summaries as both meta commentary and structural repetition when both detectors match.

### 5. Markdown overuse (`MD`, weight 15%)

Markdown overuse measures formatting that turns prose into a generated-looking outline. It should be interpreted by genre: a CLI reference can tolerate more markdown than an essay. The base score must still use the same formula so reports remain comparable.

Line and span metrics:

```text
bullet_ratio = bullet_or_numbered_list_lines / L
heading_ratio = markdown_heading_lines / L
table_ratio = markdown_table_lines / L
emphasis_density = 1000 * (
    bold_spans + italic_spans + inline_code_spans + blockquote_lines +
    horizontal_rule_lines + em_dash_count
) / W
```

Penalties:

```text
bullet_penalty = clip01((bullet_ratio - 0.12) / 0.28)
heading_penalty = clip01((heading_ratio - 0.08) / 0.17)
table_penalty = clip01(table_ratio / 0.20)
emphasis_penalty = clip01((emphasis_density - 3) / 17)

MD = (0.40 * bullet_penalty) +
     (0.20 * heading_penalty) +
     (0.20 * table_penalty) +
     (0.20 * emphasis_penalty)
```

Thresholds:

- bullet ratio `<=12%`: no bullet penalty
- bullet ratio `>=40%`: severe bullet penalty
- heading ratio `<=8%`: no heading penalty
- heading ratio `>=25%`: severe heading penalty
- table ratio `>=20%`: severe table penalty
- emphasis density `<=3` spans per 1,000 words: no emphasis penalty
- emphasis density `>=20` spans per 1,000 words: severe emphasis penalty

Implementation notes:

- Detect bullet lines that start with `-`, `*`, `+`, checkbox markers, or ordered-list markers like `1.`.
- A markdown table line contains at least two pipe characters or is a separator row like `| --- | --- |`.
- Do not include fenced code block lines in `L`.
- Inline code spans count toward emphasis density because excessive inline code can produce the same chopped-up feel as bold overuse.

## Score bands and actions

| AI Slop Score | Band | Action |
| ---: | --- | --- |
| `0..9` | Clean | Accept if the final checklist passes. |
| `10..24` | Trace | Fix only explicit findings; no broad rewrite. |
| `25..44` | Noticeable | Run one Stage 2 rewrite pass on flagged spans. |
| `45..64` | Heavy | Run cleanup plus Stage 3 fidelity audit. |
| `65..84` | Severe | Rewrite from source while preserving facts; require human review if facts are dense. |
| `85..100` | Critical | Do not publish as-is. Hold and rebuild from notes or source material. |

Component guardrail: if any component subscore is `>=0.80`, require targeted cleanup for that component even when the overall score is below `25`.

## Report schema

A scorer should emit structured data before any prose report:

```json
{
  "version": "0.1.0",
  "slop_score": 37,
  "band": "noticeable",
  "word_count": 1240,
  "sentence_count": 48,
  "components": {
    "banned_word_density": {
      "weight": 0.25,
      "score": 0.3125,
      "weighted_hits": 5,
      "density_per_1000_words": 4.03
    },
    "structural_pattern_violations": {
      "weight": 0.25,
      "score": 0.4000,
      "weighted_hits": 5,
      "density_per_1000_words": 4.03
    },
    "rhythm_monotony": {
      "weight": 0.20,
      "score": 0.2200,
      "coefficient_of_variation": 0.42,
      "rhythm_sample_confidence": "normal"
    },
    "meta_commentary_density": {
      "weight": 0.15,
      "score": 0.2688,
      "hits": 2,
      "density_per_1000_words": 1.61
    },
    "markdown_overuse": {
      "weight": 0.15,
      "score": 0.5900,
      "bullet_ratio": 0.31,
      "heading_ratio": 0.10,
      "table_ratio": 0.00,
      "emphasis_density_per_1000_words": 8.87
    }
  },
  "findings": [
    {
      "span": [120, 145],
      "line": 8,
      "category": "banned_word",
      "severity": "high",
      "text": "delve into",
      "suggested_fix": "look at",
      "component": "banned_word_density"
    }
  ]
}
```

## Worked example

Given a 1,000-word article:

```text
BWD = 0.50   # 8 weighted banned hits / 1,000 words
SPV = 0.30   # 3 weighted structural hits / 1,000 words
RHY = 0.40   # low sentence-length variation and repeated openers
META = 0.50  # 3 meta hits / 1,000 words
MD = 0.25    # some list and heading overuse
```

```text
AI_SLOP_SCORE = round(100 * (
  0.25*0.50 + 0.25*0.30 + 0.20*0.40 + 0.15*0.50 + 0.15*0.25
))
= round(39.25)
= 39
```

Band: `Noticeable`; run a targeted rewrite pass.

## Calibration rules

- The first implementation should use these constants exactly.
- Any later calibration must version the spec and include before/after score distributions on a fixed fixture set.
- Add fixtures for short text, long essay prose, technical docs, release notes, and code-heavy docs.
- Do not tune thresholds against a single author's style.
- Keep the score deterministic. If an LLM reviewer is added later, store it as a separate `llm_naturalness_score`, not as part of this 0-100 score.

## Acceptance criteria for implementation

1. Same input plus same `.slopignore` always produces the same score.
2. Component scores are clamped to `0.0..1.0`.
3. Overall score is an integer in `0..100`.
4. Findings include line numbers and source spans when a local span exists.
5. Code blocks do not create banned-word, rhythm, or meta-commentary findings.
6. Rhythm reports low confidence for documents with fewer than 5 sentences.
7. Markdown overuse is still reported for markdown-heavy prose, but consumers may decide whether the genre justifies it.
8. The report includes enough raw counts to reproduce the score by hand.
