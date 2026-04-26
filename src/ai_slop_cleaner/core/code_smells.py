"""Code-oriented AI-slop smell detection.

The prose scorer flags generated writing patterns. This module applies the
same oh-my-codex cleanup discipline to source code: lock behavior first, plan
before editing, categorize smells, and report enough evidence for a safe cleanup
pass. The detector is intentionally dependency-free and conservative; it is a
triage aid, not a compiler or coverage engine.
"""

from __future__ import annotations

import ast
from dataclasses import asdict, dataclass
from difflib import SequenceMatcher
from pathlib import Path
import re
from typing import Iterable, Sequence

CLEANUP_RULES = {
    "source": "https://github.com/Yeachan-Heo/oh-my-codex/blob/main/skills/ai-slop-cleaner/SKILL.md",
    "behavior_lock": "Lock behavior with regression tests before cleanup edits.",
    "cleanup_plan": "Create a bounded cleanup plan before changing code.",
    "issue_categories": [
        "Duplication",
        "Dead code",
        "Needless abstraction",
        "Boundary violations",
        "Missing tests",
    ],
    "pass_order": [
        "Dead code",
        "Duplicate",
        "Naming/error",
        "Test reinforcement",
    ],
    "quality_gates": [
        "regression tests stay green",
        "lint passes",
        "typecheck passes",
        "unit/integration tests pass",
        "static/security scan passes when available",
        "evidence-dense report lists changes and risks",
    ],
}

CODE_EXTENSIONS = {
    ".py": "python",
    ".js": "javascript",
    ".jsx": "javascript",
    ".ts": "javascript",
    ".tsx": "javascript",
    ".rs": "rust",
}

SEVERITY_RANK = {"low": 1, "medium": 2, "high": 3, "critical": 4}


@dataclass(frozen=True)
class CodeFinding:
    """A single code smell finding."""

    category: str
    smell: str
    severity: str
    text: str
    line: int | None = None
    symbol: str = ""
    source: str = ""
    suggested_fix: str = ""
    evidence: str = ""


@dataclass(frozen=True)
class FunctionInfo:
    name: str
    line: int
    body: str
    params: tuple[str, ...]
    language: str


def detect_language(source_path: str | Path | None = None, language: str | None = None) -> str:
    """Return a normalized language key for supported source code."""

    if language:
        normalized = language.lower().strip()
        if normalized in {"py", "python"}:
            return "python"
        if normalized in {"js", "jsx", "ts", "tsx", "javascript", "typescript"}:
            return "javascript"
        if normalized in {"rs", "rust"}:
            return "rust"
        return normalized
    if source_path:
        return CODE_EXTENSIONS.get(Path(source_path).suffix.lower(), "text")
    return "text"


def _line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, max(0, offset)) + 1


def _normalize_body(body: str) -> str:
    body = re.sub(r"#.*|//.*", "", body)
    body = re.sub(r"\s+", "", body)
    body = re.sub(r"[A-Za-z_][A-Za-z0-9_]*", "id", body)
    return body.strip()


def _similarity(left: str, right: str) -> float:
    if not left and not right:
        return 1.0
    return SequenceMatcher(None, left, right).ratio()


def _summary(findings: Sequence[CodeFinding]) -> dict:
    by_category: dict[str, int] = {}
    by_severity: dict[str, int] = {}
    for finding in findings:
        by_category[finding.category] = by_category.get(finding.category, 0) + 1
        by_severity[finding.severity] = by_severity.get(finding.severity, 0) + 1
    return {
        "total": len(findings),
        "by_category": by_category,
        "by_severity": by_severity,
        "pass_order": CLEANUP_RULES["pass_order"],
    }


def _python_functions(tree: ast.AST, source: str) -> list[FunctionInfo]:
    functions: list[FunctionInfo] = []
    lines = source.splitlines()
    for node in ast.walk(tree):
        if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        start = max(node.lineno - 1, 0)
        end = max(getattr(node, "end_lineno", node.lineno), node.lineno)
        body = "\n".join(lines[start:end])
        params = tuple(arg.arg for arg in node.args.args)
        functions.append(FunctionInfo(node.name, node.lineno, body, params, "python"))
    return functions


def _find_matching_brace(text: str, open_brace: int) -> int:
    depth = 0
    in_string: str | None = None
    escape = False
    idx = open_brace
    while idx < len(text):
        char = text[idx]
        if in_string:
            if escape:
                escape = False
            elif char == "\\":
                escape = True
            elif char == in_string:
                in_string = None
        elif char in {'"', "'", "`"}:
            in_string = char
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return idx
        idx += 1
    return len(text)


def _brace_functions(source: str, language: str) -> list[FunctionInfo]:
    if language == "rust":
        pattern = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)[^{;]*\{", re.MULTILINE)
    else:
        pattern = re.compile(
            r"(?:\bfunction\s+([A-Za-z_$][\w$]*)\s*\(([^)]*)\)\s*\{|\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*\([^)]*\)\s*=>\s*\{)",
            re.MULTILINE,
        )
    functions: list[FunctionInfo] = []
    for match in pattern.finditer(source):
        name = next(group for group in match.groups() if group and not group.strip().startswith(":"))
        params_raw = match.group(2) or ""
        open_brace = source.find("{", match.start())
        close_brace = _find_matching_brace(source, open_brace)
        body = source[open_brace + 1 : close_brace]
        params = tuple(part.strip().split(":", 1)[0].strip() for part in params_raw.split(",") if part.strip())
        functions.append(FunctionInfo(name, _line_for_offset(source, match.start()), body, params, language))
    return functions


def _duplicate_findings(functions: Sequence[FunctionInfo], source_name: str) -> list[CodeFinding]:
    findings: list[CodeFinding] = []
    by_name: dict[str, list[FunctionInfo]] = {}
    for function in functions:
        by_name.setdefault(function.name, []).append(function)
    for name, matches in by_name.items():
        if len(matches) < 2:
            continue
        normalized = [_normalize_body(match.body) for match in matches]
        for idx in range(len(matches) - 1):
            score = _similarity(normalized[idx], normalized[idx + 1])
            if score < 0.72:
                continue
            findings.append(
                CodeFinding(
                    category="Duplication",
                    smell="duplicate_function",
                    severity="high",
                    text=f"duplicate function definition '{name}' has similar body ({score:.0%} match)",
                    line=matches[idx + 1].line,
                    symbol=name,
                    source=source_name,
                    suggested_fix="Keep one implementation or extract the genuinely shared logic behind a tested API.",
                    evidence=f"definitions at lines {matches[idx].line} and {matches[idx + 1].line}",
                )
            )
    return findings


def _call_name(node: ast.AST) -> str:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        return node.attr
    return ""


def _python_dead_code(tree: ast.AST, source_name: str) -> list[CodeFinding]:
    findings: list[CodeFinding] = []
    imported: list[tuple[str, str, int]] = []
    used_names = {node.id for node in ast.walk(tree) if isinstance(node, ast.Name)}

    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                visible = alias.asname or alias.name.split(".", 1)[0]
                imported.append((visible, alias.name, node.lineno))
        elif isinstance(node, ast.ImportFrom):
            if node.module == "__future__":
                continue
            for alias in node.names:
                if alias.name == "*":
                    continue
                visible = alias.asname or alias.name
                imported.append((visible, f"{node.module or ''}.{alias.name}".strip("."), node.lineno))

    for visible, full_name, line in imported:
        if visible not in used_names:
            findings.append(
                CodeFinding(
                    category="Dead code",
                    smell="unused_import",
                    severity="medium",
                    text=f"unused import '{full_name}'",
                    line=line,
                    symbol=visible,
                    source=source_name,
                    suggested_fix="Delete the import after tests prove it is unused.",
                )
            )

    def scan_body(body: Sequence[ast.stmt]) -> None:
        terminal: ast.stmt | None = None
        for stmt in body:
            if terminal is not None and not isinstance(stmt, ast.Pass):
                findings.append(
                    CodeFinding(
                        category="Dead code",
                        smell="unreachable_branch",
                        severity="high",
                        text="unreachable statement after terminal control flow",
                        line=getattr(stmt, "lineno", None),
                        source=source_name,
                        suggested_fix="Delete or move the branch; keep a regression test for the terminal path.",
                        evidence=f"previous terminal statement at line {getattr(terminal, 'lineno', '?')}",
                    )
                )
                break
            if isinstance(stmt, (ast.Return, ast.Raise, ast.Break, ast.Continue)):
                terminal = stmt
            for attr in ("body", "orelse", "finalbody"):
                child = getattr(stmt, attr, None)
                if isinstance(child, list):
                    scan_body(child)
            handlers = getattr(stmt, "handlers", None)
            if handlers:
                for handler in handlers:
                    scan_body(getattr(handler, "body", []))

    scan_body(getattr(tree, "body", []))
    return findings


def _python_needless_abstraction(tree: ast.AST, functions: Sequence[FunctionInfo], source_name: str) -> list[CodeFinding]:
    findings: list[CodeFinding] = []
    function_nodes = [node for node in ast.walk(tree) if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))]
    calls: dict[str, int] = {}
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            name = _call_name(node.func)
            if name:
                calls[name] = calls.get(name, 0) + 1

    for node in function_nodes:
        if len(node.body) == 1 and isinstance(node.body[0], ast.Return) and isinstance(node.body[0].value, ast.Call):
            target = _call_name(node.body[0].value.func)
            if target and target != node.name:
                findings.append(
                    CodeFinding(
                        category="Needless abstraction",
                        smell="pass_through_wrapper",
                        severity="medium",
                        text=f"pass-through wrapper '{node.name}' only delegates to '{target}'",
                        line=node.lineno,
                        symbol=node.name,
                        source=source_name,
                        suggested_fix="Inline the wrapper unless it owns validation, naming, or a stable public boundary.",
                    )
                )

    function_names = {function.name for function in functions}
    for function in functions:
        if function.name.startswith("test_") or function.name.startswith("__"):
            continue
        if calls.get(function.name, 0) == 1 and function.name in function_names:
            findings.append(
                CodeFinding(
                    category="Needless abstraction",
                    smell="single_use_helper",
                    severity="low",
                    text=f"single-use helper '{function.name}' is called once",
                    line=function.line,
                    symbol=function.name,
                    source=source_name,
                    suggested_fix="Inline it if the name does not clarify a separate concept.",
                )
            )
    return findings


def _import_modules_python(tree: ast.AST) -> list[tuple[str, int]]:
    modules: list[tuple[str, int]] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            modules.extend((alias.name, node.lineno) for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            modules.append((node.module, node.lineno))
    return modules


def _boundary_findings(modules: Iterable[tuple[str, int]], source_path: str, source_name: str) -> list[CodeFinding]:
    findings: list[CodeFinding] = []
    normalized_path = source_path.replace("\\", "/").lower()
    ui_layer = any(part in normalized_path for part in ("/ui/", "/view", "/views/", "/frontend/", "app/ui/", "src/ui/"))
    domain_layer = any(part in normalized_path for part in ("/domain/", "app/domain/", "src/domain/"))
    for module, line in modules:
        normalized_module = module.lower()
        if ui_layer and any(token in normalized_module for token in (".data", ".db", ".database", ".repo", ".repository", ".persistence", "sqlalchemy")):
            findings.append(
                CodeFinding(
                    category="Boundary violations",
                    smell="wrong_layer_import",
                    severity="high",
                    text=f"UI layer imports persistence/data module '{module}'",
                    line=line,
                    source=source_name,
                    suggested_fix="Move persistence access behind an application/service boundary.",
                )
            )
        if domain_layer and any(token in normalized_module for token in (".ui", ".views", "react", "fastapi", "flask", "django")):
            findings.append(
                CodeFinding(
                    category="Boundary violations",
                    smell="wrong_layer_import",
                    severity="high",
                    text=f"Domain layer imports framework/UI module '{module}'",
                    line=line,
                    source=source_name,
                    suggested_fix="Keep domain code independent from UI/framework adapters.",
                )
            )
    return findings


def _missing_test_findings(functions: Sequence[FunctionInfo], tests_text: str | None, source_name: str) -> list[CodeFinding]:
    if tests_text is None:
        return []
    findings: list[CodeFinding] = []
    for function in functions:
        if function.name.startswith("_") or function.name.startswith("test_") or function.name in {"main"}:
            continue
        patterns = (function.name, f"test_{function.name}")
        if not any(pattern in tests_text for pattern in patterns):
            findings.append(
                CodeFinding(
                    category="Missing tests",
                    smell="missing_function_test",
                    severity="medium",
                    text=f"function '{function.name}' has no obvious test coverage",
                    line=function.line,
                    symbol=function.name,
                    source=source_name,
                    suggested_fix="Add or link a regression test before cleaning this function.",
                )
            )
    return findings


def _simple_dead_code(source: str, source_name: str) -> list[CodeFinding]:
    findings: list[CodeFinding] = []
    lines = source.splitlines()
    terminal_line: int | None = None
    terminal_indent = 0
    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()
        indent = len(line) - len(line.lstrip())
        if terminal_line is not None:
            if stripped and indent > terminal_indent and not stripped.startswith(("}", "else", "catch")):
                findings.append(
                    CodeFinding(
                        category="Dead code",
                        smell="unreachable_branch",
                        severity="high",
                        text="unreachable statement after return/throw/panic",
                        line=idx,
                        source=source_name,
                        suggested_fix="Delete or move the branch after locking the return path with a test.",
                        evidence=f"terminal statement at line {terminal_line}",
                    )
                )
            terminal_line = None
        if re.match(r"(?:return\b|throw\b|panic!\b|break\b|continue\b)", stripped):
            terminal_line = idx
            terminal_indent = indent
    return findings


def _simple_import_modules(source: str, language: str) -> list[tuple[str, int]]:
    modules: list[tuple[str, int]] = []
    for idx, line in enumerate(source.splitlines(), start=1):
        if language == "rust":
            match = re.search(r"\buse\s+([A-Za-z0-9_:]+)", line)
            if match:
                modules.append((match.group(1).replace("::", "."), idx))
        else:
            match = re.search(r"\bimport\s+(?:[^'\"]+\s+from\s+)?['\"]([^'\"]+)['\"]", line)
            if match:
                modules.append((match.group(1), idx))
    return modules


def _analyze_python(source: str, source_name: str, source_path: str, tests_text: str | None) -> list[CodeFinding]:
    try:
        tree = ast.parse(source)
    except SyntaxError as exc:
        return [
            CodeFinding(
                category="Dead code",
                smell="syntax_error_blocks_analysis",
                severity="low",
                text=f"could not parse Python source: {exc.msg}",
                line=exc.lineno,
                source=source_name,
                suggested_fix="Run the detector after syntax errors are resolved.",
            )
        ]
    functions = _python_functions(tree, source)
    findings: list[CodeFinding] = []
    findings.extend(_duplicate_findings(functions, source_name))
    findings.extend(_python_dead_code(tree, source_name))
    findings.extend(_python_needless_abstraction(tree, functions, source_name))
    findings.extend(_boundary_findings(_import_modules_python(tree), source_path, source_name))
    findings.extend(_missing_test_findings(functions, tests_text, source_name))
    return findings


def _analyze_brace_language(source: str, language: str, source_name: str, source_path: str, tests_text: str | None) -> list[CodeFinding]:
    functions = _brace_functions(source, language)
    findings: list[CodeFinding] = []
    findings.extend(_duplicate_findings(functions, source_name))
    findings.extend(_simple_dead_code(source, source_name))
    findings.extend(_boundary_findings(_simple_import_modules(source, language), source_path, source_name))
    findings.extend(_missing_test_findings(functions, tests_text, source_name))
    return findings


def analyze_code_text(
    source: str,
    *,
    language: str | None = None,
    source_path: str | Path | None = None,
    tests_text: str | None = None,
) -> dict:
    """Analyze one source text for code slop smells."""

    source_name = str(source_path or "inline")
    normalized_language = detect_language(source_path, language)
    if normalized_language == "python":
        findings = _analyze_python(source, source_name, source_name, tests_text)
    elif normalized_language in {"javascript", "rust"}:
        findings = _analyze_brace_language(source, normalized_language, source_name, source_name, tests_text)
    else:
        findings = []

    findings.sort(key=lambda item: (-SEVERITY_RANK.get(item.severity, 0), item.category, item.line or 10**9, item.text))
    return {
        "source": source_name,
        "language": normalized_language,
        "rules": CLEANUP_RULES,
        "findings": [asdict(finding) for finding in findings],
        "summary": _summary(findings),
    }


def _iter_code_files(paths: Iterable[str | Path]) -> list[Path]:
    files: list[Path] = []
    for raw in paths:
        path = Path(raw).expanduser()
        if path.is_dir():
            files.extend(p for p in path.rglob("*") if p.is_file() and p.suffix.lower() in CODE_EXTENSIONS)
        elif path.is_file() and path.suffix.lower() in CODE_EXTENSIONS:
            files.append(path)
    return sorted(dict.fromkeys(files))


def _read_tests_text(tests_path: str | Path | None = None) -> str | None:
    if tests_path is None:
        default = Path.cwd() / "tests"
        if not default.exists():
            return None
        tests_path = default
    path = Path(tests_path).expanduser()
    if not path.exists():
        return ""
    if path.is_file():
        return path.read_text(encoding="utf-8")
    chunks: list[str] = []
    for file in sorted(path.rglob("*")):
        if file.is_file() and file.suffix.lower() in (set(CODE_EXTENSIONS) | {".md", ".txt"}):
            try:
                chunks.append(file.read_text(encoding="utf-8"))
            except UnicodeDecodeError:
                continue
    return "\n".join(chunks)


def analyze_paths(paths: Iterable[str | Path], *, tests_path: str | Path | None = None) -> dict:
    """Analyze files/directories and return an aggregate code-smell report."""

    files = _iter_code_files(paths)
    tests_text = _read_tests_text(tests_path)
    reports = []
    findings: list[dict] = []
    for file in files:
        text = file.read_text(encoding="utf-8")
        report = analyze_code_text(text, source_path=file, tests_text=tests_text)
        reports.append(report)
        findings.extend(report["findings"])

    by_category: dict[str, int] = {}
    by_severity: dict[str, int] = {}
    for finding in findings:
        by_category[finding["category"]] = by_category.get(finding["category"], 0) + 1
        by_severity[finding["severity"]] = by_severity.get(finding["severity"], 0) + 1
    return {
        "rules": CLEANUP_RULES,
        "files": [str(file) for file in files],
        "reports": reports,
        "findings": findings,
        "summary": {
            "total": len(findings),
            "files_analyzed": len(files),
            "by_category": by_category,
            "by_severity": by_severity,
            "pass_order": CLEANUP_RULES["pass_order"],
        },
    }
