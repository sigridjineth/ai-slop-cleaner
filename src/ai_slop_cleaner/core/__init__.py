"""Core detection, fallback, and scoring logic for AI Slop Cleaner."""

from ai_slop_cleaner.core.detector import analyze_text
from ai_slop_cleaner.core.scorer import score_from_components

__all__ = ["analyze_text", "score_from_components"]
