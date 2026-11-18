#!/usr/bin/env bash
# Run a command that opens real windows (kivact::test, the public-API e2e,
# kit consumer and catalog tests, doctests) while holding a lock shared by
# every worktree of this repository, so window tests never run from two
# processes at once (concurrent real windows steal focus and can hang).
#
# Usage: dev/tools/window-lock.sh [--timeout SECONDS] [--if-mirror] -- <command> [args...]
#   dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- menu
#   --if-mirror runs the command without the lock when KIVACT_TEST_MIRROR=off.
#   Use it only for packages whose tests never force a mirror with
#   RenderOptions::mirror(MirrorMode::Required) (see CONTRIBUTING.md).
#
# The lock is a directory under the repository's common git dir, so all
# worktrees share it. A lock left by a process that no longer exists is
# taken over. The command runs under `timeout` (default 900 s) and its exit
# status is returned unchanged (124 when it timed out).

set -euo pipefail

limit=900
if_mirror=0
while [[ "${1:-}" == --timeout || "${1:-}" == --if-mirror ]]; do
  case "$1" in
    --timeout) limit="$2"; shift 2 ;;
    --if-mirror) if_mirror=1; shift ;;
  esac
done
if [[ "${1:-}" != -- ]]; then
  echo "usage: $0 [--timeout SECONDS] [--if-mirror] -- <command> [args...]" >&2
  exit 2
fi
shift

run() {
  set +e
  timeout "$limit" "$@"
  local status=$?
  set -e
  if (( status == 124 )); then
    echo "window-lock: timed out after ${limit}s: $*" >&2
  fi
  return "$status"
}

if (( if_mirror )) && [[ "${KIVACT_TEST_MIRROR:-auto}" == off ]]; then
  run "$@" && exit 0 || exit $?
fi

common="$(git rev-parse --git-common-dir)"
lock="$common/kivact-window-tests.lock"
waited=0
until mkdir "$lock" 2>/dev/null; do
  owner="$(cat "$lock/pid" 2>/dev/null || true)"
  if [[ -n "$owner" ]] && ! kill -0 "$owner" 2>/dev/null; then
    # Claim the stale lock with an atomic rename so that only one waiter
    # removes it; a waiter whose rename fails just retries `mkdir`.
    stale="$lock.stale.$$"
    if mv -T "$lock" "$stale" 2>/dev/null; then
      if [[ "$(cat "$stale/pid" 2>/dev/null || true)" == "$owner" ]]; then
        echo "window-lock: taking over a lock left by $owner" >&2
        rm -rf "$stale"
      else
        # Another waiter replaced the stale lock first: give its lock back.
        mv -T "$stale" "$lock" 2>/dev/null || rm -rf "$stale"
      fi
    fi
    continue
  fi
  if (( waited % 60 == 0 )); then
    echo "window-lock: waiting for $(cat "$lock/what" 2>/dev/null || echo another run)" >&2
  fi
  sleep 2
  waited=$((waited + 2))
done
echo $$ > "$lock/pid"
printf '%s (in %s)\n' "$*" "$(pwd)" > "$lock/what"
trap 'rm -rf "$lock"' EXIT

run "$@" && exit 0 || exit $?
