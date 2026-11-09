#!/usr/bin/env bash
# Collect every KPI row of SLIM_FAST_DESIGN.md 1.4 into dev/bench/results/<label>.md.
# usage: dev/bench/run.sh <label>          (B0 before any change, B1 after)
# Windows (Git Bash; wmic or PowerShell CIM) and Linux (/proc). Opens real
# windows: run it through dev/tools/window-lock.sh, with nothing else drawing
# on the machine. Never wired into CI. Any missing or failed measurement makes
# the report INVALID (exit 1); an INVALID report is not a baseline.
set -uo pipefail
label="${1:?label, e.g. B0}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$root/dev/bench/results/$label.md"
target="${CARGO_TARGET_DIR:-$root/target}"
exe=""; case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) exe=".exe";; esac
mkdir -p "$root/dev/bench/results"
: > "$out"
log() { echo "$*" | tee -a "$out"; }
invalid() { { echo; echo "INVALID: $*"; } | tee -a "$out"; exit 1; }
winpath() { cygpath -w "$1" 2>/dev/null || echo "$1"; }

log "# KPI $label ($(date -u +%Y-%m-%dT%H:%MZ), $(uname -s), commit $(git -C "$root" rev-parse --short HEAD))"

# ---- builds: one package per cargo invocation, so features never unify across
# them; each binary is copied out because later builds reuse its path.
kpi="$target/kpi"; mkdir -p "$kpi"
build() { # <copy as> <bin> <cargo args...>
  local as="$1" bin="$2"; shift 2
  (cd "$root" && CARGO_TARGET_DIR="$target" cargo build --release "$@") >"$kpi/$as.build.log" 2>&1     || invalid "build failed: cargo build --release $* (see $kpi/$as.build.log)"
  cp "$target/release/$bin$exe" "$kpi/$as$exe" || invalid "missing binary $bin after: cargo build --release $*"
}
t0=$(date +%s)
build counter kivact-example-counter -p kivact-example-counter
t1=$(date +%s)
build catalog kivact-ui-basic-catalog -p kivact-ui-basic-catalog
build kv_startup kv_startup -p kivact-bench --bin kv_startup
build kv_stress kv_stress -p kivact-bench --bin kv_stress
build kv_startup_min kv_startup -p kivact-bench --no-default-features --bin kv_startup
build kv_latency kv_latency -p kivact-bench --features latency --bin kv_latency
CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only   build kv_heap kv_heap -p kivact-bench --bin kv_heap
startup="$kpi/kv_startup$exe"; stress="$kpi/kv_stress$exe"; catalog="$kpi/catalog$exe"; counter="$kpi/counter$exe"
log ""
log "release build of kivact-example-counter (cold for its own target dir): $((t1 - t0)) s"

# ---- process metrics by image name (MSYS pids are not the Windows pids)
proc_stat() { # <exe path> -> "ws_kib private_kib cpu_100ns"
  local name; name="$(basename "$1")"
  if [ -n "$exe" ] && command -v wmic >/dev/null 2>&1; then
    wmic process where "name='$name'" get WorkingSetSize,PrivatePageCount,KernelModeTime,UserModeTime /format:list 2>/dev/null \
      | tr -d '\r' | awk -F= '/=/{v[$1]=$2} END{printf "%d %d %d\n", v["WorkingSetSize"]/1024, v["PrivatePageCount"]/1024, v["KernelModeTime"]+v["UserModeTime"]}'
  elif [ -n "$exe" ]; then
    powershell -NoProfile -Command "\$p = Get-CimInstance Win32_Process -Filter \"Name='$name'\" | Select-Object -First 1; '{0} {1} {2}' -f [int64](\$p.WorkingSetSize / 1024), [int64](\$p.PrivatePageCount / 1024), ([int64]\$p.KernelModeTime + [int64]\$p.UserModeTime)" \
      | tr -d '\r'
  else
    local pid; pid="$(pgrep -n -x "${name:0:15}")" || { echo "0 0 0"; return; }
    awk '/VmRSS/{r=$2} /RssAnon/{a=$2} END{printf "%d %d ", r, a}' "/proc/$pid/status"
    awk '{printf "%d\n", ($14+$15)*100000}' "/proc/$pid/stat"
  fi
}
kill_exe() { if [ -n "$exe" ]; then taskkill //IM "$(basename "$1")" //F >/dev/null 2>&1; else pkill -x "$(basename "$1" | cut -c1-15)"; fi; wait 2>/dev/null; }
wait_frame() { for _ in $(seq 1 6000); do grep -q '^F ' "$1" 2>/dev/null && return 0; sleep 0.005; done; return 1; }
quant() { sort -g | awk -v q="$1" '{a[NR]=$1} END{if(NR==0){print "NA"; exit}; i=int(NR*q)+1; if(i>NR)i=NR; print a[i]}'; }
need() { [ -n "$1" ] && [ "$1" != "NA" ] || invalid "$2"; }
renderer_env() { case "$1" in default) unset ARVANE_RENDERER;; *) export ARVANE_RENDERER="$1";; esac; }

# ---- K1 startup and the first (full-window) frame
log ""
log "## K1 startup (5 runs, median) and the first frame"
for r in default cpu; do
  starts=""; rasters=""
  for _ in 1 2 3 4 5; do
    flog="$(mktemp)"; elog="$(mktemp)"; renderer_env "$r"
    KIVACT_FRAME_LOG="$(winpath "$flog")" "$startup" 2>"$elog" &
    wait_frame "$flog" || { kill_exe "$startup"; invalid "kv_startup ($r) presented no frame"; }
    sleep 0.3; kill_exe "$startup"
    main=$(awk '/^T main/{print $3}' "$elog"); line=$(grep -m1 '^F ' "$flog")
    need "$main" "kv_startup ($r) printed no T main"
    starts="$starts $(echo "$line" | awk -v m="$main" '{printf "%.1f", ($2 - m) / 1e6}')"
    rasters="$rasters $(echo "$line" | awk '{printf "%.1f", $5 / 1e6}')"
    rm -f "$flog" "$elog"
  done
  log "- K1 main_to_first_present renderer=$r median_ms=$(echo $starts | tr ' ' '\n' | quant 0.5) samples=[$starts ]"
  log "- K4 first_frame_raster renderer=$r median_ms=$(echo $rasters | tr ' ' '\n' | quant 0.5) samples=[$rasters ]"
done
unset ARVANE_RENDERER
spawns=""
for _ in 1 2 3 4 5; do
  flog="$(mktemp)"; s0=$(date +%s%N)
  KIVACT_FRAME_LOG="$(winpath "$flog")" "$catalog" 2>/dev/null &
  wait_frame "$flog" || { kill_exe "$catalog"; invalid "catalog presented no frame"; }
  sleep 0.3; kill_exe "$catalog"
  spawns="$spawns $(grep -m1 '^F ' "$flog" | awk -v s="$s0" '{printf "%.1f", ($2 - s) / 1e6}')"
  rm -f "$flog"
done
log "- K1 catalog_spawn_to_first_present renderer=default median_ms=$(echo $spawns | tr ' ' '\n' | quant 0.5) samples=[$spawns ] (includes OS process creation)"

# ---- K2/K3 idle memory and CPU
idle() { # <exe> <renderer> <name> [env...]
  local e="$1" r="$2" name="$3"; shift 3
  local flog; flog="$(mktemp)"; renderer_env "$r"
  env "$@" KIVACT_FRAME_LOG="$(winpath "$flog")" "$e" 2>/dev/null &
  wait_frame "$flog" || { kill_exe "$e"; invalid "$name ($r) presented no frame"; }
  sleep 3
  read -r ws pv c0 <<<"$(proc_stat "$e")"; local f0; f0=$(grep -c '^F ' "$flog")
  local w0; w0=$(date +%s%N)
  sleep 10
  read -r _ _ c1 <<<"$(proc_stat "$e")"; local f1; f1=$(grep -c '^F ' "$flog")
  local w1; w1=$(date +%s%N)
  local gpu; gpu=$(grep -m1 'renderer=gpu' "$flog" | sed 's/^# //')
  kill_exe "$e"; rm -f "$flog"
  [ "${ws:-0}" -gt 0 ] || invalid "$name ($r): no process metrics"
  log "- $name renderer=$r ws_kib=$ws private_kib=$pv idle_cpu_ms_per_s=$(awk -v a="$c0" -v b="$c1" -v t=$((w1 - w0)) 'BEGIN{printf "%.3f", (b - a) / 1e4 / (t / 1e9)}') idle_frames=$((f1 - f0)) cpu_ticks_100ns=$((c1 - c0)) ${gpu:+[$gpu]}"
  echo "$ws"
}
log ""
log "## K2/K3 idle (3 s settle, then 10 s sampled)"
for r in default cpu gpu; do idle "$startup" "$r" counter >/dev/null || exit 1; done
for r in default cpu; do
  idle "$catalog" "$r" basic-catalog >/dev/null || exit 1
  idle "$stress" "$r" rows3000-still ROWS=3000 STILL=1 >/dev/null || exit 1
done
log ""
log "## K2 memory per item (kv_stress STILL, CPU renderer, (WS(3000) - WS(0)) / 3000)"
for mode in box text button row; do
  w0=$(idle "$stress" cpu "item-$mode-0" ROWS=0 STILL=1 MODE=$mode | tail -1) || exit 1
  w3=$(idle "$stress" cpu "item-$mode-3000" ROWS=3000 STILL=1 MODE=$mode | tail -1) || exit 1
  log "- K2 per_item mode=$mode kib_per_item=$(awk -v a="$w0" -v b="$w3" 'BEGIN{printf "%.2f", (b - a) / 3000}')"
done

log ""
log "## K2 diagnostics (kv_heap, CPU, live heap bytes; not a KPI row)"
heap="$kpi/kv_heap$exe"
for mode in box text button row; do
  h0=$(ARVANE_RENDERER=cpu MODE=$mode ROWS=0 "$heap" 2>&1 | awk '/^H total_mib/{print $3}')
  out3=$(ARVANE_RENDERER=cpu MODE=$mode ROWS=3000 "$heap" 2>&1)
  h3=$(echo "$out3" | awk '/^H total_mib/{print $3}')
  need "$h0" "kv_heap mode=$mode rows=0 failed"; need "$h3" "kv_heap mode=$mode rows=3000 failed"
  log "- heap mode=$mode live_bytes_per_item=$(awk -v a="$h0" -v b="$h3" 'BEGIN{printf "%.0f", (b - a) * 1048576 / 3000}')"
  echo "$out3" | grep '^H site' | head -10 | sed "s/^/    [$mode] /" | tee -a "$out"
done

# ---- K3/K4 continuous update
log ""
log "## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)"
for rows in 0 3000; do
  for r in default cpu; do
    flog="$(mktemp)"; renderer_env "$r"
    ROWS=$rows KIVACT_FRAME_LOG="$(winpath "$flog")" "$stress" 2>/dev/null &
    wait_frame "$flog" || { kill_exe "$stress"; invalid "kv_stress rows=$rows ($r) presented no frame"; }
    sleep 3
    read -r _ _ c0 <<<"$(proc_stat "$stress")"; l0=$(grep -c '^F ' "$flog"); w0=$(date +%s%N)
    sleep 5
    read -r ws _ c1 <<<"$(proc_stat "$stress")"; l1=$(grep -c '^F ' "$flog"); w1=$(date +%s%N)
    kill_exe "$stress"
    grep '^F ' "$flog" | sed -n "$((l0 + 1)),${l1}p" > "$flog.f"
    n=$(wc -l < "$flog.f")
    [ "$n" -ge 100 ] || invalid "kv_stress rows=$rows ($r): only $n frames in 5 s"
    col() { awk -v c="$1" '{print $c / 1e6}' "$flog.f" | quant "$2"; }
    iv=$(awk 'NR>1{print ($2 - p) / 1e6} {p=$2}' "$flog.f" | quant 0.99)
    period=$(grep -m1 -oE 'frame_period_ns=[0-9]+' "$flog" | cut -d= -f2)
    fps_now=$(awk -v n="$n" -v t=$((w1 - w0)) 'BEGIN{printf "%.1f", n / (t / 1e9)}')
    if [ -n "$period" ] && awk -v f="$fps_now" -v p="$period" 'BEGIN{exit !(f < 0.8 * 1e9 / p)}'; then
      log "  WARNING rows=$rows renderer=$r: $fps_now fps for a $(awk -v p="$period" 'BEGIN{printf "%.1f", 1e9 / p}') Hz display (the OS throttled the window; work times stay valid, fps and interval do not)"
    fi
    log "- rows=$rows renderer=$r frames=$n fps=$fps_now frame_period_ns=${period:-NA} cpu_pct_of_core=$(awk -v a="$c0" -v b="$c1" -v t=$((w1 - w0)) 'BEGIN{printf "%.2f", (b - a) / 1e4 / (t / 1e9) / 10}') ws_kib=$ws work_ms p50=$(col 3 0.5) p95=$(col 3 0.95) p99=$(col 3 0.99) scene_p95=$(col 4 0.95) raster_p95=$(col 5 0.95) interval_p99=$iv"
    rm -f "$flog" "$flog.f"
  done
done
unset ARVANE_RENDERER

# ---- K5 update latency
log ""
log "## K5 update latency (kv_latency, test driver, CPU, mirror off)"
lat="$(KIVACT_TEST_MIRROR=off "$kpi/kv_latency$exe" 2>&1)" || invalid "kv_latency exited with an error: $(echo "$lat" | tail -3)"
lines=$(echo "$lat" | grep -c '^L ')
[ "$lines" -ge 17 ] || invalid "kv_latency printed $lines of 17 scenarios"
echo "$lat" | grep -E '^(L|S) ' | sed 's/^/- /' | tee -a "$out"

# ---- K6 binary size
log ""
log "## K6 release binary size (bytes)"
log "- K6 counter default $(wc -c < "$counter")"
log "- K6 kv_startup default $(wc -c < "$startup")"
log "- K6 kv_startup no-default-features $(wc -c < "$kpi/kv_startup_min$exe")"
log "- K6 basic-catalog default $(wc -c < "$catalog")"
echo "wrote $out"
