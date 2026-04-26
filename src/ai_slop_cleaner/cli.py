"""Command-line interface for AI Slop Cleaner."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

from ai_slop_cleaner import __version__
from ai_slop_cleaner.core.detector import analyze_text
from ai_slop_cleaner.mcp.server import serve


def _read_input(path: str | None) -> tuple[str, str]:
    if path:
        p = Path(path).expanduser()
        return p.read_text(encoding="utf-8"), str(p)
    return sys.stdin.read(), "stdin"


def _add_input_arg(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("file", nargs="?", help="UTF-8 text file to analyze. Reads stdin when omitted.")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="ai-slop-cleaner", description="Agent-driven AI-slop detector and MCP server.")
    parser.add_argument("--version", action="version", version=f"ai-slop-cleaner {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    mcp_parser = sub.add_parser("mcp", help="MCP server commands.")
    mcp_sub = mcp_parser.add_subparsers(dest="mcp_command", required=True)
    serve_parser = mcp_sub.add_parser("serve", help="Start the FastMCP server.")
    serve_parser.add_argument("--transport", choices=("stdio", "sse"), default="stdio")
    serve_parser.add_argument("--host", default="localhost")
    serve_parser.add_argument("--port", type=int, default=8080)

    score_parser = sub.add_parser("score", help="Print the integer AI Slop Score for a file or stdin.")
    _add_input_arg(score_parser)
    score_parser.add_argument("--json", action="store_true", help="Print score payload as JSON instead of a single integer.")

    analyze_parser = sub.add_parser("analyze", help="Print full AI-slop analysis JSON for a file or stdin.")
    _add_input_arg(analyze_parser)
    analyze_parser.add_argument("--pretty", action="store_true", default=True, help="Pretty-print JSON (default).")
    analyze_parser.add_argument("--compact", action="store_true", help="Print compact JSON.")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "mcp" and args.mcp_command == "serve":
        serve(transport=args.transport, host=args.host, port=args.port)
        return 0

    if args.command == "score":
        text, source = _read_input(args.file)
        result = analyze_text(text, source=source)
        if args.json:
            print(json.dumps({"score": result["score"], "components": result["components"], "details": result.get("details", {})}, indent=2, ensure_ascii=False))
        else:
            print(result["score"])
        return 0

    if args.command == "analyze":
        text, source = _read_input(args.file)
        result = analyze_text(text, source=source)
        indent = None if args.compact else 2
        print(json.dumps(result, indent=indent, ensure_ascii=False))
        return 0

    parser.error("unknown command")
    return 2


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
