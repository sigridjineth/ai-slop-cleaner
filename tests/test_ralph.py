from pathlib import Path

from ai_slop_cleaner.core.ralph import run_ralph


def test_ralph_stops_when_score_is_under_threshold(tmp_path):
    source = tmp_path / "plain.md"
    source.write_text("The release notes describe the fix and the measured impact.", encoding="utf-8")

    result = run_ralph(source, threshold=100, max_iterations=3)

    assert result["status"] == "success"
    assert result["final_score"] <= 100
    assert result["iterations"][0]["changed"] is False


def test_ralph_writes_rule_based_cleanup_to_output(tmp_path):
    source = tmp_path / "draft.md"
    output = tmp_path / "clean.md"
    source.write_text(
        "**In conclusion**, let's delve into the comprehensive landscape.\n"
        "## Key Takeaway\n"
        "이 문제에 있어서 매우 중요한 변화입니다.",
        encoding="utf-8",
    )

    result = run_ralph(source, threshold=0, max_iterations=1, output=output)

    cleaned = output.read_text(encoding="utf-8")
    assert result["status"] == "max_iterations_reached"
    assert result["iterations"][0]["changed"] is True
    assert "**" not in cleaned
    assert "In conclusion" not in cleaned
    assert "delve into" not in cleaned
    assert "에 있어서" not in cleaned


def test_ralph_honors_zero_max_iterations_without_writing(tmp_path):
    source = tmp_path / "draft.md"
    output = tmp_path / "clean.md"
    source.write_text("Let's delve into the robust landscape.", encoding="utf-8")

    result = run_ralph(source, threshold=0, max_iterations=0, output=output)

    assert result["status"] == "max_iterations_reached"
    assert result["iterations"] == []
    assert not output.exists()
