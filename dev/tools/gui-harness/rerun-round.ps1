# One full real-GUI round (6 apps, check-round.ps1) while holding the shared GUI
# lock for the whole round. Child scripts see KIVACT_GUI_LOCK_HELD
# and do not take/release the lock themselves.
param([int]$Round = 1, [string]$Root = "")
. (Join-Path $PSScriptRoot "harness-paths.ps1")
if (-not $Root) { $Root = Join-Path $KivactEvidence "round" }
$lock = $KivactGuiLock
New-Item -ItemType Directory -Force $Root | Out-Null
$got = $false
for ($i = 0; $i -lt 240; $i++) {
    try {
        $fs = [System.IO.File]::Open($lock, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write)
        $b = [System.Text.Encoding]::UTF8.GetBytes("rerun-round pid=$PID $(Get-Date -Format o)")
        $fs.Write($b, 0, $b.Length); $fs.Close(); $got = $true; break
    } catch {
        Write-Host "gui.lock busy: $(Get-Content $lock -ErrorAction SilentlyContinue); waiting 30s"
        Start-Sleep -Seconds 30
    }
}
if (-not $got) { throw "could not take gui.lock" }
$env:KIVACT_GUI_LOCK_HELD = "1"
try {
    & pwsh -STA -NoProfile -File (Join-Path $PSScriptRoot "check-round.ps1") -Round $Round -Root $Root
} finally {
    Remove-Item Env:KIVACT_GUI_LOCK_HELD -ErrorAction SilentlyContinue
    Remove-Item -Force $lock -ErrorAction SilentlyContinue
}
Get-Content (Join-Path $Root "round$Round-summary.txt")
