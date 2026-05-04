#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo run --release -p sitegen

if command -v wasm-pack >/dev/null 2>&1; then
  (cd wasm && wasm-pack build --release --target web --out-dir ../public/assets/wasm)
else
  printf 'wasm-pack not found; generated static HTML without the WASM enhancer.\n' >&2
fi
