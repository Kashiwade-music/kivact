#!/usr/bin/env bash
# Run from the repository root; refuse to overwrite any previous evidence.
set -uo pipefail
# Usage: dev/tools/check-consumers.sh [log-directory] [--cpu-only]
# Renderer-capable fixtures request GPU by default, then run a CPU variant.
# CPU-only fixtures never inherit a GPU request. Explicit CPU selection skips
# GPU-feature runs; other renderer and compiler flags pass through.
out="target/check-consumers/$(date +%Y%m%d-%H%M%S)-$$"
cpu_only=false
have_out=false
for arg in "$@"; do
  case "$arg" in
    --cpu-only) cpu_only=true ;;
    --*) echo "unknown option: $arg" >&2; exit 2 ;;
    *)
      if $have_out; then echo "usage: $0 [log-directory] [--cpu-only]" >&2; exit 2; fi
      out="$arg"; have_out=true ;;
  esac
done
if [[ "${ARVANE_RENDERER:-}" == cpu || "${KIVACT_TEST_GPU:-}" == 0 ]]; then cpu_only=true; fi
if $cpu_only; then export ARVANE_RENDERER=cpu KIVACT_TEST_GPU=0; fi
if [[ -e "$out" ]]; then
  echo "log directory already exists: $out" >&2
  exit 2
fi
mkdir -p "$(dirname "$out")" || exit 2
mkdir "$out" || exit 2
failed=0
run() {
  local name="$1"; shift
  local start=$SECONDS
  echo "== $name"
  "$@" >"$out/$name.log" 2>&1
  local result=$?
  echo "$name exit=$result seconds=$((SECONDS-start))" | tee -a "$out/summary.txt"
  if ((result)); then failed=1; fi
}
for manifest in dev/tests/fixtures/*/Cargo.toml; do
  fixture="${manifest%/Cargo.toml}"
  fixture="${fixture##*/}"
  run "$fixture-fmt" cargo fmt --manifest-path "$manifest" -- --check
  features=()
  if $cpu_only; then features=(--no-default-features); fi
  run "$fixture-check" cargo check --manifest-path "$manifest" --tests "${features[@]}"
  if $cpu_only; then
    run "$fixture-clippy" cargo clippy --manifest-path "$manifest" --all-targets --no-default-features -- -D warnings
  else
    run "$fixture-clippy" cargo clippy --manifest-path "$manifest" --all-targets --all-features -- -D warnings
  fi
  if ! $cpu_only && [[ "$fixture" == official-ui-consumer || "$fixture" == external-design-system || "$fixture" == text-primitives ]]; then
    run "$fixture-tests" dev/tools/window-lock.sh --timeout 3600 -- env KIVACT_TEST_GPU=1 cargo test --manifest-path "$manifest"
    run "$fixture-cpu" dev/tools/window-lock.sh --timeout 3600 -- env ARVANE_RENDERER=cpu KIVACT_TEST_GPU=0 cargo test --manifest-path "$manifest" --no-default-features
  else
    run "$fixture-tests" dev/tools/window-lock.sh --timeout 3600 -- env KIVACT_TEST_GPU=0 cargo test --manifest-path "$manifest" "${features[@]}"
  fi
done
echo "DONE failed=$failed" | tee -a "$out/summary.txt"
exit "$failed"
