"""Banned structural-pattern loading and canonical regexes."""

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


REDEFINITION_RE = re.compile(
    r"(?i)(?:\bnot\s+\w+(?:\s+\w+){0,2}\b|rather\s+than\s+\w+(?:\s+\w+){0,2}|instead\s+of\s+\w+(?:\s+\w+){0,2})\s*[,;:]+\s*\b(?:it\s+is|is|are)\b|\bnot\s+\w+(?:\s+\w+){0,2}\s+but\s+\w+"
)
PROGRESS_RE = re.compile(
    r"(?i)\b(let's\s+(?:take a closer look|unpack(?: this)?|dig deeper|break this down|dive in|explore)|we(?:'ll| will)\s+examine(?: this)?(?: step by step)?|step by step|one by one)\b"
)
PRE_CLASSIFICATION_RE = re.compile(
    r"(?i)\b(?:there are|there is|we can divide|this divides|can be divided)\b[^.!?]{0,80}\b(?:types|branches|categories|groups|kinds)\b"
)
IMAGINE_RE = re.compile(r"(?i)\b(imagine|picture|visualize)\b[^.!?]{0,80}")
CLOSING_SUMMARY_RE = re.compile(
    r"(?i)\b(to summarize|in summary|in conclusion|in closing|in short|put simply|at the end of the day)\b"
)
BROAD_FIELD_DUMP_RE = re.compile(
    r"(?is)\b(?:fields|properties|attributes|schema|structure)\b[^.\n]{0,120}:\s*(?:\n\s*(?:[-*+]\s*)?`?[\w-]+`?\s*:[^\n]*){2,}"
)
POST_CODE_NARRATION_RE = re.compile(
    r"(?i)\b(first|then|next)\s+(?:we|the code|this)|\bwe\s+(?:define|create|call|return)\b"
)
COLON_ENUMERATION_RE = re.compile(
    r"(?i)[a-z]{3,}(?:\s+[a-z]+){0,4}\s*[:;]\s*(?:[a-z`][\w\-`]*(?:,\s*and\s+)?){2,}"
)
A_NOT_B_RE = re.compile(
    r"(?i)\b(?:is|are|was|were)\s+not\s+[a-z]+(?:\s+[a-z]+){0,3}\s*[,;]\s*(?:it\s+)?(?:is|are|was|were)\b|\bnot\s+[a-z]+(?:\s+[a-z]+){0,3}\s+but\s+(?:is\s+)?[a-z]+"
)

META_REGEXES: tuple[Pattern[str], ...] = (
    re.compile(r"(?i)\bin this section\b"),
    re.compile(r"(?i)\blet's\s+(?:dive in|explore|unpack(?: this)?|break this down|take a closer look|dig deeper)\b"),
    re.compile(r"(?i)\bhere's what you need to know\b"),
    re.compile(r"(?i)\bthe key takeaway(?: here)? is\b"),
    re.compile(r"(?i)\bto summarize\b"),
    re.compile(r"(?i)\bin summary\b"),
    re.compile(r"(?i)\bin conclusion\b"),
    re.compile(r"(?i)\bit is important to note\b"),
    re.compile(r"(?i)\bit's important to note\b"),
    re.compile(r"(?i)\bit should be noted\b"),
    re.compile(r"(?i)\bit's worth noting that\b"),
    re.compile(r"(?i)\bby now you should\b"),
    re.compile(r"(?i)\bwe(?:'ll| will)\s+examine\b"),
    re.compile(r"(?i)\b(?:step by step|one by one)\b"),
    re.compile(r"(?i)(이 장에서는|이번 장에서는|이 글에서는|정리하면|요약하면|핵심은|결론적으로|다시 말해)"),
)

STRUCTURAL_PATTERNS: tuple[StructuralPattern, ...] = (
    StructuralPattern(
        "redefinition",
        "structural_pattern",
        "medium",
        2.0,
        REDEFINITION_RE,
        'A is not X, it is Y / not X but Y redefinition.',
    ),
    StructuralPattern(
        "closing_summary",
        "structural_pattern",
        "low",
        1.0,
        CLOSING_SUMMARY_RE,
        "Closing-summary repetition.",
    ),
    StructuralPattern(
        "progress_announcement",
        "structural_pattern",
        "medium",
        1.0,
        PROGRESS_RE,
        "Progress announcement such as let's unpack or step by step.",
    ),
    StructuralPattern(
        "pre_classification",
        "structural_pattern",
        "low",
        1.0,
        PRE_CLASSIFICATION_RE,
        "Pre-classification framing before content earns it.",
    ),
    StructuralPattern(
        "imagine_prompt",
        "structural_pattern",
        "low",
        1.0,
        IMAGINE_RE,
        "Imagine/picture/visualize prompt.",
    ),
    StructuralPattern(
        "broad_field_dump",
        "structural_pattern",
        "medium",
        1.5,
        BROAD_FIELD_DUMP_RE,
        "Broad overview followed by a field dump.",
    ),
    StructuralPattern(
        "colon_enumeration",
        "structural_pattern",
        "medium",
        1.5,
        COLON_ENUMERATION_RE,
        "Colon followed by a comma-separated list of items.",
    ),
    StructuralPattern(
        "a_not_b_redefinition",
        "structural_pattern",
        "high",
        2.5,
        A_NOT_B_RE,
        "A is not X, it is Y redefinition pattern.",
    ),
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
