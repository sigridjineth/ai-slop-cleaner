# im-not-ai and H.U.M.A.N. Coverage Audit

Date: 2026-04-26  
Audited external source: `epoko77-ai/im-not-ai` at commit `3fab1d8451170f10270b3168a97dd05fca010f6f`  
Primary files read: `README.md`, `CLAUDE.md`, `.claude/agents/ai-tell-detector.md`, `.claude/agents/humanize-monolith.md`, `.claude/agents/naturalness-reviewer.md`, `.claude/skills/humanize-korean/SKILL.md`, `.claude/skills/humanize-korean/references/ai-tell-taxonomy.md`, `.claude/skills/humanize-korean/references/quick-rules.md`, `.claude/skills/humanize-korean/references/rewriting-playbook.md`.

## Inventory

### `src/ai_slop_cleaner/`

Audited source covered the package entry files, `cli.py`, the `core/` modules
for banned patterns, banned words, detector, fallback, and scorer logic, the
`mcp/` server and tool modules, and the packaged reference files for the agent
prompt, response schema, banned patterns, and banned words.

Ignored Python bytecode caches were present locally but are not implementation source files.

### `references/`

Audited references covered the agent-driven spec, agent prompt template,
response schema, banned pattern and word references, human checklist, this
audit, and the slop-score spec.

## Summary

The audit extracted 60 im-not-ai taxonomy rules. Coverage after the patch is 60
of 60, with 0 missing rules. Direct regex examples account for 57 rules
(`ko_A1_*` through `ko_J4_*`, excluding statistical E rules), while RHY covers
the 3 statistical/document-level rules (`E-1`, `E-2`, `E-3`).

## Comparison matrix

| Rule | im-not-ai heuristic | Implementation status |
|---|---|---|
| A-1 | `~에 대해(서)/대하여` translationese | Covered by `ko_A1_about_regarding` (SPV) |
| A-2 | `~를 통해/통하여` through/via calque | Covered by `ko_A2_through` (SPV) |
| A-3 | `~에 있어(서)` in-terms-of calque | Covered by `ko_A3_in_terms` (SPV) |
| A-4 | `~라는/다는 점에서` | Covered by `ko_A4_point_sense` (SPV) |
| A-5 | `~와/과 관련하여/관련된` | Covered by `ko_A5_related` (SPV) |
| A-6 | `~에 기반하여`, `~을 바탕으로` | Covered by `ko_A6_based_on` (SPV) |
| A-7 | `가지고 있다` | Covered by `ko_A7_have` (SPV) |
| A-8 | double passive `~되어진다/~지게 된다` | Covered by `ko_A8_double_passive` (SPV) |
| A-9 | by-passive `~에 의해` | Covered by `ko_A9_by_passive` (SPV) |
| A-10 | repeated can-form `~할 수 있다` | Covered by `ko_A10_can_overuse` (SPV) |
| A-11 | purpose `~을/를 위해` | Covered by `ko_A11_purpose` (SPV) |
| A-12 | `만들어지다/이루어지다` automated passive | Covered by `ko_A12_automated_passive` (SPV) |
| A-13 | English-like Korean noun stack | Covered by `ko_A13_noun_stack` (SPV) |
| A-14 | sentence-initial `그리고` | Covered by `ko_A14_and_sentence` (SPV) |
| A-15 | abstract subject with generic verb (`보여준다/제공한다/...`) | Covered by `ko_A15_abstract_subject` (SPV) |
| B-1 | Korean term with English parenthetical | Covered by `ko_B1_parenthesized_english` (SPV) |
| B-2 | untranslated buzzwords (`framework`, `pipeline`, etc.) | Covered by `ko_B2_raw_english_term` (SPV) and banned words |
| B-3 | long English quote embedded in Korean prose | Covered by `ko_B3_long_english_quote` (SPV) |
| B-4 | `~라고 알려진/~로 일컬어지는` | Covered by `ko_B4_known_as` (SPV) |
| C-1 | mechanical `첫째/둘째/셋째` | Covered by `ko_C1_mechanical_enumeration` plus numbered-sequence detector |
| C-2 | 3+ bullet block | Covered by `ko_C2_bullet_block` and MD bullet ratio |
| C-3 | generic `도입/본론/결론` headings | Covered by `ko_C3_generic_headings` and MD heading ratio |
| C-4 | paragraph-first topic sentence formula | Covered by `ko_C4_topic_sentence` (SPV) |
| C-5 | emoji decoration | Covered by `ko_C5_emoji` (SPV) |
| C-6 | heading followed by `이 섹션에서는...` summary | Covered by `ko_C6_heading_summary` and META regexes |
| C-7 | `먼저/반면/결국` three-step formula | Covered by `ko_C7_three_step` and Korean connector monotony |
| C-8 | `A인가, B인가` binary parallelism | Covered by `ko_C8_binary_parallel` (SPV) |
| C-9 | `1) 2) 3)` numeric indexing | Covered by `ko_C9_numeric_index` plus numbered-sequence detector |
| C-10 | colon-subtitle heading `X: Y` | Covered by `ko_C10_colon_heading` (SPV) |
| D-1 | conclusion/summary formulas | Covered by `ko_D1_conclusion_phrase`, META regexes, and banned words |
| D-2 | inflated importance clichés | Covered by `ko_D2_importance_cliche` and banned words |
| D-3 | list-intro clichés | Covered by `ko_D3_list_intro` and META regexes |
| D-4 | hype words/formulas | Covered by `ko_D4_hype` and banned words |
| D-5 | personified abstract subject | Covered by `ko_D5_personified_abstract` (SPV) |
| D-6 | formulaic closing `~할 때/시점` | Covered by `ko_D6_closing_formula` (SPV) |
| D-7 | transformation formula `X에서 Y로` | Covered by `ko_D7_transformation_formula` (SPV) |
| E-1 | low sentence-length variation | Covered by RHY coefficient-of-variation penalty |
| E-2 | repeated Korean sentence endings | Covered by Korean ending keys in RHY repeated-ending rate |
| E-3 | uniform 3-4 sentence paragraph mold | Covered by paragraph-length CV penalty in RHY |
| F-1 | degree adverbs from the Korean banned-word list | Covered by `ko_F1_degree_adverb` and banned words |
| F-2 | synonym double modifiers | Covered by `ko_F2_double_modifier` (SPV) |
| F-3 | `역할과 기능`, `의미와 가치` doublets | Covered by `ko_F3_role_function` (SPV) |
| F-4 | `~적 측면/관점`, `~성`, `~화` overuse | Covered by `ko_F4_suffix_abuse` (SPV) |
| F-5 | `~적 N` abstract chain | Covered by `ko_F5_jeok_chain` (SPV) |
| G-1 | hedge endings (`~것으로 보인다`, etc.) | Covered by `ko_G1_hedging` (SPV) |
| G-2 | double/triple hedges | Covered by `ko_G2_double_hedge` (SPV) |
| H-1 | sentence-initial connector overload | Covered by `ko_H1_connector`, Korean connector monotony, RHY opener repetition |
| H-2 | repeated `하지만/그러나` | Covered by `ko_H2_contrast_connector` and connector monotony |
| H-3 | `이는`, `이 점에서`, `이 관점에서`, `이 말은` meta-entry | Covered by `ko_H3_meta_entry` (SPV/META adjacency) |
| H-4 | `즉` redefinition connector | Covered by `ko_H4_ie` and connector monotony |
| I-1 | `~것이다` ending | Covered by `ko_I1_geotida` and RHY Korean ending repetition |
| I-2 | `점/바/수/데` dependent nouns | Covered by `ko_I2_dependent_noun` (SPV) |
| I-3 | `~다는 뜻이다/~다는 의미다` | Covered by `ko_I3_means_ending` (SPV) |
| I-4 | `~할 필요가 있다`, repeated `~해야 한다/합니다` | Covered by `ko_I4_need_to` (SPV) |
| I-5 | abstract `~이/가 필요하다` | Covered by `ko_I5_needed` (SPV) |
| I-6 | `N 능력` noun chain | Covered by `ko_I6_ability_noun` (SPV) |
| J-1 | bold emphasis overuse | Covered by `ko_J1_bold` and MD emphasis density |
| J-2 | quote-emphasis overuse | Covered by `ko_J2_quote_emphasis` and MD emphasis density |
| J-3 | em dash overuse | Covered by `ko_J3_em_dash` and MD emphasis density |
| J-4 | parenthetical explanatory aside | Covered by `ko_J4_parenthetical_aside` (SPV) |

## H.U.M.A.N. Framework coverage

| Element | Detection/diagnostic coverage |
|---|---|
| H: Honest human flaws | `doc_patterns.human_framework.H_honest_human_flaws` counts markers such as `솔직히 말하면`, `to be fair`, `honestly`; over-polished absence is interpreted alongside RHY/SPV rather than as standalone authorship proof. |
| U: Unpredictable structure | RHY sentence/paragraph variation, SPV repeated templates, connector monotony, section-mold detection, and MD structure ratios. |
| M: Memorable specifics | `doc_patterns.human_framework.M_memorable_specifics` counts numbers/dates/example markers such as `예를 들어`; the score does not penalize every low-specificity text because genre context matters. |
| A: Authentic perspective | `doc_patterns.human_framework.A_authentic_perspective` counts markers such as `제 경험으로는`, `개인적으로`, `in my experience`. |
| N: Natural flow | SPV connector monotony, RHY repeated openers, and `doc_patterns.human_framework.N_natural_flow` conversational connector counts. |

## Regex validation

Representative examples for all direct im-not-ai patterns are in `tests/test_im_not_ai_coverage.py`. The test suite asserts that every `ko_*` regex matches its intended real example and that the three statistical E rules feed RHY.

## Score formula adequacy

The existing five-component formula remains adequate for triage because every
im-not-ai category maps into an existing component. A/B/D/F/G/H/I map mostly to
`SPV`, with high-confidence lexical items also contributing to `BWD`/`META`.
C/J maps to `SPV` plus `MD`, and E maps to `RHY`. H.U.M.A.N. positive signals
are reported in `doc_patterns` instead of adding a sixth score component, which
preserves the published score formula and avoids genre-biased false positives.

The audit also aligned `META` to the documented `/6` density cap and upgraded `MD` to the documented bullet/heading/table/emphasis weighted formula.

## Known false-positive / false-negative considerations

Some Korean patterns are intentionally broad (`~해야 한다`, quote emphasis,
`~적 N`). They are useful triage signals, not authorship proof; severity and
density should guide action. H.U.M.A.N. positive markers are diagnostics only,
so a legal brief or API reference may not need personal perspective or honest
caveats. Span offsets in fallback are based on `clean_text` after
code/front-matter stripping; agent paths can still provide original offsets when
available.
