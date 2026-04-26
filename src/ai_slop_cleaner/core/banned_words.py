"""Banned-word catalogue loading.

The canonical source is ``references/banned-words.md``. The loader searches the
current project, this package, and editable-install parents. A compact built-in
fallback keeps uvx/wheel installs functional even if reference files are not
available on disk.
"""

from __future__ import annotations

from dataclasses import dataclass
from functools import lru_cache
from importlib import resources
from pathlib import Path
import re
from typing import Iterable

REFERENCE_NAME = "banned-words.md"

DEFAULT_REPLACEMENTS: dict[str, str] = {
    "delve": "explore",
    "delve into": "explore",
    "elucidate": "explain",
    "underscore": "stress",
    "harness": "use",
    "leverage": "use",
    "bolster": "strengthen",
    "foster": "encourage",
    "showcase": "show",
    "streamline": "simplify",
    "revolutionize": "transform",
    "unveil": "reveal",
    "orchestrate": "arrange",
    "transcend": "go beyond",
    "exemplify": "show",
    "augment": "expand",
    "surpass": "exceed",
    "pinpoint": "identify",
    "scrutinize": "examine",
    "unravel": "solve",
    "embark": "start",
    "navigate": "handle",
    "elevate": "raise",
    "unlock": "open up",
    "unleash": "release",
    "dive": "look",
    "discover": "find",
    "craft": "make",
    "illuminate": "clarify",
    "pivotal": "key",
    "meticulous": "careful",
    "intricate": "complex",
    "transformative": "major",
    "groundbreaking": "new",
    "unparalleled": "unmatched",
    "comprehensive": "thorough",
    "robust": "strong",
    "crucial": "vital",
    "notable": "noteworthy",
    "formidable": "impressive",
    "nuanced": "subtle",
    "multifaceted": "varied",
    "paramount": "top",
    "instrumental": "helpful",
    "foundational": "basic",
    "commendable": "admirable",
    "cutting-edge": "latest",
    "seamless": "smooth",
    "vibrant": "lively",
    "bustling": "busy",
    "holistic": "whole",
    "poised": "ready",
    "remarkable": "striking",
    "realm": "area",
    "tapestry": "fabric",
    "landscape": "field",
    "beacon": "guide",
    "hurdles": "obstacles",
    "testament": "proof",
    "game-changer": "breakthrough",
    "journey": "process",
    "synergy": "cooperation",
    "in today's digital age": "",
    "it's important to note": "",
    "furthermore": "",
    "moreover": "",
    "not only this, but also this": "",
    "in conclusion": "",
    "in closing": "",
    "let's dive in": "",
    "let's explore": "",
    "it's worth noting that": "",
    "as we navigate": "",
    "in an era of": "",
    "it is essential that": "make sure",
    "it should be noted": "",
    "in order to": "to",
    "due to the fact that": "because",
    "with regard to": "about",
    "in the event that": "if",
    "for the purpose of": "to",
    "at this point in time": "now",
    "in spite of the fact that": "although",
    "in the absence of": "without",
    "it is evident that": "clearly",
    "it is apparent that": "clearly",
    "there is a need for": "we need",
    "a significant number of": "many",
    "a considerable amount of": "much",
    "it is interesting to note that": "",
    "one could argue that": "",
    "it is possible that": "",
    "there is a possibility that": "",
    "it may be worth considering": "",
    "arguably": "",
    "potentially": "",
    "in some cases": "",
    "to a certain extent": "",
    "by and large": "",
}


@dataclass(frozen=True)
class BannedCatalog:
    """Normalized banned entries split by single words and phrases."""

    single_words: tuple[str, ...]
    phrases: tuple[str, ...]
    replacements: dict[str, str]
    source: str


def _candidate_reference_paths(name: str = REFERENCE_NAME) -> Iterable[Path]:
    """Yield likely reference-file locations without assuming an install mode."""

    seen: set[Path] = set()
    cwd = Path.cwd()
    for base in (cwd, *cwd.parents):
        path = base / "references" / name
        if path not in seen:
            seen.add(path)
            yield path

    here = Path(__file__).resolve()
    for base in (here.parent, *here.parents):
        for rel in (Path("references") / name, Path("..") / ".." / ".." / "references" / name):
            path = (base / rel).resolve()
            if path not in seen:
                seen.add(path)
                yield path


def read_reference_text(name: str = REFERENCE_NAME) -> tuple[str, str] | tuple[None, None]:
    """Return ``(text, source)`` for a reference markdown file if found."""

    for path in _candidate_reference_paths(name):
        if path.exists():
            return path.read_text(encoding="utf-8"), str(path)

    try:
        resource = resources.files("ai_slop_cleaner.references").joinpath(name)
        return resource.read_text(encoding="utf-8"), f"package:{name}"
    except (ModuleNotFoundError, FileNotFoundError, AttributeError):
        return None, None


def normalize_entry(raw: str) -> str:
    """Normalize a markdown catalogue entry for matching."""

    entry = raw.strip().strip("`*_ ").strip('"“”')
    entry = re.sub(r"\s+\([^)]*\)\s*$", "", entry)
    entry = re.sub(r"\s+", " ", entry).strip(" .;,:\t\n\r").lower()
    return entry


def _first_replacement(raw: str) -> str:
    cleaned = re.sub(r"\([^)]*\)", "", raw).strip()
    if not cleaned:
        return ""
    return normalize_entry(cleaned.split(",")[0])


def _add_entry(entry_map: dict[str, str], raw: str, replacement: str = "") -> None:
    entry = normalize_entry(raw)
    if not entry or entry in {"banned", "stiff"} or "---" in entry:
        return
    entry_map.setdefault(entry, replacement)


def _first_quoted(line: str) -> str:
    for start_q, end_q in (("\"", "\""), ("“", "”")):
        start = line.find(start_q)
        if start < 0:
            continue
        end = line.find(end_q, start + len(start_q))
        if end >= 0:
            return line[start + len(start_q) : end]
    return ""


def parse_banned_catalog(markdown: str, source: str = "markdown") -> BannedCatalog:
    """Parse the first-column banned entries and quoted bullet phrases."""

    entries: dict[str, str] = {}

    for line in markdown.splitlines():
        trimmed = line.strip()
        if trimmed.startswith("|") and trimmed.count("|") >= 2:
            cols = [c.strip() for c in trimmed.strip("|").split("|")]
            if len(cols) < 2:
                continue
            banned_cell = cols[0]
            replacement = _first_replacement(cols[1]) if len(cols) > 1 else ""
            if banned_cell.lower() in {"banned", "stiff"} or "---" in banned_cell:
                continue
            for part in banned_cell.split(","):
                _add_entry(entries, part, replacement)
            continue

        if trimmed.startswith("- "):
            quoted = _first_quoted(trimmed)
            if quoted:
                _add_entry(entries, quoted, "")

    if not entries:
        entries.update(DEFAULT_REPLACEMENTS)
        source = "built-in"

    single_words = sorted(k for k in entries if " " not in k)
    phrases = sorted((k for k in entries if " " in k), key=lambda value: (-len(value), value))
    return BannedCatalog(tuple(single_words), tuple(phrases), dict(entries), source)


@lru_cache(maxsize=1)
def load_banned_catalog() -> BannedCatalog:
    """Load the canonical banned-word catalogue."""

    text, source = read_reference_text(REFERENCE_NAME)
    if text:
        return parse_banned_catalog(text, source or "markdown")
    return parse_banned_catalog("", "built-in")


def banned_reference_markdown() -> str:
    """Return reference markdown for prompts, falling back to compact defaults."""

    text, _source = read_reference_text(REFERENCE_NAME)
    if text:
        return text
    entries = sorted(DEFAULT_REPLACEMENTS)
    return "# Banned Words & Phrases\n\n" + "\n".join(f"- {entry}" for entry in entries)
