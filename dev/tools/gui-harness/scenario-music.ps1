# Music player real-window scenario: seek buttons, waveform drag, keyboard,
# real playback through the default audio device, WAV loading, errors.
param([string]$Exe, [string]$Out, [int]$Round = 1)
. (Join-Path $PSScriptRoot "apps-lib.ps1")
New-Item -ItemType Directory -Force $Out | Out-Null
$script:out = $Out
$script:log = Join-Path $Out "log.txt"
$title = "Kivact Music Player"
$wav = Join-Path $Out "tone-3s.wav"
python (Join-Path $PSScriptRoot "make_wav.py") $wav 3 | Out-Null
function Pos() { return (NameOf "position") }
function Status() { return (NameOf "status") }
# Sum of |pixel - background| over the spectrum bars area (changes while playing).
function SpectrumEnergy($bmp) {
    $r = (Find "spectrum").Current.BoundingRectangle
    $w = New-Object AW+RECT; [AW]::GetWindowRect($script:hwnd, [ref]$w) | Out-Null
    $sum = 0
    for ($x = [int]($r.X + 55); $x -lt [int]($r.X + $r.Width - 20); $x += 3) {
        for ($y = [int]($r.Y + 15); $y -lt [int]($r.Y + $r.Height - 35); $y += 3) {
            $p = $bmp.GetPixel($x - $w.L, $y - $w.T)
            if ($p.B -gt 200 -and $p.R -lt 140) { $sum++ }
        }
    }
    return $sum
}

Enter-GuiLock "scenario-music round $Round"
try {
    Log "=== Music player round $Round $(Get-Date -Format o)"
    Log "exe sha256=$((Get-FileHash $Exe -Algorithm SHA256).Hash)"
    Start-App $Exe $title
    Check "demo loads on the worker" (WaitName "track-title" "Demo melody (generated PCM)")
    Check "initial position" (WaitName "position" "0:00 / 0:16  ·  Paused")
    Check "loaded status" (WaitNameLike "status" "Loaded Demo melody*44100 Hz*")
    $bmp0 = Shot "mp-initial"

    # 1. Seek buttons (mouse)
    ClickId "seek-forward"
    Check "Forward 5 seconds" (WaitName "position" "0:05 / 0:16  ·  Paused")
    ClickId "seek-forward"; ClickId "seek-back"
    Check "Back 5 seconds" (WaitName "position" "0:05 / 0:16  ·  Paused")

    # 2. Waveform: drag with the real mouse to ~75 % (12 s) and click near the start
    $r = (Find "waveform").Current.BoundingRectangle
    $y = [int]($r.Y + $r.Height / 2)
    $x12 = [int]($r.X + 45 + ($r.Width - 57) * 12.3 / 16)
    $x2 = [int]($r.X + 45 + ($r.Width - 57) * 2.3 / 16)
    [AW]::Drag($x2, $y, $x12, $y, 12)
    Check "drag seeks to 0:12" (WaitName "position" "0:12 / 0:16  ·  Paused")
    Check "slider value follows (UIA)" ((ValueOf "waveform") -eq "0:12")
    [AW]::Click($x2, $y)
    Check "click seeks to 0:02" (WaitName "position" "0:02 / 0:16  ·  Paused")
    # Drag into the left margin (left of the plot area) clamps to 0:00
    [AW]::Drag($x2, $y, [int]($r.X + 5), $y, 8)
    Check "drag beyond the plot's left edge clamps to the start" (WaitName "position" "0:00 / 0:16  ·  Paused")
    $bmp = Shot "mp-after-drag"

    # 3. Keyboard: the waveform is focusable; arrows seek
    Check "waveform has focus after the click" ((Focused) -eq "waveform")
    [AW]::Key(0x27)  # Right
    Check "ArrowRight seeks +5 s" (WaitName "position" "0:05 / 0:16  ·  Paused")
    [AW]::Key(0x25)  # Left
    Check "ArrowLeft seeks -5 s" (WaitName "position" "0:00 / 0:16  ·  Paused")
    # UIA RangeValue/Invoke is not exposed for Increment; use the slider name/role
    Check "waveform is a slider for UIA" ((Find "waveform").Current.ControlType.ProgrammaticName -eq "ControlType.Slider")

    # 4. Real playback: position advances, spectrum reacts, pause stops
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    ClickId "play-pause"
    Check "playing status" (WaitName "status" "Playing")
    Check "button label becomes Pause" (WaitName "play-pause" "Pause")
    for ($k = 0; $k -lt 5; $k++) { Start-Sleep -Milliseconds 500; Log ("  t={0:n2}s {1}" -f $sw.Elapsed.TotalSeconds, (Pos)) }
    $p1 = Pos
    $t1 = $sw.Elapsed.TotalSeconds
    $shown = [int](($p1 -split " ")[0].Split(":")[1])
    Log ("  position after {0:n2} s: {1}" -f $t1, $p1)
    Check "position advances in real time while playing (±1 s)" ([math]::Abs($shown - $t1) -le 1.2 -and $p1 -like "*Playing")
    $bmpPlay = Shot "mp-playing"
    $e0 = SpectrumEnergy $bmp0; $e1 = SpectrumEnergy $bmpPlay
    Log "  spectrum bar pixels: paused=$e0 playing=$e1"
    Check "spectrum bars grow while playing" ($e1 -gt $e0 + 20)
    ClickId "play-pause"
    Check "paused status" (WaitName "status" "Paused")
    $p2 = Pos
    Start-Sleep -Milliseconds 1200
    Check "position frozen while paused" ((Pos) -eq $p2)
    ClickId "stop"
    Check "stop rewinds" (WaitNameLike "position" "0:00 / 0:16*")

    # 5. Load a local WAV (typed path), play it to its natural end
    TypeInto "wav-path" $wav
    ClickId "load-wav"
    Check "WAV title" (WaitName "track-title" "tone-3s.wav")
    Check "WAV status" (WaitName "status" "Loaded tone-3s.wav · 44100 Hz · 2 channel(s)")
    Check "WAV duration" (WaitName "position" "0:00 / 0:03  ·  Paused")
    Shot "mp-wav-loaded" | Out-Null
    ClickId "play-pause"
    Check "WAV plays" (WaitName "status" "Playing")
    Start-Sleep -Milliseconds 3800
    Check "natural end stops playback" (WaitName "status" "Paused" 30)
    $end = Pos
    Log "  position at the end: $end (floor of about 3 s)"
    Check "position at the end" ($end -eq "0:03 / 0:03  ·  Paused" -or $end -eq "0:02 / 0:03  ·  Paused")
    Check "button label back to Play" (WaitName "play-pause" "Play")

    # 6. Errors: a missing file shows the alert; the demo clears it
    TypeInto "wav-path" "C:\no\such\file.wav"
    ClickId "load-wav"
    Check "missing file alert" (WaitNameLike "player-error" "Cannot read WAV*")
    Shot "mp-error" | Out-Null
    ClickId "load-demo"
    Check "demo restored" (WaitName "track-title" "Demo melody (generated PCM)")
    Check "error cleared" (WaitGone "player-error")

    # Probe (not counted): release the mouse outside the window. A host that
    # got no button-up would keep its pointer capture and the app the drag,
    # so the next plain mouse move over the window would seek.
    InvokeId "stop"
    $r = (Find "waveform").Current.BoundingRectangle; $y = [int]($r.Y + $r.Height / 2)
    $wr = New-Object AW+RECT; [AW]::GetWindowRect($script:hwnd, [ref]$wr) | Out-Null
    [AW]::Drag([int]($r.X + 60), $y, [int]($wr.L - 150), $y, 10)
    Start-Sleep -Milliseconds 300
    $before = Pos
    $c = Center "seek-forward"; [AW]::SetCursorPos($c[0], $c[1]); Start-Sleep -Milliseconds 100
    $c = Center "play-pause"; [AW]::SetCursorPos($c[0], $c[1]); Start-Sleep -Milliseconds 400
    $after = Pos
    Log "KNOWN-ISSUE probe: release outside the window: position '$before' -> after hovering buttons '$after' (changed = stuck capture)"
    [AW]::Click($c[0], $c[1]); Start-Sleep -Milliseconds 200; InvokeId "stop"

    # 7. Resize: the canvases follow the window width
    $w0 = (Find "waveform").Current.BoundingRectangle.Width
    [AW]::SetWindowPos($script:hwnd, [IntPtr]::Zero, 0, 0, 700, 700, 0x0002 -bor 0x0004) | Out-Null
    Start-Sleep -Milliseconds 700
    $w1 = (Find "waveform").Current.BoundingRectangle.Width
    Log "  waveform width: $w0 -> $w1"
    Check "resize re-lays out the waveform" ($w1 -lt $w0 - 150)
    Shot "mp-resized" | Out-Null
    $code = Stop-App
    Check "graceful close exits with code 0" ($code -eq 0)
} catch {
    Log "ERROR $_ $($_.ScriptStackTrace)"
    $script:failed++
    Kill-App
} finally {
    Exit-GuiLock
    Log "passed=$script:passed failed=$script:failed"
}
exit $script:failed
