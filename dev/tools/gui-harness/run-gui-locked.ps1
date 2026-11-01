# Run gui.ps1 with a scenario while holding the shared GUI lock.
param([string]$Exe, [string]$Title, [string]$Out, [string]$Scenario, [string]$AppArguments = "")
. (Join-Path $PSScriptRoot "harness-paths.ps1")
$log = Join-Path $Out "log.txt"
$lock = $KivactGuiLock
$ownsLock = $false
try {
    if (-not $env:KIVACT_GUI_LOCK_HELD) {
        for ($i = 0; $i -lt 120 -and -not $ownsLock; $i++) {
            try {
                $fs = [System.IO.File]::Open($lock, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write)
            } catch [System.IO.IOException] {
                Write-Host "gui.lock busy; waiting 30s"
                Start-Sleep -Seconds 30
                continue
            }
            $ownsLock = $true
            try {
                $b = [System.Text.Encoding]::UTF8.GetBytes("run-gui-locked $Scenario pid=$PID $(Get-Date -Format o)")
                $fs.Write($b, 0, $b.Length)
            } finally { $fs.Dispose() }
        }
        if (-not $ownsLock) { throw "GUI lock could not be acquired: $lock" }
    }
    $firstLine = if (Test-Path -LiteralPath $log) { @(Get-Content -LiteralPath $log -Encoding Unicode).Count } else { 0 }
    if ($AppArguments) {
        & pwsh -STA -NoProfile -File (Join-Path $PSScriptRoot "gui.ps1") -Exe $Exe -Title $Title -Out $Out -Scenario $Scenario -AppArguments $AppArguments
    } else {
        & pwsh -STA -NoProfile -File (Join-Path $PSScriptRoot "gui.ps1") -Exe $Exe -Title $Title -Out $Out -Scenario $Scenario
    }
    $code = $LASTEXITCODE
    $currentRun = @(Get-Content -LiteralPath $log -Encoding Unicode | Select-Object -Skip $firstLine)
    $pass = @($currentRun | Where-Object { $_ -match "^PASS " }).Count
    $fail = @($currentRun | Where-Object { $_ -match "^FAIL |^ERROR" }).Count
    "RESULT $Scenario passed=$pass failed=$fail exit=$code" | Tee-Object -FilePath (Join-Path $Out "result.txt")
} finally { if ($ownsLock) { Remove-Item -LiteralPath $lock -Force -ErrorAction SilentlyContinue } }
exit $code
