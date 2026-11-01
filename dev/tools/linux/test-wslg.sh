#!/usr/bin/env bash
# Run Kivact's Linux gates on real windows under WSLg (docs/for-kivact-developer/design.md ch. 13,
# docs/for-kivact-developer/coding-rules.md).
#
# From Windows (Git Bash) this re-runs itself inside the WSL distribution on
# the same working tree; inside Linux it runs directly. Linux build output goes
# to CARGO_TARGET_DIR (default: $HOME/kivact-target on the distribution's own
# file system), never under the Windows tree, so the two builds never collide
# and the build does not crawl over the /mnt file share.
#
# Usage: dev/tools/linux/test-wslg.sh [wayland|x11|all]   (default: all)
#
# Gates: clippy (-D warnings), Arvane's contract suite and the whole
# workspace's tests (`--no-fail-fast`) once per display server. Test cases run
# serially because native embedded editors share real OS focus: Wayland as
# WSLg offers it, and X11 (XWayland) with WAYLAND_DISPLAY unset, which is how
# winit 0.30 picks X11. kivact-music-player is left out: its audio needs ALSA's
# development package (libasound2-dev), which the build cannot do without.
# Logs: ${KIVACT_WSL_LOG_DIR:-$CARGO_TARGET_DIR/test-wslg}/<step>.log.
#
# Environment: KIVACT_WSL_DISTRO (default Ubuntu) picks the distribution;
# ARVANE_RENDERER and WGPU_* reach the tests unchanged.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

if [[ "$(uname -s)" != Linux ]]; then
  distro="${KIVACT_WSL_DISTRO:-Ubuntu}"
  win_root="$(cygpath -w "$root")"
  linux_root="$(wsl.exe -d "$distro" -e wslpath -a "$win_root" | tr -d '\r')"
  exec wsl.exe -d "$distro" -e bash -lc \
    "cd '$linux_root' && exec bash dev/tools/linux/test-wslg.sh $(printf '%q ' "$@")"
fi

which="${1:-all}"
case "$which" in
  wayland | x11 | all) ;;
  *)
    echo "usage: $0 [wayland|x11|all]" >&2
    exit 2
    ;;
esac

cd "$root"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/kivact-target}"
logs="${KIVACT_WSL_LOG_DIR:-$CARGO_TARGET_DIR/test-wslg}"
mkdir -p "$logs"

wayland_display="${WAYLAND_DISPLAY:-}"
failed=()

# step NAME COMMAND...: run, log, and remember a failure without stopping.
step() {
  local name="$1"
  shift
  echo "== $name"
  if "$@" >"$logs/$name.log" 2>&1; then
    echo "   ok"
  else
    echo "   FAILED (see $logs/$name.log)"
    failed+=("$name")
  fi
  { grep -E '^test result:' "$logs/$name.log" || true; } | awk '
    { for (i = 1; i <= NF; i++) if ($(i) ~ /^(passed|failed|ignored);?$/) s[$(i)] += $(i - 1) }
    END { if (NR) printf "   %d passed, %d failed, %d ignored\n", s["passed;"], s["failed;"], s["ignored;"] }'
}

# on DISPLAY_SERVER COMMAND...: run with only that display server visible.
on() {
  local server="$1"
  shift
  if [[ "$server" == wayland ]]; then
    if [[ -z "$wayland_display" ]]; then
      echo "WAYLAND_DISPLAY is not set: no Wayland compositor to test on" >&2
      return 1
    fi
    WAYLAND_DISPLAY="$wayland_display" "$@"
  else
    env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET "$@"
  fi
}

step clippy cargo clippy --workspace --all-targets --all-features \
  --exclude kivact-music-player -- -D warnings

servers=(wayland x11)
[[ "$which" != all ]] && servers=("$which")
for server in "${servers[@]}"; do
  step "arvane-$server" on "$server" cargo test -p arvane --all-features --test e2e \
    -- --test-threads=1
  step "workspace-$server" on "$server" cargo test --workspace --all-features \
    --exclude kivact-music-player --no-fail-fast -- --test-threads=1
done

if ((${#failed[@]})); then
  echo "failed: ${failed[*]}"
  exit 1
fi
echo "all Linux gates passed"
