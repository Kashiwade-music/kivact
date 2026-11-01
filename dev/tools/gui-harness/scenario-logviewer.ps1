# Log viewer (external-component app): gui.ps1 -Exe ...\log-viewer.exe -Title "Kivact Log Viewer" -Out <dir> -Scenario logviewer
. (Join-Path $PSScriptRoot "ext-helpers.ps1")

$searches = El-Where -Name "^Search$" -Control "Edit"
Check "two search boxes (same component twice)" ($searches.Count -eq 2)
Check "two timelines" ((El-Where -Name "^Lines per 5 minutes$").Count -eq 2)
Check "two lazy lists" ((El-Where -Name "^Log lines$").Count -eq 2)
Check "left status" (El-Wait -Name "^60000 of 60000 lines$")
Check "right status (live)" (El-Wait -Name "^200\d\d of 200\d\d lines$")
$lists = El-Where -Name "^Log lines$"
$left = $lists[0]; $right = $lists[1]
if ((El-Rect $left).X -gt (El-Rect $right).X) { $t = $left; $left = $right; $right = $t }
$rows0 = (El-Where -Control "ListItem").Count
Log "  list items mounted: $rows0 (120,000+ lines in total)"
Check "only visible rows are mounted" ($rows0 -gt 10 -and $rows0 -lt 80)

# --- lazy list: wheel, click, keyboard, detail
Check "line 0 visible" (El-Wait -Name "^line 0$")
$c = El-Center $left
for ($k = 0; $k -lt 6; $k++) { [W]::Wheel($c[0], $c[1], -240); Start-Sleep -Milliseconds 60 }
$ok = $false; for ($k = 0; $k -lt 40; $k++) { if ((El-Where -Name "^line 0$").Count -eq 1) { $ok = $true; break }; Start-Sleep -Milliseconds 100 }
Check "wheel scrolls the left list (its line 0 unmounted, right pane keeps line 0)" $ok
Shot "lv-after-wheel" | Out-Null
$row = (El-Where -Control "ListItem" -Name "^line \d+$" | Where-Object { (El-Rect $_).X -lt (El-Rect $right).X -and (El-Rect $_).Y -gt (El-Rect $left).Y + 40 })[0]
$rowName = $row.Current.Name
El-Click $row
Log "  clicked $rowName; focused=$(Focused-Name)"
[W]::Key(0x28); Start-Sleep -Milliseconds 150   # ArrowDown
[W]::Key(0x0D); Start-Sleep -Milliseconds 250   # Enter -> detail
$n = [int]($rowName -replace "line ", "") + 1
Check "keyboard ArrowDown + Enter opens line $n in the detail" (El-Wait -Id "^detail$" -Name "^#$n ")
Shot "lv-detail" | Out-Null

# --- scrollbar thumb (a Draw::region) drag
$thumb = (El-Where -Name "^Scroll position$" -Control "Slider")
Log "  thumbs: $($thumb.Count)"
if ($thumb.Count -ge 1) {
    $t = $thumb | Sort-Object { (El-Rect $_).X } | Select-Object -First 1
    $before = El-Value $t
    $r = El-Rect $t
    [W2]::Drag([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2), [int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2 + 120), 10)
    Start-Sleep -Milliseconds 300
    $t = (El-Where -Name "^Scroll position$" -Control "Slider" | Sort-Object { (El-Rect $_).X } | Select-Object -First 1)
    $after = El-Value $t
    Log "  thumb value '$before' -> '$after'; name='$($t.Current.Name)'"
    Check "scrollbar thumb drag scrolls" ($after -ne $before)
} else { Check "scrollbar thumb region exposed" $false }

# --- search box: physical typing (left) and Japanese IME (right)
$ls = $searches | Sort-Object { (El-Rect $_).X }
El-Click $ls[0]
[W]::Text("timeout")
Check "left search filters" (El-Wait -Name "^\d+ of 60000 lines$")
$st = (El-Where -Name " of 60000 lines$")[0].Current.Name
Log "  left status: $st"
Check "left status shows a filtered count" ($st -ne "60000 of 60000 lines")
Check "right pane unaffected" (El-Wait -Name "^(\d+) of \1 lines$")
El-Click $ls[1]
[W2]::KeyDown(0x16); [W2]::KeyUp(0x16); Start-Sleep -Milliseconds 300   # VK_IME_ON
foreach ($ch in "kensaku".ToCharArray()) { [W]::Key([uint16][char]::ToUpper($ch)); Start-Sleep -Milliseconds 60 }
Start-Sleep -Milliseconds 400
Shot "lv-ime-composing" | Out-Null
$rs = (El-Where -Name "^(\d+) of \1 lines$")
Check "IME preedit does not filter (binding unchanged while composing)" ($rs.Count -ge 1)
[W]::Key(0x20); Start-Sleep -Milliseconds 500    # Space: convert
Shot "lv-ime-converted" | Out-Null
[W]::Key(0x0D); Start-Sleep -Milliseconds 600    # Enter: commit
[W2]::KeyDown(0x1A); [W2]::KeyUp(0x1A); Start-Sleep -Milliseconds 200   # VK_IME_OFF
$v = El-Value ((El-Where -Name "^Search$" -Control "Edit" | Sort-Object { (El-Rect $_).X })[1])
Log "  right search value after IME commit: '$v'"
Check "IME commit writes non-ASCII text" ($v -and ($v -match "[^\x00-\x7F]"))
Check "committed query filters the right pane (0 lines)" (El-Wait -Name "^0 of \d+ lines$")
Shot "lv-ime-committed" | Out-Null
[W]::Key(0x1B); Start-Sleep -Milliseconds 300    # Escape clears (bubbles to the SearchBox)
Check "Escape clears the right query" (El-Wait -Name "^(\d+) of \1 lines$")

# --- timeline: drag a range on the left canvas, then drag a handle region
$tls = El-Where -Name "^Lines per 5 minutes$" | Sort-Object { (El-Rect $_).X }
$tl = $tls[0]
$v0 = El-Value $tl
El-DragX $tl 0.30 0.45
$v1 = El-Value $tl
Log "  timeline value '$v0' -> '$v1'"
Check "mouse drag selects a time range" ($v1 -match "^\d\d:\d\d . \d\d:\d\d \(\d+ of 48\)$")
$st2 = (El-Where -Name " of 60000 lines$")[0].Current.Name
Log "  left status with range: $st2"
Check "range restricts the list" ($st2 -ne $st)
$endh = (El-Where -Name "^Selection end$" | Sort-Object { (El-Rect $_).X })[0]
if ($endh) {
    $r = El-Rect $endh
    Log ("  end handle rect ({0:n0},{1:n0},{2:n0},{3:n0}) value={4}" -f $r.X, $r.Y, $r.Width, $r.Height, (El-Value $endh))
    [W2]::Drag([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2), [int]($r.X + $r.Width / 2 + 60), [int]($r.Y + $r.Height / 2), 10)
    Start-Sleep -Milliseconds 300
    $v2 = El-Value $tl
    Log "  after end-handle drag: '$v2'"
    Check "dragging the end handle (hit region) extends the range" ($v2 -ne $v1 -and $v2.Substring(0, 5) -eq $v1.Substring(0, 5))
} else { Check "end handle region exposed" $false }
Shot "lv-timeline" | Out-Null

# --- skins: paint changes, semantics and hit regions do not
$tr0 = El-Rect $tl
$lr0 = El-Rect $left
$sig0 = (El-All | ForEach-Object { $_.Current.ControlType.ProgrammaticName } | Group-Object | Sort-Object Name | ForEach-Object { "$($_.Name)=$($_.Count)" }) -join ","
$bg0 = Pixel-At ([int]$tr0.X - 4) ([int]$tr0.Y - 20)
El-Click (El-One -Name "^Skin: Business$" -Control "Button")
Check "skin switched" (El-Wait -Name "^Skin: Console$" -Control "Button")
Start-Sleep -Milliseconds 400
Shot "lv-console-skin" | Out-Null
$tl = (El-Where -Name "^Lines per 5 minutes$" | Sort-Object { (El-Rect $_).X })[0]
$tr1 = El-Rect $tl
$sig1 = (El-All | ForEach-Object { $_.Current.ControlType.ProgrammaticName } | Group-Object | Sort-Object Name | ForEach-Object { "$($_.Name)=$($_.Count)" }) -join ","
$bg1 = Pixel-At ([int]$tr1.X - 4) ([int]$tr1.Y - 20)
Log "  controls before: $sig0"
Log "  controls after:  $sig1"
Log ("  panel pixel {0},{1},{2} -> {3},{4},{5}" -f $bg0.R, $bg0.G, $bg0.B, $bg1.R, $bg1.G, $bg1.B)
Check "semantics (control census) unchanged by the skin" ($sig0 -eq $sig1)
Check "timeline size unchanged by the skin" ($tr0.Width -eq $tr1.Width -and $tr0.Height -eq $tr1.Height)
Check "paint changed" (($bg0.R -ne $bg1.R) -or ($bg0.G -ne $bg1.G) -or ($bg0.B -ne $bg1.B))
Check "selection kept across skins" ((El-Value $tl) -eq $v2)
El-Click (El-One -Name "^Skin: Console$" -Control "Button")
Check "skin switched back" (El-Wait -Name "^Skin: Business$" -Control "Button")
# --- select: close by clicking outside. The list is a
# non-modal overlay, so the press also reaches the control under it: the
# skin button switches the skin (pass-through; the Windows menu convention
# would consume the click instead).
$lvl = (El-Where -Name "^Level$" -Control "ComboBox" | Sort-Object { (El-Rect $_).X })[0]
El-Click $lvl
$opened = El-Wait -Name "^Warnings and errors$" -Control "ListItem"
Log "  Level select after click: options exposed=$opened"
Check "clicking the Level select opens its list (options in UIA)" $opened
Shot "lv-select-open" | Out-Null
$skinBtn = El-One -Name "^Skin: Business$" -Control "Button"
El-Click $skinBtn
$closed = El-WaitAbsent -Name "^Warnings and errors$" -Control "ListItem"
$passed = El-Wait -Name "^Skin: Console$" -Control "Button" -Tries 20
Log "  outside click on 'Skin: Business': list closed=$closed, button pressed too=$passed"
Check "an outside click closes the Level select (options gone from UIA)" $closed
Check "the outside click also reaches the clicked button (non-modal pass-through)" $passed
Shot "lv-select-dismissed" | Out-Null
if ($passed) {
    El-Click (El-One -Name "^Skin: Console$" -Control "Button")
    El-Wait -Name "^Skin: Business$" -Control "Button" | Out-Null
}
Dump "final"
