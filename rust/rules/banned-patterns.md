# Banned Patterns

Dynamically loaded structural patterns for AI slop detection.
Loaded by the Rust binary at runtime from `rules/banned-patterns.md`.

## English Structural Patterns

| Name | Lang Scope | Severity | Weight | Regex | Description |
|------|------------|----------|--------|-------|-------------|
| redefinition | english | medium | 2.0 | `(?i)(?:\bnot\s+\w+(?:\s+\w+){0,2}\b|rather\s+than\s+\w+(?:\s+\w+){0,2}|instead\s+of\s+\w+(?:\s+\w+){0,2})\s*[,;:]+\s*\b(?:it\s+is|is|are)\b|\bnot\s+\w+(?:\s+\w+){0,2}\s+but\s+\w+` | A is not X, it is Y / not X but Y redefinition. |
| closing_summary | english | low | 1.0 | `(?i)\b(to summarize|in summary|in conclusion|in closing|in short|put simply|at the end of the day)\b` | Closing-summary repetition. |
| progress_announcement | english | medium | 1.0 | `(?i)\b(let's\s+(?:take a closer look|unpack(?: this)?|dig deeper|break this down|dive in|explore)|we(?:'ll| will)\s+examine(?: this)?(?: step by step)?|step by step|one by one)\b` | Progress announcement. |
| pre_classification | english | low | 1.0 | `(?i)\b(?:there are|there is|we can divide|this divides|can be divided)\b[^.!?]{0,80}\b(?:types|branches|categories|groups|kinds)\b` | Pre-classification framing. |
| imagine_prompt | english | low | 1.0 | `(?i)\b(imagine|picture|visualize)\b[^.!?]{0,80}` | Imagine/picture/visualize prompt. |
| broad_field_dump | english | medium | 1.5 | `(?is)\b(?:fields|properties|attributes|schema|structure)\b[^.\n]{0,120}:\s*(?:\n\s*(?:[-*+]\s*)?`?[\w-]+`?\s*:[^\n]*){2,}` | Broad overview followed by a field dump. |
| colon_enumeration | english | medium | 1.5 | `(?i)\b[a-z][\w-]*(?:\s+[a-z][\w-]*){0,6}\s*[:;]\s*`?[\w-]+`?(?:\s*,\s*(?:and\s+)?`?[\w-]+`?){1,}` | Colon followed by comma-separated items. |
| a_not_b_redefinition | english | high | 2.5 | `(?i)\b(?:is|are|was|were)\s+not\s+[a-z]+(?:\s+[a-z]+){0,3}\s*[,;]\s*(?:it\s+)?(?:is|are|was|were)\b|\bnot\s+[a-z]+(?:\s+[a-z]+){0,3}\s+but\s+(?:is\s+)?[a-z]+` | A is not X, it is Y redefinition pattern. |
| a_not_b_korean | korean | high | 2.5 | `(?i)[\uac00-\ud7a3]+\s*(?:이|가)\s+아니(?:라|고)\b` | Korean A가 아니다 redefinition pattern. |
| a_not_b_english | english | high | 2.5 | `(?i)\bis\s+not\s+just\s+[a-z\s]+(?:,\s*it\s+is|[;.]\s*it\s+goes\b)` | English "is not just X" redefinition. |

## Korean Structural Patterns

| Name | Lang Scope | Severity | Weight | Regex | Description |
|------|------------|----------|--------|-------|-------------|
| ko_A1_about_regarding | korean | high | 2.0 | `[가-힣A-Za-z0-9]+\s*에\s*대(?:해(?:서)?|하여)` | Korean '~에 대해/대하여' translationese. |
| ko_A2_through | korean | high | 2.0 | `[가-힣A-Za-z0-9]+\s*(?:을|를)\s*통(?:해|하여)` | Korean '~를 통해/통하여' translationese. |
| ko_A3_in_terms | korean | high | 2.0 | `[가-힣A-Za-z0-9]+\s*에\s*있어(?:서)?` | Korean '~에 있어서' translationese. |
| ko_A4_point_sense | korean | medium | 1.0 | `(?:라는|다는)\s+점에서` | '~라는 점에서' reasoning crutch. |
| ko_A5_related | korean | medium | 1.0 | `[가-힣A-Za-z0-9]+\s*(?:와|과)\s*관련(?:하여|한|된)` | '~와 관련하여/관련된'. |
| ko_A6_based_on | korean | medium | 1.0 | `[가-힣A-Za-z0-9]+\s*에\s*기반(?:하여|한)|[가-힣A-Za-z0-9]+\s*(?:을|를)\s*바탕으로` | '~에 기반하여/바탕으로'. |
| ko_A7_have | korean | high | 2.0 | `가지고\s+있(?:다|습니다|는)` | '가지고 있다' possessive translationese. |
| ko_A8_double_passive | korean | high | 2.0 | `[가-힣]+(?:되어진|되어진다|지게\s+된|지게\s+된다|보여질\s+수\s+있)` | double passive. |
| ko_A9_by_passive | korean | medium | 1.0 | `[가-힣A-Za-z0-9]+\s*에\s*의해` | '~에 의해' by-passive. |
| ko_A10_can_overuse | korean | medium | 1.0 | `[가-힣]+(?:할|될|릴|울|낼|킬|일)\s+수\s+있(?:다|습니다|는|을)` | Korean can-form '~할 수 있다'. |
| ko_A11_purpose | korean | medium | 1.0 | `[가-힣A-Za-z0-9]+\s*(?:을|를)\s*위해(?:서)?` | '~을 위해' purpose clause. |
| ko_A12_automated_passive | korean | medium | 1.0 | `(?:만들어지(?:다|고|며|는|었다|었습니다)?|이루어지(?:다|고|며|는)?|이루어졌(?:다|습니다)?)` | '만들어지다/이루어지다'. |
| ko_A13_noun_stack | korean | medium | 1.0 | `(?:[가-힣A-Za-z0-9]+\s+){2,}[가-힣A-Za-z0-9]+\s*(?:가속화|고도화|최적화|강화|확대|발전|전환)` | English-like Korean noun stack. |
| ko_A14_and_sentence | korean | medium | 1.0 | `(?m)^\s*그리고\b` | sentence-initial '그리고'. |
| ko_A15_abstract_subject | korean | medium | 1.5 | `[가-힣A-Za-z0-9'""\s]{1,40}(?:은|는|이|가)\s+[^.!?\n]{0,50}(?:보여준(?:다|다\.|다는|습니다)|제공(?:한다|합니다)|가져온(?:다|다는)|시사(?:한다|합니다)|흔들고\s+있(?:다|습니다))` | abstract subject plus all-purpose verb. |
| ko_B1_parenthesized_english | korean | medium | 1.0 | `[가-힣]{2,}\s*\([A-Za-z][A-Za-z0-9 ._/-]{1,40}\)` | Korean term with English parenthetical. |
| ko_B2_raw_english_term | universal | medium | 1.0 | `(?i)(?:framework|pipeline|leverage|seamless|robust|scalable|insight|impact|holistic)` | untranslated English buzzword. |
| ko_B3_long_english_quote | korean | medium | 1.0 | `[""][A-Za-z][^""\n]{20,}[""]` | long English quote in Korean prose. |
| ko_B4_known_as | korean | low | 0.5 | `(?:라고\s+알려진|로\s+일컬어지는)` | '~라고 알려진/~로 일컬어지는'. |
| ko_C1_mechanical_enumeration | korean | high | 2.0 | `(?s)첫째.{0,240}둘째.{0,240}셋째` | mechanical enumeration. |
| ko_C2_bullet_block | universal | medium | 1.0 | `(?m)(?:^\s*[-*+]\s+.+\n?){3,}` | excessive bullet block. |
| ko_C3_generic_headings | korean | medium | 1.0 | `(?m)^\s{0,3}#{1,6}\s*(?:도입|서론|본론|결론)\b` | generic headings. |
| ko_C4_topic_sentence | korean | medium | 1.0 | `(?m)^\s*(?:핵심은|중요한\s+(?:점|것)은|요지는|주목할\s+점은|이\s+문단은)` | topic sentence formula. |
| ko_C5_emoji | universal | high | 2.0 | `[✅🚀💡⚠📊✨🔥👉👍🎯]` | emoji decoration in prose. |
| ko_C6_heading_summary | korean | medium | 1.0 | `(?:이\s+섹션에서는|이\s+장에서는|이번\s+장에서는|이\s+글에서는)\s+[^.!?\n]{0,80}(?:다룬|살펴본|소개한)` | heading followed by summary. |
| ko_C7_three_step | korean | medium | 1.0 | `(?is)먼저.{0,300}반면.{0,300}(?:결국|마지막으로)` | 먼저/반면/결국 formula. |
| ko_C8_binary_parallel | korean | medium | 1.0 | `[가-힣A-Za-z0-9'""]+(?:인가|인가요)\s*[,·]?\s*[가-힣A-Za-z0-9'""]+(?:인가|인가요)` | A인가, B인가 parallelism. |
| ko_C9_numeric_index | korean | medium | 1.0 | `(?:^|\s)(?:1\)|\(1\))[^\n]{0,160}(?:2\)|\(2\))[^\n]{0,160}(?:3\)|\(3\))` | 1) 2) 3) numeric indexing. |
| ko_C10_colon_heading | universal | high | 2.0 | `(?m)^\s{0,3}#{1,6}\s*(?:\*\*)?[^:\n]{1,80}(?:\*\*)?\s*:\s+\S.*$` | colon subtitle heading formula. |
| ko_D1_conclusion_phrase | korean | high | 2.0 | `(?:결론적으로|요약하면|종합하면|정리하자면|정리하면|라고\s+할\s+수\s+있다|라고\s+볼\s+수\s+있다|라\s+하겠다|라\s+할\s+것이다|에\s+다름\s+아니다)` | Korean conclusion/summary formula. |
| ko_D2_importance_cliche | korean | high | 2.0 | `(?:시사하는\s+바가\s+크다|주목할\s+만하다|간과할\s+수\s+없다|무시할\s+수\s+없다|지평을\s+연다|방점을\s+찍는다|의미가\s+적지\s+않다|의미심장하다|반드시\s+기억해야\s+한다|매우\s+중요하다)` | importance cliché. |
| ko_D3_list_intro | korean | high | 2.0 | `(?:크게\s+세\s+가지로\s+나눌\s+수\s+있다|다음과\s+같은\s+특징을\s+가진다|다음과\s+같이\s+요약할\s+수\s+있다)` | list-introduction cliché. |
| ko_D4_hype | korean | high | 2.0 | `(?:혁신적인|획기적인|전례\s+없는|압도적|막강한|폭발적|파격적|대대적|강력한|치열한|뜨거운|가능성을\s+열어준다|새로운\s+장을\s+열(?:다|었다)|시대가\s+도래했다)` | hype vocabulary. |
| ko_D5_personified_abstract | korean | medium | 1.5 | `[가-힣A-Za-z0-9'""\s]{1,40}(?:은|는|이|가)\s+[^.!?\n]{0,40}(?:질문을\s+던집니다|질문을\s+던진다|끝나지\s+않습니다|증명합니다|부른다|요구한다)` | personified abstract subject. |
| ko_D6_closing_formula | korean | medium | 1.5 | `(?:해야\s+할\s+때(?:입니다|다)|나아갈\s+시점(?:입니다|이다)|할\s+순간(?:입니다|이다)|지금이야말로[^.!?\n]{0,60}(?:때|시점))` | formulaic closing. |
| ko_D7_transformation_formula | korean | medium | 1.0 | `[가-힣A-Za-z0-9'""\s]{1,40}(?:에서|을\s+넘어|를\s+넘어)\s+[가-힣A-Za-z0-9'""\s]{1,40}(?:로|으로)` | X에서 Y로 transformation slogan. |
| ko_E1_sentence_length_uniformity | korean | medium | 1.0 | `(?:[가-힣A-Za-z0-9 ,]{8,36}[.!?]\s*){5,}` | uniform Korean sentence lengths; also reflected in RHY. |
| ko_E2_repeated_sentence_endings | korean | medium | 1.0 | `(?s)(?:것이다|합니다|된다|있다)[.!?]?\s*(?:[^.!?\n]{0,50}(?:것이다|합니다|된다|있다)[.!?]?\s*){2,}` | repeated Korean sentence endings; also reflected in RHY. |
| ko_E3_uniform_paragraph_blocks | korean | low | 0.5 | `(?m)^[^\n]{20,80}$\n\n^[^\n]{20,80}$\n\n^[^\n]{20,80}$` | uniform paragraph block lengths; also reflected in RHY. |
| ko_F1_degree_adverb | korean | medium | 1.0 | `(?:매우|정말|진짜로|대단히|극히)\s+[가-힣]+` | excessive degree adverb. |
| ko_F2_double_modifier | korean | medium | 1.0 | `(?:중요하고\s+핵심적인|새롭고\s+혁신적인|지속적이고\s+꾸준한|[가-힣]+고\s+[가-힣]+적인\s+(?:역할|접근|노력|변화))` | synonym double modifier. |
| ko_F3_role_function | korean | medium | 1.0 | `(?:역할과\s+기능|의미와\s+가치)` | role/function doublet. |
| ko_F4_suffix_abuse | korean | low | 0.5 | `(?:[가-힣]+적\s+(?:측면|관점)|[가-힣]+성\b|[가-힣]+화\b)` | abstract suffix overuse. |
| ko_F5_jeok_chain | korean | medium | 1.0 | `[가-힣]+적\s+[가-힣]+` | ~적 N abstract chain. |
| ko_G1_hedging | korean | medium | 1.0 | `(?:할\s+수\s+있을\s+것으로\s+보인다|인\s+것으로\s+판단된다|라고\s+여겨진다|인\s+듯하다|것으로\s+보인다)` | Korean hedge ending. |
| ko_G2_double_hedge | korean | medium | 1.5 | `(?:가능성이\s+있을\s+수\s+있다|보여질\s+수\s+있다|할\s+수\s+있을\s+것으로\s+보일\s+수\s+있다)` | double/triple hedge. |
| ko_H1_connector | korean | medium | 1.0 | `(?m)^\s*(?:또한|따라서|즉|나아가|아울러|게다가|더욱이)\b` | sentence-initial connector. |
| ko_H2_contrast_connector | korean | medium | 1.0 | `(?m)^\s*(?:하지만|그러나)\b` | contrast connector. |
| ko_H3_meta_entry | korean | high | 1.5 | `(?:이는\s+[^.!?\n]{0,60}의미한다|이\s+점에서|이\s+관점에서\s+보면|이\s+말은)` | 이는/이 점에서 meta-entry. |
| ko_H4_ie | korean | medium | 0.5 | `즉` | redefining connector 즉. |
| ko_I1_geotida | korean | high | 2.0 | `[가-힣]+(?:인|한|일|다는)\s+것이다` | ~것이다 formal ending. |
| ko_I2_dependent_noun | korean | medium | 1.0 | `(?:라는\s+점(?:에\s+있다|에서)?|다는\s+점(?:에\s+있다|에서)?|주목할\s+점은|나아갈\s+바|할\s+수(?:가)?\s+있|하는\s+데(?:에)?)` | dependent-noun crutch. |
| ko_I3_means_ending | korean | medium | 1.0 | `(?:라는\s+것|다는\s+것이다|다는\s+뜻이다|다는\s+의미다|다는\s+점이다)` | ~다는 뜻/의미/점 ending. |
| ko_I4_need_to | korean | medium | 1.0 | `(?:할\s+필요가\s+있다|[가-힣]*해야\s+(?:한다|합니다)|[가-힣]+야\s+(?:한다|합니다))` | ~필요가 있다/해야 한다. |
| ko_I5_needed | korean | medium | 1.0 | `[가-힣]+(?:이|가)\s+필요하다` | abstract need formula. |
| ko_I6_ability_noun | korean | medium | 1.0 | `[가-힣A-Za-z0-9]+\s+능력` | N 능력 ability noun. |
| ko_J1_bold | universal | medium | 0.5 | `(?:\*\*|__)[^*_\n]{1,80}(?:\*\*|__)` | bold emphasis decoration. |
| ko_J2_quote_emphasis | korean | medium | 0.5 | `["'"'][가-힣A-Za-z0-9\s]{1,30}["'"']` | quote emphasis decoration. |
| ko_J3_em_dash | universal | low | 0.5 | `—` | em dash decoration. |
| ko_J4_parenthetical_aside | korean | medium | 1.0 | `\([^)]{0,80}(?:의미한다|뜻이다|시사한다|말한다)[^)]{0,80}\)` | parenthetical explanatory aside. |
