#!/usr/bin/env bash
# AI-Slop-Cleaner OMX Delegation Wrapper
# Usage: ./omx-delegate.sh <input-file> [output-dir]
#
# Runs the Rust analyze subcommand, packages raw structural matches plus the
# universal agent categories, and asks an LLM/Codex agent to rewrite the complete
# text by inference. This script never asks agents to regex-replace text.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BINARY="${PROJECT_ROOT}/rust/target/release/ai-slop-cleaner"
RULES_DIR="${PROJECT_ROOT}/rust/rules"
PATTERNS_AGENT="${RULES_DIR}/patterns-agent.md"

INPUT_FILE="${1:-}"
OUTPUT_DIR="${2:-${PROJECT_ROOT}/.omx/output}"

if [[ -z "${INPUT_FILE}" ]]; then
    echo "Usage: $0 <input-file> [output-dir]"
    exit 1
fi

if [[ ! -f "${INPUT_FILE}" ]]; then
    echo "Input file not found: ${INPUT_FILE}"
    exit 1
fi

if [[ ! -f "${BINARY}" ]]; then
    echo "Binary not found at ${BINARY}. Building..."
    cargo build --release --manifest-path "${PROJECT_ROOT}/rust/Cargo.toml"
fi

mkdir -p "${OUTPUT_DIR}"

ANALYSIS_JSON="${OUTPUT_DIR}/analysis.json"
CONTEXT_FILE="${OUTPUT_DIR}/omx-context.md"
INPUT_COPY="${OUTPUT_DIR}/input.txt"

cp "${INPUT_FILE}" "${INPUT_COPY}"

echo "[omx-delegate] Running ai-slop-cleaner analyze on ${INPUT_FILE}..."
"${BINARY}" --rules-dir "${RULES_DIR}" analyze "${INPUT_FILE}" > "${ANALYSIS_JSON}"

MATCHES=$(python3 -c "import json,sys; print(len(json.load(open(sys.argv[1]))))" "${ANALYSIS_JSON}")
TOP_MATCHES=$(python3 - "${ANALYSIS_JSON}" <<'PY'
import json
import sys

matches = json.load(open(sys.argv[1]))
matches = sorted(matches, key=lambda match: match.get("weight", 0), reverse=True)[:8]
for index, match in enumerate(matches, 1):
    print(
        "{index}. [{severity}] {name} line {line}: {text}".format(
            index=index,
            severity=match.get("severity", "unknown"),
            name=match.get("pattern_name", "pattern"),
            line=match.get("line_number", 0),
            text=match.get("matched_text", ""),
        )
    )
PY
)

echo "[omx-delegate] Structural matches: ${MATCHES}"

cat > "${CONTEXT_FILE}" <<EOF_CONTEXT
# OMX Context for AI-Slop-Cleaner Rewrite

## Input
- Source file: ${INPUT_FILE}
- Input copy: ${INPUT_COPY}
- Structural analysis JSON: ${ANALYSIS_JSON}
- Universal category catalog: ${PATTERNS_AGENT}

## Analyze result
Rust \`analyze\` returned ${MATCHES} raw structural matches. These matches are
evidence only; they are not a score and not rewrite instructions.

Top structural evidence:

${TOP_MATCHES}

## Required architecture
Use full-text LLM inference:

1. Read the complete input text.
2. Read all raw structural matches from \`analysis.json\`.
3. Read \`patterns-agent.md\` universal categories.
4. Judge semantic slop by category intent in whatever language the text uses.
5. Rewrite the complete text naturally while preserving meaning and grammar.
6. Never regex-replace, never produce patches, and never cut words out of compound terms.

## Deliverables
- Complete rewritten text at ${OUTPUT_DIR}/cleaned.txt
- Fresh post-rewrite analysis JSON at ${OUTPUT_DIR}/final-analysis.json
- Brief note at ${OUTPUT_DIR}/final-report.md with match counts and remaining risks
EOF_CONTEXT

echo "[omx-delegate] Context written to ${CONTEXT_FILE}"

if command -v omx &>/dev/null; then
    echo "[omx-delegate] Delegating to oh-my-codex..."
    omx exec --skip-git-repo-check --dangerously-bypass-approvals-and-sandbox "
Read ${CONTEXT_FILE}. Rewrite ${INPUT_COPY} with full-text LLM inference using ${ANALYSIS_JSON} and ${PATTERNS_AGENT}. Do not use regex substitution or patch-style edits. Write the complete rewrite to ${OUTPUT_DIR}/cleaned.txt. Then run:

${BINARY} --rules-dir ${RULES_DIR} analyze ${OUTPUT_DIR}/cleaned.txt > ${OUTPUT_DIR}/final-analysis.json

Finally write ${OUTPUT_DIR}/final-report.md with the before/after structural match counts and any remaining risks.
"
else
    echo "[omx-delegate] omx not found in PATH. Context is ready for manual delegation: ${CONTEXT_FILE}"
fi

echo "[omx-delegate] Done. Output in ${OUTPUT_DIR}/"
