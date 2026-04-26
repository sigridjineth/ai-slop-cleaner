import pytest

from ai_slop_cleaner.core import fallback
from ai_slop_cleaner.core.banned_patterns import load_structural_patterns


PATTERN_EXAMPLES = {
    "ko_A1_about_regarding": "AI 규제에 대해 논의할 필요가 있다.",
    "ko_A2_through": "데이터 분석을 통해 인사이트를 얻는다.",
    "ko_A3_in_terms": "이 문제에 있어서 중요한 것은 속도다.",
    "ko_A4_point_sense": "확장성이 뛰어나다는 점에서 의미가 있다.",
    "ko_A5_related": "보안과 관련하여 주의해야 한다.",
    "ko_A6_based_on": "데이터에 기반하여 판단한다.",
    "ko_A7_have": "강한 경쟁력을 가지고 있다.",
    "ko_A8_double_passive": "그 결정은 판단되어진다.",
    "ko_A9_by_passive": "AI에 의해 생성된 이미지다.",
    "ko_A10_can_overuse": "효율을 높일 수 있다.",
    "ko_A11_purpose": "고객 만족을 위해 노력한다.",
    "ko_A12_automated_passive": "합의가 이루어졌다.",
    "ko_A13_noun_stack": "AI 기술 발전 속도 가속화 문제가 남았다.",
    "ko_A14_and_sentence": "그리고 자리에 앉았다.",
    "ko_A15_abstract_subject": "이 전략은 시장 지형을 흔들고 있습니다.",
    "ko_B1_parenthesized_english": "인공지능(AI)은 중요하다.",
    "ko_B2_raw_english_term": "이 framework를 leverage하여 성과를 낸다.",
    "ko_B3_long_english_quote": '그는 "This is a very long English quotation"라고 말했다.',
    "ko_B4_known_as": "AGI라고 알려진 범용 인공지능.",
    "ko_C1_mechanical_enumeration": "첫째, 속도다. 둘째, 비용이다. 셋째, 품질이다.",
    "ko_C2_bullet_block": "- 속도\n- 비용\n- 품질\n",
    "ko_C3_generic_headings": "## 본론\n내용",
    "ko_C4_topic_sentence": "핵심은 비용 구조다.",
    "ko_C5_emoji": "✅ 핵심 기능을 확인한다.",
    "ko_C6_heading_summary": "이 섹션에서는 주요 쟁점을 다룬다.",
    "ko_C7_three_step": "먼저 원인을 본다. 반면 비용은 낮다. 결국 전환이 필요하다.",
    "ko_C8_binary_parallel": "독점인가, 확산인가가 쟁점이다.",
    "ko_C9_numeric_index": "1) 표준화가 빨라졌다. 2) 비용이 낮아졌다. 3) 활용이 늘었다.",
    "ko_C10_colon_heading": "### 서론: 제조업의 미래\n본문",
    "ko_D1_conclusion_phrase": "결론적으로 변화가 필요하다.",
    "ko_D2_importance_cliche": "이는 시사하는 바가 크다.",
    "ko_D3_list_intro": "크게 세 가지로 나눌 수 있다.",
    "ko_D4_hype": "압도적 성과와 혁신적인 변화다.",
    "ko_D5_personified_abstract": "AI 대전은 질문을 던집니다.",
    "ko_D6_closing_formula": "지금이야말로 전환을 고민할 때입니다.",
    "ko_D7_transformation_formula": "규모의 경쟁에서 전략의 경쟁으로 이동한다.",
    "ko_F1_degree_adverb": "매우 중요한 변화다.",
    "ko_F2_double_modifier": "새롭고 혁신적인 접근이다.",
    "ko_F3_role_function": "플랫폼으로서의 역할과 기능을 수행한다.",
    "ko_F4_suffix_abuse": "근본적 관점에서 구조적 변화가 필요하다.",
    "ko_F5_jeok_chain": "기술적 안정성이 필요하다.",
    "ko_G1_hedging": "성장할 수 있을 것으로 보인다.",
    "ko_G2_double_hedge": "개선 가능성이 있을 수 있다.",
    "ko_H1_connector": "또한 이는 중요하다.",
    "ko_H2_contrast_connector": "그러나 문제는 남는다.",
    "ko_H3_meta_entry": "이는 변화를 의미한다.",
    "ko_H4_ie": "즉 비용 문제다.",
    "ko_I1_geotida": "핵심인 것이다.",
    "ko_I2_dependent_noun": "주목할 점은 비용이다.",
    "ko_I3_means_ending": "민감하다는 뜻이다.",
    "ko_I4_need_to": "균형을 맞춰야 한다.",
    "ko_I5_needed": "혁신이 필요하다.",
    "ko_I6_ability_noun": "추론 능력이 중요하다.",
    "ko_J1_bold": "**핵심 단어**를 강조한다.",
    "ko_J2_quote_emphasis": "'금융 슈퍼앱' 전략이다.",
    "ko_J3_em_dash": "AI는 도구 — 그 이상은 아니다.",
    "ko_J4_parenthetical_aside": "전환이 필요하다(이는 비용 절감을 의미한다).",
}


@pytest.mark.parametrize("name, example", sorted(PATTERN_EXAMPLES.items()))
def test_im_not_ai_structural_regex_examples_match(name, example):
    patterns = {pattern.name: pattern.regex for pattern in load_structural_patterns()}
    assert name in patterns
    assert patterns[name].search(example), name


def test_korean_rhythm_rules_feed_rhy_component(monkeypatch):
    monkeypatch.setenv("HOME", "/tmp/no-slopignore-home")
    text = "\n\n".join(
        [
            "시장은 빠르게 변한다. 비용은 계속 오른다. 조직은 대응한다.",
            "정부는 기준을 만든다. 기업은 투자를 늘린다. 학교는 교육을 바꾼다.",
            "현장은 다시 묻는다. 팀은 다시 답한다. 고객은 결과를 본다.",
        ]
    )
    result = fallback.analyze_text(text)

    assert result["components"]["RHY"] > 0
    assert any(finding["category"] == "rhythm_issue" for finding in result["findings"])


def test_human_framework_positive_signals_are_reported(monkeypatch):
    monkeypatch.setenv("HOME", "/tmp/no-slopignore-home")
    text = "솔직히 말하면 제 경험으로는 2026년 3월의 장애가 컸다. 예를 들어 결제 지연이 17분 이어졌다. 그런데 팀은 바로 롤백했다."
    result = fallback.analyze_text(text)
    human = result["doc_patterns"]["human_framework"]

    assert human["H_honest_human_flaws"]["marker_count"] >= 1
    assert human["M_memorable_specifics"]["specific_marker_count"] >= 2
    assert human["A_authentic_perspective"]["marker_count"] >= 1
    assert human["N_natural_flow"]["conversational_connector_count"] >= 1
