#!/usr/bin/env bash
# Verify GPU texture sharing (the experimental feature `gpu-interop`) on a
# Linux machine with a hardware Vulkan GPU, from `git clone` to a report
# (docs/for-kivact-developer/manual-verification.md, "GPU texture 共有").
#
# Self-contained: copy this one file to the machine and run it from a
# terminal inside the desktop session (the tests open real windows, so
# WAYLAND_DISPLAY or DISPLAY must be set; a plain SSH login has neither).
#
#   bash verify-gpu-interop.sh [options]
#
# Options:
#   --dir DIR        clone into DIR (default: $HOME/kivact-gpu-interop-check);
#                    an existing clone there is fetched and checked out again
#   --repo URL       repository (default: https://github.com/Kashiwade-music/kivact.git;
#                    use the SSH URL when HTTPS needs credentials)
#   --branch NAME    branch to verify (default: main)
#   --install-deps   install the runtime packages (Vulkan loader and Mesa
#                    driver, vulkan-tools, the validation layer, fonts) with
#                    sudo through apt, dnf or pacman first
#   --x11            in a Wayland session, also run the window tests on X11
#                    (XWayland)
#   --no-validation  skip the run under the Khronos validation layer
#   --jobs N         cargo build jobs (CARGO_BUILD_JOBS; lower it when the
#                    link step runs out of memory)
#
# Steps, each logged and none stopping the others: build the four test
# targets, the window-less read-back test, Arvane's contract tests, the
# public API's tests and the creative kit's tests (window tests go through
# dev/tools/window-lock.sh), then the same tests again under the validation
# layer when it is installed. Direct3D 12 tests print `SKIPPED ..._dx12` on
# Linux, as expected; a skipped Vulkan test, or a creative test (they run on
# the window's own backend), means the GPU path was not exercised and fails
# the verification.
#
# Output: <dir>/target/gpu-interop-check/ holds a log per step, system.txt and
# report.txt, packed into kivact-gpu-interop-report-<time>.tar.gz next to the
# clone: send that file back. The exit status is 0 only when every step
# passed and the GPU path was exercised.

set -uo pipefail

dir="$HOME/kivact-gpu-interop-check"
repo="https://github.com/Kashiwade-music/kivact.git"
branch="main"
install_deps=0
x11=0
validation=1

usage() { sed -n '2,/^$/s/^# \{0,1\}//p' "$0"; }

while (($#)); do
  case "$1" in
    --dir) dir="$2"; shift 2 ;;
    --repo) repo="$2"; shift 2 ;;
    --branch) branch="$2"; shift 2 ;;
    --install-deps) install_deps=1; shift ;;
    --x11) x11=1; shift ;;
    --no-validation) validation=0; shift ;;
    --jobs) export CARGO_BUILD_JOBS="$2"; shift 2 ;;
    -h | --help) usage; exit 0 ;;
    *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

say() { printf '\n== %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 2; }
have() { command -v "$1" >/dev/null 2>&1; }

[[ "$(uname -s)" == Linux ]] || die "run this on Linux"

# ----- packages ---------------------------------------------------------------

if ((install_deps)); then
  say "installing runtime packages (sudo)"
  if have apt-get; then
    sudo apt-get update &&
      sudo apt-get install -y git build-essential pkg-config libvulkan1 \
        mesa-vulkan-drivers vulkan-tools vulkan-validationlayers fontconfig \
        fonts-dejavu-core
  elif have dnf; then
    sudo dnf install -y git gcc pkgconf-pkg-config vulkan-loader \
      mesa-vulkan-drivers vulkan-tools vulkan-validation-layers fontconfig \
      dejavu-sans-fonts
  elif have pacman; then
    sudo pacman -S --needed --noconfirm git base-devel vulkan-icd-loader \
      vulkan-intel vulkan-radeon vulkan-tools vulkan-validation-layers \
      fontconfig ttf-dejavu
  else
    die "no apt-get, dnf or pacman: install the Vulkan loader, a Vulkan driver, vulkan-tools and the validation layer by hand"
  fi || die "package installation failed"
fi

# ----- preflight --------------------------------------------------------------

say "preflight"
for tool in git cargo timeout awk grep tar; do
  have "$tool" || die "'$tool' is not installed"
done
have rustup || echo "warning: rustup not found; the repository pins its toolchain in rust-toolchain.toml"
if grep -qi microsoft /proc/version 2>/dev/null; then
  echo "warning: this is WSL; WSLg offers no hardware Vulkan GPU, so the GPU tests will be skipped"
fi
if [[ -z "${WAYLAND_DISPLAY:-}" && -z "${DISPLAY:-}" ]]; then
  die "neither WAYLAND_DISPLAY nor DISPLAY is set: run this from a terminal inside the desktop session"
fi
# (Captured first: under pipefail, `grep -q` closing the pipe early would
# fail ldconfig with SIGPIPE.)
libraries="$(ldconfig -p 2>/dev/null || /sbin/ldconfig -p 2>/dev/null)"
if ! grep -q 'libvulkan\.so\.1' <<<"$libraries"; then
  die "the Vulkan loader (libvulkan.so.1) is missing: install it (or rerun with --install-deps)"
fi
if have vulkaninfo; then
  types="$(vulkaninfo --summary 2>/dev/null | grep -o 'PHYSICAL_DEVICE_TYPE_[A-Z_]*' | sort -u)"
  echo "Vulkan device types: ${types//$'\n'/ }"
  if ! grep -qE 'INTEGRATED_GPU|DISCRETE_GPU' <<<"$types"; then
    echo "warning: no hardware Vulkan device is listed; Arvane refuses software adapters and the GPU tests will be skipped"
  fi
else
  echo "warning: vulkaninfo not found (vulkan-tools); the report will not name the driver"
fi
validation_layer=0
if ((validation)); then
  for d in /usr/share/vulkan/explicit_layer.d /usr/local/share/vulkan/explicit_layer.d /etc/vulkan/explicit_layer.d "$HOME/.local/share/vulkan/explicit_layer.d"; do
    if ls "$d"/*khronos_validation*.json >/dev/null 2>&1 || grep -qs VK_LAYER_KHRONOS_validation "$d"/*.json; then
      validation_layer=1
    fi
  done
  ((validation_layer)) || echo "warning: the Khronos validation layer is not installed; the validation run is skipped"
fi

# ----- clone ------------------------------------------------------------------

say "repository: $repo ($branch) in $dir"
# With an SSH URL, reach the submodule (an https://github.com/ URL in
# .gitmodules) through SSH as well.
git_ssh=()
[[ "$repo" == git@github.com:* ]] && git_ssh=(-c 'url.git@github.com:.insteadOf=https://github.com/')
if ! GIT_TERMINAL_PROMPT=0 timeout 60 git "${git_ssh[@]}" ls-remote --exit-code "$repo" \
  "refs/heads/$branch" >/dev/null 2>&1; then
  echo "warning: $repo cannot be read without credentials (or the branch is missing)."
  echo "  A private repository needs credentials: 'gh auth login' then 'gh auth setup-git',"
  echo "  or an SSH key on GitHub and --repo git@github.com:Kashiwade-music/kivact.git."
  echo "  Trying anyway; git may ask for a user name and a personal access token."
fi
if [[ -d "$dir/.git" ]]; then
  if [[ -n "$(git -C "$dir" status --porcelain --ignore-submodules=dirty)" ]]; then
    die "$dir has local changes; remove the directory or pass another --dir"
  fi
  git "${git_ssh[@]}" -C "$dir" fetch --prune origin "+refs/heads/$branch:refs/remotes/origin/$branch" ||
    die "fetch failed"
  git -C "$dir" checkout -q -B "$branch" "origin/$branch" || die "checkout failed"
else
  [[ -e "$dir" ]] && die "$dir exists and is not a clone; remove it or pass another --dir"
  git "${git_ssh[@]}" clone --filter=blob:none --branch "$branch" "$repo" "$dir" ||
    die "clone failed (a private repository needs credentials: try --repo git@github.com:Kashiwade-music/kivact.git, or 'gh auth login')"
fi
git "${git_ssh[@]}" -C "$dir" submodule update --init --recursive || die "submodule update failed"
cd "$dir" || die "cannot enter $dir"
commit="$(git rev-parse HEAD)"
echo "commit: $commit"

# The pinned toolchain (rust-toolchain.toml) installs on first use.
cargo --version || die "cargo cannot run the pinned toolchain (rust-toolchain.toml)"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$dir/target}"
logs="$CARGO_TARGET_DIR/gpu-interop-check"
rm -rf "$logs"
mkdir -p "$logs"

# ----- system information -----------------------------------------------------

{
  echo "date: $(date -Is)"
  echo "commit: $commit ($branch)"
  echo "kernel: $(uname -srm)"
  [[ -r /etc/os-release ]] && grep -E '^(PRETTY_NAME|VERSION_ID)=' /etc/os-release
  echo "cpu: $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | sed 's/^ //')"
  echo "memory: $(awk '/MemTotal/ { printf "%.1f GiB", $2 / 1048576 }' /proc/meminfo)"
  echo "session: XDG_SESSION_TYPE=${XDG_SESSION_TYPE:-} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-} DISPLAY=${DISPLAY:-}"
  echo "desktop: ${XDG_CURRENT_DESKTOP:-}"
  echo "toolchain: $(cargo --version) / $(rustc --version)"
  have lspci && { echo "pci display devices:"; lspci -nn | grep -Ei 'vga|3d|display'; }
  if have vulkaninfo; then
    echo "--- vulkaninfo --summary"
    vulkaninfo --summary 2>&1
  fi
} >"$logs/system.txt" 2>&1
sed -n '1,12p' "$logs/system.txt"

# ----- steps ------------------------------------------------------------------

declare -a names=() results=()
failed=0

# step NAME COMMAND...: run with the output in NAME.log; never stops the script.
step() {
  local name="$1" status
  shift
  printf '\n== %s\n   %s\n' "$name" "$*"
  "$@" >"$logs/$name.log" 2>&1
  status=$?
  names+=("$name")
  if ((status == 0)); then
    results+=("ok")
    echo "   ok"
  else
    results+=("FAILED (exit $status)")
    failed=1
    echo "   FAILED (exit $status, see $logs/$name.log)"
  fi
  { grep -E '^test result:' "$logs/$name.log" || true; } | awk '
    { for (i = 1; i <= NF; i++) if ($(i) ~ /^(passed|failed|ignored);?$/) s[$(i)] += $(i - 1) }
    END { if (NR) printf "   %d passed, %d failed, %d ignored\n", s["passed;"], s["failed;"], s["ignored;"] }'
  return 0
}

# Through bash: the file mode may lack the execute bit on some checkouts.
lock=(bash dev/tools/window-lock.sh --timeout 1800 --)
producer=(-p gpu-interop-producer --features wgpu)
arvane=(-p arvane --features gpu-interop,testing --test e2e)
public=(-p kivact-public-api-tests --features gpu-interop --test e2e)
creative=(-p kivact-ui-creative --features gpu-interop --test consumer)

# Build first, outside the window lock (its time limit would count the build).
step build-producer cargo test --no-run "${producer[@]}"
step build-arvane cargo test --no-run "${arvane[@]}"
step build-public cargo test --no-run "${public[@]}"
step build-creative cargo test --no-run "${creative[@]}"

# run SUFFIX ENV...: the four test runs, with ENV set (env(1) assignments).
run() {
  local suffix="$1"
  shift
  step "read-back$suffix" env "$@" cargo test "${producer[@]}" -- --nocapture
  step "arvane$suffix" env "$@" "${lock[@]}" cargo test "${arvane[@]}" external -- --nocapture
  step "public$suffix" env "$@" "${lock[@]}" cargo test "${public[@]}" gpu_interop -- --nocapture
  step "creative$suffix" env "$@" "${lock[@]}" cargo test "${creative[@]}" gpu_interop -- --nocapture
}

run ""
if ((x11)) && [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
  # winit 0.30 picks X11 when WAYLAND_DISPLAY is unset.
  run "-x11" -u WAYLAND_DISPLAY -u WAYLAND_SOCKET
fi
if ((validation && validation_layer)); then
  # Every Vulkan instance of the process (wgpu's and the test engine's own)
  # loads the layer, which writes errors to the output when no messenger
  # takes them.
  run "-validation" \
    VK_LOADER_LAYERS_ENABLE='*validation' \
    VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation \
    VK_KHRONOS_VALIDATION_DEBUG_ACTION=VK_DBG_LAYER_ACTION_LOG_MSG \
    VK_KHRONOS_VALIDATION_REPORT_FLAGS=error
fi

# ----- report -----------------------------------------------------------------

{
  echo "GPU texture sharing on Linux: verification report"
  echo
  cat "$logs/system.txt" | sed -n '1,12p'
  echo
  echo "steps:"
  for i in "${!names[@]}"; do
    printf '  %-24s %s\n' "${names[$i]}" "${results[$i]}"
  done
  echo
  echo "skipped tests (Direct3D 12 is expected to skip on Linux):"
  grep -h '^SKIPPED' "$logs"/*.log | sort -u | sed 's/^/  /' || true
} >"$logs/report.txt"

# A skipped test returns normally and is reported `ok` by the harness: it
# names itself on a `SKIPPED <name>: <reason>` line. Direct3D 12 always skips
# on Linux; storage textures skip where the driver cannot share them
# (optional); any other skip means the GPU path was not exercised.
skipped_names="$(grep -h '^SKIPPED' "$logs"/*.log | sed -E 's/^SKIPPED ([A-Za-z0-9_]+).*/\1/' | sort -u)"
unexpected="$(grep -h '^SKIPPED' "$logs"/*.log | grep -v '_dx12' | grep -v 'storage' | sort -u)"
optional="$(grep -h '^SKIPPED' "$logs"/*.log | grep -v '_dx12' | grep 'storage' | sort -u)"
if [[ -n "$optional" ]]; then
  {
    echo
    echo "optional, skipped (the driver cannot share storage textures):"
    sed 's/^/  /' <<<"$optional"
  } >>"$logs/report.txt"
fi
if [[ -n "$unexpected" ]]; then
  failed=1
  {
    echo
    echo "NOT VERIFIED: these tests did not reach the GPU path:"
    sed 's/^/  /' <<<"$unexpected"
  } >>"$logs/report.txt"
fi

# Vulkan tests the harness ran that did not skip ("test
# module::name_on_vulkan ... "; whether they passed is the step's exit
# status: a test's own output can split the harness's line before its
# `ok`). The module prefix is dropped to match the SKIPPED lines, which name
# the function alone.
ran_names="$(grep -hoE '^test [A-Za-z0-9_:]+_vulkan \.\.\.' "$logs"/*.log |
  sed -E 's/^test ([A-Za-z0-9_:]+) \.\.\./\1/; s/.*:://' | sort -u |
  grep -vxF -f <(printf '%s\n' "$skipped_names") || true)"
ran_vulkan=0
[[ -n "$ran_names" ]] && ran_vulkan="$(wc -l <<<"$ran_names")"
{
  echo
  echo "Vulkan tests that ran on the GPU (not skipped): $ran_vulkan"
  [[ -n "$ran_names" ]] && sed 's/^/  /' <<<"$ran_names"
} >>"$logs/report.txt"
if ((ran_vulkan == 0)); then
  failed=1
  echo "NOT VERIFIED: no Vulkan test ran on the GPU" >>"$logs/report.txt"
fi

if ((validation && validation_layer)); then
  # One error is one "Validation Error: [ VUID-... ]" line (the lines after
  # it quote the spec). SPIR-V checks of shader modules
  # (VUID-StandaloneSpirv-*, at vkCreateShaderModule) judge the renderer's
  # shaders as wgpu's shader compiler (naga) writes them, not texture
  # sharing, which creates no shader: they are listed apart and do not fail
  # the verification.
  all_errors="$(cat "$logs"/*-validation.log | grep -E 'Validation Error' || true)"
  shader_errors="$(grep -E 'VUID-StandaloneSpirv-' <<<"$all_errors" || true)"
  other_errors="$(grep -vE 'VUID-StandaloneSpirv-' <<<"$all_errors" | grep . || true)"
  count() { [[ -n "$1" ]] && wc -l <<<"$1" || echo 0; }
  summary() { sed -E 's/ \| MessageID.*//; s/^(.{0,200}).*/\1/' <<<"$1" | sort | uniq -c | sort -rn | sed 's/^/  /'; }
  {
    echo
    echo "validation errors outside shader compilation: $(count "$other_errors")"
    [[ -n "$other_errors" ]] && summary "$other_errors"
    echo "validation errors in shader modules (naga's SPIR-V, not texture sharing): $(count "$shader_errors")"
    [[ -n "$shader_errors" ]] && summary "$shader_errors"
  } >>"$logs/report.txt"
  [[ -z "$other_errors" ]] || failed=1
elif ((validation)); then
  echo $'\nvalidation run: not run (the layer is not installed)' >>"$logs/report.txt"
fi

if ((failed)); then
  echo $'\nRESULT: FAILED or NOT VERIFIED' >>"$logs/report.txt"
else
  echo $'\nRESULT: PASSED (the Vulkan GPU path was exercised)' >>"$logs/report.txt"
fi

archive="$(dirname "$dir")/kivact-gpu-interop-report-$(date +%Y%m%d-%H%M%S).tar.gz"
tar -czf "$archive" -C "$logs" .

say "report"
cat "$logs/report.txt"
echo
echo "logs:    $logs"
echo "archive: $archive  (send this file back)"
exit "$failed"
