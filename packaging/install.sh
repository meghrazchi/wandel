#!/usr/bin/env bash
# Wandel headless CLI & MCP installer for macOS and Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/meghrazchi/wandel/main/packaging/install.sh | bash
set -euo pipefail

REPO="meghrazchi/wandel"
VERSION="${WANDEL_VERSION:-latest}"
INSTALL_DIR="${WANDEL_INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}:${ARCH}" in
  Darwin:arm64)  ARCHIVE="wandel-cli-macos-arm64.tar.gz" ;;
  Darwin:x86_64) ARCHIVE="wandel-cli-macos-x64.tar.gz" ;;
  Linux:x86_64)  ARCHIVE="wandel-cli-linux-x64.tar.gz" ;;
  *)
    echo "Unsupported platform: ${OS} ${ARCH}" >&2
    echo "Download manually from https://github.com/${REPO}/releases" >&2
    exit 1
    ;;
esac

if [[ "${VERSION}" == "latest" ]]; then
  BASE_URL="https://github.com/${REPO}/releases/latest/download"
else
  BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "Downloading ${ARCHIVE}..."
curl -fsSL "${BASE_URL}/${ARCHIVE}" -o "${TMP_DIR}/${ARCHIVE}"
curl -fsSL "${BASE_URL}/${ARCHIVE}.sha256" -o "${TMP_DIR}/${ARCHIVE}.sha256"

echo "Verifying SHA-256 checksum..."
if command -v shasum >/dev/null 2>&1; then
  (cd "${TMP_DIR}" && shasum -a 256 -c "${ARCHIVE}.sha256")
elif command -v sha256sum >/dev/null 2>&1; then
  (cd "${TMP_DIR}" && sha256sum -c "${ARCHIVE}.sha256")
fi

mkdir -p "${INSTALL_DIR}"
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "${INSTALL_DIR}" wandel
chmod +x "${INSTALL_DIR}/wandel"

echo ""
echo "Installed Wandel CLI & MCP to: ${INSTALL_DIR}/wandel"
if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
  echo "Add to your shell profile if needed:"
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi
"${INSTALL_DIR}/wandel" info || true
