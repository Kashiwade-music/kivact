# Host benchmark runner (PowerShell 7 in STA mode). Builds the hostbench
# binary (dev/benchmarks/hostbench), runs every scenario -Runs times per host
# configuration (configurations interleaved), and writes raw results and a
# summary of the medians to <evidence>\hostbench\<timestamp>\ (summary.md,
# raw.jsonl, the frame logs and stderr). Holds the GUI lock.
#
#   pwsh -STA -NoProfile -File dev\tools\gui-harness\hostbench.ps1 [-Runs 3] [-Hosts arvane,arvane-cpu] [-Scenarios text,idle] [-NoBuild]
#
# Hosts: arvane (the Arvane host's default: the GPU, started with the window)
# and arvane-cpu (the same binary with ARVANE_RENDERER=cpu).
#
# The app measures presentation from the host's frame log (KIVACT_FRAME_LOG);
# this script adds what only another process can see: process CPU time and
# memory sampled at the app's MARK lines, process creation -> first frame
# (startup), and the window resizes of the resize scenario (SetWindowPos; the
# public API cannot resize). The bench window is brought to the foreground
# (the conditions of the recorded tables). No UI Automation client is
# attached (with one, the host keeps its accessibility tree up to date).
param(
    [int]$Runs = 3,
    [string[]]$Hosts = @("arvane", "arvane-cpu"),
    [string[]]$Scenarios = @("startup", "scroll_virtual", "scroll_column", "animation", "text", "churn_1k", "churn_10k", "churn_50k", "relayout_10k", "relayout_50k", "resize", "idle", "drag_backdrop", "drag_slider", "hover_tooltip"),
    [switch]$NoBuild,
    [string]$Out = ""
)
. (Join-Path $PSScriptRoot "apps-lib.ps1")
# `-File` passes "a,b" as one string.
$Hosts = @($Hosts | ForEach-Object { $_ -split "," } | Where-Object { $_ })
$Scenarios = @($Scenarios | ForEach-Object { $_ -split "," } | Where-Object { $_ })
if (-not ("HB" -as [type])) {
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class HB {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string cls, string title);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  // The bench window of process `pid` (its title is "hostbench <scenario>").
  // Process.MainWindowHandle may pick a small helper window instead.
  public static IntPtr Window(int pid, string title) {
    IntPtr h = FindWindowW(null, title);
    uint owner;
    if (h == IntPtr.Zero) return h;
    GetWindowThreadProcessId(h, out owner);
    return owner == (uint)pid ? h : IntPtr.Zero;
  }
}
"@
}
if (-not $Out) { $Out = Join-Path $KivactEvidence ("hostbench\" + (Get-Date -Format "yyyyMMdd-HHmmss")) }
New-Item -ItemType Directory -Force $Out | Out-Null
$script:out = $Out
$script:log = Join-Path $Out "log.txt"
$bin = Join-Path $KivactTarget "hostbench-bin"

# ---- build ---------------------------------------------------------------------
# Every host configuration runs the same binary; it is copied out so a later
# build in the target dir does not replace it during the runs.
if (-not $NoBuild) {
    Log "build"
    Push-Location $KivactRepo
    # cargo reports progress on stderr, which "Stop" would turn into an error.
    $eap = $ErrorActionPreference; $ErrorActionPreference = "Continue"
    try { & cargo build --release -p kivact-hostbench 2>&1 | ForEach-Object { "$_" } | Out-Host }
    finally { $ErrorActionPreference = $eap; Pop-Location }
    if ($LASTEXITCODE -ne 0) { throw "build failed" }
    New-Item -ItemType Directory -Force $bin | Out-Null
    Copy-Item -Force (Join-Path $KivactTarget "release\hostbench.exe") (Join-Path $bin "hostbench.exe")
}

# ---- one run -------------------------------------------------------------------
function Set-Prop($o, $name, $value) { $o | Add-Member -NotePropertyName $name -NotePropertyValue $value -Force }
function UnixNs([DateTime]$t) { return ([long]$t.ToUniversalTime().Ticks - 621355968000000000) * 100 }

function Sample($p, [IntPtr]$hwnd) {
    $p.Refresh()
    return [ordered]@{
        cpu_ms = $p.TotalProcessorTime.TotalMilliseconds
        ws_mb = $p.WorkingSet64 / 1MB
        peak_ws_mb = $p.PeakWorkingSet64 / 1MB
        private_mb = $p.PrivateMemorySize64 / 1MB
        # PeakPagefileUsage: the peak private commit.
        peak_private_mb = $p.PeakPagedMemorySize64 / 1MB
        foreground = ($hwnd -ne [IntPtr]::Zero -and [HB]::GetForegroundWindow() -eq $hwnd)
    }
}

function Activate([IntPtr]$hwnd) {
    if ([HB]::SetForegroundWindow($hwnd) -and [HB]::GetForegroundWindow() -eq $hwnd) { return $true }
    # The foreground lock: holding Alt lets another process take the foreground.
    [HB]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
    [HB]::SetForegroundWindow($hwnd) | Out-Null
    [HB]::BringWindowToTop($hwnd) | Out-Null
    [HB]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    return ([HB]::GetForegroundWindow() -eq $hwnd)
}

# 40 resizes between two outer sizes, 50 ms apart. SetWindowPos returns once
# the window procedure has handled the size messages.
function Resize([IntPtr]$hwnd) {
    $times = @()
    for ($i = 0; $i -lt 40; $i++) {
        if ($i % 2 -eq 0) { $w = 1280; $h = 860 } else { $w = 976; $h = 759 }
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        [HB]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, $w, $h, 0x0002 -bor 0x0004 -bor 0x0010) | Out-Null
        $times += $sw.Elapsed.TotalMilliseconds
        Start-Sleep -Milliseconds 50
    }
    return $times
}

# Screenshot of the window (screen pixels, for comparing text rendering).
function Capture([IntPtr]$hwnd, $path) {
    $r = New-Object AW+RECT
    [AW]::GetWindowRect($hwnd, [ref]$r) | Out-Null
    $bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
    $bmp.Save($path)
    $g.Dispose(); $bmp.Dispose()
}

function Invoke-Bench($h, $s, $k) {
    $exe = Join-Path $bin "hostbench.exe"
    $frames = Join-Path $Out "frames-$h-$s-$k.txt"
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.Arguments = $s
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.EnvironmentVariables["KIVACT_FRAME_LOG"] = $frames
    if ($h -eq "arvane-cpu") { $psi.EnvironmentVariables["ARVANE_RENDERER"] = "cpu" }
    else { $psi.EnvironmentVariables.Remove("ARVANE_RENDERER") }
    $p = [System.Diagnostics.Process]::Start($psi)
    $null = $p.Handle
    $errTask = $p.StandardError.ReadToEndAsync()
    $marks = [ordered]@{}
    $result = $null
    $resizeMs = $null
    $activated = $false
    $tries = 0
    $hwnd = [IntPtr]::Zero
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $pending = $p.StandardOutput.ReadLineAsync()
    while ($true) {
        if (-not $activated -and $tries -lt 20 -and -not $p.HasExited) {
            $hwnd = [HB]::Window($p.Id, "hostbench $s")
            if ($hwnd -ne [IntPtr]::Zero) { $activated = Activate $hwnd; $tries++ }
        }
        if ($pending.Wait(15)) {
            $line = $pending.Result
            if ($null -eq $line) { break }
            if ($line -match '^MARK (\S+) (\d+)') {
                $m = Sample $p $hwnd
                $m["unix_ns"] = [long]$matches[2]
                $marks[$matches[1]] = $m
                if ($matches[1] -eq "ready" -and $hwnd -ne [IntPtr]::Zero) { $resizeMs = Resize $hwnd }
                if ($matches[1] -eq "end" -and $k -eq 1 -and ($s -eq "startup" -or $s -eq "text") -and $hwnd -ne [IntPtr]::Zero) {
                    try { Capture $hwnd (Join-Path $Out "shot-$h-$s.png") } catch { Log "capture failed: $_" }
                }
            } elseif ($line -match '^RESULT (.*)$') {
                $result = $matches[1]
            }
            $pending = $p.StandardOutput.ReadLineAsync()
        }
        if ($sw.Elapsed.TotalSeconds -gt 240) { Log "TIMEOUT $h $s run$k"; Stop-Process -Id $p.Id -Force; break }
    }
    if (-not $p.WaitForExit(15000)) { Stop-Process -Id $p.Id -Force }
    $errTask.Result | Out-File -FilePath (Join-Path $Out "stderr-$h-$s-$k.txt") -Encoding utf8
    if (-not $result) { Log "NO RESULT $h $s run$k exit=$($p.ExitCode)"; return $null }
    $o = $result | ConvertFrom-Json
    Set-Prop $o "run" $k
    Set-Prop $o "activated" $activated
    # Phase CPU time from the marks (ms of CPU, and % of one core).
    foreach ($pair in @(@("phase_begin", "phase_end", "phase"), @("ready", "phase_end", "phase"), @("mount_begin", "mount_end", "mount"),
                        @("update_begin", "update_end", "update"), @("idle_begin", "idle_end", "idle"), @("caret_begin", "caret_end", "caret"))) {
        $a = $marks[$pair[0]]; $b = $marks[$pair[1]]
        if ($a -and $b) {
            $cpu = $b.cpu_ms - $a.cpu_ms
            $wall = ($b.unix_ns - $a.unix_ns) / 1e6
            Set-Prop $o ("cpu_" + $pair[2] + "_ms") ([math]::Round($cpu, 1))
            if ($wall -gt 0) { Set-Prop $o ("cpu_" + $pair[2] + "_pct") ([math]::Round(100 * $cpu / $wall, 1)) }
            Set-Prop $o ("foreground_" + $pair[2]) $b.foreground
        }
    }
    $end = $marks["end"]
    if ($end) {
        foreach ($key in @("cpu_ms", "ws_mb", "peak_ws_mb", "private_mb", "peak_private_mb")) {
            Set-Prop $o ("end_" + $key) ([math]::Round($end[$key], 1))
        }
    }
    if ($result -match '"first_present_unix_ns":(\d+)') {
        Set-Prop $o "startup_ms" ([math]::Round(([long]$matches[1] - (UnixNs $p.StartTime)) / 1e6, 2))
    }
    if ($resizeMs) {
        $sorted = $resizeMs | Sort-Object
        Set-Prop $o "resize_calls" $sorted.Count
        Set-Prop $o "resize_call_p50_ms" ([math]::Round($sorted[[int][math]::Floor(($sorted.Count - 1) / 2)], 2))
        Set-Prop $o "resize_call_max_ms" ([math]::Round($sorted[-1], 2))
    }
    return $o
}

# ---- runs ----------------------------------------------------------------------
$machine = @{
    gpu = ((Get-CimInstance Win32_VideoController | ForEach-Object { "$($_.Name) ($($_.CurrentRefreshRate) Hz)" }) -join "; ")
    cpu = ((Get-CimInstance Win32_Processor | ForEach-Object { $_.Name.Trim() }) -join "; ")
    os = (Get-CimInstance Win32_OperatingSystem).Caption
}
Log "=== hostbench $(Get-Date -Format o) runs=$Runs hosts=$($Hosts -join ',')"
Log "gpu: $($machine.gpu) | cpu: $($machine.cpu)"
$raw = Join-Path $Out "raw.jsonl"
$all = @()
Enter-GuiLock "hostbench"
try {
    for ($k = 1; $k -le $Runs; $k++) {
        foreach ($s in $Scenarios) {
            foreach ($h in $Hosts) {
                $o = Invoke-Bench $h $s $k
                if ($o) {
                    $line = $o | ConvertTo-Json -Compress -Depth 5
                    $line | Out-File -FilePath $raw -Append -Encoding utf8
                    Log "$h $s run$k ok"
                    $all += $o
                }
                Start-Sleep -Milliseconds 300
            }
        }
    }
} catch { Log "ERROR $_" } finally { Exit-GuiLock }

# ---- summary (medians) ---------------------------------------------------------
function Median($values) {
    $v = @($values | Where-Object { $null -ne $_ } | Sort-Object)
    if ($v.Count -eq 0) { return $null }
    if ($v.Count % 2 -eq 1) { return $v[($v.Count - 1) / 2] }
    return ($v[$v.Count / 2 - 1] + $v[$v.Count / 2]) / 2
}
function Field($o, $path) {
    $x = $o
    foreach ($part in $path.Split(".")) { if ($null -eq $x) { return $null }; $x = $x.$part }
    return $x
}
$frameRows = @(
    @("fps", "fps"), @("interval p50 ms", "interval_ms.p50"), @("interval p95 ms", "interval_ms.p95"),
    @("interval p99 ms", "interval_ms.p99"), @("interval max ms", "interval_ms.max"),
    @("host work p50 ms", "work_ms.p50"), @("host work p95 ms", "work_ms.p95"), @("raster p50 ms", "raster_ms.p50"),
    @("CPU % of a core", "cpu_phase_pct"), @("peak working set MB", "end_peak_ws_mb"), @("peak private MB", "end_peak_private_mb")
)
$table = [ordered]@{
    "startup" = @(@("process start -> first frame ms", "startup_ms"), @("main -> root setup ms", "main_to_setup_ms"),
                  @("root setup -> first frame ms", "setup_to_first_present_ms"), @("first frame host work ms", "first_work_ms"),
                  @("peak working set MB", "end_peak_ws_mb"), @("peak private MB", "end_peak_private_mb"))
    "scroll_virtual" = $frameRows
    "scroll_column" = $frameRows
    "animation" = $frameRows
    "text" = $frameRows
    "drag_backdrop" = $frameRows
    "drag_slider" = $frameRows
    "hover_tooltip" = $frameRows
    "churn_1k" = @(@("mount -> frame ms", "mount_ms"), @("mount frame work ms", "mount_work_ms"), @("mount CPU ms", "cpu_mount_ms"),
                   @("update -> frame p50 ms", "update_latency_ms.p50"), @("update -> frame p95 ms", "update_latency_ms.p95"),
                   @("update frame work p50 ms", "update_work_ms.p50"), @("peak working set MB", "end_peak_ws_mb"), @("peak private MB", "end_peak_private_mb"))
    "resize" = @(@("frames in 4 s", "frames"), @("host work p50 ms", "work_ms.p50"), @("host work p95 ms", "work_ms.p95"), @("host work max ms", "work_ms.max"),
                 @("interval p95 ms", "interval_ms.p95"), @("SetWindowPos p50 ms", "resize_call_p50_ms"), @("SetWindowPos max ms", "resize_call_max_ms"),
                 @("CPU % of a core", "cpu_phase_pct"))
    "idle" = @(@("CPU ms over 3 s", "cpu_idle_ms"), @("CPU ms over 3 s, focused input", "cpu_caret_ms"), @("frames over 3 s", "frames_idle"),
               @("frames over 3 s, focused input", "frames_caret"), @("working set MB", "end_ws_mb"), @("private MB", "end_private_mb"))
}
$table["churn_10k"] = $table["churn_1k"]
$table["churn_50k"] = $table["churn_1k"]
$table["relayout_10k"] = $table["churn_1k"]
$table["relayout_50k"] = $table["churn_1k"]
$md = @("# hostbench $(Get-Date -Format 'yyyy-MM-dd HH:mm')", "",
        "- GPU: $($machine.gpu)", "- CPU: $($machine.cpu)", "- OS: $($machine.os)",
        "- runs: $Runs per host and scenario; each cell is the median of the runs",
        "- frame times come from the host's frame log (layout + scene + raster + present on the UI thread)",
        "- process CPU time is sampled at the app's marks; Windows updates it in 15.6 ms steps", "")
foreach ($s in $Scenarios) {
    $rows = $table[$s]
    if (-not $rows) { continue }
    $md += "## $s", "", ("| metric | " + ($Hosts -join " | ") + " |"), ("|---|" + (($Hosts | ForEach-Object { "---:" }) -join "|") + "|")
    foreach ($r in $rows) {
        $cells = foreach ($h in $Hosts) {
            $v = Median (@($all | Where-Object { $_.host -eq $h -and $_.scenario -eq $s } | ForEach-Object { Field $_ $r[1] }))
            if ($null -eq $v) { "-" } else { "{0:0.##}" -f [double]$v }
        }
        $md += "| $($r[0]) | " + ($cells -join " | ") + " |"
    }
    $renderers = ($all | Where-Object { $_.scenario -eq $s } | ForEach-Object { "$($_.host)=$($_.renderer)" } | Sort-Object -Unique) -join ", "
    $inactive = @($all | Where-Object { $_.scenario -eq $s -and $_.activated -ne $true }).Count
    $a11y = @($all | Where-Object { $_.scenario -eq $s -and $_.accessibility -eq "on" } | ForEach-Object { "$($_.host) run$($_.run)" }) -join ", "
    if (-not $a11y) { $a11y = "none" }
    $md += "", "renderers: $renderers; runs whose window could not be brought to the foreground: $inactive", "",
           "runs with a UI Automation client attached (the host keeps its accessibility tree): $a11y", ""
}
$md | Out-File -FilePath (Join-Path $Out "summary.md") -Encoding utf8
Log "summary: $(Join-Path $Out 'summary.md')"
