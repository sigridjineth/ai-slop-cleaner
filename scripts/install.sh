#!/bin/bash
# AI Slop Cleaner installer
# Usage: curl -fsSL https://raw.githubusercontent.com/sigridjineth/ai-slop-cleaner/main/scripts/install.sh | bash
set -euo pipefail

REPO="sigridjineth/ai-slop-cleaner"
INSTALL_DIR="${HOME}/.local/share/ai-slop-cleaner"
BIN_DIR="${HOME}/.local/bin"
WRAPPER="${BIN_DIR}/ai-slop-cleaner"

echo "AI Slop Cleaner Installer"
echo

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac
case "$OS" in
  linux) OS="linux" ;;
  darwin) OS="macos" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

echo "Platform: ${OS}-${ARCH}"
echo

# Try release binary first
echo "Checking GitHub releases..."
LATEST=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -n "$LATEST" ] && [ "$LATEST" != "null" ]; then
  ASSET="ai-slop-cleaner-${OS}-${ARCH}.tar.gz"
  URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"

  if curl -fsSL -o /tmp/ai-slop-cleaner.tar.gz "$URL" 2>/dev/null; then
    echo "Release:  ${LATEST}"
    echo "Asset:    ${ASSET}"
    rm -rf "$INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
    tar xzf /tmp/ai-slop-cleaner.tar.gz -C "$INSTALL_DIR" --strip-components=1
    BINARY="${INSTALL_DIR}/bin/ai-slop-cleaner"
    RULES_DIR="${INSTALL_DIR}/rules"
  else
    echo "No prebuilt binary for ${OS}-${ARCH}. Falling back to source build..."
    LATEST=""
  fi
else
  echo "No release found. Falling back to source build..."
fi

# Fallback: clone and build from source
if [ -z "$LATEST" ] || [ "$LATEST" = "null" ]; then
  if ! command -v cargo &>/dev/null; then
    echo "Rust toolchain not found. Installing rustup..."
    if ! command -v rustup &>/dev/null; then
      curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    fi
    source "${HOME}/.cargo/env"
  fi

  echo "Cargo:  $(cargo --version)"

  if [ -d "${INSTALL_DIR}/.git" ]; then
    echo "Repo:   updating existing clone..."
    git -C "$INSTALL_DIR" pull --quiet
  else
    echo "Repo:   cloning into ${INSTALL_DIR}..."
    rm -rf "$INSTALL_DIR"
    git clone --depth 1 --quiet "https://github.com/${REPO}" "$INSTALL_DIR"
  fi

  RUST_DIR="${INSTALL_DIR}/rust"
  BINARY="${RUST_DIR}/target/release/ai-slop-cleaner"

  if [ ! -x "$BINARY" ]; then
    echo "Build:  compiling release binary..."
    cd "$RUST_DIR"
    cargo build --release
  fi

  RULES_DIR="${RUST_DIR}/rules"
fi

echo "Binary: ${BINARY}"

# Install wrapper
mkdir -p "$BIN_DIR"
cat > "$WRAPPER" <<EOF
#!/bin/bash
exec "${BINARY}" --rules-dir "${RULES_DIR}" "\$@"
EOF
chmod +x "$WRAPPER"

# Ensure PATH
if ! command -v ai-slop-cleaner &>/dev/null; then
  if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
    echo
    echo "Add ${BIN_DIR} to your PATH:"
    echo '  export PATH="${HOME}/.local/bin:${PATH}"'
  fi
fi

echo
echo "Done. Get started:"
echo
echo '  ai-slop-cleaner score document.md'
echo '  ai-slop-cleaner --help'
