#!/usr/bin/env bash
# AI-Slop-Cleaner OMX Delegation Wrapper
# Usage: ./omx-delegate.sh <input-file> [output-dir]
#
# This script orchestrates the ai-slop-cleaner Rust binary through oh-my-codex
# by running analysis, generating a TASK.md context, and delegating cleanup.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BINARY="${PROJECT_ROOT}/rust/target/release/ai-slop-cleaner"
RULES_DIR="${PROJECT_ROOT}/rust/rules"

INPUT_FILE="${1:-}"
OUTPUT_DIR="${2:-${PROJECT_ROOT}/.omx/output}"

if [[ -z "${INPUT_FILE}" ]]; then
    echo "Usage: $0 <input-file> [output-dir]"
    exit 1
fi

if [[ ! -f "${BINARY}" ]]; then
    echo "Binary not found at ${BINARY}. Building..."
    cd "${PROJECT_ROOT}/rust"
    cargo build --release
fi

mkdir -p "${OUTPUT_DIR}"

# Step 1: Run analysis and capture JSON
JSON_OUT="${OUTPUT_DIR}/analysis.json"
echo "[omx-delegate] Running ai-slop-cleaner analysis on ${INPUT_FILE}..."
"${BINARY}" --rules-dir "${RULES_DIR}" score "${INPUT_FILE}" --format json > "${JSON_OUT}"

SCORE=$(python3 -c "import json; print(json.load(open('${JSON_OUT}')).get('overall_score', 0))")
MATCHES=$(python3 -c "import json; print(len(json.load(open('${JSON_OUT}')).get('matches', [])))")
WORDS=$(python3 -c "import json; print(len(json.load(open('${JSON_OUT}')).get('word_matches', [])))")

echo "[omx-delegate] Score: ${SCORE}, Pattern matches: ${MATCHES}, Word matches: ${WORDS}"

# Step 2: Generate OMX context document
CONTEXT_FILE="${OUTPUT_DIR}/omx-context.md"
cat > "${CONTEXT_FILE}" <<EOF
# OMX Context: AI-Slop-Cleaner Analysis

## Input File
- Path: ${INPUT_FILE}
- Analysis JSON: ${JSON_OUT}

## Score Summary
- Overall Score: ${SCORE}/100 (higher = more AI slop detected)
- Pattern Matches: ${MATCHES}
- Word Matches: ${WORDS}

## Ruleset
- Patterns: 70 banned structural patterns
- Words: 93 banned words/phrases
- Rules Directory: ${RULES_DIR}

## Task
Improve the input text to reduce the AI slop score below 15.0.
Focus on:
1. Removing "A is not X, it is Y" redefinition patterns
2. Replacing banned words with natural alternatives
3. Breaking uniform sentence structures
4. Removing markdown artifacts (bullets, tables, excessive headings)
5. Making Korean text sound natural (avoid ~것입니다, ~할 수 있습니다)

## Deliverable
A revised version of the input text with AI slop score < 15.0.
EOF

echo "[omx-delegate] Context written to ${CONTEXT_FILE}"

# Step 3: Check if omx is available and delegate
if command -v omx &>/dev/null; then
    echo "[omx-delegate] Delegating to oh-my-codex..."
    # Use npx to run latest codex since global install is outdated
    npx @openai/codex@latest exec --sandbox danger-full-access --dangerously-bypass-approvals-and-sandbox "
## \$team tasks
1. Read the analysis JSON at ${JSON_OUT}
2. Read the original input at ${INPUT_FILE}
3. Identify the top 5 highest-weight pattern matches
4. Plan specific rewrites for each match

## \$ralph tasks
1. Create a ralph plan: iterative cleanup with 3 rounds max
2. Round 1: Fix high-severity patterns (redefinition, banned words)
3. Round 2: Fix medium-severity patterns (structure, flow)
4. Round 3: Polish for natural human voice
5. After each round, run: ${BINARY} --rules-dir ${RULES_DIR} score <file> --format json
6. Stop when score < 15.0 or after 3 rounds

## \$ultrawork tasks
1. Implement the cleanup edits
2. Run the Rust binary after each round to verify score improvement
3. Write final cleaned text to ${OUTPUT_DIR}/cleaned.txt
4. Write final score report to ${OUTPUT_DIR}/final-report.json
5. Return the cleaned text and final score
"
else
    echo "[omx-delegate] omx not found in PATH. Skipping delegation."
    echo "[omx-delegate] To delegate manually, run:"
    echo "  omx exec --skip-git-repo-check --dangerously-bypass-approvals-and-sandbox '...'"
fi

echo "[omx-delegate] Done. Output in ${OUTPUT_DIR}/"
