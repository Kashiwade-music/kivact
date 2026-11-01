# Real-GUI round: 3 sample apps, workbench (its scenario and the toolbar's
# icon buttons), 2 external-component apps (debug build).
param([int]$Round = 1, [string]$Root = "")
$h = $PSScriptRoot
. (Join-Path $h "harness-paths.ps1")
if (-not $Root) { $Root = Join-Path $KivactEvidence "round" }
New-Item -ItemType Directory -Force $Root | Out-Null
$bin = Join-Path $KivactTarget "debug"
$env:SCENARIO_JSON = Join-Path $KivactRepo "dev\apps\workbench\gui_scenario.json"
$summary = Join-Path $Root "round$Round-summary.txt"
"=== round $Round $(Get-Date -Format o) HEAD $(git -C $KivactRepo rev-parse --short HEAD)" | Out-File -Append -Encoding utf8 $summary
function Count($log, $enc) {
    $p = @(Select-String -Path $log -Pattern "^PASS " -Encoding $enc).Count
    $f = @(Select-String -Path $log -Pattern "^FAIL |^ERROR" -Encoding $enc).Count
    return "passed=$p failed=$f"
}
foreach ($s in @(@("check-taskboard.ps1", "task-board.exe", "taskboard"), @("check-postman.ps1", "mini-postman.exe", "postman"), @("check-music.ps1", "music-player.exe", "music"))) {
    $o = Join-Path $Root "$($s[2])-r$Round"
    Remove-Item -Recurse -Force $o -ErrorAction SilentlyContinue
    & pwsh -STA -NoProfile -File (Join-Path $h $s[0]) -Exe (Join-Path $bin $s[1]) -Out $o -Round $Round | Out-Null
    "$($s[2]): $(Count (Join-Path $o 'log.txt') 'UTF8')" | Tee-Object -Append -FilePath $summary
}
foreach ($s in @(@("workbench-extra", "workbench.exe", "Kivact Workbench", "workbench"), @("workbench-icons", "workbench.exe", "Kivact Workbench", "wbicons"), @("logviewer-extra", "log-viewer.exe", "Kivact Log Viewer", "logviewer"), @("devicemon-extra", "device-monitor.exe", "Kivact Device Monitor", "devicemon"))) {
    $o = Join-Path $Root "$($s[3])-r$Round"
    Remove-Item -Recurse -Force $o -ErrorAction SilentlyContinue
    & pwsh -STA -NoProfile -File (Join-Path $h "run-gui-locked.ps1") -Exe (Join-Path $bin $s[1]) -Title $s[2] -Out $o -Scenario $s[0] | Out-Null
    "$($s[3]): $(Count (Join-Path $o 'log.txt') 'Unicode')" | Tee-Object -Append -FilePath $summary
}
