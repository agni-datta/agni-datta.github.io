#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo run --locked -p sitegen

wasm-pack build \
  --release \
  --target web \
  --out-dir ../public/assets/wasm \
  --out-name agni_datta_site_wasm \
  wasm
