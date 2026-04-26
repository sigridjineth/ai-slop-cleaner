# Banned Structural Patterns

These structural habits are reliable AI fingerprints. Avoid them in final output.

---

## 1. "A is not X, it is Y" redefinition

AI loves to negate and redefine. Humans just say what something is.

**Bad:**
"LLM is not a truth engine; it is a probability engine."

**Good:**
"LLM is a probability engine."

**Also bad:**
"Rather than a storage warehouse, it is closer to a pattern-weaving function."

**Good:**
"It weaves patterns more than it stores facts."

---

## 2. Identical sentence-template repetition

Repeating the same grammatical frame feels mechanical.

**Bad:**
> "By the end of this chapter you will understand X.
> By the end of this chapter you will be able to Y.
> By the end of this chapter you will have learned Z."

**Good:**
> "This chapter covers X, Y, and Z."

**Bad:**
> "Section 1 discusses A.
> Section 2 discusses B.
> Section 3 discusses C."

**Good:**
> "The first section covers A. B comes next, and C wraps up the chapter."

---

## 3. Over-numbered lists

"First... Second... Third..." everywhere makes text read like a presentation deck.

**Bad:**
> "First, gather requirements. Second, design the schema. Third, write the code."

**Good:**
> "Gather requirements, design the schema, then write the code."

Reserve numbered lists for genuine sequences that would confuse readers otherwise. Even then, keep them rare.

---

## 4. Predictable paragraph architecture

If every subsection follows the same 4-step mold, the text feels templated.

**Mold to break:**
1. Subheading
2. 1–2 sentence intro
3. 3–6 bullet points
4. 1–2 sentence closing

**Vary instead:**
- Start some sections with an anecdote.
- Start others with a direct answer.
- Skip the closing sentence entirely when the point is obvious.
- Use bullets only when you have 5+ distinct items that resist prose.

---

## 5. Bullet-point overuse

Breaking every idea into bullets creates a staccato, manual-like rhythm.

**Bad:**
> - Apples are red.
> - Bananas are yellow.
> - Grapes are purple.

**Good:**
> "Apples are red, bananas yellow, grapes purple."

Fold ≤4 short items into a sentence. Reserve lists for complex items that need breathing room.

---

## 6. Meta commentary overload

Constantly telling the reader what you are doing interrupts the actual content.

**Remove:**
- "In this section we will explore..."
- "By now you should have a mental model of..."
- "The key takeaway here is..."
- "Here's what you need to know."
- "Let's break this down."

**Exception:** Brief roadmap at the very top of a long document is fine. Do not repeat it inside every section.

---

## 7. Closing-summary repetition

Re-stating what you just said drains energy.

**Bad:**
> "To summarize, we discussed A, B, and C."

**Good:**
> End with a forward-looking question or a single crisp observation.

If a summary is mandatory (e.g., executive summary), write it once at the top, not again at the bottom.

---

## 8. Connector monotony

Starting every paragraph with the same word or phrase is hypnotic in a bad way.

**Bad streak:**
> "Therefore... Therefore... As a result... Consequently... Therefore..."

**Fix:**
Drop the connector and start with the subject.

> "The server crashed. We restarted it. Traffic normalized within minutes."

---

## 9. Identical sentence endings

If every sentence ends with "~is possible" or "~is essential," the rhythm flatlines.

**Bad:**
> "Optimization is essential. Testing is essential. Documentation is essential."

**Good:**
> "Optimization matters. So does testing. Don't ship without documentation either."

Mix endings: statements, fragments, questions, imperatives.

---

## 10. Table and diagram prohibition

No markdown tables, comparison charts, or ASCII diagrams in the finished prose.

**Bad:**
> | Feature | A | B |
> |---------|---|---|
> | Speed   | Fast | Slow |

**Good:**
> "A is faster than B."

---

## 11. "You" overload

Addressing the reader as "you" too often sounds like a tutorial video script.

**Bad:**
> "You need to configure the port. You then restart the service. You can verify it works by..."

**Good:**
> "Configure the port, restart the service, and verify with curl."

Use "you" sparingly — once or twice per section at most.

---

## 12. Post-code textbook narration

Explaining every line of code after showing it is an AI hallmark.

**Bad:**
> "First we define the DOCUMENTS list. Then when a question arrives we..."

**Good:**
> Put necessary context in code comments. Outside the block, one sentence of high-level purpose is enough:
> "This snippet matches a question against documents and returns the best fit."

---

## 13. "Broad overview → field list" pattern

Introducing a structure and immediately colon-listing fields feels like auto-generated docs.

**Bad:**
> "Each step usually contains these fields: step_index: the step number..."

**Good:**
> "A step tracks its number, description, and outcome."

Weave field names into natural sentences. Never dump them after a colon.

---

## 13a. Colon enumeration (comma-list after colon)

A sentence ending with a colon followed by a comma-separated list of items is a strong AI signal.

**Bad:**
> "It follows the same pattern as Ouroboros: Python package, pyproject.toml, FastMCP server, MCP tools."

**Good:**
> "It mirrors Ouroboros — a Python package with pyproject.toml, a FastMCP server, and MCP tools."

**Also bad:**
> "The stack includes: React, TypeScript, and Tailwind."

**Good:**
> "The stack uses React, TypeScript, and Tailwind."

If the list is short, fold it into the sentence. If the list is long, use a bullet list with full sentences, not comma-separated fragments.

---

## 14. Pre-classification framing

"There are three types of..." sets up a taxonomy before the reader cares.

**Bad:**
> "We can divide this into three branches."

**Good:**
> Jump straight into the first item. Let the structure emerge organically.

---

## 15. Progress announcements

Phrases that announce an inspection feel procedural and hollow.

**Remove:**
- "Let's take a closer look."
- "We'll examine this step by step."
- "Let's unpack this."
- "Let's dig deeper."

Just show the content. The reader will look closer if it's interesting.

---

## 16. Inline one-sentence summaries

Compressing already-clear content into a "bottom line" sentence adds bloat.

**Bad:**
> "In short, the lesson is X."
> "Put simply, this means Y."
> "At the end of the day, it's Z."

If the preceding paragraph is clear, these add nothing. Delete them.

---

## 17. External framework dependency / comparison

Do not pivot the discussion to LangChain, LangGraph, AutoGen, CrewAI, etc., unless the document is explicitly a comparison of those tools. Do not use them as scaffolding to explain your own ideas.

---

## 18. "Imagine / picture / visualize" prompts

Asking readers to imagine a scene is an AI padding tactic.

**Bad:**
> "Imagine a world where..."
> "Picture a system that..."
> "Visualize the following scenario."

**Good:**
State the scenario directly.

---

## 19. "Roughly / kind of / feel" vagueness

If you do not know the exact structure, say so honestly. Do not paper over it with vague impressionism.

**Bad:**
> "The structure looks roughly like this."
> "It has a kind of layered feel."

**Good:**
> "I don't have the exact layout, but the layers seem to be presentation, logic, and data."

---

## 20. "A bit more specifically" transitions

Restating the same point with slightly more detail is classic AI recursion.

**Bad:**
> "More specifically..."
> "To be more precise..."
> "In more detail..."

**Good:**
Be specific the first time. If you need a second pass, make sure it genuinely adds new information, not just rephrasing.

---

## 21. "Typically / usually / generally" filler

These weaken statements into mush.

**Bad:**
> "Typically, a request contains these fields."

**Good:**
> "A request contains these fields."
> Or, if there are genuine exceptions: "Most requests contain these fields, except batch jobs."

---

## 17. Korean AI-tell taxonomy from im-not-ai

`epoko77-ai/im-not-ai` defines a Korean-specific AI-slop taxonomy. The fallback detector maps those rules into regex-backed structural patterns with source-stable names (`ko_A1_*` through `ko_J4_*`) plus RHY/MD document metrics where a rule is statistical rather than span-based.

### A. Translation-ese

Detect Korean calques such as `~에 대해(서)`, `~를 통해/통하여`, `~에 있어서`, `~라는 점에서`, `~와 관련하여`, `~에 기반하여`, `가지고 있다`, double passives like `~되어진다/~지게 된다`, by-passive `~에 의해`, repeated `~할 수 있다`, purpose clauses `~을 위해`, automated passives `만들어지다/이루어지다`, English-like noun stacks, sentence-initial `그리고`, and abstract subjects paired with generic verbs like `보여준다/제공한다/가져온다/시사한다`.

### B. English citation and term overuse

Detect repeated Korean+English parentheticals, untranslated buzzwords (`framework`, `pipeline`, `leverage`, `seamless`, `robust`, `scalable`, `insight`, `impact`, `holistic`), long English quotes embedded in Korean prose, and `~라고 알려진/~로 일컬어지는` known-as phrasing.

### C. Structural AI patterns

Detect mechanical `첫째/둘째/셋째` enumeration, 3+ bullet blocks, generic `도입/본론/결론` headings, topic-sentence paragraph openers, emoji decoration, heading-summary lines (`이 섹션에서는...`), `먼저/반면/결국` three-step formulas, repeated `A인가, B인가` binary parallelism, `1) 2) 3)` numeric indexing, and colon-subtitle headings such as `### 서론: ...`.

### D. Korean AI signature phrases

Detect conclusion formulas (`결론적으로`, `요약하면`, `정리하자면`, `~라고 할 수 있다`), inflated importance clichés (`시사하는 바가 크다`, `주목할 만하다`, `지평을 연다`), list-intro clichés (`크게 세 가지로 나눌 수 있다`), hype vocabulary (`혁신적인`, `압도적`, `파격적`, `폭발적`, etc.), personified abstract subjects, formulaic closings (`~할 때입니다`, `~할 시점입니다`, `지금이야말로...`), and transformation slogans (`X에서 Y로`, `X을 넘어 Y로`).

### E. Rhythm and paragraph uniformity

Detect low sentence-length variation, repeated Korean sentence endings (`~이다`, `~한다`, `~것이다`, `~합니다`), repeated sentence/paragraph openers, and overly uniform paragraph lengths through the RHY component.

### F. Modifier and abstraction overload

Detect degree adverbs (`매우`, `정말`, `대단히`), synonym double modifiers (`중요하고 핵심적인`, `새롭고 혁신적인`), role/function doublets, abstract suffix overuse (`~적 측면`, `~성`, `~화`), and `~적 N` abstract chains.

### G. Hedging overload

Detect Korean hedge endings such as `~할 수 있을 것으로 보인다`, `~인 것으로 판단된다`, `~라고 여겨진다`, `~인 듯하다`, plus double hedges like `가능성이 있을 수 있다` and `보여질 수 있다`.

### H. Connector overload

Detect sentence-initial `또한`, `따라서`, `즉`, `나아가`, `아울러`, `게다가`, `더욱이`, repeated `하지만/그러나`, meta-entry phrases (`이는 ~을 의미한다`, `이 점에서`, `이 관점에서 보면`, `이 말은`), and `즉` as an overused redefinition connector.

### I. Formal/dependent noun overload

Detect `~것이다`, `~라는 점에 있다`, `점/바/수/데` crutches, `~다는 뜻이다/~다는 의미다`, `~할 필요가 있다`, repeated `~해야 한다/~해야 합니다`, abstract `~이 필요하다`, and `N 능력` noun chains.

### J. Visual decoration overload

Detect excessive bold, quote-emphasis, em dashes, and parenthetical explanatory asides. These feed the markdown-overuse component and, when span-specific, structural findings.
