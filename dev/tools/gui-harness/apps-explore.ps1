# Launch an app, dump its UIA tree and take a screenshot (layout exploration).
param([string]$Exe, [string]$Title, [string]$Out, [string]$LocalAppData = "")
. (Join-Path $PSScriptRoot "apps-lib.ps1")
New-Item -ItemType Directory -Force $Out | Out-Null
$script:out = $Out
$script:log = Join-Path $Out "explore-log.txt"
if ($LocalAppData) { New-Item -ItemType Directory -Force $LocalAppData | Out-Null; $env:LOCALAPPDATA = $LocalAppData }
Enter-GuiLock "apps-explore"
try {
    Start-App $Exe $Title
    Start-Sleep -Milliseconds 1500
    Dump "initial"
    Shot "explore" | Out-Null
    $code = Stop-App
} catch {
    Log "ERROR $_"
    Kill-App
} finally {
    Exit-GuiLock
}
