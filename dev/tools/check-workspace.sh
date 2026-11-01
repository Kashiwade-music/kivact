#!/usr/bin/env bash
# Run from the repository root; refuse to overwrite any previous evidence.
set -uo pipefail
# Usage: dev/tools/check-workspace.sh [log-directory]
# Each real-window command acquires the shared lock. Do not wrap this whole
# script in window-lock (that would nest the same lock).
if (($# > 1)); then
  echo "usage: $0 [log-directory]" >&2
  exit 2
fi
out="${1:-target/check-workspace/$(date +%Y%m%d-%H%M%S)-$$}"
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
run fmt cargo fmt --all -- --check
run xtask cargo xtask check
run architecture bash -c '[[ -z "$(git status --porcelain -- architecture)" ]]'
run clippy cargo clippy --workspace --all-targets --all-features -- -D warnings
run test_all dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --all-features --no-fail-fast
run test_nodef dev/tools/window-lock.sh --timeout 5400 -- cargo test --workspace --no-default-features --no-fail-fast
run features bash dev/tools/check-features.sh
run policy cargo test -p kivact-workspace-policy-tests
run docs dev/tools/window-lock.sh --timeout 3600 -- cargo test -p kivact-docs --doc
echo "DONE failed=$failed" | tee -a "$out/summary.txt"
exit "$failed"
