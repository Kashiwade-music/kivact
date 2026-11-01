# Log viewer: external-component scenario + extra checks.
. (Join-Path $PSScriptRoot "scenario-logviewer.ps1")

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

$tl = (El-Where -Name "^Lines per 5 minutes$" | Sort-Object { (El-Rect $_).X })[0]
Fin-TimelineDragOut $tl "lv"
$ls = (El-Where -Name "^Search$" -Control "Edit" | Sort-Object { (El-Rect $_).X })
Fin-TabRoundTrip $ls[0] "lv"
Fin-Clipboard $ls[0] "timeout" "lv"
[W]::Key(0x1B); Start-Sleep -Milliseconds 300
$right = (El-Where -Name "^Search$" -Control "Edit" | Sort-Object { (El-Rect $_).X })[1]
$committed = Fin-Ime $right "kensaku" "けんさく" "lv"
Fin-Reconvert $right $committed "lv"
[W]::Key(0x1B); Start-Sleep -Milliseconds 300
Fin-Dpi $right "lv"
Fin-Monitor $right "lv"
Fin-Resize $right 900 700
