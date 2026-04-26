"""Score aggregation for the AI Slop Score.

The score follows the 5-component weighted formula requested for v2:

``round(100 * (0.25*BWD + 0.25*SPV + 0.20*RHY + 0.15*META + 0.15*MD))``
"""

from __future__ import annotations

import math
from typing import Mapping

COMPONENT_KEYS = ("BWD", "SPV", "RHY", "META", "MD")
WEIGHTS: dict[str, float] = {
    "BWD": 0.25,
    "SPV": 0.25,
    "RHY": 0.20,
    "META": 0.15,
    "MD": 0.15,
}


def clip01(value: float | int | None) -> float:
    """Clamp a numeric value into ``0.0..1.0``."""

    try:
        numeric = float(value if value is not None else 0.0)
    except (TypeError, ValueError):
        return 0.0
    if math.isnan(numeric) or numeric < 0:
        return 0.0
    if numeric > 1:
        return 1.0
    return numeric


def round_half_up(value: float) -> int:
    """Round positive scores like Go/JavaScript instead of Python bankers rounding."""

    if math.isnan(value) or value <= 0:
        return 0
    return int(math.floor(value + 0.5))


def normalize_components(components: Mapping[str, float | int | None] | None) -> dict[str, float]:
    """Return all five components, clipped and rounded for repeatable reports."""

    components = components or {}
    return {key: round(clip01(components.get(key, 0.0)), 4) for key in COMPONENT_KEYS}


def score_from_components(components: Mapping[str, float | int | None] | None = None, **kwargs: float) -> int:
    """Aggregate normalized component subscores into a 0-100 integer."""

    merged: dict[str, float | int | None] = {}
    if components:
        merged.update(components)
    merged.update(kwargs)
    normalized = normalize_components(merged)
    raw = 100.0 * sum(WEIGHTS[key] * normalized[key] for key in COMPONENT_KEYS)
    return min(100, max(0, round_half_up(raw)))


def with_score(payload: dict) -> dict:
    """Attach normalized components and final score to an analysis payload."""

    components = normalize_components(payload.get("components"))
    payload = dict(payload)
    payload["components"] = components
    payload["score"] = score_from_components(components)
    return payload
