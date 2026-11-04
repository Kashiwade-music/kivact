#!/usr/bin/env bash
# Check each non-default feature of the library crates on its own.
#
# Workspace builds unify features through dev-dependencies (for example
# kivact-style's dev-dependency on `kivact` with `gpu-interop`), so a library
# feature enabled on its own is checked only here: a crate's feature must
# change only that crate's behaviour, never an enum or struct another crate
# matches or builds. Add an entry here when a crate gains a feature.
#
# Usage: dev/tools/check-features.sh
#
# Shared values live in kivact-types; engine client contracts live in kivact-engine.
# Each entry is `crate/feature`. The facade (`kivact`, `kivact-ui`) is built
# with only that feature turned on; the first failure stops the run with a
# non-zero status.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

entries=(
  arvane-native/embed
  kivact-core/embed
  kivact/embed
  kivact/gpu
  kivact/gpu-interop
  kivact/tokio
  kivact/test
  arvane-render/gpu
  arvane-render/gpu-interop
  arvane-native/test
  arvane-virtual/mirror
  kivact-gpu-share/export
  kivact-gpu-share/import
  kivact-gpu-share/wgpu
  kivact-core/gpu-interop
  kivact-test/embed
  kivact-test/gpu-interop
  kivact-engine/testing
  kivact-ui-creative/gpu-interop
)

for entry in "${entries[@]}"; do
  crate="${entry%%/*}"
  packages=(-p kivact -p kivact-ui)
  if [[ "$crate" != kivact && "$crate" != kivact-ui ]]; then
    packages+=(-p "$crate")
  fi
  if output="$(cargo check "${packages[@]}" --no-default-features --lib --features "$entry" 2>&1)"; then
    echo "ok   $entry"
  else
    echo "FAIL $entry" >&2
    echo "$output" | tail -40 >&2
    exit 1
  fi
done
