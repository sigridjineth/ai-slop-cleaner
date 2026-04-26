"""Ralph iterative cleanup mode for AI-slop prose."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
from typing import Any

from ai_slop_cleaner.core.banned_words import load_banned_catalog
from ai_slop_cleaner.core.detector import analyze_text

SEVERITY_RANK = {"low": 1, "medium": 2, "high": 3, "critical": 4}

TRANSLATIONESE_REPLACEMENTS: tuple[tuple[re.Pattern[str], str], ...] = (
    (re.compile(r"\b[Ii]n order to\b"), "To"),
    (re.compile(r"\b[Dd]ue to the fact that\b"), "Because"),
    (re.compile(r"\b[Ww]ith regard to\b"), "About"),
    (re.compile(r"\b[Ii]n the event that\b"), "If"),
    (re.compile(r"\b[Ff]or the purpose of\b"), "To"),
    (re.compile(r"\b[Aa]t this point in time\b"), "Now"),
    (re.compile(r"에\s*있어(?:서)?"), "에서"),
    (re.compile(r"에\s*대(?:해(?:서)?|하여)"), "을"),
    (re.compile(r"을\s*통(?:해|하여)|를\s*통(?:해|하여)"), "로"),
    (re.compile(r"매우\s+"), ""),
    (re.compile(r"결론적으로\s*"), ""),
    (re.compile(r"요약하면\s*"), ""),
)


@dataclass(frozen=True)
class RalphOptions:
    threshold: int = 25
    max_iterations: int = 10
    output: Path | None = None
    overwrite: bool = False
    prefer_agents: bool = False


def _finding_rank(finding: dict[str, Any]) -> tuple[int, int]:
    severity = SEVERITY_RANK.get(str(finding.get("severity", "medium")), 2)
    has_line = 0 if finding.get("line") else 1
    return (-severity, has_line)


def top_findings(analysis: dict[str, Any], limit: int = 3) -> list[dict[str, Any]]:
    findings = [finding for finding in analysis.get("findings", []) if isinstance(finding, dict)]
    return sorted(findings, key=_finding_rank)[:limit]


def _entry_regex(entry: str) -> re.Pattern[str]:
    escaped = re.escape(entry)
    if re.search(r"[A-Za-z0-9]", entry):
        return re.compile(rf"(?<![\w-]){escaped}(?![\w-])", re.IGNORECASE)
    return re.compile(escaped, re.IGNORECASE)


def _match_case(replacement: str, matched: str) -> str:
    if not replacement:
        return ""
    if matched.isupper():
        return replacement.upper()
    if matched[:1].isupper():
        return replacement[:1].upper() + replacement[1:]
    return replacement


def replace_banned_words(text: str) -> str:
    """Replace banned words/phrases with plain alternatives from the catalogue."""

    catalog = load_banned_catalog()
    entries = sorted((*catalog.phrases, *catalog.single_words), key=len, reverse=True)
    cleaned = text
    for entry in entries:
        replacement = catalog.replacements.get(entry, "")
        if replacement == entry:
            continue
        pattern = _entry_regex(entry)
        cleaned = pattern.sub(lambda match: _match_case(replacement, match.group(0)), cleaned)
    return re.sub(r"[ \t]+([,.!?;:])", r"\1", cleaned)


def remove_markdown_decoration_overload(text: str) -> str:
    """Reduce emphasis, heading, blockquote, and horizontal-rule decoration."""

    cleaned = re.sub(r"(\*\*|__)([^\n]+?)\1", r"\2", text)
    cleaned = re.sub(r"(?:\*\*|__){1,}", "", cleaned)
    cleaned = re.sub(r"(?<!\*)\*([^*\n]{1,120})\*(?!\*)", r"\1", cleaned)
    cleaned = re.sub(r"_([^_\n]{1,120})_", r"\1", cleaned)
    lines: list[str] = []
    for line in cleaned.splitlines():
        stripped = line.strip()
        if re.fullmatch(r"(?:---+|\*\*\*+|___+)", stripped):
            continue
        line = re.sub(r"^\s{0,3}#{1,6}\s+", "", line)
        line = re.sub(r"^\s{0,3}>\s?", "", line)
        lines.append(line.rstrip())
    return "\n".join(lines)


def simplify_translationese(text: str) -> str:
    cleaned = text
    for pattern, replacement in TRANSLATIONESE_REPLACEMENTS:
        cleaned = pattern.sub(replacement, cleaned)
    cleaned = re.sub(r"\s+([,.!?;:])", r"\1", cleaned)
    cleaned = re.sub(r" {2,}", " ", cleaned)
    return cleaned


def break_repetitive_sentence_structures(text: str) -> str:
    """Vary repeated sentence starts with small, meaning-preserving edits."""

    pieces = re.split(r"([.!?]+\s+)", text)
    if len(pieces) < 5:
        return text
    sentences: list[str] = []
    separators: list[str] = []
    for idx, piece in enumerate(pieces):
        if idx % 2 == 0:
            sentences.append(piece)
        else:
            separators.append(piece)
    opener_counts: dict[str, int] = {}
    rewritten: list[str] = []
    for sentence in sentences:
        match = re.match(r"(\s*)([A-Za-z][A-Za-z'’-]*)(\s+)(.*)", sentence, flags=re.DOTALL)
        if not match:
            rewritten.append(sentence)
            continue
        opener = match.group(2).lower()
        opener_counts[opener] = opener_counts.get(opener, 0) + 1
        if opener_counts[opener] >= 3 and opener in {"this", "the", "it", "we", "you"}:
            tail = match.group(4).lstrip()
            rewritten.append(match.group(1) + (tail[:1].upper() + tail[1:] if tail else sentence))
        else:
            rewritten.append(sentence)
    rebuilt: list[str] = []
    for idx, sentence in enumerate(rewritten):
        rebuilt.append(sentence)
        if idx < len(separators):
            rebuilt.append(separators[idx])
    return "".join(rebuilt)


def clean_text_once(text: str, findings: list[dict[str, Any]] | None = None) -> str:
    """Apply one rule-based cleanup pass guided by the top findings."""

    cleaned = text
    # The top findings decide what gets reported for the iteration. The actual
    # fix pass stays deterministic and cheap by applying all safe transforms;
    # otherwise three banned-word findings can starve markdown or translationese
    # cleanup in the same short document.
    cleaned = replace_banned_words(cleaned)
    cleaned = break_repetitive_sentence_structures(cleaned)
    cleaned = remove_markdown_decoration_overload(cleaned)
    cleaned = simplify_translationese(cleaned)

    cleaned = re.sub(r"\n{3,}", "\n\n", cleaned)
    cleaned = re.sub(r"[ \t]+\n", "\n", cleaned)
    return cleaned.strip() + ("\n" if cleaned.endswith("\n") else "")


def _write_target(input_path: Path, text: str, output: Path | None, overwrite: bool) -> str | None:
    if output is not None:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(text, encoding="utf-8")
        return str(output)
    if overwrite:
        input_path.write_text(text, encoding="utf-8")
        return str(input_path)
    return None


def run_ralph(
    file: str | Path,
    *,
    threshold: int = 25,
    max_iterations: int = 10,
    output: str | Path | None = None,
    overwrite: bool = False,
    prefer_agents: bool = False,
) -> dict[str, Any]:
    """Run Ralph cleanup until score is low enough or iterations are exhausted."""

    input_path = Path(file).expanduser()
    output_path = Path(output).expanduser() if output is not None else None
    text = input_path.read_text(encoding="utf-8")
    iterations: list[dict[str, Any]] = []
    last_score: int | None = None
    written_to: str | None = None

    if max_iterations <= 0:
        analysis = analyze_text(text, source=str(input_path), prefer_agents=prefer_agents)
        return {
            "status": "max_iterations_reached",
            "message": "max iterations reached before cleanup",
            "final_score": int(analysis["score"]),
            "threshold": threshold,
            "iterations": iterations,
            "output": None,
            "text": text,
        }

    for iteration in range(1, max_iterations + 1):
        analysis = analyze_text(text, source=str(input_path), prefer_agents=prefer_agents)
        last_score = int(analysis["score"])
        findings = top_findings(analysis, limit=3)
        if last_score <= threshold:
            return {
                "status": "success",
                "message": f"score {last_score} is <= threshold {threshold}",
                "final_score": last_score,
                "threshold": threshold,
                "iterations": [*iterations, {"iteration": iteration, "score": last_score, "top_findings": findings, "changed": False}],
                "output": written_to,
                "text": text,
            }

        cleaned = clean_text_once(text, findings)
        changed = cleaned != text
        iterations.append({"iteration": iteration, "score": last_score, "top_findings": findings, "changed": changed})
        if not changed:
            return {
                "status": "stalled",
                "message": "no rule-based cleanup changed the text",
                "final_score": last_score,
                "threshold": threshold,
                "iterations": iterations,
                "output": written_to,
                "text": text,
            }
        text = cleaned
        written_to = _write_target(input_path, text, output_path, overwrite)

    final_analysis = analyze_text(text, source=str(output_path or input_path), prefer_agents=prefer_agents)
    return {
        "status": "max_iterations_reached",
        "message": f"max iterations reached with score {int(final_analysis['score'])}",
        "final_score": int(final_analysis["score"]),
        "threshold": threshold,
        "iterations": iterations,
        "output": written_to,
        "text": text,
    }
