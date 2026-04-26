"""Agent-driven detector.

The primary detector delegates judgment to coding-agent CLIs, matching the
Ouroboros pattern of MCP tools that launch agent work. It tries Claude Code
first, then Codex CLI, then the deterministic regex fallback.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import shutil
import subprocess
from typing import Any

from ai_slop_cleaner.core.banned_patterns import load_banned_patterns_markdown
from ai_slop_cleaner.core.banned_words import banned_reference_markdown, read_reference_text
from ai_slop_cleaner.core import fallback
from ai_slop_cleaner.core.scorer import normalize_components, score_from_components

AGENT_TIMEOUT_SECONDS = float(os.environ.get("AI_SLOP_CLEANER_AGENT_TIMEOUT", "45"))


def _read_reference(name: str) -> str:
    text, _source = read_reference_text(name)
    return text or ""


def _load_prompt_template() -> str:
    text, _source = read_reference_text("agent-prompt-template.md")
    if text:
        return text
    return """You are an AI-slop detection subagent. Analyze the text and return only JSON.

Banned words:
{{BANNED_WORDS}}

Banned structural patterns:
{{BANNED_PATTERNS}}

Expected JSON:
{"findings": [], "components": {"BWD": 0, "SPV": 0, "RHY": 0, "META": 0, "MD": 0}}

Text:
{{TEXT}}
"""


def build_agent_prompt(text: str, source: str = "") -> str:
    """Build the exact prompt sent to Claude/Codex subagents."""

    template = _load_prompt_template()
    replacements = {
        "{{BANNED_WORDS}}": banned_reference_markdown(),
        "{{BANNED_PATTERNS}}": load_banned_patterns_markdown(),
        "{{RESPONSE_SCHEMA}}": _read_reference("agent-response-schema.json"),
        "{{SOURCE}}": source or "inline text",
        "{{TEXT}}": text,
    }
    for key, value in replacements.items():
        template = template.replace(key, value)
    return template


def _extract_json(stdout: str) -> dict[str, Any] | None:
    """Extract a JSON object from raw agent output."""

    raw = stdout.strip()
    if not raw:
        return None
    candidates = [raw]
    fenced = re.findall(r"```(?:json)?\s*(\{.*?\})\s*```", raw, flags=re.DOTALL)
    candidates.extend(fenced)
    first = raw.find("{")
    last = raw.rfind("}")
    if first >= 0 and last > first:
        candidates.append(raw[first : last + 1])
    for candidate in candidates:
        try:
            parsed = json.loads(candidate)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict):
            return parsed
    return None


def _normalize_finding(raw: Any) -> dict[str, Any] | None:
    if not isinstance(raw, dict):
        return None
    category = str(raw.get("category", "unknown"))
    severity = str(raw.get("severity", "medium"))
    text = str(raw.get("text", ""))
    span_raw = raw.get("span")
    span = None
    if isinstance(span_raw, (list, tuple)) and len(span_raw) == 2:
        try:
            span = (int(span_raw[0]), int(span_raw[1]))
        except (TypeError, ValueError):
            span = None
    return {
        "line": raw.get("line"),
        "span": span,
        "category": category,
        "severity": severity,
        "text": text,
        "context": str(raw.get("context", raw.get("evidence", ""))),
        "suggested_fix": str(raw.get("suggested_fix", raw.get("suggest", ""))),
        "detector": str(raw.get("detector", "agent")),
    }


def _validate_agent_payload(payload: dict[str, Any], agent_name: str, source: str) -> dict[str, Any] | None:
    components = normalize_components(payload.get("components"))
    if set(components) != {"BWD", "SPV", "RHY", "META", "MD"}:
        return None
    findings_raw = payload.get("findings", [])
    if not isinstance(findings_raw, list):
        return None
    findings = [item for item in (_normalize_finding(raw) for raw in findings_raw) if item]
    for finding in findings:
        finding["detector"] = agent_name
    return {
        "score": score_from_components(components),
        "components": components,
        "findings": findings,
        "details": dict(payload.get("details", {})) if isinstance(payload.get("details"), dict) else {},
        "doc_patterns": dict(payload.get("doc_patterns", {})) if isinstance(payload.get("doc_patterns"), dict) else {},
        "source": source,
        "detector": agent_name,
    }


def _run_agent(command: list[str], prompt: str, timeout: float) -> str | None:
    try:
        result = subprocess.run(
            command,
            input=prompt,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except (FileNotFoundError, PermissionError, subprocess.TimeoutExpired, OSError):
        return None
    if result.returncode != 0:
        return None
    return result.stdout


def _try_claude(prompt: str, timeout: float) -> str | None:
    if not shutil.which("claude"):
        return None
    # Claude Code accepts --print with stdin in current releases; stdin avoids
    # giant argv payloads for long documents.
    return _run_agent(["claude", "--print"], prompt, timeout)


def _try_codex(prompt: str, timeout: float) -> str | None:
    if not shutil.which("codex"):
        return None
    return _run_agent(["codex", "exec", "-"], prompt, timeout)


def analyze_text(text: str, source: str = "", *, prefer_agents: bool = True, timeout: float | None = None) -> dict[str, Any]:
    """Analyze text by trying Claude, Codex, then regex fallback."""

    if os.environ.get("AI_SLOP_CLEANER_DISABLE_AGENTS", "").lower() in {"1", "true", "yes", "on"}:
        return fallback.analyze_text(text, source=source)

    timeout = AGENT_TIMEOUT_SECONDS if timeout is None else timeout
    if prefer_agents:
        prompt = build_agent_prompt(text, source=source)
        for agent_name, runner in (("claude", _try_claude), ("codex", _try_codex)):
            stdout = runner(prompt, timeout)
            if not stdout:
                continue
            parsed = _extract_json(stdout)
            if not parsed:
                continue
            normalized = _validate_agent_payload(parsed, agent_name, source)
            if normalized:
                return normalized

    return fallback.analyze_text(text, source=source)
