# Workbench: gui_scenario.json + extra checks.
. (Join-Path $PSScriptRoot "scenario-workbench.ps1")

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

$list = El-One -Id "^records$"
$lr = El-Rect $list
$items = El-Where -Control "ListItem"
$above = @($items | Where-Object { (El-Rect $_).Bottom -le $lr.Top }).Count
$below = @($items | Where-Object { (El-Rect $_).Top -ge $lr.Bottom }).Count
Log "  records: $($items.Count) ListItems exposed; entirely outside the viewport: above=$above below=$below"
Check "UIA hides clipped overscan rows (at most one neighbour each side)" ($above -le 1 -and $below -le 1)
Fin-TimelineDragOut (El-One -Id "^external-timeline$") "wb"
Fin-TabRoundTrip (El-One -Id "^input$") "wb"
$in = El-One -Id "^input$"
Fin-Clipboard $in "timeout" "wb"
$vc = (El-One -Id "^visible-count$").Current.Name
Log "  after paste: $vc"
Check "pasted query filters the list" ($vc -match "^\d{1,5} visible records / 100000 total$")
[W]::Key(0x1B); Start-Sleep -Milliseconds 300
Check "Escape clears the search (keyboard)" ((El-Value (El-One -Id "^input$")) -eq "")
$committed = Fin-Ime (El-One -Id "^input$") "kensaku" "けんさく" "wb"
Check "IME query filters (0 records)" (El-Wait -Id "^visible-count$" -Name "^0 visible records")
Fin-Reconvert (El-One -Id "^input$") $committed "wb"
[W]::Key(0x1B); Start-Sleep -Milliseconds 300
Fin-Dpi $in "wb"
Fin-Monitor (El-One -Id "^input$") "wb"
Fin-Resize (El-One -Id "^input$") 760 640
