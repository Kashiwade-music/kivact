# Task Board real-window scenario: SendInput (mouse/keyboard) + UIA checks,
# JSON persistence across a graceful restart, 100k generated board.
param([string]$Exe, [string]$Out, [int]$Round = 1)
. (Join-Path $PSScriptRoot "apps-lib.ps1")
New-Item -ItemType Directory -Force $Out | Out-Null
$script:out = $Out
$script:log = Join-Path $Out "log.txt"
$title = "Kivact · Task Board"
$lad = Join-Path $Out "localappdata"
Remove-Item -Recurse -Force $lad -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $lad | Out-Null
$env:LOCALAPPDATA = $lad
Remove-Item Env:\KIVACT_TASK_BOARD_GENERATED -ErrorAction SilentlyContinue
$json = Join-Path $lad "KivactExamples\task-board.json"
function Saved() { return (Read-AtomicJsonSnapshot $json) }
function SavedTask($id) { $s = Saved; if ($s) { return $s.tasks | Where-Object { $_.id -eq $id } } return $null }
function WaitSaved($id, $lane, $titleText) {
    for ($k = 0; $k -lt 50; $k++) {
        $t = SavedTask $id
        if ($t -and ($lane -eq $null -or $t.lane -eq $lane) -and ($titleText -eq $null -or $t.title -eq $titleText)) { return $true }
        Start-Sleep -Milliseconds 100
    }
    Log "  saved task $id = $(SavedTask $id | ConvertTo-Json -Compress)"
    return $false
}

Enter-GuiLock "scenario-taskboard round $Round"
try {
    Log "=== Task Board round $Round $(Get-Date -Format o)"
    Log "exe sha256=$((Get-FileHash $Exe -Algorithm SHA256).Hash)"
    Start-App $Exe $title
    Check "three lanes with the default cards" ((WaitName "lane-planned" "予定 (1件)") -and (WaitName "lane-progress" "進行中 (1件)") -and (WaitName "lane-done" "完了 (1件)"))
    Check "status starts ready and nothing is saved" ((WaitName "board-status" "準備完了") -and -not (Test-Path $json))

    # 1. Add through physical clicks and SendInput unicode text (Japanese)
    TypeInto "task-title" "GUIカード"
    TypeInto "task-notes" "UIAで保存"
    Check "typed title is in the field (UIA value)" ((ValueOf "task-title") -eq "GUIカード")
    ClickId "add-task"
    Check "added card appears in Planned" ((WaitName "lane-planned" "予定 (2件)") -and (WaitName "card-4" "GUIカード"))
    Check "form is cleared after add" ((ValueOf "task-title") -eq "")
    Check "status says saved" (WaitName "board-status" "保存しました")
    Check "JSON contains the new task" (WaitSaved 4 "Planned" "GUIカード")
    Shot "tb-added" | Out-Null

    # 2. Select (card button) opens the editor and loads the form
    ClickId "select-4"
    Check "editor shows the selection" (WaitName "editor-label" "選択: GUIカード / 予定")
    Check "card meta shows visits and selection" (WaitName "card-meta-4" "予定 · 閲覧 1 · 選択中")
    Check "form loaded from the card" ((ValueOf "task-notes") -eq "UIAで保存")

    # 3. Editor: move right
    ClickId "move-right"
    Check "moved to In progress" ((WaitName "lane-progress" "進行中 (2件)") -and (WaitName "lane-planned" "予定 (1件)"))
    Check "JSON lane InProgress" (WaitSaved 4 "InProgress" $null)
    Check "visits kept across the lane move" (WaitNameLike "card-meta-4" "進行中 · 閲覧 1*")

    # 4. Checkbox click completes; the moved card takes the focus back
    #    (it is a new row in the Done lane), so Space reopens it
    ClickId "done-4"
    Check "checkbox completes the task" ((WaitName "lane-done" "完了 (2件)") -and (WaitSaved 4 "Done" $null))
    $cb = (Find "done-4").GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Current.ToggleState
    Log "  done-4 toggle state: $cb"
    Check "UIA toggle state is On" ("$cb" -eq "On")
    Start-Sleep -Milliseconds 200
    Log "  focused after the move: $(Focused)"
    Check "focus follows the moved card's checkbox" ((Focused) -eq "done-4")
    [AW]::Key(0x20)
    Check "Space unchecks (back to In progress)" ((WaitName "lane-progress" "進行中 (2件)") -and (WaitSaved 4 "InProgress" $null))
    Start-Sleep -Milliseconds 200
    Check "focus stays on the card after the second move" ((Focused) -eq "done-4")

    # 5. Select (combo box) in the editor, keyboard only (check-taskboard.ps1
    #    also opens the popup with the mouse).
    ClickId "edit-task"
    [AW]::Key(0x09); [AW]::Key(0x09); [AW]::Key(0x09); Start-Sleep -Milliseconds 200
    Check "Tab reaches the lane select" ((Focused) -eq "lane-select")
    Check "select shows the current lane" ((ValueOf "lane-select") -eq "進行中")
    [AW]::Key(0x26)  # ArrowUp
    Check "ArrowUp on the select moves the task to Planned" ((WaitName "lane-planned" "予定 (2件)") -and (WaitSaved 4 "Planned" $null))
    Check "select value follows" ((ValueOf "lane-select") -eq "予定")

    # 6. Edit the title (Ctrl+A, Delete, type) and update
    TypeInto "task-title" "編集済みカード"
    ClickId "edit-task"
    Check "card title updated" (WaitName "card-4" "編集済みカード")
    Check "JSON title updated" (WaitSaved 4 $null "編集済みカード")
    # Empty title is rejected with an alert
    TypeInto "task-title" ""
    ClickId "edit-task"
    Check "empty title shows an alert" (WaitName "notice" "題名を入力してください")
    TypeInto "task-title" "編集済みカード"

    # 7. Search filters every lane
    TypeInto "task-search" "済み"
    Check "search keeps only the matching card" ((WaitName "lane-planned" "予定 (1件)") -and (WaitName "lane-progress" "進行中 (0件)") -and (WaitName "lane-done" "完了 (0件)"))
    Shot "tb-search" | Out-Null
    TypeInto "task-search" ""
    Check "clearing the search restores the lanes" (WaitName "lane-progress" "進行中 (1件)")

    # 8. Keyboard only: Tab from the search field reaches the title field, Enter on a focused button activates
    ClickId "task-search"
    [AW]::Key(0x09); Start-Sleep -Milliseconds 200
    Check "Tab moves from search to title" ((Focused) -eq "task-title")
    ClickId "select-1"
    Check "select-1 has focus after the click" ((Focused) -eq "select-1")
    [AW]::Key(0x09); Start-Sleep -Milliseconds 200
    Check "Tab moves to the card's right button" ((Focused) -eq "right-1")
    [AW]::Key(0x0D)
    Check "Enter on the focused button moves card 1" (WaitName "lane-progress" "進行中 (2件)")
    Start-Sleep -Milliseconds 200
    Check "focus follows the moved card" ((Focused) -eq "right-1")
    [AW]::Key(0x20)
    Check "Space moves it again (keyboard only)" ((WaitName "lane-done" "完了 (2件)") -and (WaitSaved 1 "Done" $null))
    Check "a done card focuses its checkbox" ((Focused) -eq "done-1")
    ClickId "select-4"
    Check "card 4 selected again" (WaitNameLike "card-meta-4" "*選択中")
    $bmp = Shot "tb-before-restart"
    $sel = (Find "card-4").Current.BoundingRectangle
    $px = PixelAt $bmp ($sel.X + 1) ($sel.Y + $sel.Height / 2)
    Log ("  selected card left border rgb=({0},{1},{2})" -f $px.R, $px.G, $px.B)
    Check "selected card has the accent border" ($px.B -gt 180 -and $px.R -lt 90)

    # 9. Graceful close drains the save queue; relaunch restores the board
    $code = Stop-App
    Check "graceful close exits with code 0" ($code -eq 0)
    Start-App $Exe $title
    Check "restored card after restart" ((WaitName "card-4" "編集済みカード") -and (WaitName "card-meta-4" "予定 · 閲覧 0"))
    Check "restored lanes after restart" ((WaitName "lane-done" "完了 (2件)") -and (WaitName "lane-progress" "進行中 (1件)"))

    # 10. Delete via UIA Invoke (a11y path) closes the editor
    InvokeId "select-4"
    Check "editor visible" (WaitName "editor-label" "選択: 編集済みカード / 予定")
    InvokeId "delete-task"
    Check "deleted card is gone" ((WaitGone "card-4") -and (WaitName "lane-planned" "予定 (0件)"))
    Check "editor unmounts after delete" (WaitGone "task-editor")
    for ($k = 0; $k -lt 30 -and (SavedTask 4); $k++) { Start-Sleep -Milliseconds 100 }
    Check "JSON no longer has the task" ($null -eq (SavedTask 4))
    Shot "tb-deleted" | Out-Null
    $code = Stop-App
    Check "second close exits with code 0" ($code -eq 0)

    # 11. 100k generated board (unsaved) in a fresh profile
    $lad2 = Join-Path $Out "localappdata-100k"
    Remove-Item -Recurse -Force $lad2 -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force $lad2 | Out-Null
    $env:LOCALAPPDATA = $lad2
    $env:KIVACT_TASK_BOARD_GENERATED = "100000"
    $t0 = Get-Date
    Start-App $Exe $title
    Check "100k board loads" (WaitName "lane-planned" "予定 (100000件)" 300)
    Log ("  100k: window+load {0:n0} ms" -f ((Get-Date) - $t0).TotalMilliseconds)
    $n = (Root).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition).Count
    Log "  UIA elements with 100k tasks: $n"
    Check "only a window of cards is materialized" ($n -lt 400)
    $list = (Find "list-planned").Current.BoundingRectangle
    for ($k = 0; $k -lt 10; $k++) { [AW]::Wheel([int]($list.X + 40), [int]($list.Y + 100), -1200); Start-Sleep -Milliseconds 60 }
    Start-Sleep -Milliseconds 500
    Check "wheel scrolls the virtual list (card 1 disposed)" (WaitGone "card-1")
    $visible = (Root).FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::ListItem)))
    $names = @($visible | ForEach-Object { $_.Current.Name })
    Log "  visible after wheel: $($names[0]) .. $($names[-1]) ($($names.Count))"
    Check "later cards are visible" ($names.Count -gt 0 -and $names[0] -ne "Generated task 1")
    $listTop = (Find "list-planned").Current.BoundingRectangle.Y
    $firstVisible = $visible | Where-Object { $_.Current.BoundingRectangle.Y -ge $listTop } | Select-Object -First 1
    Log "  overscan rows above the viewport are exposed to UIA: $(@($visible | Where-Object { $_.Current.BoundingRectangle.Y -lt $listTop }).Count)"
    $firstId = $firstVisible.Current.AutomationId -replace "card-", ""
    Log "  first fully visible card: $firstId"
    $t1 = Get-Date
    ClickId "right-$firstId"
    Check "moving one of 100k cards updates the counts" ((WaitName "lane-progress" "進行中 (1件)") -and (WaitName "lane-planned" "予定 (99999件)"))
    Log ("  100k: move round trip {0:n0} ms (UIA polling included)" -f ((Get-Date) - $t1).TotalMilliseconds)
    Shot "tb-100k" | Out-Null
    $code = Stop-App
    Check "100k close exits with code 0" ($code -eq 0)
    Remove-Item Env:\KIVACT_TASK_BOARD_GENERATED
} catch {
    Log "ERROR $_ $($_.ScriptStackTrace)"
    $script:failed++
    Kill-App
} finally {
    Exit-GuiLock
    Log "passed=$script:passed failed=$script:failed"
}
exit $script:failed
