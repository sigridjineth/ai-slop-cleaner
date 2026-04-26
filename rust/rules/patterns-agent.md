# AI-Slop Pattern Definitions (Agent-Readable)

This file contains all AI-slop detection patterns in plain markdown format.
The Rust binary loads these and passes them to an LLM agent for multilingual judgment.
Each pattern includes: name, severity, weight, description, and examples.

## How to Use

For each text segment, the agent should:
1. Read the pattern descriptions below
2. Judge whether the text matches the pattern's intent (not just regex)
3. Consider context, language, and cultural nuance
4. Return structured matches with severity and weight

---

## English Structural Patterns

### redefinition
- **Severity:** medium
- **Lang Scope:** english
- **Weight:** 2.0
- **Description:** Sentences that negate one concept and immediately redefine it with another. The pattern "A is not X, it is Y" or "not X but Y".
- **Examples:**
  - "This is not just a tool, it is a revolution."
  - "The system is not a database but rather a function."
  - "It is not merely a suggestion; it is a requirement."
- **Multilingual note:** Same intent in any language — negating A to assert B.

### closing_summary
- **Severity:** low
- **Lang Scope:** english
- **Weight:** 1.0
- **Description:** Explicit closing phrases that signal a summary is coming.
- **Examples:**
  - "In conclusion, ..."
  - "To summarize, ..."
  - "In short, ..."
  - "At the end of the day, ..."

### progress_announcement
- **Severity:** medium
- **Lang Scope:** english
- **Weight:** 1.0
- **Description:** Phrases that announce what the writer is about to do, rather than just doing it.
- **Examples:**
  - "Let's dive in."
  - "Let's take a closer look."
  - "We will examine this step by step."
  - "Let's unpack this."

### pre_classification
- **Severity:** low
- **Lang Scope:** english
- **Weight:** 1.0
- **Description:** Framing a topic by announcing how many categories it has before listing them.
- **Examples:**
  - "There are three types of..."
  - "This can be divided into two categories..."
  - "We can divide this into several groups..."

### imagine_prompt
- **Severity:** low
- **Lang Scope:** english
- **Weight:** 1.0
- **Description:** Prompting the reader to imagine or visualize something.
- **Examples:**
  - "Imagine a world where..."
  - "Picture this scenario..."
  - "Visualize the following..."

### broad_field_dump
- **Severity:** medium
- **Lang Scope:** english
- **Weight:** 1.5
- **Description:** A broad overview sentence followed immediately by a colon and a list of fields/attributes.
- **Examples:**
  - "The object has the following properties: name, age, height..."
  - "The system consists of these components: A, B, C..."

### colon_enumeration
- **Severity:** medium
- **Lang Scope:** english
- **Weight:** 1.5
- **Description:** A short phrase followed by a colon, then comma-separated items.
- **Examples:**
  - "Key features: speed, reliability, cost"
  - "Required fields: name, email, phone"

### a_not_b_redefinition
- **Severity:** high
- **Lang Scope:** english
- **Weight:** 2.5
- **Description:** Stronger variant of redefinition. Explicit "is not X, it is Y" with clear negation and assertion.
- **Examples:**
  - "This is not a problem, it is an opportunity."
  - "It is not just code, it is poetry."
- **Note:** Overlaps with `redefinition` but weighted higher for explicit negation+assertion pairs.

### a_not_b_english
- **Severity:** high
- **Lang Scope:** english
- **Weight:** 2.5
- **Description:** English-specific "is not just X" pattern.
- **Examples:**
  - "It is not just a framework, it is a philosophy."
  - "This is not merely a tool; it goes beyond that."

---

## Korean Structural Patterns

### ko_A1_about_regarding
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Translationese pattern using '~에 대해/대하여' where simpler phrasing would work.
- **Examples:**
  - "이 문제에 대해 논의하겠습니다" → "이 문제를 논의하겠습니다"
  - "AI에 대하여 설명드리겠습니다" → "AI를 설명드리겠습니다"

### ko_A2_through
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Overuse of '~를 통해/통하여' as a connector.
- **Examples:**
  - "이 방법을 통해 해결합니다" → "이 방법으로 해결합니다"
  - "데이터를 통하여 분석합니다" → "데이터로 분석합니다"

### ko_A3_in_terms
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Translationese '~에 있어서' pattern.
- **Examples:**
  - "성능에 있어서 우수합니다" → "성능이 우수합니다"
  - "이 점에 있어서 중요합니다" → "이 점이 중요합니다"

### ko_A4_point_sense
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** The formulaic '~라는 점에서' reasoning connector.
- **Examples:**
  - "효율성이라는 점에서 좋습니다"
  - "속도라는 점에서 뛰어납니다"

### ko_A5_related
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '~와 관련하여/관련된' when a simpler connection would suffice.
- **Examples:**
  - "이 문제와 관련하여" → "이 문제와 관련해서" or just "이 문제에 대해"

### ko_A6_based_on
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '~에 기반하여/바탕으로' overuse.
- **Examples:**
  - "이 데이터에 기반하여" → "이 데이터를 바탕으로"
  - "연구에 기반한 결과" → "연구 결과"

### ko_A7_have
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** '가지고 있다' possessive translationese.
- **Examples:**
  - "많은 기능을 가지고 있습니다" → "많은 기능이 있습니다"
  - "장점을 가지고 있는 시스템" → "장점이 있는 시스템"

### ko_A8_double_passive
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Double passive constructions that sound unnatural in Korean.
- **Examples:**
  - "변경되어진다" → "변경된다"
  - "보여질 수 있다" → "보일 수 있다"

### ko_A9_by_passive
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '~에 의해' passive construction.
- **Examples:**
  - "시스템에 의해 처리된다" → "시스템이 처리한다"

### ko_A10_can_overuse
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Overuse of '~할 수 있다' can-form.
- **Examples:**
  - "이 방법으로 해결할 수 있습니다" → "이 방법으로 해결합니다"
  - "활용할 수 있는 기능" → "활용하는 기능"

### ko_A11_purpose
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '~을 위해(서)' purpose clause overuse.
- **Examples:**
  - "성능 향상을 위해" → "성능을 높이려고"

### ko_A12_automated_passive
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '만들어지다/이루어지다' automated passive forms.
- **Examples:**
  - "자동으로 만들어진다" → "자동으로 만든다"

### ko_A13_noun_stack
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** English-like noun stacking with abstract suffixes.
- **Examples:**
  - "디지털 전환 가속화" → "디지털 전환을 빠르게 하다"
  - "서비스 고도화" → "서비스를 높은 수준으로 만들다"

### ko_A14_and_sentence
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Starting sentences with '그리고' repeatedly.
- **Examples:**
  - "그리고 이 방법은..."
  - "그리고 또한..."

### ko_A15_abstract_subject
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.5
- **Description:** Abstract subject + all-purpose verb combinations.
- **Examples:**
  - "이 연구는 중요한 결과를 보여준다"
  - "이 데이터는 유의미한 정보를 제공한다"

### ko_B1_parenthesized_english
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Korean term followed by English in parentheses.
- **Examples:**
  - "인공지능(Artificial Intelligence)"
  - "머신러닝(Machine Learning)"

### ko_B2_raw_english_term
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Untranslated English buzzwords in Korean text.
- **Examples:**
  - "이 framework는 robust합니다"
  - "pipeline을 leverage합니다"

### ko_B3_long_english_quote
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Long English quotes embedded in Korean prose.
- **Examples:**
  - '한국어 문장 중간에 "This is a very long English sentence that continues for many words" 라고 쓰는 경우'

### ko_B4_known_as
- **Severity:** low
- **Lang Scope:** korean
- **Weight:** 0.5
- **Description:** '~라고 알려진/~로 일컬어지는' formulaic expressions.
- **Examples:**
  - "세계적으로 유명한, 일컬어지는 도시"

### ko_C1_mechanical_enumeration
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Mechanical use of '첫째, 둘째, 셋째' enumeration.
- **Examples:**
  - "첫째, ... 둘째, ... 셋째, ..."

### ko_C2_bullet_block
- **Severity:** medium
- **Lang Scope:** universal
- **Weight:** 1.0
- **Description:** Excessive bullet point blocks.
- **Examples:**
  - Three or more consecutive bullet points in prose.

### ko_C3_generic_headings
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Generic headings like '도입', '서론', '본론', '결론'.

### ko_C4_topic_sentence
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Formulaic topic sentences.
- **Examples:**
  - "핵심은 ..."
  - "중요한 것은 ..."
  - "이 문단은 ..."

### ko_C5_emoji
- **Severity:** high
- **Lang Scope:** universal
- **Weight:** 2.0
- **Description:** Emoji decoration in prose text.
- **Examples:**
  - "✅ 완료했습니다"
  - "🚀 새로운 기능"

### ko_C6_heading_summary
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Heading immediately followed by a summary sentence.
- **Examples:**
  - "## 소개\n\n이 섹션에서는 시스템을 소개합니다."

### ko_C7_three_step
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** The '먼저/반면/결국' formulaic structure.
- **Examples:**
  - "먼저 ... 반면 ... 결국 ..."

### ko_C8_binary_parallel
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Binary parallelism with 'A인가, B인가'.
- **Examples:**
  - "성공인가, 실패인가"

### ko_C9_numeric_index
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Numeric indexing like '1) 2) 3)'.

### ko_C10_colon_heading
- **Severity:** high
- **Lang Scope:** universal
- **Weight:** 2.0
- **Description:** Headings with colons that split title and subtitle.
- **Examples:**
  - "## 주제: 부제목"

### ko_D1_conclusion_phrase
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Formulaic conclusion phrases.
- **Examples:**
  - "결론적으로"
  - "요약하면"
  - "정리하자면"
  - "라고 할 수 있다"

### ko_D2_importance_cliche
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Overused importance expressions.
- **Examples:**
  - "시사하는 바가 크다"
  - "간과할 수 없다"
  - "무시할 수 없다"

### ko_D3_list_intro
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Formulaic list introductions.
- **Examples:**
  - "크게 세 가지로 나눌 수 있다"
  - "다음과 같은 특징을 가진다"

### ko_D4_hype
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** Hype vocabulary and exaggerated claims.
- **Examples:**
  - "혁신적인"
  - "획기적인"
  - "전례 없는"
  - "새로운 장을 열었다"

### ko_D5_personified_abstract
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.5
- **Description:** Personifying abstract subjects.
- **Examples:**
  - "기술은 우리에게 질문을 던진다"
  - "시대는 끝나지 않는다"

### ko_D6_closing_formula
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.5
- **Description:** Formulaic closing expressions.
- **Examples:**
  - "해야 할 때입니다"
  - "나아갈 시점입니다"

### ko_D7_transformation_formula
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** X에서 Y로 transformation slogan.
- **Examples:**
  - "디지털로의 전환"
  - "AI 시대로의 도약"

### ko_E1_sentence_length_uniformity
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Uniform sentence lengths (rhythm detection).
- **Note:** Detected via rhythm analysis, not just regex.

### ko_E2_repeated_sentence_endings
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Repeated sentence endings like '것이다', '합니다'.

### ko_E3_uniform_paragraph_blocks
- **Severity:** low
- **Lang Scope:** korean
- **Weight:** 0.5
- **Description:** Paragraphs of uniform length.

### ko_F1_degree_adverb
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Excessive degree adverbs.
- **Examples:**
  - "매우 중요한"
  - "대단히 효과적인"

### ko_F2_double_modifier
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Synonym double modifiers.
- **Examples:**
  - "중요하고 핵심적인 역할"
  - "새롭고 혁신적인 접근"

### ko_F3_role_function
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '역할과 기능/의미와 가치' doublets.

### ko_F4_suffix_abuse
- **Severity:** low
- **Lang Scope:** korean
- **Weight:** 0.5
- **Description:** Abstract suffix overuse (-적, -성, -화).
- **Examples:**
  - "효율적 측면"
  - "혁신성"
  - "현대화"

### ko_F5_jeok_chain
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** ~적 N abstract chains.
- **Examples:**
  - "효율적 관리"
  - "체계적 접근"

### ko_G1_hedging
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Hedge endings that weaken statements.
- **Examples:**
  - "할 수 있을 것으로 보인다"
  - "인 것으로 판단된다"

### ko_G2_double_hedge
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.5
- **Description:** Double or triple hedges.
- **Examples:**
  - "가능성이 있을 수 있다"
  - "보여질 수 있다"

### ko_H1_connector
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Sentence-initial connectors.
- **Examples:**
  - "또한 ..."
  - "따라서 ..."
  - "나아가 ..."

### ko_H2_contrast_connector
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Contrast connectors at sentence start.
- **Examples:**
  - "하지만 ..."
  - "그러나 ..."

### ko_H3_meta_entry
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 1.5
- **Description:** Meta-commentary entries.
- **Examples:**
  - "이는 ... 의미한다"
  - "이 점에서 ..."

### ko_H4_ie
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 0.5
- **Description:** Overuse of '즉' as a redefining connector.

### ko_I1_geotida
- **Severity:** high
- **Lang Scope:** korean
- **Weight:** 2.0
- **Description:** '~것이다' formal ending overuse.
- **Examples:**
  - "중요한 것이다"
  - "그렇다는 것이다"

### ko_I2_dependent_noun
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Dependent noun crutches.
- **Examples:**
  - "라는 점에 있다"
  - "주목할 점은"
  - "나아갈 바"

### ko_I3_means_ending
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '~다는 뜻/의미/점이다' endings.
- **Examples:**
  - "그렇다는 뜻이다"
  - "중요하다는 의미다"

### ko_I4_need_to
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** '~필요가 있다/해야 한다' formulas.
- **Examples:**
  - "노력할 필요가 있다"
  - "개선해야 한다"

### ko_I5_needed
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Abstract '필요하다' formulas.
- **Examples:**
  - "변화가 필요하다"

### ko_I6_ability_noun
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** N + 능력 ability nouns.
- **Examples:**
  - "학습 능력"
  - "분석 능력"

### ko_J1_bold
- **Severity:** medium
- **Lang Scope:** universal
- **Weight:** 0.5
- **Description:** Bold emphasis decoration in prose.

### ko_J2_quote_emphasis
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 0.5
- **Description:** Quote emphasis decoration.

### ko_J3_em_dash
- **Severity:** low
- **Lang Scope:** universal
- **Weight:** 0.5
- **Description:** Em dash decoration.

### ko_J4_parenthetical_aside
- **Severity:** medium
- **Lang Scope:** korean
- **Weight:** 1.0
- **Description:** Parenthetical explanatory asides.
- **Examples:**
  - "(이는 ... 의미한다)"
  - "(즉, ... 뜻이다)"

---

## Banned Words (English)

These words should be flagged when used in contexts where simpler alternatives would work better.

### Verbs
- delve, elucidate, underscore, harness, leverage, bolster, foster, showcase, streamline, revolutionize, unveil, orchestrate, transcend, exemplify, augment, surpass, pinpoint, scrutinize, unravel, embark, navigate, elevate, unlock, unleash, dive, discover, craft, illuminate

### Adjectives/Adverbs
- pivotal, meticulous, intricate, transformative, groundbreaking, unparalleled, comprehensive, robust, crucial, notable, formidable, nuanced, multifaceted, paramount, instrumental, foundational, commendable, cutting-edge, seamless, vibrant, bustling, holistic, poised, remarkable

### Nouns
- realm, tapestry, landscape, beacon, hurdles, testament, game-changer, journey, synergy

### Cliché Phrases
- "In today's digital age"
- "It's important to note"
- "Furthermore", "Moreover", "However"
- "...not just this, but also this"
- "In conclusion", "In closing"
- "Let's dive in", "Let's explore"

---

## Banned Words (Korean)

### Verbs (with alternatives)
- 활용하다 → 쓰다, 이용하다, 살리다
- 강조하다, 부각하다 → 중요하다, 분명히 보여 주다
- 제공하다, 부여하다 → 주다, 맡기다, 할 수 있게 하다
- 제고하다 → 높이다, 나아지게 하다
- 구축하다 → 만들다, 쌓다, 세우다
- 도래하다 → 오다, 시작되다
- 함양하다 → 기르다, 키우다
- 영위하다 → 하다, 이어 가다, 살아가다
- 결부되다, 연계되다 → 관련되다, 이어지다
- 간과하다 → 놓치다, 빠뜨리다
- 명시하다 → 분명히 밝히다, 또렷하게 적다
- 야기하다 → 일으키다, 가져오다
- 발발하다 → 일어나다, 터지다
- 수반하다 → 따라오다, 함께 오다
- 투영하다 → 비추다, 반영하다
- 상정하다 → 가정하다, 그렇게 본다
- 개진하다 → 말하다, 의견을 내다
- 조명하다 → 살펴보다, 짚어 보다
- 견지하다 → 유지하다, 지키다
- 초래하다 → 가져오다, 부르다
- 기여하다 → 돕다, 도움이 되다
- 표명하다 → 입장을 밝히다, 드러내다
- 전개하다 → 펼치다, 이어 가다, 진행하다
- 모색하다 → 찾다, 방법을 생각하다
- 재고하다 → 다시 생각하다

### Adjectives/Adverbs (with alternatives)
- 중추적인, 핵심적인 → 중요한, 중심이 되는
- 복잡한, 정교한 → 섬세한, 꼼꼼한
- 변혁적인, 획기적인 → 큰 변화를 가져오는, 새로운
- 비할 데 없는, 독보적인 → 아주 뛰어난, 특별한
- 포괄적인, 전반적인 → 폭넓은, 전체적인
- 강력한, 견고한 → 튼튼한, 안정적인
- 결정적인 → 중요한
- 주목할 만한 → 눈에 띄는, 특별한
- 다면적인 → 여러 면을 가진
- 궁극적으로, 결과적으로 → 결국, 끝내
- 본질적인 → 기본적인, 원래의
- 유의미한 → 의미 있는, 의미가 큰
- 고도화된, 첨단화된 → 많이 발전한, 수준이 높은
- 압도적인 → 훨씬 뛰어난, 매우 강한
- 혁신적인 → 새로운, 색다른
- 폭발적인 → 매우 빠른, 급격한
- 치명적인 → 매우 큰, 치명적 결과를 낳는
- 각광받는 → 주목받는, 인기 있는

### Translationese Patterns
- "~에 있어(서)" → "~에서", "~에는", "~에서는"
- "~을 기반으로" → "~을 바탕으로", "~에 따라", "~을 토대로"
- "~을 통해" (overuse) → "~으로", "~하면서", "~을 써서"
- "~와 같은" (overuse) → context-specific or delete
- "~적이다" overuse → "효과적이다" → "효과가 좋다"
- "~화, ~성" overuse → "현대화" → "현대로 바뀌다"
- "~라는 점" → "~라는 것", "~라는 사실"
- "~이 될 것입니다" → "~입니다", "~가 됩니다"
- "~이라고 할 수 있습니다" → "~입니다"
- "~으로 하여금" → "~이 ~하게 하다"
- "면모" → "모습"
- "접근 방식" → "방법", "방식"
- "이러한 맥락에서" → "이런 상황에서", "이처럼"
- "본격적으로" → "제대로", or go straight to the point
- "당사는, 저희는" → "우리는", or "저는"

### AI-Specific Expressions (avoid)
- 떠올리다, 떠올릴 수 있다, 떠올려 보면
- 대략, 대략적으로, 대략 이런
- 느낌입니다, 느낌의, "~같은 느낌"
- "조금 더 구체적으로", "좀 더 자세히"
- "보통 ~에는", "일반적으로 ~가 있습니다"
- "~와 같은 모습입니다", "~처럼 생각할 수 있습니다"
- "~를 상상해 볼 수 있습니다", "~를 그려볼 수 있습니다"
