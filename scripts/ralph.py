#!/usr/bin/env python3
"""
Ralph — standalone LLM-inference deslop loop for ai-slop-cleaner.

This script never rewrites with regex substitutions. It repeatedly:

    analyze current full text -> ask an LLM to rewrite the complete text -> re-analyze

The Rust binary supplies raw structural matches only. The LLM receives those
matches, the full current text, and the universal categories from
rules/patterns-agent.md, then rewrites by holistic inference in any language.

Usage:
    python3 scripts/ralph.py <input-file> [--max-rounds 3] [--target-matches 0] [--force-rewrite]

LLM backend priority:
    1. `claude -p` CLI if installed; reuses Claude Code authentication
    2. `aichat` CLI when AICHAT_MODEL is set
    3. OPENAI_API_KEY
    4. ANTHROPIC_API_KEY

If no API key or CLI is available, prompts are written to .ralph/round-N-prompt.md.
Paste the LLM's complete rewritten text into .ralph/round-N-response.md, then
press Enter to continue.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path


DEFAULT_OPENAI_MODEL = os.environ.get("OPENAI_MODEL", "gpt-4o-mini")
DEFAULT_ANTHROPIC_MODEL = os.environ.get("ANTHROPIC_MODEL", "claude-3-5-haiku-20241022")


def find_binary():
    """Locate the ai-slop-cleaner binary."""
    script_dir = Path(__file__).parent.resolve()
    project_root = script_dir.parent
    candidates = [
        # Release install path
        Path.home() / ".local" / "share" / "ai-slop-cleaner" / "bin" / "ai-slop-cleaner",
        # Source build paths
        project_root / "rust" / "target" / "release" / "ai-slop-cleaner",
        project_root / "target" / "release" / "ai-slop-cleaner",
    ]
    for candidate in candidates:
        if candidate.exists():
            return str(candidate)

    # Try PATH, which may point to a wrapper script installed by install.sh.
    path_result = subprocess.run(["which", "ai-slop-cleaner"], capture_output=True, text=True)
    if path_result.returncode == 0:
        return path_result.stdout.strip()
    return None


def build_binary(project_root):
    """Build the Rust binary."""
    rust_dir = project_root / "rust"
    if not (rust_dir / "Cargo.toml").exists():
        print("[ralph] Cargo.toml not found. Cannot build.")
        sys.exit(1)
    print("[ralph] Building ai-slop-cleaner...")
    subprocess.run(["cargo", "build", "--release"], cwd=str(rust_dir), check=True)
    return str(rust_dir / "target" / "release" / "ai-slop-cleaner")


def run_analyze(binary, rules_dir, input_file):
    """Run ai-slop-cleaner analyze and return raw structural matches."""
    cmd = [binary, "--rules-dir", str(rules_dir), "analyze", str(input_file)]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"[ralph] analyze failed: {result.stderr}")
        sys.exit(1)
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        print(f"[ralph] analyze returned invalid JSON: {exc}")
        print(result.stdout)
        sys.exit(1)


def load_agent_patterns(rules_dir):
    """Load universal semantic categories for the LLM prompt."""
    path = Path(rules_dir) / "patterns-agent.md"
    if not path.exists():
        print(f"[ralph] patterns-agent.md not found at {path}")
        sys.exit(1)
    return path.read_text(encoding="utf-8")


def extract_top_violations(matches, limit=8):
    """Return the highest-weight structural matches for a compact summary."""
    violations = []
    for match in matches:
        violations.append(
            {
                "type": match.get("pattern_name", "pattern"),
                "line": match.get("line_number", 0),
                "severity": match.get("severity", "unknown"),
                "weight": match.get("weight", 0),
                "matched_text": match.get("matched_text", ""),
            }
        )
    violations.sort(key=lambda item: item["weight"], reverse=True)
    return violations[:limit]


def make_prompt(input_text, structural_matches, agent_patterns, round_num):
    """Generate a full-text LLM rewrite prompt for the holistic judge."""
    top_violations = extract_top_violations(structural_matches)
    violation_text = "\n".join(
        f"{index + 1}. [{v['severity']}] {v['type']} (weight {v['weight']}) "
        f"at line {v['line']}: {v['matched_text']}"
        for index, v in enumerate(top_violations)
    ) or "No structural matches. Use the universal categories to judge semantic slop."

    structural_json = json.dumps(structural_matches, ensure_ascii=False, indent=2)

    return f"""You are Ralph, a language-agnostic holistic judge and editor for AI slop.

CRITICAL SAFETY RULE:
Never clean text with regex-style substitutions, search-and-replace recipes, or
word chopping. Rewrite the entire text by inference. Preserve grammar and
meaning. Never cut connector-looking syllables out of compound terms. For
example, Korean `즉흥적으로` and `즉흥성` are complete words, not the connector
`즉`; `느낌입니다` must not become `다`.

## Round {round_num}

## Your task
Read all inputs below, then output a COMPLETE rewritten version of the text.
Do not output patches, diffs, replacement tables, comments, or explanations.
Do not wrap the whole response in a markdown fence unless the original document
itself is a fenced block.

## How to judge slop
1. Read every universal category in `patterns-agent.md`.
2. Judge semantic slop by CATEGORY INTENT, not by string matching examples.
3. Use Rust structural matches as evidence only. They are not rewrite commands.
4. Judge structural plus semantic slop in whatever language the text uses.
5. Keep natural phrases that are appropriate in context.
6. Preserve facts, names, code, quotations, formatting that serves a real purpose,
   and the original language/register unless the text itself changes register.
7. REWRITE the entire text naturally: concise, grammatical, specific, human-sounding.
8. Do NOT regex-replace, do NOT patch, and do NOT delete substrings from words.

## Top structural evidence from Rust analyze
{violation_text}

## Full structural matches JSON from Rust analyze ({len(structural_matches)} found)
```json
{structural_json}
```

## Universal pattern categories from patterns-agent.md
```markdown
{agent_patterns}
```

## Full current text to rewrite
```text
{input_text}
```

## Output contract
Return only the complete rewritten text. The output must be a full replacement
for the input text, not a partial edit.
"""


def call_llm(prompt):
    """Call an available LLM backend. Returns response text or None."""
    if shutil.which("claude"):
        result = subprocess.run(
            ["claude", "-p"],
            input=prompt,
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            return result.stdout
        print(f"[ralph] claude -p error: {result.stderr}")

    if shutil.which("aichat") and os.environ.get("AICHAT_MODEL"):
        model = os.environ["AICHAT_MODEL"]
        result = subprocess.run(
            ["aichat", "--model", model, "--no-stream"],
            input=prompt,
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            return result.stdout
        print(f"[ralph] aichat error: {result.stderr}")

    openai_key = os.environ.get("OPENAI_API_KEY")
    if openai_key:
        request = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=json.dumps(
                {
                    "model": DEFAULT_OPENAI_MODEL,
                    "messages": [{"role": "user", "content": prompt}],
                    "temperature": 0.3,
                }
            ).encode(),
            headers={
                "Authorization": f"Bearer {openai_key}",
                "Content-Type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                data = json.loads(response.read())
                return data["choices"][0]["message"]["content"]
        except Exception as exc:  # pragma: no cover - depends on external API
            print(f"[ralph] OpenAI API error: {exc}")

    anthropic_key = os.environ.get("ANTHROPIC_API_KEY")
    if anthropic_key:
        request = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=json.dumps(
                {
                    "model": DEFAULT_ANTHROPIC_MODEL,
                    "max_tokens": 8192,
                    "messages": [{"role": "user", "content": prompt}],
                }
            ).encode(),
            headers={
                "x-api-key": anthropic_key,
                "anthropic-version": "2023-06-01",
                "Content-Type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                data = json.loads(response.read())
                return data["content"][0]["text"]
        except Exception as exc:  # pragma: no cover - depends on external API
            print(f"[ralph] Anthropic API error: {exc}")

    return None


def interactive_round(ralph_dir, round_num, prompt):
    """Write prompt and wait for the user to provide a complete rewrite file."""
    prompt_file = ralph_dir / f"round-{round_num}-prompt.md"
    response_file = ralph_dir / f"round-{round_num}-response.md"
    prompt_file.write_text(prompt, encoding="utf-8")
    print(f"[ralph] Prompt written to {prompt_file}")
    print(f"[ralph] Paste the COMPLETE rewritten text into {response_file} and press Enter...")
    input()
    if not response_file.exists():
        print("[ralph] No response file found. Exiting.")
        sys.exit(1)
    return response_file.read_text(encoding="utf-8")


def clean_response(text):
    """Strip accidental outer markdown fences without editing the prose itself."""
    text = text.strip()
    if text.startswith("```"):
        lines = text.splitlines()
        if lines and lines[0].startswith("```"):
            lines = lines[1:]
        if lines and lines[-1].startswith("```"):
            lines = lines[:-1]
        text = "\n".join(lines).strip()
    return text


def write_analysis(ralph_dir, round_num, matches, suffix="analysis"):
    path = ralph_dir / f"round-{round_num}-{suffix}.json"
    path.write_text(json.dumps(matches, ensure_ascii=False, indent=2), encoding="utf-8")
    return path


def main():
    parser = argparse.ArgumentParser(
        description="Iteratively deslop text with full-text LLM inference, never regex replacement."
    )
    parser.add_argument("input_file", help="File to clean up")
    parser.add_argument("--max-rounds", type=int, default=3, help="Maximum rewrite iterations")
    parser.add_argument(
        "--target-matches",
        type=int,
        default=0,
        help="Stop when Rust structural match count is at or below this value",
    )
    parser.add_argument("--rules-dir", default=None, help="Path to rules directory")
    parser.add_argument(
        "--force-rewrite",
        action="store_true",
        help="Run at least one LLM rewrite even when analyze finds no structural matches",
    )
    args = parser.parse_args()

    input_file = Path(args.input_file).resolve()
    if not input_file.exists():
        print(f"[ralph] File not found: {input_file}")
        sys.exit(1)

    script_dir = Path(__file__).parent.resolve()
    project_root = script_dir.parent
    ralph_dir = project_root / ".ralph"
    ralph_dir.mkdir(exist_ok=True)

    binary = find_binary() or build_binary(project_root)

    if args.rules_dir:
        rules_dir = Path(args.rules_dir)
    else:
        release_rules = Path.home() / ".local" / "share" / "ai-slop-cleaner" / "rules"
        source_rules = project_root / "rust" / "rules"
        rules_dir = release_rules if release_rules.exists() else source_rules

    agent_patterns = load_agent_patterns(rules_dir)

    current_file = ralph_dir / "current.txt"
    current_file.write_text(input_file.read_text(encoding="utf-8"), encoding="utf-8")

    print(f"[ralph] Binary: {binary}")
    print(f"[ralph] Rules:  {rules_dir}")
    round_limit = max(args.max_rounds, 1) if args.force_rewrite else args.max_rounds

    print(f"[ralph] Target: <= {args.target_matches} structural matches | Max rounds: {args.max_rounds}")
    if args.force_rewrite and args.max_rounds == 0:
        print("[ralph] Force rewrite enabled: raising effective max rounds to 1")
    elif args.force_rewrite:
        print("[ralph] Force rewrite enabled: will run at least one LLM rewrite")
    print("[ralph] Rewrite mode: full-text LLM inference, never regex substitution")
    print("-" * 60)

    forced_rewrite_done = False
    for round_num in range(1, round_limit + 1):
        before_matches = run_analyze(binary, rules_dir, current_file)
        write_analysis(ralph_dir, round_num, before_matches, "before")
        before_count = len(before_matches)
        print(f"\n[ralph] Round {round_num}: {before_count} structural matches before rewrite")

        force_this_round = args.force_rewrite and not forced_rewrite_done
        if before_count <= args.target_matches and not force_this_round:
            print("[ralph] Target reached before rewrite. Cleaning complete.")
            break
        if before_count <= args.target_matches and force_this_round:
            print("[ralph] Force rewrite enabled; rewriting despite target already being reached.")

        prompt = make_prompt(
            current_file.read_text(encoding="utf-8"),
            before_matches,
            agent_patterns,
            round_num,
        )
        (ralph_dir / f"round-{round_num}-prompt.md").write_text(prompt, encoding="utf-8")

        response = call_llm(prompt)
        if response is None:
            response = interactive_round(ralph_dir, round_num, prompt)

        rewritten = clean_response(response)
        if not rewritten:
            print("[ralph] LLM returned an empty rewrite. Exiting to avoid data loss.")
            sys.exit(1)

        forced_rewrite_done = forced_rewrite_done or force_this_round
        current_file.write_text(rewritten, encoding="utf-8")
        (ralph_dir / f"round-{round_num}-rewrite.txt").write_text(rewritten, encoding="utf-8")

        after_matches = run_analyze(binary, rules_dir, current_file)
        write_analysis(ralph_dir, round_num, after_matches, "after")
        after_count = len(after_matches)
        print(f"[ralph] Round {round_num}: {after_count} structural matches after rewrite")

        if after_count <= args.target_matches:
            print("[ralph] Target reached after rewrite. Cleaning complete.")
            break
    else:
        final_matches = run_analyze(binary, rules_dir, current_file)
        print(f"\n[ralph] Max rounds reached. Final structural matches: {len(final_matches)}")

    final_path = ralph_dir / "cleaned.txt"
    final_path.write_text(current_file.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"[ralph] Final text: {final_path}")


if __name__ == "__main__":
    main()
