# Workbench toolbar icons: the IconButton's SVG
# icon is drawn (not blank), takes the button's hover and pressed text
# colors, and loses the hover color when the pointer leaves the window.
# Saves 8x crops of the icon (idle, hover, pressed, after leaving) for a
# visual check of the SVG's sharpness.
# Driven by gui.ps1: run-gui-locked.ps1 -Exe ...\workbench.exe -Title "Kivact Workbench" -Out <dir> -Scenario workbench-icons
. (Join-Path $PSScriptRoot "ext-helpers.ps1")
. (Join-Path $PSScriptRoot "check-lib.ps1")
$script:hwnd = $hwnd

# The darkest pixel of an element's rect (the icon's ink on a light
# background) and an 8x nearest-neighbour crop saved as <tag>.png.
function Icon-Ink($e, [string]$tag) {
    $r = El-Rect $e
    $w = [int]$r.Width; $h = [int]$r.Height
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$r.X, [int]$r.Y, 0, 0, $bmp.Size)
    $g.Dispose()
    $best = $null; $bestL = 9999; $inked = 0
    for ($i = 0; $i -lt $w; $i++) {
        for ($j = 0; $j -lt $h; $j++) {
            $p = $bmp.GetPixel($i, $j)
            $l = 0.299 * $p.R + 0.587 * $p.G + 0.114 * $p.B
            if ($l -lt 200) { $inked++ }
            if ($l -lt $bestL) { $bestL = $l; $best = $p }
        }
    }
    $big = New-Object System.Drawing.Bitmap ($w * 8), ($h * 8)
    $g2 = [System.Drawing.Graphics]::FromImage($big)
    $g2.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $g2.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $g2.DrawImage($bmp, 0, 0, $w * 8, $h * 8)
    $g2.Dispose()
    $big.Save((Join-Path $Out "$tag.png")); $big.Dispose(); $bmp.Dispose()
    $ink = [pscustomobject]@{ R = [int]$best.R; G = [int]$best.G; B = [int]$best.B; L = [int]$bestL; Inked = $inked }
    Log ("  {0}: darkest ({1},{2},{3}) luminance {4}, {5} inked px of {6}x{7}" -f $tag, $ink.R, $ink.G, $ink.B, $ink.L, $ink.Inked, $w, $h)
    return $ink
}
function Same-Ink($a, $b) { [Math]::Abs($a.R - $b.R) + [Math]::Abs($a.G - $b.G) + [Math]::Abs($a.B - $b.B) -le 12 }

# A point outside the window, beside it (left when there is room, else right).
function Outside-Point {
    $wr = Fx-WinRect
    $x = if ($wr.L -ge 60) { $wr.L - 40 } else { $wr.R + 40 }
    return @([int]$x, [int]($wr.T + 200))
}

Log "=== step icon-buttons"
El-Click (El-One -Id "^record-open-0$")
Check "a record is selected (Clear selection is enabled)" (El-Wait -Id "^detail-body$" -Name "^Record 0: diagnostic detail$")
$tb = El-One -Id "^toolbar$" -Name "^Record tools$" -Control "ToolBar"
$inTb = @(El-Where -Scope $tb)
Log "  toolbar descendants: $(($inTb | ForEach-Object { $_.Current.ControlType.ProgrammaticName + " '" + $_.Current.Name + "'" }) -join ', ')"
Check "the toolbar exposes its two buttons by name, no icon image" ($inTb.Count -eq 2 -and (@($inTb | Where-Object { $_.Current.ControlType.ProgrammaticName -match "Button" })).Count -eq 2)
$btn = El-One -Id "^clear-selection$" -Control "Button"
$range = El-One -Id "^clear-range$" -Control "Button"
$away = Outside-Point
[FX]::MoveTo($away[0], $away[1]); Start-Sleep -Milliseconds 400
$idle = Icon-Ink $btn "wb-icon-idle"
$disabled = Icon-Ink $range "wb-icon-disabled"
Check "the icon is drawn (inked pixels darker than the background)" ($idle.Inked -ge 12 -and $idle.L -lt 170)
Check "a disabled icon is drawn lighter (opacity 0.45)" ($disabled.Inked -ge 1 -and $disabled.L -gt $idle.L + 20)

$c = El-Center $btn
[FX]::MoveTo($c[0], $c[1]); Start-Sleep -Milliseconds 400
$hover = Icon-Ink $btn "wb-icon-hover"
Check "hover paints the icon in the button's hover text color (darker)" ($hover.L -le $idle.L - 20)

[FX]::Down(); Start-Sleep -Milliseconds 400
$pressed = Icon-Ink $btn "wb-icon-pressed"
Check "press paints the icon in the button's pressed text color (accent blue)" ($pressed.B -ge $pressed.R + 60 -and $pressed.B -ge $pressed.G + 20)
[FX]::Up(); Start-Sleep -Milliseconds 300
Check "a click on the icon presses its button (selection cleared)" (El-WaitAbsent -Id "^detail-body$")

El-Click (El-One -Id "^record-open-1$")
Check "a record is selected again" (El-Wait -Id "^detail-body$" -Name "^Record 1: diagnostic detail$")
$btn = El-One -Id "^clear-selection$" -Control "Button"
$c = El-Center $btn
[FX]::MoveTo($c[0], $c[1]); Start-Sleep -Milliseconds 400
$hover2 = Icon-Ink $btn "wb-icon-hover-again"
Check "hover again before leaving the window" ($hover2.L -le $idle.L - 20)
$away = Outside-Point
[FX]::MoveTo($away[0], $away[1]); Start-Sleep -Milliseconds 500
$left = Icon-Ink $btn "wb-icon-left-window"
Check "hover ends when the pointer leaves the window (idle color again)" (Same-Ink $left $idle)
Shot "wb-icons" | Out-Null
