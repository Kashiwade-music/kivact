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
# Each entry is `crate/feature`. The facade (`kivact`, `kivact-ui`) is built
# with only that feature turned on; the first failure stops the run with a
# non-zero status.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

entries=(
  arvane-native/embed
  arvane/embed
  kivact-engine/embed
  kivact-core/embed
  kivact/embed
  arvane-render/gpu
  arvane-render/gpu-interop
  arvane-native/testing
  arvane/gpu
  arvane/gpu-interop
  arvane/testing
  gpu-share/export
  gpu-share/import
  gpu-share/wgpu
  kivact-core/gpu
  kivact-core/gpu-interop
  kivact-core/test
  kivact-engine/gpu
  kivact-engine/gpu-interop
  kivact-engine/testing
  kivact-lower/gpu-interop
  kivact-style/gpu-interop
  kivact-ui-creative/gpu-interop
)

for entry in "${entries[@]}"; do
  crate="${entry%%/*}"
  packages=(-p kivact -p kivact-ui)
  if [[ "$crate" != kivact && "$crate" != kivact-ui ]]; then
    packages+=(-p "$crate")
  fi
  if output="$(cargo check "${packages[@]}" --lib --features "$entry" 2>&1)"; then
    echo "ok   $entry"
  else
    echo "FAIL $entry" >&2
    echo "$output" | tail -40 >&2
    exit 1
  fi
done
