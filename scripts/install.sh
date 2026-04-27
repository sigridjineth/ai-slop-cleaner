#!/bin/bash
# AI Slop Cleaner installer — one command, everything auto-built.
# Usage: curl -fsSL https://raw.githubusercontent.com/sigridjineth/ai-slop-cleaner/main/scripts/install.sh | bash
set -euo pipefail

REPO_URL="https://github.com/sigridjineth/ai-slop-cleaner"
INSTALL_DIR="${HOME}/.local/share/ai-slop-cleaner"
BIN_DIR="${HOME}/.local/bin"
WRAPPER="${BIN_DIR}/ai-slop-cleaner"

echo "╭──────────────────────────────────────╮"
echo "│     AI Slop Cleaner Installer        │"
echo "╰──────────────────────────────────────╯"
echo

# 1. Ensure cargo is available
if ! command -v cargo &>/dev/null; then
  echo "Rust toolchain not found. Installing rustup..."
  if ! command -v rustup &>/dev/null; then
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  fi
  # shellcheck source=/dev/null
  source "${HOME}/.cargo/env"
fi

echo "  Cargo:  $(cargo --version)"

# 2. Clone or update repository
if [ -d "${INSTALL_DIR}/.git" ]; then
  echo "  Repo:   updating existing clone..."
  git -C "${INSTALL_DIR}" pull --quiet
else
  echo "  Repo:   cloning into ${INSTALL_DIR}..."
  rm -rf "${INSTALL_DIR}"
  git clone --depth 1 --quiet "${REPO_URL}" "${INSTALL_DIR}"
fi

# 3. Build release binary
RUST_DIR="${INSTALL_DIR}/rust"
BINARY="${RUST_DIR}/target/release/ai-slop-cleaner"

if [ ! -x "${BINARY}" ]; then
  echo "  Build:  compiling release binary..."
  cd "${RUST_DIR}"
  cargo build --release
fi

echo "  Binary: ${BINARY}"

# 4. Install wrapper that points to built-in rules
mkdir -p "${BIN_DIR}"
RULES_DIR="${RUST_DIR}/rules"

cat > "${WRAPPER}" <<EOF
#!/bin/bash
# AI Slop Cleaner wrapper — auto-points to default rules.
exec "${BINARY}" --rules-dir "${RULES_DIR}" "\$@"
EOF

chmod +x "${WRAPPER}"

# 5. Ensure PATH
if ! command -v ai-slop-cleaner &>/dev/null; then
  if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
    echo
    echo "  Add ${BIN_DIR} to your PATH:"
    echo '    export PATH="${HOME}/.local/bin:${PATH}"'
  fi
fi

echo
echo "Done! Get started:"
echo
echo '  ai-slop-cleaner score document.md'
echo '  ai-slop-cleaner --help'
