<p align="right">
  <strong>English</strong> | <a href="./README.ko.md">한국어</a>
</p>

# AI Slop Cleaner

AI Slop Cleaner finds generated-looking prose patterns without pretending that a
regex can understand every language. The Rust CLI reports raw structural matches.
LLM agents use those matches, the full text, and universal category descriptions
to judge intent and rewrite naturally.

The important rule is simple: detection can use structural regex evidence, but
deslop rewriting must be full-text LLM inference. Never clean prose by regex
substitution.

## Current architecture

| Layer | Role |
| :---- | :--- |
| Rust `analyze` | Fast structural scan for language-agnostic formatting and cadence evidence. |
| `patterns-agent.md` | Universal semantic categories that an LLM judges by intent in any language. |
| `scripts/ralph.py` | Iterative loop: analyze the full text, ask an LLM for a complete rewrite, then re-analyze. |
| `scripts/omx-delegate.sh` | Packages the same analyze evidence for OMX or manual agent delegation. |

The Rust binary does not compute an overall slop score. It returns raw
`MatchResult` JSON from `analyze` so an agent can use the evidence without being
trapped by a fake numeric certainty.

## Quick start

Build the CLI:

```bash
cd rust
cargo build --release
```

Analyze a file:

```bash
./target/release/ai-slop-cleaner --rules-dir rules analyze ../README.md
```

From the repository root, the same command is:

```bash
./rust/target/release/ai-slop-cleaner -r rust/rules analyze README.md
```

Pipe text from stdin:

```bash
cat draft.md | ./rust/target/release/ai-slop-cleaner -r rust/rules stdin --format json
```

List loaded rules:

```bash
./rust/target/release/ai-slop-cleaner -r rust/rules rules
```

The old `score` subcommand remains for compatibility, but it is deprecated. It
prints raw structural matches and a warning. New workflows should use `analyze`.

## Universal pattern categories

`rust/rules/patterns-agent.md` no longer contains legacy language-prefixed
pattern IDs. It defines universal categories instead.
Each category explains the intent that makes text feel generated. Multilingual
examples are illustrations only, not trigger phrases.

| Category | What the LLM judges |
| :------- | :------------------ |
| Translationese | Calqued grammar, awkward passives, literal connectors, imported word order. |
| Structural Monotony | Repeated document molds, field dumps, over-numbered lists, same section shape. |
| Cliche/Formulaic | Stock conclusions, importance inflation, generic hype, redefinition formulas. |
| Rhythm | Repeated openers, endings, sentence skeletons, or paragraph cadence. |
| Modifier Abuse | Stacked intensifiers and vague praise that add emphasis without precision. |
| Hedging | Repeated possibility or capability markers that avoid a clear claim. |
| Meta-Commentary | Narrating the explanation instead of directly explaining the subject. |
| Connector Abuse | Mechanical transitions, forced contrast, and symbol-style conjunctions. |
| Dependency Clause Overuse | Clause stacks that delay the main point and make prose bureaucratic. |

The detection rule is the same for every category: does the text exhibit this
intent in its own language, genre, and context?

## LLM-inference deslop loop

`scripts/ralph.py` performs cleanup without regex replacement:

```bash
python3 scripts/ralph.py draft.md --max-rounds 3 --target-matches 0
python3 scripts/ralph.py draft.md --force-rewrite  # semantic-only slop
```

Each round does the following:

1. Run `ai-slop-cleaner analyze` on the current full text.
2. Read `patterns-agent.md` universal categories.
3. Send the LLM the full text, all structural matches, and the category catalog.
4. Ask for a complete rewritten text, not patches or substitutions.
5. Re-run `analyze` on the rewritten text.
6. Stop when the structural match target is reached or the round limit is hit.
   With `--force-rewrite`, run at least one LLM rewrite even when structural
   analysis finds zero matches. Use this for semantic-only slop such as
   translationese or connector abuse.

The prompt explicitly warns against destructive cleanup such as removing `즉`
from `즉흥적으로` or chopping `느낌입니다` into `다`. Connector-looking text
inside a word is part of that word.

LLM backend priority is `claude -p`, `AICHAT_MODEL`, `OPENAI_API_KEY`, then
`ANTHROPIC_API_KEY`. The `claude -p` path reuses Claude Code authentication, so
it works inside the skill without separate API keys. If none are available,
Ralph writes `.ralph/round-N-prompt.md` and waits for you to paste the complete
rewrite into `.ralph/round-N-response.md`.

## Agent prompt contract

`references/agent-prompt-template.md` describes the holistic judge prompt. The
agent receives:

- the full source text;
- raw structural matches from `analyze`;
- the universal categories from `patterns-agent.md`;
- the requirement to rewrite the complete text by inference.

The score formula is retained for LLM calibration only:

```text
AI_SLOP_SCORE = round(100 * (
    0.25 * BWD +
    0.25 * SPV +
    0.20 * RHY +
    0.15 * META +
    0.15 * MD
))
```

Components are banned word or canned phrase density (`BWD`), structural pattern
violations (`SPV`), rhythm monotony (`RHY`), meta-commentary density (`META`),
and markdown or formatting overuse (`MD`). The Rust binary does not compute this
score.

## Repository layout

```text
rust/
  src/main.rs              CLI entrypoint for analyze, stdin, rules, and deprecated score.
  src/scorer.rs            Returns raw structural MatchResult values.
  src/pattern_loader.rs    Loads markdown rule files.
  rules/banned-patterns.md Structural regex evidence, all universal scope.
  rules/banned-words.md    Reference vocabulary for agents and docs.
  rules/patterns-agent.md  Universal LLM categories with multilingual examples.

scripts/
  ralph.py                 Standalone full-text rewrite loop.
  omx-delegate.sh          OMX context generator using analyze output.

references/
  agent-prompt-template.md Holistic judge and rewrite prompt.
  slop-score-spec.md       Five-component scoring model for agent calibration.
```

## Development

Run the Rust checks:

```bash
cd rust
cargo test
cargo build --release
./target/release/ai-slop-cleaner -r rules analyze ../README.md
```

Script syntax checks:

```bash
python3 -m py_compile scripts/ralph.py
bash -n scripts/omx-delegate.sh
```

## Safety rules for contributors

- Do not add regex-based rewrite rules.
- Do not add language-specific semantic pattern IDs.
- Keep structural regex patterns language-agnostic.
- Treat examples as illustrations, not match lists.
- Preserve meaning and grammar when rewriting.
- Never cut substrings out of compound words.
