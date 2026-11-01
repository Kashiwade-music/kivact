# Catalogue comparisons use the existing shared lock and input guard.
# Only gui.ps1's newly started process/window is resized or operated.
. (Join-Path $PSScriptRoot "harness-paths.ps1")
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class CatalogueWindow {
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
}
"@
if (-not $env:KIVACT_GUI_APPROVED_UNTIL) { throw "Set KIVACT_GUI_APPROVED_UNTIL to the end of an explicitly approved desktop-use window" }
$approvedUntil = [DateTimeOffset]::Parse($env:KIVACT_GUI_APPROVED_UNTIL).UtcDateTime
$scene = $env:KIVACT_CATALOG_SCENE
$chapters = switch ($scene) {
    "basic" { @("overview", "tokens", "buttons", "forms", "selection", "display", "table", "feedback", "overlays", "navigation", "messaging", "schedule", "cards") }
    "game" { @("foundations", "buttons", "inputs", "selection", "navigation", "display", "table", "feedback", "overlays", "layout") }
    "creative" { @("buttons", "numeric", "selection", "indicators", "transport", "knobs", "meters", "mixer", "properties", "docking", "outliner", "media", "layers", "timeline", "animation", "nodes", "grading", "picker", "viewport", "menus", "dialogs", "jobs") }
    default { throw "KIVACT_CATALOG_SCENE must select basic/game/creative" }
}
if ([DateTime]::UtcNow -ge $approvedUntil) {
    throw "The user's current desktop-use window has ended"
}
$width = if ($scene -eq "creative") { 1861 } else { 1846 }
$height = 969
$outer = New-Object W+RECT
$client = New-Object CatalogueWindow+RECT
[W]::GetWindowRect($hwnd, [ref]$outer) | Out-Null
[CatalogueWindow]::GetClientRect($hwnd, [ref]$client) | Out-Null
$borderWidth = ($outer.R - $outer.L) - ($client.R - $client.L)
$borderHeight = ($outer.B - $outer.T) - ($client.B - $client.T)
[W]::SetWindowPos($hwnd, [IntPtr]::Zero, 18, 18, ($width + $borderWidth), ($height + $borderHeight), 0x0004) | Out-Null
Start-Sleep -Milliseconds 800
[CatalogueWindow]::GetClientRect($hwnd, [ref]$client) | Out-Null
Check "native client matches source content width and CSS height" (($client.R - $client.L) -eq $width -and ($client.B - $client.T) -eq $height)
$dpi = [CatalogueWindow]::GetDpiForWindow($hwnd)
Check "source comparison scale is 1" ($dpi -eq 96)
@{ scene = $scene; width = $client.R; height = $client.B; dpi = $dpi; pid = $p.Id; utc = [DateTime]::UtcNow.ToString("o"); codeCommit = (git -C $KivactRepo rev-parse HEAD); binarySha256 = (FileSha256 $Exe); captureMethod = "native client CopyFromScreen"; sourceCssViewport = @(1861,969); sourceContentWidth = $width; sourceImage = $(if ($scene -eq "creative") { @(1861,969) } else { @(1846,961) }); reducedMotion = "OS setting preserved; public driver records explicit condition" } |
    ConvertTo-Json | Out-File -Encoding UTF8 (Join-Path $Out "conditions.json")

function CatalogueShot([string]$name) {
    $origin = New-Object CatalogueWindow+POINT
    [CatalogueWindow]::ClientToScreen($hwnd, [ref]$origin) | Out-Null
    $image = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($image)
    try {
        $graphics.CopyFromScreen($origin.X, $origin.Y, 0, 0, $image.Size)
        $path = Join-Path $Out "$name.png"
        $image.Save($path)
        Log "client screenshot $path ${width}x${height}"
    } finally { $graphics.Dispose(); $image.Dispose() }
}
CatalogueShot "$scene-initial"
if ($scene -eq "creative") {
    $origin = New-Object CatalogueWindow+POINT
    [CatalogueWindow]::ClientToScreen($hwnd, [ref]$origin) | Out-Null
    $names = @("Tool Pointer", "Tool Move", "Tool Rotate", "Tool Scale", "Tool Scissors", "Tool Pencil")
    $tools = @(All | Where-Object {
        $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button -and
        $_.Current.Name -in $names -and $_.Current.BoundingRectangle.Y -ge ($origin.Y + 42) -and
        $_.Current.BoundingRectangle.Bottom -le ($origin.Y + 88)
    })
    $undo = (All | Where-Object { $_.Current.Name -eq "Undo" -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button } | Select-Object -First 1)
    $aligned = $tools.Count -eq 6 -and $null -ne $undo
    if ($aligned) {
        $u = $undo.Current.BoundingRectangle
        foreach ($tool in $tools) {
            $r = $tool.Current.BoundingRectangle
            if ([Math]::Abs(($r.Y + $r.Height / 2) - ($u.Y + $u.Height / 2)) -gt 0.5) { $aligned = $false }
        }
    }
    Check "creative six toolbar tools align vertically with Undo" $aligned
}
function CatalogueClick([string]$id) {
    $point = Center $id
    [W]::Click($point[0], $point[1])
}
foreach ($chapter in $chapters) {
    if ([DateTime]::UtcNow -ge $approvedUntil) { throw "Desktop-use window ended" }
    $id = "$scene-nav-$chapter"
    if ($scene -eq "creative") {
        $item = (Find $id).Current.BoundingRectangle
        $viewport = (Find "creative-nav-scroll").Current.BoundingRectangle
        if ($item.Bottom -gt $viewport.Bottom) {
            [W]::Wheel([int]($viewport.X + $viewport.Width / 2), [int]($viewport.Y + $viewport.Height / 2), -240)
            Start-Sleep -Milliseconds 300
        }
    }
    $point = Center $id
    [W]::Click($point[0], $point[1])
    Start-Sleep -Milliseconds 650
    $p.Refresh()
    Check "$scene/$chapter native navigation remains responsive" (-not $p.HasExited)
    CatalogueShot "$scene-$chapter"
}
switch ($scene) {
    "basic" {
        CatalogueClick "basic-nav-overlays"
        Start-Sleep -Milliseconds 500
        $dialog = (All | Where-Object { $_.Current.Name -eq "確認ダイアログ" -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button } | Select-Object -First 1)
        if (-not $dialog) { throw "Basic confirmation opener was not exposed to UIA" }
        $r = $dialog.Current.BoundingRectangle
        [W]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
    }
    "game" {
        CatalogueClick "game-nav-overlays"
        Start-Sleep -Milliseconds 500
        CatalogueClick "game-create-open"
    }
    "creative" {
        CatalogueClick "creative-nav-dialogs"
        Start-Sleep -Milliseconds 500
        # window_button returns a sibling wrapper; its test id belongs to the
        # retained tree. Native UIA exposes the actual named opener button.
        $opener = (All | Where-Object { $_.Current.Name -eq "Render Settings…" -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button } | Select-Object -First 1)
        if (-not $opener) { throw "Creative render settings opener was not exposed to UIA" }
        $r = $opener.Current.BoundingRectangle
        [W]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
    }
}
Start-Sleep -Milliseconds 300
CatalogueShot "$scene-native-dialog"
Check "$scene modal is exposed through UIA" (@(All | Where-Object { $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Window -and $_.Current.Name -ne $Title }).Count -gt 0)
if ($scene -eq "creative") {
    $render = (All | Where-Object { $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Window -and $_.Current.Name -eq "Render Settings" } | Select-Object -First 1)
    $geometry = $false
    if ($render) {
        $r = $render.Current.BoundingRectangle
        $geometry = [Math]::Abs($r.Width - 340) -le 0.5 -and [Math]::Abs($r.Height - 285) -le 5 -and
            [Math]::Abs($r.Y - $origin.Y - 120) -le 0.5 -and [Math]::Abs($origin.X + $width - $r.Right - 40) -le 0.5
    }
    Check "creative Render Settings matches source width and top-right position" $geometry
}
[W]::Key(0x1B)
Start-Sleep -Milliseconds 250
CatalogueShot "$scene-native-dialog-dismissed"
if ($scene -eq "creative") {
    $opener = (All | Where-Object { $_.Current.Name -eq "Close Project…" -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button } | Select-Object -First 1)
    if (-not $opener) { throw "Creative close project opener was not exposed to UIA" }
    $r = $opener.Current.BoundingRectangle
    [W]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
    Start-Sleep -Milliseconds 350
    CatalogueShot "$scene-native-close-project"
    $modal = (All | Where-Object { $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Window -and $_.Current.Name -eq "Close Project" } | Select-Object -First 1)
    $geometry = $false
    if ($modal) {
        $r = $modal.Current.BoundingRectangle
        $geometry = [Math]::Abs($r.Width - 440) -le 0.5 -and [Math]::Abs($r.Height - 199.4375) -le 4 -and
            [Math]::Abs($r.X + $r.Width / 2 - $origin.X - $width / 2) -le 0.5 -and
            [Math]::Abs($r.Y + $r.Height / 2 - $origin.Y - $height / 2) -le 0.5
    }
    Check "creative Close Project matches source modal geometry" $geometry
    $save = (All | Where-Object { $_.Current.Name -eq "Save" -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button -and $_.Current.HasKeyboardFocus } | Select-Object -First 1)
    Check "creative Close Project initially focuses Save" ($null -ne $save)
    [W]::Key(0x1B)
    Start-Sleep -Milliseconds 250
    CatalogueShot "$scene-native-close-project-dismissed"
}
Dump "$scene-complete"
