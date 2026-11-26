#!/usr/bin/env bash
set -euo pipefail

# Sets up FFmpeg and FFprobe sidecar binaries for Tauri v2 in src-tauri/binaries/.
# Usage: ./scripts/fetch-ffmpeg.sh [target-triple]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${SCRIPT_DIR}/../src-tauri/binaries"
mkdir -p "${BIN_DIR}"

TARGET_TRIPLE="${1:-$(rustc -vV | awk '/host:/ { print $2 }')}"
echo "Configuring FFmpeg sidecars for target: ${TARGET_TRIPLE}"

EXT=""
if [[ "${TARGET_TRIPLE}" == *"windows"* ]]; then
  EXT=".exe"
fi

FFMPEG_DEST="${BIN_DIR}/ffmpeg-${TARGET_TRIPLE}${EXT}"
FFPROBE_DEST="${BIN_DIR}/ffprobe-${TARGET_TRIPLE}${EXT}"

if command -v ffmpeg >/dev/null 2>&1 && command -v ffprobe >/dev/null 2>&1; then
  cp "$(command -v ffmpeg)" "${FFMPEG_DEST}"
  cp "$(command -v ffprobe)" "${FFPROBE_DEST}"
  chmod +x "${FFMPEG_DEST}" "${FFPROBE_DEST}"
  echo "Copied system ffmpeg & ffprobe to ${BIN_DIR}/"
  echo "Note: For standalone release distribution, replace these with static builds from:"
  echo "  - macOS:   https://evermeet.cx/ffmpeg/ or https://ffmpeg.martin-riedl.de/"
  echo "  - Windows: https://www.gyan.dev/ffmpeg/builds/ (release-essentials)"
  echo "  - Linux:   https://johnvansickle.com/ffmpeg/ (static builds)"
else
  echo "Error: ffmpeg/ffprobe not found in PATH. Please install ffmpeg or place static binaries at:"
  echo "  ${FFMPEG_DEST}"
  echo "  ${FFPROBE_DEST}"
  exit 1
fi
