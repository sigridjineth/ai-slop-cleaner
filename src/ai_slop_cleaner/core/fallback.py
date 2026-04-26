"""Regex fallback detector used when Claude/Codex subagents are unavailable.

This ports the essential scoring and detection behavior from the former Go CLI:
code/front matter stripping, banned-word density, structural patterns, rhythm
monotony, meta commentary density, markdown overuse, and ``.slopignore``.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
import math
from pathlib import Path
import re
from statistics import mean, pstdev
from typing import Iterable, Pattern

from ai_slop_cleaner.core.banned_patterns import (
    META_REGEXES,
    POST_CODE_NARRATION_RE,
    REDEFINITION_RE,
    load_structural_patterns,
)
from ai_slop_cleaner.core.banned_words import load_banned_catalog
from ai_slop_cleaner.core.scorer import clip01, score_from_components

BANNED_DENSITY_MAX = 16.0
STRUCTURAL_DENSITY_MAX = 10.0
META_DENSITY_MAX = 8.0
MARKDOWN_DENSITY_MAX = 12.0

WORD_RE = re.compile(r"[\w]+(?:['’][\w]+)*", re.UNICODE)
SENTENCE_END_RE = re.compile(r"[.!?]+(?:\s+|$)")
BULLET_LINE_RE = re.compile(r"^\s*(?:[-*+]\s+|\d+[.)]\s+)")
ORDERED_LINE_RE = re.compile(r"^\s*\d+[.)]\s+")
HEADING_LINE_RE = re.compile(r"^\s{0,3}#{1,6}\s+")
BOLD_RE = re.compile(r"(?:\*\*|__)[^*_][^\n]*?(?:\*\*|__)")
TABLE_LINE_RE = re.compile(r"^\s*\|.+\|\s*$")
BLOCKQUOTE_RE = re.compile(r"^\s{0,3}>")
HORIZONTAL_RULE_RE = re.compile(r"^\s{0,3}(?:---+|\*\*\*+|___+)\s*$")
INLINE_CODE_RE = re.compile(r"`[^`\n]*`")

PRONOUNS = {
    "i", "me", "my", "mine", "you", "your", "yours", "he", "him", "his", "she", "her",
    "hers", "it", "its", "we", "us", "our", "ours", "they", "them", "their", "theirs",
    "who", "whom", "whose", "which", "what",
}
DETERMINERS = {"a", "an", "the", "this", "that", "these", "those", "each", "every", "some", "any", "few", "many", "much"}
PREPOSITIONS = {"of", "to", "in", "for", "with", "on", "at", "by", "from", "about", "as", "into", "over", "after", "before", "between", "through", "without", "within", "under", "around"}
CONJUNCTIONS = {"and", "or", "but", "nor", "yet", "so", "because", "although", "while", "if", "when", "unless"}
COMMON_VERBS = {"is", "are", "was", "were", "be", "being", "been", "am", "have", "has", "had", "do", "does", "did", "make", "makes", "made", "use", "uses", "used", "need", "needs", "contains", "includes", "shows", "show", "discusses", "covers", "highlights", "matters", "means"}
COMMON_ADVERBS = {"very", "more", "most", "often", "also", "then"}
COMMON_ADJECTIVES = {"good", "bad", "new", "old", "key", "same", "clear"}
FUNCTION_WORDS = {"a", "an", "the", "of", "to", "in", "for", "with", "and", "or", "but", "is", "are", "was", "were"}
CONNECTOR_OPENERS = {"furthermore", "moreover", "however", "therefore", "consequently", "additionally", "meanwhile", "overall", "first", "second", "third", "as a result", "in addition", "on the other"}


@dataclass(frozen=True)
class Finding:
    """A single fallback finding."""

    line: int | None
    span: tuple[int, int] | None
    category: str
    severity: str
    text: str
    context: str = ""
    suggested_fix: str = ""
    detector: str = "fallback"


@dataclass(frozen=True)
class PreparedDocument:
    original: str
    clean_text: str
    prose_lines: tuple[str, ...]
    sentences: tuple[str, ...]
    word_count: int
    inline_code_spans: int
    line_offsets: tuple[int, ...]


def load_slopignore() -> tuple[Pattern[str], ...]:
    """Load local and home ``.slopignore`` regexes."""

    patterns: list[Pattern[str]] = []
    for path in (Path.cwd() / ".slopignore", Path.home() / ".slopignore"):
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except OSError:
            continue
        for line in lines:
            raw = line.strip()
            if not raw or raw.startswith("#"):
                continue
            try:
                patterns.append(re.compile(raw))
            except re.error:
                continue
    return tuple(patterns)


def _should_ignore(line: str, patterns: Iterable[Pattern[str]]) -> bool:
    return any(pattern.search(line) for pattern in patterns)


def remove_front_matter(text: str) -> str:
    lines = text.splitlines()
    if not lines:
        return text
    first = lines[0].strip()
    if first not in {"---", "+++"}:
        return text
    for idx, line in enumerate(lines[1:], start=1):
        if line.strip() == first:
            return "\n".join(lines[idx + 1 :])
    return text


def _fence_marker(trimmed: str) -> str:
    if trimmed.startswith("```"):
        return "```"
    if trimmed.startswith("~~~"):
        return "~~~"
    return ""


def _strip_inline_code(line: str) -> tuple[str, int]:
    spans = list(INLINE_CODE_RE.finditer(line))
    if not spans:
        return line, 0
    chars = list(line)
    for match in spans:
        for idx in range(match.start(), match.end()):
            chars[idx] = " "
    return "".join(chars), len(spans)


def word_tokens(text: str) -> list[str]:
    return WORD_RE.findall(text)


def split_sentences(text: str) -> tuple[str, ...]:
    sentences: list[str] = []
    for paragraph in re.split(r"\n\s*\n+", text):
        paragraph = paragraph.strip()
        if not paragraph:
            continue
        start = 0
        for match in re.finditer(r"[.!?]+", paragraph):
            part = paragraph[start : match.start()].strip()
            if part:
                sentences.append(part)
            start = match.end()
        tail = paragraph[start:].strip()
        if tail:
            sentences.append(tail)
    return tuple(sentences)


def prepare_document(text: str, ignore_patterns: Iterable[Pattern[str]] | None = None) -> PreparedDocument:
    ignore_patterns = tuple(ignore_patterns or load_slopignore())
    text = remove_front_matter(text)
    lines = text.split("\n")
    in_fence = False
    clean_lines: list[str] = []
    prose_lines: list[str] = []
    line_offsets: list[int] = []
    offset = 0
    inline_code_spans = 0

    for line in lines:
        line_offsets.append(offset)
        offset += len(line) + 1
        trimmed = line.strip()
        if _fence_marker(trimmed):
            in_fence = not in_fence
            continue
        if in_fence or _should_ignore(line, ignore_patterns):
            continue
        stripped, spans = _strip_inline_code(line)
        inline_code_spans += spans
        clean_lines.append(stripped)
        if stripped.strip():
            prose_lines.append(stripped)

    clean_text = "\n".join(clean_lines)
    return PreparedDocument(
        original=text,
        clean_text=clean_text,
        prose_lines=tuple(prose_lines),
        sentences=split_sentences(clean_text),
        word_count=max(len(word_tokens(clean_text)), 1),
        inline_code_spans=inline_code_spans,
        line_offsets=tuple(line_offsets),
    )


def _line_for_span(text: str, start: int) -> int:
    return text.count("\n", 0, start) + 1


def _line_context(text: str, line: int) -> str:
    lines = text.splitlines()
    if 1 <= line <= len(lines):
        return lines[line - 1].strip()
    return ""


def _entry_regex(entry: str) -> Pattern[str]:
    return re.compile(r"(?i)(?<![\w'’])" + re.escape(entry) + r"(?![\w'’])")


def _phrase_regex(phrase: str) -> Pattern[str]:
    parts = [re.escape(p) for p in phrase.split()]
    return re.compile(r"(?i)(?<![\w'’])" + r"\s+".join(parts) + r"(?![\w'’])")


def _span_overlaps(span: tuple[int, int], spans: Iterable[tuple[int, int]]) -> bool:
    return any(span[0] < other[1] and span[1] > other[0] for other in spans)


def _banned_word_density(doc: PreparedDocument) -> tuple[float, int, list[Finding]]:
    catalog = load_banned_catalog()
    phrase_spans: list[tuple[int, int]] = []
    findings: list[Finding] = []
    phrase_hits = 0

    for phrase in catalog.phrases:
        for match in _phrase_regex(phrase).finditer(doc.clean_text):
            phrase_hits += 1
            span = match.span()
            phrase_spans.append(span)
            line = _line_for_span(doc.clean_text, span[0])
            findings.append(
                Finding(
                    line=line,
                    span=span,
                    category="banned_phrase",
                    severity="high",
                    text=match.group(0),
                    context=_line_context(doc.clean_text, line),
                    suggested_fix=catalog.replacements.get(phrase, ""),
                )
            )

    single_hits = 0
    for word in catalog.single_words:
        for match in _entry_regex(word).finditer(doc.clean_text):
            span = match.span()
            if _span_overlaps(span, phrase_spans):
                continue
            single_hits += 1
            line = _line_for_span(doc.clean_text, span[0])
            findings.append(
                Finding(
                    line=line,
                    span=span,
                    category="banned_word",
                    severity="high",
                    text=match.group(0),
                    context=_line_context(doc.clean_text, line),
                    suggested_fix=catalog.replacements.get(word, ""),
                )
            )

    weighted_hits = single_hits + (2 * phrase_hits)
    density = 1000.0 * weighted_hits / doc.word_count
    return clip01(density / BANNED_DENSITY_MAX), weighted_hits, findings


def _coarse_token_class(token: str) -> str:
    token = token.lower()
    if token.isdigit():
        return "number"
    if token in PRONOUNS:
        return "pronoun"
    if token in DETERMINERS:
        return "determiner"
    if token in PREPOSITIONS:
        return "preposition"
    if token in CONJUNCTIONS:
        return "conjunction"
    if token in COMMON_VERBS or token.endswith("ed") or token.endswith("ing"):
        return "verb"
    if token in COMMON_ADVERBS or token.endswith("ly"):
        return "adverb"
    if token in COMMON_ADJECTIVES or token.endswith(("ive", "al", "ous", "ful", "less", "able", "ible", "ic", "ary")):
        return "adjective"
    return "noun"


def sentence_skeleton(sentence: str) -> str:
    tokens = [token.lower() for token in word_tokens(sentence)]
    if not tokens:
        return ""
    return "-".join(_coarse_token_class(token) for token in tokens[:8])


def count_sentence_template_streaks(sentences: Iterable[str]) -> int:
    streaks = 0
    last = ""
    run = 0
    for sentence in sentences:
        skeleton = sentence_skeleton(sentence)
        if skeleton and skeleton == last:
            run += 1
        else:
            if run >= 3:
                streaks += 1
            last = skeleton
            run = 1 if skeleton else 0
    if run >= 3:
        streaks += 1
    return streaks


def max_sentence_template_streak(sentences: Iterable[str]) -> int:
    max_streak = 0
    last = ""
    run = 0
    for sentence in sentences:
        skeleton = sentence_skeleton(sentence)
        if skeleton and skeleton == last:
            run += 1
        else:
            max_streak = max(max_streak, run)
            last = skeleton
            run = 1 if skeleton else 0
    return max(max_streak, run)


def _has_repeated_section_mold(lines: Iterable[str]) -> bool:
    lines = list(lines)
    mold_count = 0
    for idx, line in enumerate(lines):
        if not HEADING_LINE_RE.search(line):
            continue
        intro_lines = 0
        bullet_lines = 0
        j = idx + 1
        while j < len(lines) and not BULLET_LINE_RE.search(lines[j]) and not HEADING_LINE_RE.search(lines[j]):
            if lines[j].strip():
                intro_lines += 1
            j += 1
        while j < len(lines) and BULLET_LINE_RE.search(lines[j]):
            bullet_lines += 1
            j += 1
        if 1 <= intro_lines <= 2 and 3 <= bullet_lines <= 6:
            mold_count += 1
    return mold_count >= 3


def _count_over_numbered_sequences(text: str, lines: Iterable[str]) -> int:
    count = 0
    if len(re.findall(r"(?i)\b(first|second|third|fourth|fifth|sixth|finally|lastly)\b", text)) >= 3:
        count += 1
    run = 0
    for line in lines:
        if ORDERED_LINE_RE.search(line):
            run += 1
        else:
            if run >= 3:
                count += 1
            run = 0
    if run >= 3:
        count += 1
    return count


def _count_connector_monotony(text: str) -> int:
    counts: dict[str, int] = {}
    for paragraph in re.split(r"\n\s*\n+", text):
        tokens = [token.lower() for token in word_tokens(paragraph)]
        if not tokens:
            continue
        opener = tokens[0]
        for size in (3, 2):
            if len(tokens) >= size:
                candidate = " ".join(tokens[:size])
                if candidate in CONNECTOR_OPENERS:
                    opener = candidate
                    break
        if opener in CONNECTOR_OPENERS:
            counts[opener] = counts.get(opener, 0) + 1
    return sum(1 for count in counts.values() if count >= 3)


def _count_post_code_narration(text: str) -> int:
    in_fence = False
    just_closed = False
    count = 0
    for line in text.splitlines():
        trimmed = line.strip()
        if _fence_marker(trimmed):
            if in_fence:
                just_closed = True
            in_fence = not in_fence
            continue
        if in_fence or not trimmed:
            continue
        if just_closed:
            if POST_CODE_NARRATION_RE.search(line):
                count += 1
            just_closed = False
    return count


def _structural_pattern_subscore(doc: PreparedDocument) -> tuple[float, float, list[Finding]]:
    weighted_hits = 0.0
    findings: list[Finding] = []

    for pattern in load_structural_patterns():
        for match in pattern.regex.finditer(doc.clean_text):
            weighted_hits += pattern.weight
            line = _line_for_span(doc.clean_text, match.start())
            findings.append(
                Finding(
                    line=line,
                    span=match.span(),
                    category=pattern.category,
                    severity=pattern.severity,
                    text=match.group(0).strip(),
                    context=_line_context(doc.clean_text, line),
                    suggested_fix=pattern.description,
                )
            )

    streaks = count_sentence_template_streaks(doc.sentences)
    if streaks:
        weighted_hits += 2.0 * streaks
        findings.append(Finding(None, None, "rhythm_issue", "medium", f"{streaks} repeated sentence-template streak(s)", suggested_fix="Vary adjacent sentence structure."))

    if _has_repeated_section_mold(doc.prose_lines):
        weighted_hits += 2.0
        findings.append(Finding(None, None, "structural_pattern", "medium", "repeated heading-intro-bullets section mold", suggested_fix="Vary section architecture."))

    numbered = _count_over_numbered_sequences(doc.clean_text, doc.prose_lines)
    if numbered:
        weighted_hits += 1.5 * numbered
        findings.append(Finding(None, None, "structural_pattern", "medium", f"{numbered} over-numbered list/framing sequence(s)", suggested_fix="Fold simple sequences into prose unless order matters."))

    connector = _count_connector_monotony(doc.clean_text)
    if connector:
        weighted_hits += float(connector)
        findings.append(Finding(None, None, "structural_pattern", "low", f"{connector} repeated connector opener group(s)", suggested_fix="Start paragraphs with the subject instead."))

    post_code = _count_post_code_narration(doc.original)
    if post_code:
        weighted_hits += float(post_code)
        findings.append(Finding(None, None, "structural_pattern", "low", f"{post_code} post-code textbook narration line(s)", suggested_fix="Keep code explanation high-level."))

    density = 1000.0 * weighted_hits / doc.word_count
    return clip01(density / STRUCTURAL_DENSITY_MAX), weighted_hits, findings


def _repeated_opener_rate(sentences: Iterable[str]) -> float:
    openers: list[str] = []
    counts: dict[str, int] = {}
    sentences = list(sentences)
    for sentence in sentences:
        tokens = [token.lower() for token in word_tokens(sentence)]
        opener = tokens[0] if tokens else ""
        openers.append(opener)
        if opener:
            counts[opener] = counts.get(opener, 0) + 1
    if not sentences:
        return 0.0
    repeated = sum(1 for opener in openers if opener and counts.get(opener, 0) >= 3)
    return repeated / len(sentences)


def _sentence_ending_key(sentence: str) -> str:
    tokens = [token.lower() for token in word_tokens(sentence)]
    content = [token for token in tokens if token not in FUNCTION_WORDS]
    if len(content) >= 2:
        return " ".join(content[-2:])
    if content:
        return content[0]
    return tokens[-1] if tokens else ""


def _repeated_ending_rate(sentences: Iterable[str]) -> float:
    endings: list[str] = []
    counts: dict[str, int] = {}
    sentences = list(sentences)
    for sentence in sentences:
        ending = _sentence_ending_key(sentence)
        endings.append(ending)
        if ending:
            counts[ending] = counts.get(ending, 0) + 1
    if not sentences:
        return 0.0
    repeated = sum(1 for ending in endings if ending and counts.get(ending, 0) >= 2)
    return repeated / len(sentences)


def _rhythm_monotony_subscore(doc: PreparedDocument) -> tuple[float, float, str, list[Finding]]:
    findings: list[Finding] = []
    if len(doc.sentences) < 5:
        return 0.0, 0.0, "low", findings
    lengths = [len(word_tokens(sentence)) for sentence in doc.sentences if word_tokens(sentence)]
    if len(lengths) < 5:
        return 0.0, 0.0, "low", findings
    avg = mean(lengths)
    sd = pstdev(lengths)
    cv = sd / max(avg, 1.0)
    cv_penalty = clip01((0.55 - cv) / 0.35)
    if avg < 8:
        cv_penalty *= 0.5
    opener_rate = _repeated_opener_rate(doc.sentences)
    opener_penalty = clip01((opener_rate - 0.15) / 0.25)
    ending_rate = _repeated_ending_rate(doc.sentences)
    ending_penalty = clip01((ending_rate - 0.15) / 0.25)
    template_penalty = clip01((max_sentence_template_streak(doc.sentences) - 2) / 4)
    rhy = (0.50 * cv_penalty) + (0.20 * opener_penalty) + (0.20 * ending_penalty) + (0.10 * template_penalty)
    if rhy >= 0.25:
        findings.append(Finding(None, None, "rhythm_issue", "medium", "sentence rhythm is unusually uniform", suggested_fix="Mix short, medium, and longer sentences."))
    return clip01(rhy), cv, "normal", findings


def _meta_commentary_subscore(doc: PreparedDocument) -> tuple[float, int, list[Finding]]:
    findings: list[Finding] = []
    total = 0
    first_start: int | None = None
    for regex in META_REGEXES:
        for match in regex.finditer(doc.clean_text):
            total += 1
            if first_start is None or match.start() < first_start:
                first_start = match.start()
            line = _line_for_span(doc.clean_text, match.start())
            findings.append(
                Finding(line, match.span(), "meta_commentary", "medium", match.group(0), _line_context(doc.clean_text, line), "Delete the meta commentary or replace it with the actual point.")
            )
    if total == 1 and doc.word_count >= 800 and first_start is not None and first_start <= 500:
        total = 0
        findings = []
    density = 1000.0 * total / doc.word_count
    return clip01(density / META_DENSITY_MAX), total, findings


def _markdown_overuse_subscore(doc: PreparedDocument) -> tuple[float, int, list[Finding]]:
    hits = 0
    findings: list[Finding] = []
    for idx, line in enumerate(doc.prose_lines, start=1):
        line_hits = 0
        if HEADING_LINE_RE.search(line):
            line_hits += 1
        if BULLET_LINE_RE.search(line):
            line_hits += 1
        if TABLE_LINE_RE.search(line):
            line_hits += 1
        if BLOCKQUOTE_RE.search(line):
            line_hits += 1
        if HORIZONTAL_RULE_RE.search(line):
            line_hits += 1
        line_hits += len(BOLD_RE.findall(line))
        if line_hits:
            hits += line_hits
            findings.append(Finding(idx, None, "markdown_overuse", "low", line.strip(), line.strip(), "Use formatting only when it helps the reader."))
    em_dash_hits = doc.clean_text.count("—")
    hits += em_dash_hits
    density = 1000.0 * hits / doc.word_count
    return clip01(density / MARKDOWN_DENSITY_MAX), hits, findings


def analyze_doc_patterns(text: str) -> dict:
    doc = prepare_document(text)
    lengths = [len(sentence) for sentence in doc.sentences]
    avg_len = mean(lengths) if lengths else 0.0
    stddev = pstdev(lengths) if len(lengths) > 1 else 0.0
    opener_counts: dict[str, int] = {}
    for sentence in doc.sentences:
        tokens = word_tokens(sentence.lower())
        if tokens and len(tokens[0]) > 3:
            opener_counts[tokens[0]] = opener_counts.get(tokens[0], 0) + 1
    repeated = sorted(f"{word} ({count}x)" for word, count in opener_counts.items() if count >= 3)
    bullet_count = sum(1 for line in text.splitlines() if BULLET_LINE_RE.search(line.strip()))
    line_count = max(len(text.splitlines()), 1)
    ratio = bullet_count / line_count
    bullet_density = "high" if ratio > 0.3 else "medium" if ratio > 0.15 else "low"
    catalog = load_banned_catalog()
    lower_text = doc.clean_text.lower()
    banned_count = sum(len(_entry_regex(word).findall(lower_text)) for word in catalog.single_words)
    return {
        "avg_sentence_length": round(avg_len, 1),
        "sentence_length_std_dev": round(stddev, 1),
        "repeated_openers": repeated,
        "bullet_density": bullet_density,
        "heading_count": sum(1 for line in text.splitlines() if HEADING_LINE_RE.search(line)),
        "banned_word_count": banned_count,
    }


def analyze_text(text: str, source: str = "") -> dict:
    """Run the deterministic fallback detector and scorer."""

    doc = prepare_document(text)
    bwd, weighted_banned_hits, banned_findings = _banned_word_density(doc)
    spv, weighted_structural_hits, structural_findings = _structural_pattern_subscore(doc)
    rhy, rhythm_cv, rhythm_confidence, rhythm_findings = _rhythm_monotony_subscore(doc)
    meta, meta_hits, meta_findings = _meta_commentary_subscore(doc)
    md, markdown_hits, markdown_findings = _markdown_overuse_subscore(doc)

    components = {
        "BWD": round(bwd, 4),
        "SPV": round(spv, 4),
        "RHY": round(rhy, 4),
        "META": round(meta, 4),
        "MD": round(md, 4),
    }
    findings = [*banned_findings, *structural_findings, *rhythm_findings, *meta_findings, *markdown_findings]
    findings.sort(key=lambda f: (f.line is None, f.line or 10**9, f.category, f.text))
    payload = {
        "score": score_from_components(components),
        "components": components,
        "findings": [asdict(f) for f in findings],
        "details": {
            "word_count": doc.word_count,
            "sentence_count": len(doc.sentences),
            "weighted_banned_hits": weighted_banned_hits,
            "weighted_structural_hits": round(weighted_structural_hits, 4),
            "meta_hits": meta_hits,
            "markdown_hits": markdown_hits,
            "rhythm_cv": round(rhythm_cv, 4),
            "rhythm_sample_confidence": rhythm_confidence,
        },
        "doc_patterns": analyze_doc_patterns(text),
        "source": source,
        "detector": "fallback",
    }
    return payload


def score_text(text: str) -> int:
    return int(analyze_text(text)["score"])
