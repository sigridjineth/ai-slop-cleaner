"""FastMCP server for AI Slop Cleaner.

This mirrors the Ouroboros MCP shape: a Python package exposes a CLI command
that starts a FastMCP stdio server and registers tool functions from a separate
``mcp.tools`` module.
"""

from __future__ import annotations

from typing import Literal

from ai_slop_cleaner import __version__
from ai_slop_cleaner.mcp import tools

VALID_TRANSPORTS = {"stdio", "sse"}


def validate_transport(transport: str) -> Literal["stdio", "sse"]:
    normalized = transport.lower()
    if normalized not in VALID_TRANSPORTS:
        raise ValueError("transport must be 'stdio' or 'sse'")
    return normalized  # type: ignore[return-value]


def create_server(name: str = "ai-slop-cleaner", version: str = __version__):
    """Create a FastMCP server with all AI-slop tools registered."""

    try:
        from mcp.server.fastmcp import FastMCP
    except ImportError as exc:  # pragma: no cover - exercised before dependency install only
        raise ImportError("mcp package not installed. Install with: pip install 'ai-slop-cleaner'.") from exc

    server = FastMCP(name)

    server.tool(
        name="ai_slop_score",
        description="Return the 0-100 AI Slop Score with BWD, SPV, RHY, META, and MD component breakdown.",
    )(tools.ai_slop_score)
    server.tool(
        name="ai_slop_analyze",
        description="Return full AI-slop findings, score components, details, and document patterns.",
    )(tools.ai_slop_analyze)
    server.tool(
        name="ai_slop_check",
        description="Return pass/fail for text or a file. Passing means score <= threshold (default 25).",
    )(tools.ai_slop_check)

    # FastMCP does not expose arbitrary serverInfo version on all SDK versions;
    # keep this attribute for tests and diagnostics without touching protocol I/O.
    server.ai_slop_cleaner_version = version
    return server


async def serve_async(transport: str = "stdio", host: str = "localhost", port: int = 8080) -> None:
    """Run the FastMCP server using stdio or SSE transport."""

    transport = validate_transport(transport)
    try:
        from mcp.server.fastmcp import FastMCP
    except ImportError as exc:  # pragma: no cover
        raise ImportError("mcp package not installed. Install with: pip install 'ai-slop-cleaner'.") from exc

    if transport == "sse":
        server = FastMCP("ai-slop-cleaner", host=host, port=port)
        server.tool(name="ai_slop_score", description="Return the 0-100 AI Slop Score with component breakdown.")(tools.ai_slop_score)
        server.tool(name="ai_slop_analyze", description="Return full AI-slop findings and document patterns.")(tools.ai_slop_analyze)
        server.tool(name="ai_slop_check", description="Return pass/fail for text or a file.")(tools.ai_slop_check)
        await server.run_sse_async()
        return

    server = create_server()
    await server.run_stdio_async()


def serve(transport: str = "stdio", host: str = "localhost", port: int = 8080) -> None:
    """Synchronous wrapper used by the CLI."""

    import asyncio

    asyncio.run(serve_async(transport=transport, host=host, port=port))
