"""Core detection, fallback, and scoring logic for AI Slop Cleaner."""

from ai_slop_cleaner.core.code_smells import analyze_code_text, analyze_paths
from ai_slop_cleaner.core.detector import analyze_text
from ai_slop_cleaner.core.ralph import run_ralph
from ai_slop_cleaner.core.scorer import score_from_components

__all__ = ["analyze_text", "analyze_code_text", "analyze_paths", "run_ralph", "score_from_components"]
