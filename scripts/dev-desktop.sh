#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
command -v pnpm >/dev/null || { echo '需要 Node.js 与 pnpm 11；参阅 CONTRIBUTING.md。' >&2; exit 1; }
pnpm dev &
STUDY_VITE_PID=$!
trap 'kill "$STUDY_VITE_PID" 2>/dev/null || true' EXIT
export CARGO_HOME="${STUDY_CARGO_HOME:-$PWD/.cache/cargo}"
cargo run -p study-assistant-desktop --no-default-features -- "$@"
