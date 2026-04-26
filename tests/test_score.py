from ai_slop_cleaner.core.scorer import normalize_components, score_from_components


def test_overall_formula_rounds_once():
    assert score_from_components({"BWD": 0.50, "SPV": 0.30, "RHY": 0.40, "META": 0.50, "MD": 0.25}) == 39


def test_components_are_clipped_and_complete():
    assert normalize_components({"BWD": 2, "SPV": -1, "RHY": 0.123456}) == {
        "BWD": 1.0,
        "SPV": 0.0,
        "RHY": 0.1235,
        "META": 0.0,
        "MD": 0.0,
    }
