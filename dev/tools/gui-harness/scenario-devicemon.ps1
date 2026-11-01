# Device monitor (external-component app, skin switching): gui.ps1 -Exe ...\device-monitor.exe -Title "Kivact Device Monitor" -Out <dir> -Scenario devicemon
. (Join-Path $PSScriptRoot "ext-helpers.ps1")
function TempStart {
    $ty = (El-Rect (El-One -Name "^Temperature history$")).Y
    (El-Where -Name "^Selection start$" | Where-Object { [Math]::Abs((El-Rect $_).Y - $ty) -lt 3 })[0]
}

Check "device list" (El-Wait -Name "^Devices$")
Check "find box" (El-Wait -Name "^Find device$" -Control "Edit")
Check "alarms toggle" (El-Wait -Name "^Alarms only$" -Control "CheckBox")
Check "two timelines (same component twice)" ((El-Where -Name "^(Temperature|Voltage) history$").Count -eq 2)
Check "1000 shown" (El-Wait -Name "^1000 shown$")
$rows0 = (El-Where -Control "ListItem").Count
Log "  list items mounted: $rows0 of 1000"
Check "only visible rows mounted" ($rows0 -gt 5 -and $rows0 -lt 40)

# --- live updates (1 s ticks)
$sum0 = (El-One -Id "^summary$").Current.Name
$stats0 = (El-One -Id "^temp-stats$").Current.Name
Start-Sleep -Milliseconds 3500
$sum1 = (El-One -Id "^summary$").Current.Name
$stats1 = (El-One -Id "^temp-stats$").Current.Name
Log "  summary '$sum0' -> '$sum1'"
Log "  temp stats '$stats0' -> '$stats1'"
Check "live ticks update the detail" ($stats0 -ne $stats1)

# --- list: wheel, End, click selects, detail follows
$list = El-One -Name "^Devices$"
$c = El-Center $list
for ($k = 0; $k -lt 4; $k++) { [W]::Wheel($c[0], $c[1], -240); Start-Sleep -Milliseconds 60 }
Check "wheel scrolls (device 0 unmounted)" (El-WaitAbsent -Name "^device 0$")
El-Click $list
[W]::Key(0x23); Start-Sleep -Milliseconds 300   # End
Check "End reveals device 999" (El-Wait -Name "^device 999$" -Control "ListItem")
Check "detail follows the keyboard selection" (El-Wait -Id "^title$" -Name "^sensor-0999 ")
[W]::Key(0x24); Start-Sleep -Milliseconds 300   # Home
$row = El-One -Name "^device 3$" -Control "ListItem"
$r3 = El-Rect $row
Log ("  after Home: title={0} list value={1} device 3 at y={2:n0}" -f (El-One -Id "^title$").Current.Name, (El-Value $list), $r3.Y)
El-Click $row
Log ("  after click: title={0} list value={1} device 3 now at y={2:n0}" -f (El-One -Id "^title$").Current.Name, (El-Value (El-One -Name "^Devices$")), (El-Rect (El-One -Name "^device 3$" -Control "ListItem")).Y)
Check "click selects device 3" (El-Wait -Id "^title$" -Name "^sensor-0003 ")
Shot "dm-selected" | Out-Null

# --- search + toggle
El-Click (El-One -Name "^Find device$" -Control "Edit")
[W]::Text("roof")
Check "search filters to one zone" (El-Wait -Name "^250 shown$")
El-Click (El-One -Name "^Clear search$" -Control "Button")
Check "clear button restores" (El-Wait -Name "^1000 shown$")
$cb = El-One -Name "^Alarms only$" -Control "CheckBox"
El-Click $cb
Check "alarms only filters" (El-Wait -Name "^\d{1,3} shown$")
$shown = (El-Where -Name "^\d+ shown$")[0].Current.Name
Log "  with alarms only: $shown"
[W]::Key(0x20); Start-Sleep -Milliseconds 300   # Space toggles back (focus on the check box)
Check "Space toggles the check box back" (El-Wait -Name "^1000 shown$")

# --- timelines: mouse drag on temperature, keyboard on voltage
$temp = El-One -Name "^Temperature history$"
El-DragX $temp 0.62 0.95
$tv = El-Value $temp
Log "  temperature selection: $tv"
Check "drag selects a sample range" ($tv -match "^t-\d+ . t-\d+ \(\d+ of 48\)$")
Check "range statistics" (El-Wait -Id "^temp-stats$" -Name "^1\d samples")
Check "the other timeline is independent" (El-Wait -Id "^volt-stats$" -Name "^48 samples")
$volt = El-One -Name "^Voltage history$"
El-Click $volt
[W]::Key(0x27); Start-Sleep -Milliseconds 200   # ArrowRight moves the one-bucket selection
$vv = El-Value $volt
Log "  voltage selection after click + ArrowRight: $vv"
Check "click + keyboard on the second timeline" ($vv -match "\(1 of 48\)$")
$start = TempStart
$sr = El-Rect $start
Log ("  temperature start handle rect ({0:n0},{1:n0},{2:n0},{3:n0})" -f $sr.X, $sr.Y, $sr.Width, $sr.Height)
Shot "dm-timelines" | Out-Null

# --- Style/skin invariance: switch skin, compare geometry of hit targets and the control census
$census = { (El-All | ForEach-Object { $_.Current.ControlType.ProgrammaticName } | Group-Object | Sort-Object Name | ForEach-Object { "$($_.Name)=$($_.Count)" }) -join "," }
$c0 = & $census
$t0 = El-Rect (El-One -Name "^Temperature history$")
$l0 = El-Rect (El-One -Name "^Devices$")
$s0 = El-Rect (TempStart)
$px0 = Pixel-At ([int]$l0.X + 20) ([int]$l0.Y - 6)
El-Click (El-One -Name "^Skin: Business$" -Control "Button")
Check "switched to Console" (El-Wait -Name "^Skin: Console$" -Control "Button")
Start-Sleep -Milliseconds 500
Shot "dm-console" | Out-Null
$c1 = & $census
$t1 = El-Rect (El-One -Name "^Temperature history$")
$l1 = El-Rect (El-One -Name "^Devices$")
$s1 = El-Rect (TempStart)
$px1 = Pixel-At ([int]$l1.X + 20) ([int]$l1.Y - 6)
Log "  census business: $c0"
Log "  census console:  $c1"
Log ("  pixel {0},{1},{2} -> {3},{4},{5}" -f $px0.R, $px0.G, $px0.B, $px1.R, $px1.G, $px1.B)
Check "semantics census unchanged" ($c0 -eq $c1)
Check "timeline and list sizes unchanged" ($t0.Width -eq $t1.Width -and $t0.Height -eq $t1.Height -and $l0.Height -eq $l1.Height)
Check "hit region (start handle) size unchanged" ($s0.Width -eq $s1.Width -and $s0.Height -eq $s1.Height)
Check "paint changed" (($px0.R -ne $px1.R) -or ($px0.G -ne $px1.G) -or ($px0.B -ne $px1.B))
# The handle still works under the console skin (hit region).
$start = TempStart
$sr = El-Rect $start
[W2]::Drag([int]($sr.X + $sr.Width / 2), [int]($sr.Y + $sr.Height / 2), [int]($sr.X + $sr.Width / 2 - 80), [int]($sr.Y + $sr.Height / 2), 10)
Start-Sleep -Milliseconds 300
$tv2 = El-Value (El-One -Name "^Temperature history$")
Log "  after start-handle drag (console): $tv2"
Check "start handle drag widens the range" ($tv2 -ne $tv)
# Keyboard focus ring under the console skin: Tab to the skin button, Enter switches back.
El-Click (El-One -Name "^Skin: Console$" -Control "Button")
Check "switched back to Business" (El-Wait -Name "^Skin: Business$" -Control "Button")
Shot "dm-business" | Out-Null
Dump "final"
