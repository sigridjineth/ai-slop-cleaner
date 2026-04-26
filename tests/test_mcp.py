import os

from ai_slop_cleaner.mcp.server import create_server, validate_transport
from ai_slop_cleaner.mcp.tools import ai_slop_analyze, ai_slop_check, ai_slop_score


def test_validate_transport():
    assert validate_transport("stdio") == "stdio"
    assert validate_transport("SSE") == "sse"


def test_create_fastmcp_server_registers_version():
    server = create_server()
    assert getattr(server, "ai_slop_cleaner_version") == "2.0.0"


def test_mcp_tool_functions_use_fallback_when_agents_disabled(monkeypatch):
    monkeypatch.setenv("AI_SLOP_CLEANER_DISABLE_AGENTS", "1")
    text = "Let's dive in and delve into the landscape."

    score = ai_slop_score(text=text)
    assert score["score"] > 0
    assert set(score["components"]) == {"BWD", "SPV", "RHY", "META", "MD"}

    analysis = ai_slop_analyze(text=text)
    assert analysis["findings"]

    check = ai_slop_check(text=text, threshold=100)
    assert check["pass"] is True
