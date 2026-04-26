"""Banned structural-pattern loading and canonical regexes.

The fallback detector keeps these regexes deliberately explicit.  The markdown
reference is still the human-facing SSOT, but code-level names make coverage
against external taxonomies auditable.
"""

from __future__ import annotations

from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
import re
from typing import Iterable, Pattern

from ai_slop_cleaner.core.banned_words import read_reference_text

REFERENCE_NAME = "banned-patterns.md"


@dataclass(frozen=True)
class StructuralPattern:
    """Regex-backed structural pattern used by the fallback detector."""

    name: str
    category: str
    severity: str
    weight: float
    regex: Pattern[str]
    description: str


def _re(pattern: str, flags: int = re.IGNORECASE) -> Pattern[str]:
    return re.compile(pattern, flags)


# Original English/H.U.M.A.N.-oriented structural tells.
REDEFINITION_RE = _re(
    r"(?:\bnot\s+\w+(?:\s+\w+){0,2}\b|rather\s+than\s+\w+(?:\s+\w+){0,2}|instead\s+of\s+\w+(?:\s+\w+){0,2})\s*[,;:]+\s*\b(?:it\s+is|is|are)\b|\bnot\s+\w+(?:\s+\w+){0,2}\s+but\s+\w+"
)
PROGRESS_RE = _re(
    r"\b(let's\s+(?:take a closer look|unpack(?: this)?|dig deeper|break this down|dive in|explore)|we(?:'ll| will)\s+examine(?: this)?(?: step by step)?|step by step|one by one)\b"
)
PRE_CLASSIFICATION_RE = _re(
    r"\b(?:there are|there is|we can divide|this divides|can be divided)\b[^.!?]{0,80}\b(?:types|branches|categories|groups|kinds)\b"
)
IMAGINE_RE = _re(r"\b(imagine|picture|visualize)\b[^.!?]{0,80}")
CLOSING_SUMMARY_RE = _re(
    r"\b(to summarize|in summary|in conclusion|in closing|in short|put simply|at the end of the day)\b"
)
BROAD_FIELD_DUMP_RE = _re(
    r"\b(?:fields|properties|attributes|schema|structure)\b[^.\n]{0,120}:\s*(?:\n\s*(?:[-*+]\s*)?`?[\w-]+`?\s*:[^\n]*){2,}",
    re.IGNORECASE | re.DOTALL,
)
POST_CODE_NARRATION_RE = _re(
    r"\b(first|then|next)\s+(?:we|the code|this)|\bwe\s+(?:define|create|call|return)\b"
)
COLON_ENUMERATION_RE = _re(
    r"\b[a-z][\w-]*(?:\s+[a-z][\w-]*){0,6}\s*[:;]\s*`?[\w-]+`?(?:\s*,\s*(?:and\s+)?`?[\w-]+`?){1,}"
)
A_NOT_B_RE = _re(
    r"\b(?:is|are|was|were)\s+not\s+[a-z]+(?:\s+[a-z]+){0,3}\s*[,;]\s*(?:it\s+)?(?:is|are|was|were)\b|\bnot\s+[a-z]+(?:\s+[a-z]+){0,3}\s+but\s+(?:is\s+)?[a-z]+"
)

# im-not-ai / Humanize KR taxonomy regexes (A-J).  Names preserve the source
# rule IDs so the audit matrix can be generated and reviewed mechanically.
KO_A1_ABOUT_RE = _re(r"[가-힣A-Za-z0-9]+\s*에\s*대(?:해(?:서)?|하여)")
KO_A2_THROUGH_RE = _re(r"[가-힣A-Za-z0-9]+\s*(?:을|를)\s*통(?:해|하여)")
KO_A3_IN_TERMS_RE = _re(r"[가-힣A-Za-z0-9]+\s*에\s*있어(?:서)?")
KO_A4_POINT_SENSE_RE = _re(r"(?:라는|다는)\s+점에서")
KO_A5_RELATED_RE = _re(r"[가-힣A-Za-z0-9]+\s*(?:와|과)\s*관련(?:하여|한|된)")
KO_A6_BASED_ON_RE = _re(r"[가-힣A-Za-z0-9]+\s*에\s*기반(?:하여|한)|[가-힣A-Za-z0-9]+\s*(?:을|를)\s*바탕으로")
KO_A7_HAVE_RE = _re(r"가지고\s+있(?:다|습니다|는)")
KO_A8_DOUBLE_PASSIVE_RE = _re(r"[가-힣]+(?:되어진|되어진다|지게\s+된|지게\s+된다|보여질\s+수\s+있)")
KO_A9_BY_PASSIVE_RE = _re(r"[가-힣A-Za-z0-9]+\s*에\s*의해")
KO_A10_CAN_OVERUSE_RE = _re(r"[가-힣]+(?:할|될|릴|울|낼|킬|일)\s+수\s+있(?:다|습니다|는|을)")
KO_A11_PURPOSE_RE = _re(r"[가-힣A-Za-z0-9]+\s*(?:을|를)\s*위해(?:서)?")
KO_A12_AUTOMATED_PASSIVE_RE = _re(r"(?:만들어지(?:다|고|며|는|었다|었습니다)?|이루어지(?:다|고|며|는)?|이루어졌(?:다|습니다)?)")
KO_A13_NOUN_STACK_RE = _re(r"(?:[가-힣A-Za-z0-9]+\s+){2,}[가-힣A-Za-z0-9]+\s*(?:가속화|고도화|최적화|강화|확대|발전|전환)")
KO_A14_AND_SENTENCE_RE = _re(r"(?m)^\s*그리고\b")
KO_A15_ABSTRACT_SUBJECT_RE = _re(
    r"[가-힣A-Za-z0-9'\"“”\s]{1,40}(?:은|는|이|가)\s+[^.!?\n]{0,50}(?:보여준(?:다|다\.|다는|습니다)|제공(?:한다|합니다)|가져온(?:다|다는)|시사(?:한다|합니다)|흔들고\s+있(?:다|습니다))"
)

KO_B1_PAREN_ENGLISH_RE = _re(r"[가-힣]{2,}\s*\([A-Za-z][A-Za-z0-9 ._/-]{1,40}\)")
KO_B2_RAW_ENGLISH_RE = _re(r"(?<![A-Za-z])(?:framework|pipeline|leverage|seamless|robust|scalable|insight|impact|holistic)(?![A-Za-z])")
KO_B3_LONG_ENGLISH_QUOTE_RE = _re(r"[\"“][A-Za-z][^\"”\n]{20,}[\"”]")
KO_B4_KNOWN_AS_RE = _re(r"(?:라고\s+알려진|로\s+일컬어지는)")

KO_C1_ENUMERATION_RE = _re(r"첫째[\s\S]{0,240}둘째[\s\S]{0,240}셋째")
KO_C2_BULLET_BLOCK_RE = _re(r"(?m)(?:^\s*[-*+]\s+.+\n?){3,}")
KO_C3_GENERIC_HEADINGS_RE = _re(r"(?m)^\s{0,3}#{1,6}\s*(?:도입|서론|본론|결론)\b")
KO_C4_TOPIC_SENTENCE_RE = _re(r"(?m)^\s*(?:핵심은|중요한\s+(?:점|것)은|요지는|주목할\s+점은|이\s+문단은)")
KO_C5_EMOJI_RE = _re(r"[✅🚀💡⚠️📊✨🔥👉👍🎯]")
KO_C6_HEADING_SUMMARY_RE = _re(r"(?:이\s+섹션에서는|이\s+장에서는|이번\s+장에서는|이\s+글에서는)\s+[^.!?\n]{0,80}(?:다룬|살펴본|소개한)")
KO_C7_THREE_STEP_RE = _re(r"먼저[^\n]{0,300}반면[^\n]{0,300}(?:결국|마지막으로)", re.IGNORECASE | re.DOTALL)
KO_C8_BINARY_PARALLEL_RE = _re(r"[가-힣A-Za-z0-9'\"“”]+(?:인가|인가요)\s*[,·]?\s*[가-힣A-Za-z0-9'\"“”]+(?:인가|인가요)")
KO_C9_NUMERIC_INDEX_RE = _re(r"(?:^|\s)(?:1\)|\(1\))[^\n]{0,160}(?:2\)|\(2\))[^\n]{0,160}(?:3\)|\(3\))")
KO_C10_COLON_HEADING_RE = _re(r"(?m)^\s{0,3}#{1,6}\s*(?:\*\*)?[^:\n]{1,80}(?:\*\*)?\s*:\s+\S.*$")

KO_D1_CONCLUSION_RE = _re(r"(?:결론적으로|요약하면|종합하면|정리하자면|정리하면|라고\s+할\s+수\s+있다|라고\s+볼\s+수\s+있다|라\s+하겠다|라\s+할\s+것이다|에\s+다름\s+아니다)")
KO_D2_IMPORTANCE_RE = _re(r"(?:시사하는\s+바가\s+크다|주목할\s+만하다|간과할\s+수\s+없다|무시할\s+수\s+없다|지평을\s+연다|방점을\s+찍는다|의미가\s+적지\s+않다|의미심장하다|반드시\s+기억해야\s+한다|매우\s+중요하다)")
KO_D3_LIST_INTRO_RE = _re(r"(?:크게\s+세\s+가지로\s+나눌\s+수\s+있다|다음과\s+같은\s+특징을\s+가진다|다음과\s+같이\s+요약할\s+수\s+있다)")
KO_D4_HYPE_RE = _re(r"(?:혁신적인|획기적인|전례\s+없는|압도적|막강한|폭발적|파격적|대대적|강력한|치열한|뜨거운|가능성을\s+열어준다|새로운\s+장을\s+열(?:다|었다)|시대가\s+도래했다)")
KO_D5_PERSONIFIED_ABSTRACT_RE = _re(r"[가-힣A-Za-z0-9'\"“”\s]{1,40}(?:은|는|이|가)\s+[^.!?\n]{0,40}(?:질문을\s+던집니다|질문을\s+던진다|끝나지\s+않습니다|증명합니다|부른다|요구한다)")
KO_D6_CLOSING_FORMULA_RE = _re(r"(?:해야\s+할\s+때(?:입니다|다)|나아갈\s+시점(?:입니다|이다)|할\s+순간(?:입니다|이다)|지금이야말로[^.!?\n]{0,60}(?:때|시점))")
KO_D7_TRANSFORMATION_RE = _re(r"[가-힣A-Za-z0-9'\"“”\s]{1,40}(?:에서|을\s+넘어|를\s+넘어)\s+[가-힣A-Za-z0-9'\"“”\s]{1,40}(?:로|으로)")

KO_F1_DEGREE_ADVERB_RE = _re(r"(?:매우|정말|진짜로|대단히|극히)\s+[가-힣]+")
KO_F2_DOUBLE_MODIFIER_RE = _re(r"(?:중요하고\s+핵심적인|새롭고\s+혁신적인|지속적이고\s+꾸준한|[가-힣]+고\s+[가-힣]+적인\s+(?:역할|접근|노력|변화))")
KO_F3_ROLE_FUNCTION_RE = _re(r"(?:역할과\s+기능|의미와\s+가치)")
KO_F4_SUFFIX_ABUSE_RE = _re(r"(?:[가-힣]+적\s+(?:측면|관점)|[가-힣]+성\b|[가-힣]+화\b)")
KO_F5_JEOK_CHAIN_RE = _re(r"[가-힣]+적\s+[가-힣]+")

KO_G1_HEDGING_RE = _re(r"(?:할\s+수\s+있을\s+것으로\s+보인다|인\s+것으로\s+판단된다|라고\s+여겨진다|인\s+듯하다|것으로\s+보인다)")
KO_G2_DOUBLE_HEDGE_RE = _re(r"(?:가능성이\s+있을\s+수\s+있다|보여질\s+수\s+있다|할\s+수\s+있을\s+것으로\s+보일\s+수\s+있다)")

KO_H1_CONNECTOR_RE = _re(r"(?m)^\s*(?:또한|따라서|즉|나아가|아울러|게다가|더욱이)\b")
KO_H2_CONTRAST_RE = _re(r"(?m)^\s*(?:하지만|그러나)\b")
KO_H3_META_ENTRY_RE = _re(r"(?:이는\s+[^.!?\n]{0,60}의미한다|이\s+점에서|이\s+관점에서\s+보면|이\s+말은)")
KO_H4_IE_RE = _re(r"\b즉\b")

KO_I1_GEOTIDA_RE = _re(r"[가-힣]+(?:인|한|일|다는)\s+것이다")
KO_I2_DEPENDENT_NOUN_RE = _re(r"(?:라는\s+점(?:에\s+있다|에서)?|다는\s+점(?:에\s+있다|에서)?|주목할\s+점은|나아갈\s+바|할\s+수(?:가)?\s+있|하는\s+데(?:에)?)")
KO_I3_MEANS_ENDING_RE = _re(r"(?:라는\s+것|다는\s+것이다|다는\s+뜻이다|다는\s+의미다|다는\s+점이다)")
KO_I4_NEED_TO_RE = _re(r"(?:할\s+필요가\s+있다|[가-힣]*해야\s+(?:한다|합니다)|[가-힣]+야\s+(?:한다|합니다))")
KO_I5_NEEDED_RE = _re(r"[가-힣]+(?:이|가)\s+필요하다")
KO_I6_ABILITY_NOUN_RE = _re(r"[가-힣A-Za-z0-9]+\s+능력")

KO_J1_BOLD_OVERUSE_RE = _re(r"(?:\*\*|__)[^*_\n]{1,80}(?:\*\*|__)")
KO_J2_QUOTE_EMPHASIS_RE = _re(r"[\"'“‘][가-힣A-Za-z0-9\s]{1,30}[\"'”’]")
KO_J3_EM_DASH_RE = _re(r"—")
KO_J4_PAREN_ASIDE_RE = _re(r"\([^)]{0,80}(?:의미한다|뜻이다|시사한다|말한다)[^)]{0,80}\)")

META_REGEXES: tuple[Pattern[str], ...] = (
    _re(r"\bin this section\b"),
    _re(r"\blet's\s+(?:dive in|explore|unpack(?: this)?|break this down|take a closer look|dig deeper)\b"),
    _re(r"\bhere's what you need to know\b"),
    _re(r"\bthe key takeaway(?: here)? is\b"),
    _re(r"\bto summarize\b"),
    _re(r"\bin summary\b"),
    _re(r"\bin conclusion\b"),
    _re(r"\bit is important to note\b"),
    _re(r"\bit's important to note\b"),
    _re(r"\bit should be noted\b"),
    _re(r"\bit's worth noting that\b"),
    _re(r"\bby now you should\b"),
    _re(r"\bwe(?:'ll| will)\s+examine\b"),
    _re(r"\b(?:step by step|one by one)\b"),
    _re(r"(?:이\s+섹션에서는|이\s+장에서는|이번\s+장에서는|이\s+글에서는|정리하면|요약하면|종합하면|정리하자면|결론적으로|핵심은|다시\s+말해|다음과\s+같은|다음과\s+같이)")
)


def sp(name: str, severity: str, weight: float, regex: Pattern[str], description: str) -> StructuralPattern:
    return StructuralPattern(name, "structural_pattern", severity, weight, regex, description)


STRUCTURAL_PATTERNS: tuple[StructuralPattern, ...] = (
    sp("redefinition", "medium", 2.0, REDEFINITION_RE, "A is not X, it is Y / not X but Y redefinition."),
    sp("closing_summary", "low", 1.0, CLOSING_SUMMARY_RE, "Closing-summary repetition."),
    sp("progress_announcement", "medium", 1.0, PROGRESS_RE, "Progress announcement such as let's unpack or step by step."),
    sp("pre_classification", "low", 1.0, PRE_CLASSIFICATION_RE, "Pre-classification framing before content earns it."),
    sp("imagine_prompt", "low", 1.0, IMAGINE_RE, "Imagine/picture/visualize prompt."),
    sp("broad_field_dump", "medium", 1.5, BROAD_FIELD_DUMP_RE, "Broad overview followed by a field dump."),
    sp("colon_enumeration", "medium", 1.5, COLON_ENUMERATION_RE, "Colon followed by a comma-separated list of items."),
    sp("a_not_b_redefinition", "high", 2.5, A_NOT_B_RE, "A is not X, it is Y redefinition pattern."),
    sp("ko_A1_about_regarding", "high", 2.0, KO_A1_ABOUT_RE, "im-not-ai A-1: Korean '~에 대해/대하여' translationese."),
    sp("ko_A2_through", "high", 2.0, KO_A2_THROUGH_RE, "im-not-ai A-2: Korean '~를 통해/통하여' translationese."),
    sp("ko_A3_in_terms", "high", 2.0, KO_A3_IN_TERMS_RE, "im-not-ai A-3: Korean '~에 있어서' translationese."),
    sp("ko_A4_point_sense", "medium", 1.0, KO_A4_POINT_SENSE_RE, "im-not-ai A-4: '~라는 점에서' reasoning crutch."),
    sp("ko_A5_related", "medium", 1.0, KO_A5_RELATED_RE, "im-not-ai A-5: '~와 관련하여/관련된'."),
    sp("ko_A6_based_on", "medium", 1.0, KO_A6_BASED_ON_RE, "im-not-ai A-6: '~에 기반하여/바탕으로'."),
    sp("ko_A7_have", "high", 2.0, KO_A7_HAVE_RE, "im-not-ai A-7: '가지고 있다' possessive translationese."),
    sp("ko_A8_double_passive", "high", 2.0, KO_A8_DOUBLE_PASSIVE_RE, "im-not-ai A-8: double passive '~되어진다/~지게 된다'."),
    sp("ko_A9_by_passive", "medium", 1.0, KO_A9_BY_PASSIVE_RE, "im-not-ai A-9: '~에 의해' by-passive."),
    sp("ko_A10_can_overuse", "medium", 1.0, KO_A10_CAN_OVERUSE_RE, "im-not-ai A-10: Korean can-form '~할 수 있다'."),
    sp("ko_A11_purpose", "medium", 1.0, KO_A11_PURPOSE_RE, "im-not-ai A-11: '~을 위해' purpose clause."),
    sp("ko_A12_automated_passive", "medium", 1.0, KO_A12_AUTOMATED_PASSIVE_RE, "im-not-ai A-12: '만들어지다/이루어지다'."),
    sp("ko_A13_noun_stack", "medium", 1.0, KO_A13_NOUN_STACK_RE, "im-not-ai A-13: English-like Korean noun stack."),
    sp("ko_A14_and_sentence", "medium", 1.0, KO_A14_AND_SENTENCE_RE, "im-not-ai A-14: sentence-initial '그리고'."),
    sp("ko_A15_abstract_subject", "medium", 1.5, KO_A15_ABSTRACT_SUBJECT_RE, "im-not-ai A-15: abstract subject plus all-purpose verb."),
    sp("ko_B1_parenthesized_english", "medium", 1.0, KO_B1_PAREN_ENGLISH_RE, "im-not-ai B-1: Korean term with repeated English parenthetical."),
    sp("ko_B2_raw_english_term", "medium", 1.0, KO_B2_RAW_ENGLISH_RE, "im-not-ai B-2: untranslated English buzzword."),
    sp("ko_B3_long_english_quote", "medium", 1.0, KO_B3_LONG_ENGLISH_QUOTE_RE, "im-not-ai B-3: long English quote embedded in Korean prose."),
    sp("ko_B4_known_as", "low", 0.5, KO_B4_KNOWN_AS_RE, "im-not-ai B-4: '~라고 알려진/~로 일컬어지는'."),
    sp("ko_C1_mechanical_enumeration", "high", 2.0, KO_C1_ENUMERATION_RE, "im-not-ai C-1: mechanical 첫째/둘째/셋째 enumeration."),
    sp("ko_C2_bullet_block", "medium", 1.0, KO_C2_BULLET_BLOCK_RE, "im-not-ai C-2: excessive bullet block."),
    sp("ko_C3_generic_headings", "medium", 1.0, KO_C3_GENERIC_HEADINGS_RE, "im-not-ai C-3: generic 도입/본론/결론 headings."),
    sp("ko_C4_topic_sentence", "medium", 1.0, KO_C4_TOPIC_SENTENCE_RE, "im-not-ai C-4: paragraph-opening topic sentence formula."),
    sp("ko_C5_emoji", "high", 2.0, KO_C5_EMOJI_RE, "im-not-ai C-5: emoji decoration in prose."),
    sp("ko_C6_heading_summary", "medium", 1.0, KO_C6_HEADING_SUMMARY_RE, "im-not-ai C-6: heading followed by one-line summary."),
    sp("ko_C7_three_step", "medium", 1.0, KO_C7_THREE_STEP_RE, "im-not-ai C-7: 먼저/반면/결국 three-step paragraph formula."),
    sp("ko_C8_binary_parallel", "medium", 1.0, KO_C8_BINARY_PARALLEL_RE, "im-not-ai C-8: repeated A인가, B인가 parallelism."),
    sp("ko_C9_numeric_index", "medium", 1.0, KO_C9_NUMERIC_INDEX_RE, "im-not-ai C-9: 1) 2) 3) numeric indexing."),
    sp("ko_C10_colon_heading", "high", 2.0, KO_C10_COLON_HEADING_RE, "im-not-ai C-10: colon subtitle heading formula."),
    sp("ko_D1_conclusion_phrase", "high", 2.0, KO_D1_CONCLUSION_RE, "im-not-ai D-1: Korean conclusion/summary formula."),
    sp("ko_D2_importance_cliche", "high", 2.0, KO_D2_IMPORTANCE_RE, "im-not-ai D-2: overblown importance cliché."),
    sp("ko_D3_list_intro", "high", 2.0, KO_D3_LIST_INTRO_RE, "im-not-ai D-3: list-introduction cliché."),
    sp("ko_D4_hype", "high", 2.0, KO_D4_HYPE_RE, "im-not-ai D-4: hype vocabulary and new-era formulas."),
    sp("ko_D5_personified_abstract", "medium", 1.5, KO_D5_PERSONIFIED_ABSTRACT_RE, "im-not-ai D-5: personified abstract subject."),
    sp("ko_D6_closing_formula", "medium", 1.5, KO_D6_CLOSING_FORMULA_RE, "im-not-ai D-6: formulaic closing '~할 때/시점'."),
    sp("ko_D7_transformation_formula", "medium", 1.0, KO_D7_TRANSFORMATION_RE, "im-not-ai D-7: X에서 Y로 transformation slogan."),
    sp("ko_F1_degree_adverb", "medium", 1.0, KO_F1_DEGREE_ADVERB_RE, "im-not-ai F-1: excessive degree adverb."),
    sp("ko_F2_double_modifier", "medium", 1.0, KO_F2_DOUBLE_MODIFIER_RE, "im-not-ai F-2: synonym double modifier."),
    sp("ko_F3_role_function", "medium", 1.0, KO_F3_ROLE_FUNCTION_RE, "im-not-ai F-3: role/function or meaning/value doublet."),
    sp("ko_F4_suffix_abuse", "low", 0.5, KO_F4_SUFFIX_ABUSE_RE, "im-not-ai F-4: abstract suffix overuse."),
    sp("ko_F5_jeok_chain", "medium", 1.0, KO_F5_JEOK_CHAIN_RE, "im-not-ai F-5: '~적 N' abstract chain."),
    sp("ko_G1_hedging", "medium", 1.0, KO_G1_HEDGING_RE, "im-not-ai G-1: Korean hedge ending."),
    sp("ko_G2_double_hedge", "medium", 1.5, KO_G2_DOUBLE_HEDGE_RE, "im-not-ai G-2: double/triple hedge."),
    sp("ko_H1_connector", "medium", 1.0, KO_H1_CONNECTOR_RE, "im-not-ai H-1: sentence-initial Korean connector."),
    sp("ko_H2_contrast_connector", "medium", 1.0, KO_H2_CONTRAST_RE, "im-not-ai H-2: repeated 하지만/그러나 contrast connector."),
    sp("ko_H3_meta_entry", "high", 1.5, KO_H3_META_ENTRY_RE, "im-not-ai H-3: 이는/이 점에서 meta-entry."),
    sp("ko_H4_ie", "medium", 0.5, KO_H4_IE_RE, "im-not-ai H-4: redefining connector '즉'."),
    sp("ko_I1_geotida", "high", 2.0, KO_I1_GEOTIDA_RE, "im-not-ai I-1: '~것이다' formal-noun ending."),
    sp("ko_I2_dependent_noun", "medium", 1.0, KO_I2_DEPENDENT_NOUN_RE, "im-not-ai I-2: 점/바/수/데 dependent-noun crutch."),
    sp("ko_I3_means_ending", "medium", 1.0, KO_I3_MEANS_ENDING_RE, "im-not-ai I-3: '~다는 뜻/의미/점' ending."),
    sp("ko_I4_need_to", "medium", 1.0, KO_I4_NEED_TO_RE, "im-not-ai I-4: '~필요가 있다/해야 한다' recommendation ending."),
    sp("ko_I5_needed", "medium", 1.0, KO_I5_NEEDED_RE, "im-not-ai I-5: abstract '~이 필요하다'."),
    sp("ko_I6_ability_noun", "medium", 1.0, KO_I6_ABILITY_NOUN_RE, "im-not-ai I-6: repeated 'N 능력' ability noun."),
    sp("ko_J1_bold", "medium", 0.5, KO_J1_BOLD_OVERUSE_RE, "im-not-ai J-1: bold emphasis decoration."),
    sp("ko_J2_quote_emphasis", "medium", 0.5, KO_J2_QUOTE_EMPHASIS_RE, "im-not-ai J-2: quote emphasis decoration."),
    sp("ko_J3_em_dash", "low", 0.5, KO_J3_EM_DASH_RE, "im-not-ai J-3: em dash decoration."),
    sp("ko_J4_parenthetical_aside", "medium", 1.0, KO_J4_PAREN_ASIDE_RE, "im-not-ai J-4: parenthetical explanatory aside."),
)


def _candidate_paths(name: str = REFERENCE_NAME) -> Iterable[Path]:
    cwd = Path.cwd()
    for base in (cwd, *cwd.parents):
        yield base / "references" / name


@lru_cache(maxsize=1)
def load_banned_patterns_markdown() -> str:
    """Load the canonical structural-pattern reference markdown."""

    text, _source = read_reference_text(REFERENCE_NAME)
    if text:
        return text
    return "# Banned Structural Patterns\n\n" + "\n".join(
        f"- {pattern.name}: {pattern.description}" for pattern in STRUCTURAL_PATTERNS
    )


def load_structural_patterns() -> tuple[StructuralPattern, ...]:
    """Return regex-backed structural patterns.

    The markdown reference is loaded by ``load_banned_patterns_markdown`` for agent
    prompts; regex execution stays explicit so fallback behavior is stable.
    """

    return STRUCTURAL_PATTERNS
