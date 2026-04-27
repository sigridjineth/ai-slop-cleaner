# AI-Slop Universal Pattern Categories (Agent-Readable)

This file defines language-universal categories for LLM-based AI-slop detection.
The Rust binary does not use these categories as regex rules. It only reports
structural evidence from `banned-patterns.md`. An LLM agent reads this catalog,
the full source text, and the Rust structural matches, then judges whether the
text exhibits each category's intent in whatever language the text uses.

Examples below are illustrations only. They are not trigger phrases, word lists,
or replacement recipes. Never detect or rewrite by string matching these examples.
For every category, the detection rule is: does the prose exhibit the described
slop intent in its own language, genre, and context?

## Agent contract

1. Read the whole text before judging any category.
2. Treat category descriptions as intent-level smells, not literal phrase lists.
3. Consider language, register, genre, and audience.
4. Preserve meaning, facts, names, terms of art, and useful structure.
5. Rewrite by full-text inference when asked to clean prose. Never regex-replace.
6. Never cut connector-looking syllables out of compound terms. For example,
   Korean words such as `즉흥적으로` and `즉흥성` must not be treated as the
   connector `즉`; endings such as `느낌입니다` must not be chopped into `다`.

---

### translationese
- **Severity:** high
- **Weight:** 2.0
- **Description:** Prose sounds translated or calqued rather than native to its language. Signals include imported grammar, literal connector choices, unnatural passive or possessive forms, awkward nominalizations, and source-language word order. The issue is not that a phrase exists in a language; it is that the sentence feels mechanically carried over from another language when a native construction would be simpler.
- **Detection rule:** Judge whether the wording feels like translation residue in the current language, not whether it contains any particular token.
- **Examples:**
  - Korean: `이 문제에 대해 논의하겠습니다` may be natural in some contexts, but in plain prose `이 문제를 논의하겠습니다` is often cleaner.
  - Korean: `많은 기능을 가지고 있습니다` can sound like a possessive calque when `기능이 많습니다` would be direct.
  - English: `Through this method, we can achieve optimization` can sound imported when `This method optimizes it` is enough.
  - Japanese: `〜において` repeated in casual explanatory prose can feel bureaucratic or translated.
  - Chinese: `通过该方法来进行实现` may be heavier than `用这个方法实现`.
  - Spanish: `a través de este método se realiza` may be over-literal when `este método realiza` or `con este método` fits.
  - Hindi: `के माध्यम से` repeated as a universal connector may sound translated when a postposition or verb choice would do.
  - Arabic: `من خلال` repeated for ordinary means can sound formulaic when a direct verb or preposition is natural.
- **Multilingual note:** Every language has legitimate formal connectors and passive forms. Flag only when the surrounding prose relies on them mechanically or unnaturally.

### structural_monotony
- **Severity:** high
- **Weight:** 2.0
- **Description:** The document is organized with repeated templates rather than organic structure. Signals include repeated heading-intro-bullets-summary blocks, mechanical enumerations, identical section shapes, colon-led field dumps, numbered ladders that over-explain, and parallel binary contrasts used section after section.
- **Detection rule:** Judge whether the structure feels generated, templated, or over-packaged for the content.
- **Examples:**
  - English: every section follows `What it is:` then three bullets then `Why it matters:`.
  - Korean: each paragraph starts with `첫째`, `둘째`, `셋째` even when the ideas are not a ranked list.
  - Japanese: repeated `概要 / 特徴 / メリット / まとめ` blocks for small topics.
  - Chinese: repeated `定义：… 特点：… 应用：… 总结：…` for every subsection.
  - Spanish: `Características: rapidez, seguridad, eficiencia` repeated as a default paragraph shape.
  - Turkish: every section uses the same `Nedir? / Neden önemlidir? / Nasıl çalışır?` mold.
- **Multilingual note:** Useful headings and lists are allowed. The smell is template dominance: structure replacing thought.

### cliche_formulaic
- **Severity:** medium
- **Weight:** 1.5
- **Description:** The prose leans on canned conclusions, generic importance claims, hype, redefinition formulas, or motivational phrasing that could appear in any document. It sounds like a stock answer rather than a specific author deciding how to end or emphasize the point.
- **Detection rule:** Judge whether phrases are functioning as formulaic filler rather than carrying document-specific meaning.
- **Examples:**
  - English: `In conclusion`, `It is important to note`, `not just a tool but a revolution`.
  - Korean: `요약하면`, `중요한 역할을 합니다`, `새로운 가능성을 열어줍니다` when used as a canned wrap-up.
  - Japanese: `結論として`, `非常に重要です`, `新たな可能性を切り開きます` as generic closure.
  - Chinese: `总而言之`, `具有重要意义`, `开启新的可能性` when unsupported by specifics.
  - Spanish: `En conclusión`, `es importante destacar`, `no es solo X, sino Y`.
  - Arabic: `في الختام`, `من المهم ملاحظة`, `ليس مجرد X بل Y`.
- **Multilingual note:** Common conclusion phrases can be natural in essays, speeches, or teaching material. Flag them when they substitute for a real ending or inflate an ordinary point.

### rhythm
- **Severity:** medium
- **Weight:** 1.5
- **Description:** Sentences move with machine-like cadence. Signals include similar sentence lengths, repeated openers, repeated endings, repeated subject-verb skeletons, same-length paragraphs, and an explanatory rhythm that never varies emphasis or pace.
- **Detection rule:** Judge the music of the prose across paragraphs, not isolated sentences.
- **Examples:**
  - English: five sentences in a row start with `This means` and end with `for users`.
  - Korean: repeated `〜합니다. 또한 〜합니다. 이를 통해 〜할 수 있습니다.` cadence.
  - Japanese: repeated `〜できます。さらに〜できます。そのため〜できます。` endings.
  - Chinese: repeated `它可以… 同时… 因此…` sentence frames.
  - Spanish: repeated `Esto permite... Además... Esto ayuda...` patterns.
  - Hindi: repeated `यह ... करता है। इसके माध्यम से ... होता है।` cadence.
- **Multilingual note:** Rhythm requires document-level judgment. Do not flag short samples with too little evidence.

### modifier_abuse
- **Severity:** medium
- **Weight:** 1.25
- **Description:** The text pads claims with stacked intensifiers, vague quality adjectives, repeated abstract suffixes, or modifier chains that add emphasis without precision. The sentence sounds decorated rather than sharpened.
- **Detection rule:** Judge whether modifiers earn their place by adding specific information.
- **Examples:**
  - English: `highly robust and extremely efficient comprehensive solution`.
  - Korean: `매우 혁신적이고 효과적인 종합적 접근 방식` when the modifiers are not supported.
  - Japanese: `非常に効果的で包括的な革新的ソリューション` as empty praise.
  - Chinese: `非常全面且高效的创新性解决方案` without specifics.
  - Spanish: `una solución sumamente innovadora, robusta y eficiente` as generic gloss.
  - Turkish: `son derece güçlü ve kapsamlı yenilikçi yaklaşım` without concrete detail.
- **Multilingual note:** Technical modifiers and necessary degree words are legitimate. Flag padding, not precision.

### hedging
- **Severity:** medium
- **Weight:** 1.25
- **Description:** Claims are weakened by stacked possibility, capability, or uncertainty markers. The prose repeatedly says something can, may, might, could, appears to, tends to, or is able to do a thing instead of stating the actual claim and conditions.
- **Detection rule:** Judge whether hedges are protecting uncertainty that matters or merely avoiding commitment.
- **Examples:**
  - English: `may potentially be able to help improve`.
  - Korean: `개선할 수 있을 것으로 보일 수 있습니다`.
  - Japanese: `改善できる可能性があると考えられます` repeated without need.
  - Chinese: `可能可以有助于提升`.
  - Spanish: `podría llegar a ayudar a mejorar`.
  - Arabic: `قد يكون من الممكن أن يساعد في تحسين`.
- **Multilingual note:** Scientific uncertainty, legal caution, and safety caveats may require hedging. Flag repeated evasive hedges that obscure simple claims.

### meta_commentary
- **Severity:** medium
- **Weight:** 1.5
- **Description:** The prose talks about the act of explaining, exploring, summarizing, or organizing instead of directly explaining the subject. It announces the journey, the takeaway, or the section purpose in a way that feels generated.
- **Detection rule:** Judge whether the author is narrating the structure rather than delivering content.
- **Examples:**
  - English: `Let's dive in`, `This section explores`, `Now that we have covered`.
  - Korean: `이제부터 살펴보겠습니다`, `다음으로 알아보겠습니다`, `정리해보면`.
  - Japanese: `ここでは見ていきます`, `次に確認しましょう`.
  - Chinese: `下面我们来看看`, `接下来让我们探讨`.
  - Spanish: `Veamos en detalle`, `A continuación exploraremos`.
  - Hindi: `आइए अब देखते हैं`, `अब हम समझेंगे`.
- **Multilingual note:** A single roadmap can help readers in long documentation. Flag repeated or unnecessary self-navigation.

### connector_abuse
- **Severity:** medium
- **Weight:** 1.25
- **Description:** Transitions become mechanical. The text repeatedly starts sentences or paragraphs with the same connectors, forces contrast or causality where it is not needed, or uses symbols such as `+` as prose conjunctions for style rather than clarity.
- **Detection rule:** Judge whether connectors improve flow or create a visible generated cadence.
- **Examples:**
  - English: repeated `Additionally`, `Moreover`, `Furthermore`, `However`, or `A + B` list style.
  - Korean: repeated `또한`, `그리고`, `하지만`, `즉` as paragraph machinery.
  - Japanese: repeated `また`, `さらに`, `一方で`, `つまり`.
  - Chinese: repeated `此外`, `同时`, `然而`, `也就是说`.
  - Spanish: repeated `Además`, `Sin embargo`, `Por lo tanto`, `Es decir`.
  - Arabic: repeated `بالإضافة إلى ذلك`, `ومع ذلك`, `أي`.
- **Multilingual note:** Never remove connector-looking text from inside words or compounds. Decide at phrase/sentence level by meaning.

### dependency_clause_overuse
- **Severity:** medium
- **Weight:** 1.25
- **Description:** Sentences are loaded with dependent clauses, nominalized purpose clauses, and chained conditions that delay the main verb. The prose reads as if it is constantly qualifying how, why, based on what, in relation to what, or for the purpose of what before saying the point.
- **Detection rule:** Judge whether clause stacking makes the sentence indirect, abstract, or bureaucratic.
- **Examples:**
  - English: `In order to achieve X, based on Y, through Z, the system is able to...`.
  - Korean: `〜하기 위해`, `〜에 기반하여`, `〜와 관련하여` stacked before the main claim.
  - Japanese: `〜するために`, `〜に基づいて`, `〜に関して` piled into one sentence.
  - Chinese: `为了… 基于… 通过… 从而…` stacked mechanically.
  - Spanish: `con el fin de`, `basado en`, `en relación con` chained before the verb.
  - Turkish: `amacıyla`, `temelinde`, `aracılığıyla` piled into a single sentence.
- **Multilingual note:** Complex ideas sometimes require subordinate clauses. Flag avoidable stacking, not legitimate syntax.
