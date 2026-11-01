#!/usr/bin/env bash
# Run inside WSL while an outer Windows-side dev/tools/window-lock.sh is held.
# Usage: dev/tools/linux/verify-workspace.sh [log-directory] [--clean-workspace]
# Never acquire a second lock inside WSL: Windows and WSL have different PIDs.
# CARGO_TARGET_DIR defaults to the distribution's own filesystem. Give each
# worktree its own directory; never share it with Windows or concurrent builds.
set -uo pipefail
if [[ "$(uname -s)" != Linux ]]; then
  echo "run inside WSL with the outer Windows-side window lock held" >&2
  exit 2
fi
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/kivact-target}"
out="target/verify-workspace/$(date +%Y%m%d-%H%M%S)-$$"
clean_workspace=false
have_out=false
for arg in "$@"; do
  case "$arg" in
    --clean-workspace) clean_workspace=true ;;
    --*) echo "unknown option: $arg" >&2; exit 2 ;;
    *)
      if $have_out; then echo "usage: $0 [log-directory] [--clean-workspace]" >&2; exit 2; fi
      out="$arg"; have_out=true ;;
  esac
done
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
# No clean by default. Opt-in removes workspace packages only, retaining the
# external dependency cache. Profile and compiler flags belong to the caller.
if $clean_workspace; then
  run clean python3 -c 'import json, subprocess; packages = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"]))["packages"]; subprocess.check_call(["cargo", "clean", *[argument for package in packages for argument in ("-p", package["name"])]])'
fi
# Preserve the caller's legacy runner logs too; all logs for this run go below
# the new output directory rather than overwrite test-wslg's default directory.
run wslg env KIVACT_WSL_LOG_DIR="$out/wslg" bash dev/tools/linux/test-wslg.sh x11
run architecture cargo xtask check --root "$PWD" --no-write
run demo env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET cargo run -p kivact-embed-host -- --smoke
run demo_two env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET cargo run -p kivact-embed-host -- --two --scale 1.25 --smoke
echo "DONE failed=$failed" | tee -a "$out/summary.txt"
exit "$failed"
