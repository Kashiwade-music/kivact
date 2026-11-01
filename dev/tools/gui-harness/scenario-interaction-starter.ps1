# The public starter route, driven through guarded OS input and native UIA.
# The runner passes its dedicated output file as AppArguments.
. (Join-Path $PSScriptRoot "ext-helpers.ps1")
. (Join-Path $PSScriptRoot "check-lib.ps1")
$script:hwnd = $hwnd
if (-not $env:KIVACT_GUI_APPROVED_UNTIL) { throw "Set KIVACT_GUI_APPROVED_UNTIL to the end of an explicitly approved desktop-use window" }
if ([DateTime]::UtcNow -ge [DateTimeOffset]::Parse($env:KIVACT_GUI_APPROVED_UNTIL).UtcDateTime) {
    throw "The user's current desktop-use window has ended"
}
Check "startup load completes" (El-Wait -Name "^Rows: 3$")
$search = El-One -Name "^Search devices$" -Control "Edit"
El-Click $search
[W]::Text("Pump")
Check "native typing filters the list" (El-Wait -Name "^Rows: 1$")
El-Click (El-One -Name "^Pump$" -Control "Button")
Check "selected row retains domain key" (El-Wait -Name "^device:pump$")
El-Invoke (El-One -Name "^Save note$" -Control "Button")
Check "native AT Invoke exposes validation failure" (El-Wait -Name "^Save failed: Write a note before saving\.$")
$note = El-One -Name "^Device note$" -Control "Edit"
El-Click $note
[W]::Text("Checked today")
Check "raw editor publishes native ValuePattern" ((El-Value $note) -eq "Checked today")
[W]::Key(0x0D)
Check "native Enter submit survives custom No handler" (El-Wait -Name "^Saved$")
$output = $AppArguments.Trim('"')
Check "blocking save writes the real file" ((Test-Path -LiteralPath $output) -and [System.IO.File]::ReadAllText($output) -eq "device:pump`n50`nChecked today")
[W]::Key(0x09)
Start-Sleep -Milliseconds 200
Check "Tab moves from raw editor to the slider" ((Focused-Name) -eq "Calibration")
[W]::KeyCombo(0x10, 0x09)
Start-Sleep -Milliseconds 200
Check "Shift Tab restores raw editor focus" ((Focused-Name) -eq "Device note")
$slider = El-One -Name "^Calibration$"
$sliderRect = El-Rect $slider
$dragY = [int]($sliderRect.Y + $sliderRect.Height / 2)
[FX]::MoveTo([int]($sliderRect.X + $sliderRect.Width * 0.5), $dragY)
[FX]::Down()
Start-Sleep -Milliseconds 100
$secondaryDown = New-Object W+INPUT
$secondaryDown.type = 0
$mouse = New-Object W+MOUSEINPUT
$mouse.flags = 0x0008
$secondaryDown.mi = $mouse
$secondaryUp = New-Object W+INPUT
$secondaryUp.type = 0
$mouse.flags = 0x0010
$secondaryUp.mi = $mouse
$chord = [W+INPUT[]]@($secondaryDown, $secondaryUp)
[W]::SendInput(2, $chord, [System.Runtime.InteropServices.Marshal]::SizeOf([type][W+INPUT])) | Out-Null
[FX]::MoveTo([int]($sliderRect.Right + 20), $dragY)
Start-Sleep -Milliseconds 200
[FX]::Up()
Start-Sleep -Milliseconds 200
$range = [System.Windows.Automation.RangeValuePattern]::Pattern
$afterRelease = $slider.GetCurrentPattern($range).Current.Value
[FX]::MoveTo([int]($sliderRect.X + $sliderRect.Width * 0.1), $dragY)
Start-Sleep -Milliseconds 200
Check "secondary release preserves drag and primary release ends it" ($afterRelease -eq 100 -and $slider.GetCurrentPattern($range).Current.Value -eq $afterRelease)
El-Click (El-One -Name "^Simulate load error$" -Control "Button")
Check "load failure is displayed" (El-Wait -Name "^Load failed\. Retry\.$")
El-Click (El-One -Name "^Retry load$" -Control "Button")
Check "retry restores the filtered list" (El-Wait -Name "^Rows: 1$")
Check "retry preserves caller owned draft" ((El-Value (El-One -Name "^Device note$" -Control "Edit")) -eq "Checked today")
$committed = Fin-Ime (El-One -Name "^Device note$" -Control "Edit") "kensaku" "けんさく" "starter"
Check "IME commit preserves prior draft text" ($committed -like "Checked today*")
Shot "starter-final" | Out-Null
Dump "starter-final"
