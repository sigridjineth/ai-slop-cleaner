from ai_slop_cleaner.core.code_smells import CLEANUP_RULES, analyze_code_text


def categories(result):
    return {finding["category"] for finding in result["findings"]}


def test_cleanup_rules_capture_oh_my_codex_workflow():
    assert CLEANUP_RULES["issue_categories"] == [
        "Duplication",
        "Dead code",
        "Needless abstraction",
        "Boundary violations",
        "Missing tests",
    ]
    assert CLEANUP_RULES["pass_order"] == [
        "Dead code",
        "Duplicate",
        "Naming/error",
        "Test reinforcement",
    ]
    assert "regression tests" in " ".join(CLEANUP_RULES["quality_gates"])


def test_python_code_smell_detector_finds_requested_categories():
    source = """
import os
import sys
from app.data import repo

def wrapper(value):
    return normalize(value)

def only_once(value):
    return value.strip()

def public_api(value):
    cleaned = only_once(value)
    return cleaned
    print("unreachable")

def public_api(value):
    cleaned = only_once(value)
    return cleaned
"""

    result = analyze_code_text(source, language="python", source_path="app/ui/view.py", tests_text="")

    assert result["summary"]["total"] >= 5
    assert {
        "Duplication",
        "Dead code",
        "Needless abstraction",
        "Boundary violations",
        "Missing tests",
    }.issubset(categories(result))
    assert any("public_api" in finding["text"] for finding in result["findings"])
    assert any("unused import" in finding["text"] for finding in result["findings"])
    assert any("app.data" in finding["text"] for finding in result["findings"])


def test_javascript_and_rust_duplicate_function_detection():
    javascript = """
function makeId(value) {
  return String(value).trim();
}
function makeId(value) {
  return String(value).trim();
}
"""
    rust = """
fn make_id(value: &str) -> String {
    value.trim().to_string()
}
fn make_id(value: &str) -> String {
    value.trim().to_string()
}
"""

    js_result = analyze_code_text(javascript, language="javascript", tests_text="makeId")
    rust_result = analyze_code_text(rust, language="rust", tests_text="make_id")

    assert "Duplication" in categories(js_result)
    assert "Duplication" in categories(rust_result)
