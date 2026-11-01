# Driven by gui.ps1 (functions Find/NameOf/Center/Shot/Check/Log in scope).
function WaitName($id, $expected) {
    for ($k = 0; $k -lt 30; $k++) {
        if ((NameOf $id) -eq $expected) { return $true }
        Start-Sleep -Milliseconds 100
    }
    Log "  $id name='$(NameOf $id)' expected '$expected'"
    return $false
}
function Focused() {
    $f = [System.Windows.Automation.AutomationElement]::FocusedElement
    return $f.Current.AutomationId
}

Check "count text initial" (WaitName "count" "count: 0")

# 1. Physical mouse click on Increment (x2)
$c = Center "inc"; [W]::Click($c[0], $c[1]); Start-Sleep -Milliseconds 150; [W]::Click($c[0], $c[1])
Check "mouse click increments twice" (WaitName "count" "count: 2")

# 2. Pixel check: Increment button background is the accent blue
$bmp = Shot "after-click"
$r = New-Object W+RECT; [W]::GetWindowRect($hwnd, [ref]$r) | Out-Null
$b = (Find "inc").Current.BoundingRectangle
$px = $bmp.GetPixel([int]($b.X - $r.L + 4), [int]($b.Y - $r.T + $b.Height / 2))
Log ("  inc pixel rgb=({0},{1},{2})" -f $px.R, $px.G, $px.B)
Check "button paints accent background" ($px.B -gt 180 -and $px.R -lt 80)

# 3. Type into the text field (click to focus, SendInput unicode incl. Japanese)
$c = Center "name"; [W]::Click($c[0], $c[1]); Start-Sleep -Milliseconds 200
Check "click focuses the text field" ((Focused) -eq "name")
[W]::Text("Kivact 日本")
Check "typed text echoes into label" (WaitName "greeting" "hello, Kivact 日本")
[W]::Key(0x08) # Backspace
Check "backspace edits" (WaitName "greeting" "hello, Kivact 日")
$v = (Find "name").GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
Check "UIA value of the field" ($v -eq "Kivact 日")

# 4. Keyboard only: Tab to checkbox, Space toggles; Shift+Tab goes back
[W]::Key(0x09); Start-Sleep -Milliseconds 150
Check "Tab moves focus to the checkbox" ((Focused) -eq "done")
[W]::Key(0x20)
Check "Space toggles the checkbox" (WaitName "done-label" "done")
[W]::KeyCombo(0x10, 0x09); Start-Sleep -Milliseconds 150
Check "Shift+Tab returns to the field" ((Focused) -eq "name")
[W]::Key(0x09); [W]::Key(0x09); Start-Sleep -Milliseconds 150
Check "Tab reaches Add item" ((Focused) -eq "add")
[W]::Key(0x0D)
Check "Enter activates the focused button" (WaitName "item-count" "3 items")

# 5. UIA Invoke on Reset (a11y action path)
$inv = (Find "reset").GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
$inv.Invoke()
Check "UIA Invoke activates Reset" (WaitName "count" "count: 0")

# 6. Add many items, then wheel-scroll the list
for ($k = 0; $k -lt 14; $k++) { $c = Center "add"; [W]::Click($c[0], $c[1]); Start-Sleep -Milliseconds 60 }
Check "items added" (WaitName "item-count" "17 items")
Shot "before-wheel" | Out-Null
$list = (Root).FindFirst([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Pane)))
$lr = $list.Current.BoundingRectangle
$firstBefore = $list.FindFirst([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition).Current.BoundingRectangle.Y
[W]::Wheel([int]($lr.X + 50), [int]($lr.Y + 40), -240)
Start-Sleep -Milliseconds 300
$firstAfter = $list.FindFirst([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition).Current.BoundingRectangle.Y
Log "  first list row y: $firstBefore -> $firstAfter"
Check "wheel scrolls the list content" ($firstAfter -lt $firstBefore)
Shot "after-wheel" | Out-Null

# 7. Resize: the full-width Add item button follows the window width
$w0 = (Find "add").Current.BoundingRectangle.Width
[W]::SetWindowPos($hwnd, [IntPtr]::Zero, 0, 0, 640, 560, 0x0002 -bor 0x0004) | Out-Null
Start-Sleep -Milliseconds 600
$w1 = (Find "add").Current.BoundingRectangle.Width
Log "  add width: $w0 -> $w1"
Check "resize re-lays out the window content" ($w1 -lt $w0 - 200 -and $w1 -gt 300)
Shot "after-resize" | Out-Null
Dump "final"
