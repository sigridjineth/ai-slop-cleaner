"""MCP tool implementations for AI Slop Cleaner."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from ai_slop_cleaner.core.detector import analyze_text

PASS_THRESHOLD = 25


def _text_from_args(text: str | None = None, file: str | None = None) -> tuple[str, str]:
    if text:
        return text, "inline text"
    if not file:
        raise ValueError("tool requires either text or file")
    path = Path(file).expanduser()
    return path.read_text(encoding="utf-8"), str(path)


def ai_slop_score(text: str | None = None, file: str | None = None) -> dict[str, Any]:
    """Return the 0-100 AI Slop Score with component breakdown."""

    content, source = _text_from_args(text, file)
    analysis = analyze_text(content, source=source)
    return {
        "score": analysis["score"],
        "components": analysis["components"],
        "details": analysis.get("details", {}),
        "detector": analysis.get("detector", "unknown"),
        "source": source,
    }


def ai_slop_analyze(text: str | None = None, file: str | None = None) -> dict[str, Any]:
    """Return findings, components, score, details, and document patterns."""

    content, source = _text_from_args(text, file)
    return analyze_text(content, source=source)


def ai_slop_check(text: str | None = None, file: str | None = None, threshold: int = PASS_THRESHOLD) -> dict[str, Any]:
    """Return a quick pass/fail result. Passing means score <= threshold."""

    result = ai_slop_score(text=text, file=file)
    score = int(result["score"])
    passed = score <= threshold
    return {
        "status": "pass" if passed else "fail",
        "pass": passed,
        "score": score,
        "threshold": threshold,
        "components": result["components"],
        "detector": result["detector"],
    }
