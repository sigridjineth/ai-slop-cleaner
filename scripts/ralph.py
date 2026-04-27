#!/usr/bin/env python3
"""
Ralph — Standalone iterative cleanup harness for ai-slop-cleaner.
No OMX dependency. Runs locally with optional LLM API integration.

Usage:
    python3 scripts/ralph.py <input-file> [--max-rounds 3] [--target-score 15]

Environment variables:
    OPENAI_API_KEY    - Use OpenAI API for rewrites
    ANTHROPIC_API_KEY - Use Anthropic API for rewrites
    AICHAT_MODEL      - Use `aichat` CLI if installed (e.g. "claude", "gpt-4o")

If no API key or CLI is available, the script enters interactive mode:
    prompts are written to .ralph/round-N-prompt.md; you paste the LLM
    response into .ralph/round-N-response.md and the loop continues.
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


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
    for c in candidates:
        if c.exists():
            return str(c)
    # Try PATH (may be wrapper script)
    result = subprocess.run(["which", "ai-slop-cleaner"], capture_output=True, text=True)
    if result.returncode == 0:
        return result.stdout.strip()
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
    # Parse JSON array from stdout
    return json.loads(result.stdout)


def extract_top_violations(matches, n=5):
    """Return the top-N highest-weight structural matches."""
    all_violations = []
    for m in matches:
        all_violations.append({
            "type": m.get("pattern_name", "pattern"),
            "line": m.get("line_number", 0),
            "severity": m.get("severity", "unknown"),
            "weight": m.get("weight", 0),
            "matched_text": m.get("matched_text", ""),
        })
    all_violations.sort(key=lambda x: x["weight"], reverse=True)
    return all_violations[:n]


def make_prompt(input_text, structural_matches, violations, round_num):
    """Generate the cleanup prompt for the LLM holistic judge."""
    violation_text = "\n".join(
        f"{i+1}. [{v['severity']}] {v['type']} (weight {v['weight']}) at line {v['line']}: {v['matched_text']}"
        for i, v in enumerate(violations)
    )
    prompt = f"""You are Ralph, an expert editor and holistic judge of AI-generated slop.

Your job is to read the FULL TEXT plus STRUCTURAL MATCHES from a dumb pattern matcher, then:
1. Identify BOTH structural slop (bold, em dashes, bullets, etc.) AND semantic slop (overuse of "So", "That makes it...", repetitive explanations, conversational filler, excessive structuring)
2. Rewrite the text to remove ALL slop while preserving the original meaning.
3. Make it sound like a human wrote it — direct, concise, no fluff.

## Round {round_num}

## Full Text
```
{input_text}
```

## Structural Matches from Dumb Pattern Matcher ({len(structural_matches)} found)
{violation_text}

## Instructions
1. Consider structural matches AND your own semantic judgment together.
2. Remove markdown artifacts (excessive bullets, tables, bold, em dashes) only if they feel mechanical.
3. Fix semantic slop: rewrite "So..." openings, "That makes it..." transitions, repetitive explanations.
4. Output ONLY the rewritten text. No commentary, no markdown code fences around the whole output.
"""
    return prompt


def call_llm(prompt):
    """Call an available LLM backend. Returns the response text."""
    # 1. Try aichat CLI
    if shutil.which("aichat") and os.environ.get("AICHAT_MODEL"):
        model = os.environ["AICHAT_MODEL"]
        result = subprocess.run(
            ["aichat", "--model", model, "--no-stream", prompt],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            return result.stdout

    # 2. Try OpenAI API
    openai_key = os.environ.get("OPENAI_API_KEY")
    if openai_key:
        import urllib.request
        req = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=json.dumps({
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.3,
            }).encode(),
            headers={
                "Authorization": f"Bearer {openai_key}",
                "Content-Type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=120) as resp:
                data = json.loads(resp.read())
                return data["choices"][0]["message"]["content"]
        except Exception as e:
            print(f"[ralph] OpenAI API error: {e}")

    # 3. Try Anthropic API
    anthropic_key = os.environ.get("ANTHROPIC_API_KEY")
    if anthropic_key:
        import urllib.request
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=json.dumps({
                "model": "claude-3-5-haiku-20241022",
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}],
            }).encode(),
            headers={
                "x-api-key": anthropic_key,
                "anthropic-version": "2023-06-01",
                "Content-Type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(req, timeout=120) as resp:
                data = json.loads(resp.read())
                return data["content"][0]["text"]
        except Exception as e:
            print(f"[ralph] Anthropic API error: {e}")

    return None


def interactive_round(ralph_dir, round_num, prompt):
    """Write prompt and wait for user to provide response file."""
    prompt_file = ralph_dir / f"round-{round_num}-prompt.md"
    response_file = ralph_dir / f"round-{round_num}-response.md"
    prompt_file.write_text(prompt, encoding="utf-8")
    print(f"[ralph] Prompt written to {prompt_file}")
    print(f"[ralph] Paste the LLM response into {response_file} and press Enter...")
    input()
    if not response_file.exists():
        print("[ralph] No response file found. Exiting.")
        sys.exit(1)
    return response_file.read_text(encoding="utf-8")


def clean_response(text):
    """Strip markdown fences if the LLM wrapped the output."""
    text = text.strip()
    if text.startswith("```"):
        lines = text.splitlines()
        # Remove first line if it starts with ```
        if lines[0].startswith("```"):
            lines = lines[1:]
        # Remove last line if it starts with ```
        if lines and lines[-1].startswith("```"):
            lines = lines[:-1]
        text = "\n".join(lines).strip()
    return text


def main():
    parser = argparse.ArgumentParser(description="Ralph standalone cleanup harness")
    parser.add_argument("input_file", help="File to clean up")
    parser.add_argument("--max-rounds", type=int, default=3, help="Maximum iterations")
    parser.add_argument("--target-matches", type=int, default=0, help="Stop when structural matches at or below this")
    parser.add_argument("--rules-dir", default=None, help="Path to rules directory")
    args = parser.parse_args()

    input_file = Path(args.input_file).resolve()
    if not input_file.exists():
        print(f"[ralph] File not found: {input_file}")
        sys.exit(1)

    script_dir = Path(__file__).parent.resolve()
    project_root = script_dir.parent
    ralph_dir = project_root / ".ralph"
    ralph_dir.mkdir(exist_ok=True)

    binary = find_binary()
    if not binary:
        binary = build_binary(project_root)

    # Resolve rules directory
    if args.rules_dir:
        rules_dir = Path(args.rules_dir)
    else:
        # Try release install path first, then source path
        release_rules = Path.home() / ".local" / "share" / "ai-slop-cleaner" / "rules"
        source_rules = project_root / "rust" / "rules"
        rules_dir = release_rules if release_rules.exists() else source_rules

    current_file = ralph_dir / "current.txt"
    current_file.write_text(input_file.read_text(encoding="utf-8"), encoding="utf-8")

    print(f"[ralph] Binary: {binary}")
    print(f"[ralph] Rules:  {rules_dir}")
    print(f"[ralph] Target: <= {args.target_matches} structural matches | Max rounds: {args.max_rounds}")
    print("-" * 40)

    for round_num in range(1, args.max_rounds + 1):
        structural_matches = run_analyze(binary, rules_dir, current_file)
        match_count = len(structural_matches)

        print(f"\n[ralph] Round {round_num}: {match_count} structural matches")

        if match_count <= args.target_matches:
            print(f"[ralph] Target reached. Cleaning complete.")
            final_path = ralph_dir / "cleaned.txt"
            final_path.write_text(current_file.read_text(encoding="utf-8"), encoding="utf-8")
            print(f"[ralph] Final text: {final_path}")
            sys.exit(0)

        violations = extract_top_violations(structural_matches, n=5)
        prompt = make_prompt(current_file.read_text(encoding="utf-8"), structural_matches, violations, round_num)

        response = call_llm(prompt)
        if response is None:
            response = interactive_round(ralph_dir, round_num, prompt)

        cleaned = clean_response(response)
        current_file.write_text(cleaned, encoding="utf-8")

    # Final analyze after max rounds
    structural_matches = run_analyze(binary, rules_dir, current_file)
    match_count = len(structural_matches)
    print(f"\n[ralph] Max rounds reached. Final structural matches: {match_count}")
    final_path = ralph_dir / "cleaned.txt"
    final_path.write_text(current_file.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"[ralph] Final text: {final_path}")


if __name__ == "__main__":
    import shutil
    main()
