# Device monitor: external-component / skin scenario + extra checks.
. (Join-Path $PSScriptRoot "scenario-devicemon.ps1")

# ---- extra checks -----------------------------------------------------------
. (Join-Path $PSScriptRoot "check-lib.ps1")
$script:hwnd = $hwnd

# Drag on a timeline canvas, leave the window, release outside; afterwards
# plain hovering must not change the selection (pointer cancel).
function Fin-TimelineDragOut($tl, [string]$tag) {
    $r = El-Rect $tl
    $y = [int]($r.Y + $r.Height * 0.5)
    Fx-DragOutLeft ([int]($r.X + $r.Width * 0.55)) $y
    Start-Sleep -Milliseconds 300
    $v1 = El-Value $tl
    [FX]::MoveTo([int]($r.X + $r.Width * 0.7), $y); Start-Sleep -Milliseconds 150
    [FX]::MoveTo([int]($r.X + $r.Width * 0.85), $y); Start-Sleep -Milliseconds 400
    $v2 = El-Value $tl
    Log "  ${tag} timeline after release outside: '$v1'; after hovering: '$v2'"
    Check "${tag}: drag released outside the window ends (hover does not change the range)" ($v1 -eq $v2 -and $v1)
    Shot "$tag-dragout" | Out-Null
}

# Tab / Shift+Tab round trip from a focused element.
function Fin-TabRoundTrip($el, [string]$tag) {
    El-Click $el
    $f0 = Focused-Name
    [W]::Key(0x09); Start-Sleep -Milliseconds 250
    $f1 = Focused-Name
    [W]::KeyCombo(0x10, 0x09); Start-Sleep -Milliseconds 250
    $f2 = Focused-Name
    Log "  ${tag} focus: '$f0' -Tab-> '$f1' -Shift+Tab-> '$f2'"
    Check "${tag}: Tab moves focus and Shift+Tab returns" ($f1 -ne $f0 -and $f2 -eq $f0)
}

Fin-TimelineDragOut (El-One -Name "^Temperature history$") "dm"
$find = El-One -Name "^Find device$" -Control "Edit"
Fin-TabRoundTrip $find "dm"
Fin-Clipboard $find "roof" "dm"
Check "pasted query filters" (El-Wait -Name "^250 shown$")
$committed = Fin-Ime (El-One -Name "^Find device$" -Control "Edit") "kensaku" "けんさく" "dm"
Check "IME query filters (0 shown)" (El-Wait -Name "^0 shown$")
Fin-Reconvert (El-One -Name "^Find device$" -Control "Edit") $committed "dm"
$clear = El-Where -Name "^Clear search$" -Control "Button"
if ($clear.Count -ge 1) { El-Click $clear[0] }
Check "search cleared after IME (Escape / Clear search)" (El-Wait -Name "^1000 shown$")
Fin-Dpi $find "dm"
Fin-Monitor (El-One -Name "^Find device$" -Control "Edit") "dm"
Fin-Resize (El-One -Name "^Temperature history$") 1100 760
