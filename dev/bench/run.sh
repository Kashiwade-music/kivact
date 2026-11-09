#!/usr/bin/env bash
# Collect the six KPIs of SLIM_FAST_DESIGN.md chapter 1 into one Markdown file.
# usage: dev/bench/run.sh <label>          (e.g. B0 before any change, B1 after)
# Windows (Git Bash, wmic) and Linux (/proc). Opens real windows: run it
# through dev/tools/window-lock.sh, with nothing else drawing on the machine.
# Never wired into CI.
set -u
label="${1:?label, e.g. B0}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$root/dev/bench/results/$label.md"
target="${CARGO_TARGET_DIR:-$root/target}"
bin="$target/release"
exe=""; case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) exe=".exe";; esac
mkdir -p "$root/dev/bench/results"
log() { echo "$*" | tee -a "$out"; }
: > "$out"
log "# KPI $label ($(date -u +%Y-%m-%dT%H:%MZ), $(uname -s), commit $(git -C "$root" rev-parse --short HEAD))"

echo "building release probes and examples..."
(cd "$root" && cargo build --release -p kivact-bench -p kivact-example-counter -p kivact-ui-basic-catalog) >/dev/null 2>&1 \
  || { echo "release build failed"; exit 1; }
(cd "$root" && CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_TARGET_DIR="$target/bench-heap" cargo build --release -p kivact-bench --bin kv_heap) >/dev/null 2>&1

# Process metrics by image name (MSYS pids are not the Windows pids).
proc_stat() { # <exe path> -> "ws_kib private_kib cpu_100ns"
  local name; name="$(basename "$1")"
  if [ -n "$exe" ] && command -v wmic >/dev/null 2>&1; then
    wmic process where "name='$name'" get WorkingSetSize,PrivatePageCount,KernelModeTime,UserModeTime /format:list 2>/dev/null \
      | tr -d '\r' | awk -F= '/=/{v[$1]=$2} END{printf "%d %d %d\n", v["WorkingSetSize"]/1024, v["PrivatePageCount"]/1024, v["KernelModeTime"]+v["UserModeTime"]}'
  elif [ -n "$exe" ]; then
    # Windows without wmic (removed by default from Windows 11 24H2): the same Win32_Process fields through CIM.
    powershell -NoProfile -Command "\$p = Get-CimInstance Win32_Process -Filter \"Name='$name'\" | Select-Object -First 1; '{0} {1} {2}' -f [int64](\$p.WorkingSetSize / 1024), [int64](\$p.PrivatePageCount / 1024), ([int64]\$p.KernelModeTime + [int64]\$p.UserModeTime)" \
      | tr -d '\r'
  else
    local pid; pid="$(pgrep -n -x "$name")"
    awk '/VmRSS/{r=$2} /RssAnon/{a=$2} END{printf "%d %d ", r, a}' "/proc/$pid/status"
    awk '{printf "%d\n", ($14+$15)*100000}' "/proc/$pid/stat"
  fi
}
kill_exe() { if [ -n "$exe" ]; then taskkill //IM "$(basename "$1")" //F >/dev/null 2>&1; else pkill -x "$(basename "$1")"; fi; }
wait_frame() { for _ in $(seq 1 6000); do grep -q '^F ' "$1" 2>/dev/null && return 0; sleep 0.005; done; return 1; }
pct() { sort -n | awk -v q="$1" '{a[NR]=$1} END{if(NR==0){print 0}else{i=int(NR*q)+1; if(i>NR)i=NR; print a[i]}}'; }

log ""
log "## K1 startup (main -> first presented frame, 800x600, median of 5)"
for renderer in default cpu; do
  samples=""
  for _ in 1 2 3 4 5; do
    flog="$(mktemp)"; elog="$(mktemp)"
    if [ "$renderer" = cpu ]; then export ARVANE_RENDERER=cpu; else unset ARVANE_RENDERER; fi
    KIVACT_FRAME_LOG="$(cygpath -w "$flog" 2>/dev/null || echo "$flog")" "$bin/kv_startup$exe" 2>"$elog" &
    wait_frame "$flog"; sleep 0.3; kill_exe "$bin/kv_startup$exe"; wait 2>/dev/null
    main=$(awk '/^T main/{print $3}' "$elog"); end=$(awk '/^F /{print $2; exit}' "$flog")
    samples="$samples $(( (end - main) / 1000000 ))"
    rm -f "$flog" "$elog"
  done
  log "- renderer=$renderer startup_ms median=$(echo $samples | tr ' ' '\n' | pct 0.5) samples=[$samples ]"
done
unset ARVANE_RENDERER

log ""
log "## K2/K3 memory and CPU (idle 3 s, then 10 s sampled)"
idle() { # <exe> <renderer> <label> [env...]
  local e="$1" r="$2" name="$3"; shift 3
  flog="$(mktemp)"
  if [ "$r" = cpu ]; then export ARVANE_RENDERER=cpu; else unset ARVANE_RENDERER; fi
  env "$@" KIVACT_FRAME_LOG="$(cygpath -w "$flog" 2>/dev/null || echo "$flog")" "$e" 2>/dev/null &
  wait_frame "$flog"; sleep 3
  read -r ws pv c0 <<<"$(proc_stat "$e")"; f0=$(grep -c '^F ' "$flog")
  sleep 10
  read -r _ _ c1 <<<"$(proc_stat "$e")"; f1=$(grep -c '^F ' "$flog")
  kill_exe "$e"; wait 2>/dev/null
  log "- $name renderer=$r ws_mib=$((ws/1024)) private_mib=$((pv/1024)) idle_cpu_ms_per_s=$(( (c1-c0) / 100000 )) idle_frames=$((f1-f0))"
  rm -f "$flog"
}
for r in default cpu; do
  idle "$bin/kv_startup$exe" "$r" counter
  idle "$bin/kivact-ui-basic-catalog$exe" "$r" basic-catalog
  idle "$bin/kv_stress$exe" "$r" rows3000-still ROWS=3000 STILL=1
done

log ""
log "## K2 memory per item (kv_heap, 3000 items, live bytes / item of the top sites)"
for mode in box text button row; do
  ARVANE_RENDERER=cpu MODE=$mode ROWS=3000 "$target/bench-heap/release/kv_heap$exe" 2>&1 | grep '^H ' | head -12 | sed "s/^/  [$mode] /" | tee -a "$out"
done

log ""
log "## K3/K4 frame time and CPU under continuous update (kv_stress, 5 s)"
for rows in 0 3000; do
  for r in default cpu; do
    flog="$(mktemp)"
    if [ "$r" = cpu ]; then export ARVANE_RENDERER=cpu; else unset ARVANE_RENDERER; fi
    ROWS=$rows KIVACT_FRAME_LOG="$(cygpath -w "$flog" 2>/dev/null || echo "$flog")" "$bin/kv_stress$exe" 2>/dev/null &
    wait_frame "$flog"; sleep 3
    read -r _ _ c0 <<<"$(proc_stat "$bin/kv_stress$exe")"; l0=$(grep -c '^F ' "$flog")
    sleep 5
    read -r ws _ c1 <<<"$(proc_stat "$bin/kv_stress$exe")"; l1=$(grep -c '^F ' "$flog")
    kill_exe "$bin/kv_stress$exe"; wait 2>/dev/null
    grep '^F ' "$flog" | sed -n "$((l0+1)),${l1}p" > "$flog.f"
    w() { awk -v c="$1" '{print $c/1e6}' "$flog.f" | pct "$2"; }
    iv=$(awk 'NR>1{print ($2-p)/1e6} {p=$2}' "$flog.f" | pct 0.99)
    log "- rows=$rows renderer=$r fps=$(( (l1-l0) / 5 )) cpu_pct_of_core=$(( (c1-c0) / 500000 )) ws_mib=$((ws/1024)) work_ms p50=$(w 3 0.5) p95=$(w 3 0.95) p99=$(w 3 0.99) scene_p95=$(w 4 0.95) raster_p95=$(w 5 0.95) interval_p99=$iv"
    rm -f "$flog" "$flog.f"
  done
done
unset ARVANE_RENDERER

log ""
log "## K5 update latency (kv_latency, test driver, CPU, mirror off)"
KIVACT_TEST_MIRROR=off "$bin/kv_latency$exe" 2>&1 | grep -E '^(L|S) ' | sed 's/^/- /' | tee -a "$out"

log ""
log "## K6 release binary size (bytes)"
for b in kivact-example-counter kivact-ui-basic-catalog; do
  log "- $b $(wc -c < "$bin/$b$exe")"
done
echo "wrote $out"
