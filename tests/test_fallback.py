import math

from ai_slop_cleaner.core import fallback


def repeated_words(word: str, count: int) -> str:
    return " ".join([word] * count)


def test_banned_word_density_uses_phrase_weight_and_exclusion(monkeypatch):
    monkeypatch.setenv("HOME", "/tmp/no-slopignore-home")
    text = repeated_words("plain", 990) + " delve into leverage"
    result = fallback.analyze_text(text)

    assert result["details"]["weighted_banned_hits"] == 3
    want = (1000 * 3.0 / result["details"]["word_count"]) / 16.0
    assert math.isclose(result["components"]["BWD"], round(want, 4), abs_tol=0.0001)


def test_score_ignores_code_blocks_for_banned_words_and_meta(monkeypatch):
    monkeypatch.setenv("HOME", "/tmp/no-slopignore-home")
    text = "\n".join([
        "```",
        "Let's dive in and delve into the comprehensive landscape.",
        "This chapter highlights pivotal choices. This chapter highlights pivotal choices.",
        "```",
        "Plain prose stays outside code.",
    ])
    result = fallback.analyze_text(text)

    assert result["details"]["weighted_banned_hits"] == 0
    assert result["details"]["meta_hits"] == 0
    assert result["components"]["RHY"] == 0
    assert result["details"]["rhythm_sample_confidence"] == "low"


def test_sloppy_prose_scores_high(monkeypatch):
    monkeypatch.setenv("HOME", "/tmp/no-slopignore-home")
    sloppy = "\n".join([
        "## Robust Landscape",
        "**In this section** we delve into a comprehensive and transformative landscape.",
        "This chapter highlights pivotal choices. This chapter highlights seamless workflows. This chapter highlights nuanced outcomes.",
        "The result is not a helper, it is a game-changer.",
        "- First, orchestrate the journey.",
        "- Second, navigate the hurdles.",
        "- Third, elucidate the tapestry.",
        "- Fourth, unlock a beacon.",
        "- Fifth, leverage synergy.",
        "Let's unpack this. 정리하면 핵심은 이 장에서는 메타 설명을 반복한다는 점입니다.",
    ])
    assert fallback.score_text(sloppy) >= 70
