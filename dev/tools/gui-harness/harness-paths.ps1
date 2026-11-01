# Repo-relative paths for the GUI harness (dot-source this file).
#   $KivactRepo      repository root (three levels above this directory)
#   $KivactTarget    cargo target dir: $env:CARGO_TARGET_DIR, else <repo>\target
#   $KivactEvidence  evidence root: $env:KIVACT_GUI_EVIDENCE, else <repo>\target\gui-evidence
#   $KivactGuiLock   machine-wide "one GUI run at a time" lock file:
#                    $env:KIVACT_GUI_LOCK, else %TEMP%\kivact-gui.lock
$script:KivactRepo = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$script:KivactTarget = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $script:KivactRepo "target" }
$script:KivactEvidence = if ($env:KIVACT_GUI_EVIDENCE) { $env:KIVACT_GUI_EVIDENCE } else { Join-Path $script:KivactRepo "target\gui-evidence" }
$script:KivactGuiLock = if ($env:KIVACT_GUI_LOCK) { $env:KIVACT_GUI_LOCK } else { Join-Path ([System.IO.Path]::GetTempPath()) "kivact-gui.lock" }
# Input guard (the SendInput helpers): keys and presses go only to programs
# built under the target dir, so a run never types into another app that
# took the foreground. Child scripts inherit it.
if (-not $env:KIVACT_GUI_GUARD_ROOT) { $env:KIVACT_GUI_GUARD_ROOT = $script:KivactTarget }
