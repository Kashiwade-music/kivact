# App startup times on the real window (release builds of the six apps):
# process start -> window with the title -> first expected UIA element, three
# runs per app. Holds the GUI lock. Host frame and update costs are
# hostbench.ps1's.
param([string]$Out = "")
. (Join-Path $PSScriptRoot "apps-lib.ps1")
if (-not $Out) { $Out = Join-Path $KivactEvidence "perf" }
New-Item -ItemType Directory -Force $Out | Out-Null
$script:out = $Out
$script:log = Join-Path $Out "log.txt"
$rel = Join-Path $KivactTarget "release"
Enter-GuiLock "perf-gui"
try {
    Log "=== perf $(Get-Date -Format o)"
    # Startup: process start -> window with the title -> first expected UIA element.
    $apps = @(
        @("task-board.exe", "Kivact · Task Board", "lane-planned"),
        @("mini-postman.exe", "Kivact · Mini Postman", "postman-status"),
        @("music-player.exe", "Kivact Music Player", "track-title"),
        @("workbench.exe", "Kivact Workbench", "visible-count"),
        @("log-viewer.exe", "Kivact Log Viewer", "timeline"),
        @("device-monitor.exe", "Kivact Device Monitor", "summary")
    )
    $lad = Join-Path $Out "localappdata"; New-Item -ItemType Directory -Force $lad | Out-Null
    $env:LOCALAPPDATA = $lad
    $env:KIVACT_POSTMAN_HISTORY = Join-Path $Out "history.json"
    foreach ($a in $apps) {
        for ($k = 1; $k -le 3; $k++) {
            $sw = [System.Diagnostics.Stopwatch]::StartNew()
            $p = Start-Process -FilePath "$rel\$($a[0])" -PassThru
            $null = $p.Handle
            $script:hwnd = [IntPtr]::Zero
            while ($sw.ElapsedMilliseconds -lt 20000) {
                $p.Refresh()
                if ($p.MainWindowHandle -ne [IntPtr]::Zero -and $p.MainWindowTitle -eq $a[1]) { $script:hwnd = $p.MainWindowHandle; break }
                Start-Sleep -Milliseconds 10
            }
            $tWin = $sw.ElapsedMilliseconds
            $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, $a[2])
            $el = $null
            while (-not $el -and $sw.ElapsedMilliseconds -lt 20000) {
                try { $el = [System.Windows.Automation.AutomationElement]::FromHandle($script:hwnd).FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond) } catch {}
                if (-not $el) { Start-Sleep -Milliseconds 10 }
            }
            $tUia = $sw.ElapsedMilliseconds
            Log ("STARTUP {0} run{1}: window {2} ms, UIA '{3}' {4} ms" -f $a[0], $k, $tWin, $a[2], $tUia)
            $p.CloseMainWindow() | Out-Null
            if (-not $p.WaitForExit(8000)) { Stop-Process -Id $p.Id -Force }
            Start-Sleep -Milliseconds 500
        }
    }
} catch { Log "ERROR $_" } finally { Exit-GuiLock }
