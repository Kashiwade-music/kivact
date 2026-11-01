# Shared helpers for the sample-app GUI scenarios (dot-source this file).
# PowerShell 7 in STA mode (UIAutomationClient). Keep this file UTF-8 with BOM.
$ErrorActionPreference = "Stop"
# Read one committed file snapshot without preventing an atomic replacement.
function Read-AtomicJsonSnapshot([string]$Path) {
    $stream = $null
    $reader = $null
    try {
        $share = [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete
        $stream = [IO.FileStream]::new($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, $share)
        $reader = [IO.StreamReader]::new($stream, [Text.Encoding]::UTF8, $true)
        return ($reader.ReadToEnd() | ConvertFrom-Json -ErrorAction Stop)
    } catch {
        $snapshotError = $_.Exception
        while ($null -ne $snapshotError) {
            if ($snapshotError -is [IO.IOException]) {
                $code = $snapshotError.HResult -band 0xffff
                if ($code -in 2, 3, 32, 33) { return $null }
                break
            }
            $snapshotError = $snapshotError.InnerException
        }
        throw
    } finally {
        if ($null -ne $reader) { $reader.Dispose() }
        if ($null -ne $stream) { $stream.Dispose() }
    }
}
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Drawing
if (-not ("AW" -as [type])) {
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class AW {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint data, flags, time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort vk, scan; public uint flags, time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct INPUT {
    [FieldOffset(0)] public uint type;
    [FieldOffset(8)] public MOUSEINPUT mi;
    [FieldOffset(8)] public KEYBDINPUT ki;
  }
  // Input guard: with KIVACT_GUI_GUARD_ROOT set (harness-paths.ps1), keys
  // go out only while the foreground window belongs to a program under it
  // (the app under test), and a button press or wheel only onto such a
  // window; otherwise the run stops instead of typing into another app.
  [DllImport("user32.dll", EntryPoint = "GetForegroundWindow")] static extern IntPtr GuardForeground();
  [DllImport("user32.dll", EntryPoint = "GetWindowThreadProcessId")] static extern uint GuardPid(IntPtr h, out uint pid);
  [DllImport("user32.dll", EntryPoint = "GetCursorPos")] static extern bool GuardCursor(out GuardPoint p);
  [DllImport("user32.dll", EntryPoint = "WindowFromPoint")] static extern IntPtr GuardWindowAt(GuardPoint p);
  [StructLayout(LayoutKind.Sequential)] public struct GuardPoint { public int X, Y; }
  static readonly System.Collections.Generic.Dictionary<uint, bool> guardSeen = new System.Collections.Generic.Dictionary<uint, bool>();
  static bool GuardOurs(IntPtr h) {
    string root = Environment.GetEnvironmentVariable("KIVACT_GUI_GUARD_ROOT");
    if (string.IsNullOrEmpty(root)) return true;
    if (h == IntPtr.Zero) return false;
    uint pid; GuardPid(h, out pid);
    bool ours;
    if (guardSeen.TryGetValue(pid, out ours)) return ours;
    try {
      string path = System.Diagnostics.Process.GetProcessById((int)pid).MainModule.FileName;
      ours = path.StartsWith(System.IO.Path.GetFullPath(root), StringComparison.OrdinalIgnoreCase);
    } catch { ours = false; }
    guardSeen[pid] = ours;
    return ours;
  }
  static void GuardInput(INPUT[] inputs) {
    foreach (var i in inputs) {
      if (i.type == 1 && !GuardOurs(GuardForeground()))
        throw new InvalidOperationException("input guard: the foreground window is not the app under test; no key sent");
      if (i.type == 0 && (i.mi.flags & (0x0002 | 0x0008 | 0x0020 | 0x0080 | 0x0800 | 0x1000)) != 0) {
        GuardPoint p; GuardCursor(out p);
        if (!GuardOurs(GuardWindowAt(p)))
          throw new InvalidOperationException("input guard: the window under the cursor is not the app under test; no press sent");
      }
    }
  }
  [DllImport("user32.dll", EntryPoint = "SendInput", SetLastError=true)] static extern uint SendInputRaw(uint n, INPUT[] i, int size);
  public static uint SendInput(uint n, INPUT[] i, int size) { GuardInput(i); return SendInputRaw(n, i, size); }
  static void Mouse(uint flags) {
    var a = new INPUT[1]; a[0].type = 0; a[0].mi.flags = flags;
    SendInput(1, a, Marshal.SizeOf(typeof(INPUT)));
  }
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int i);
  // SetCursorPos alone reports the move lazily, after a button or wheel
  // message sent right behind it, and winit hosts take the click position from
  // the last WM_MOUSEMOVE. Absolute input moves (away and back, so the cursor
  // really moves) queue the position in order, as a real mouse does.
  static INPUT Abs(int x, int y) {
    var i = new INPUT(); i.type = 0; i.mi.flags = 0x0001 | 0x8000 | 0x4000;
    i.mi.dx = (int)Math.Ceiling((x - GetSystemMetrics(76) + 0.5) * 65536.0 / GetSystemMetrics(78));
    i.mi.dy = (int)Math.Ceiling((y - GetSystemMetrics(77) + 0.5) * 65536.0 / GetSystemMetrics(79));
    return i;
  }
  public static void PointTo(int x, int y) {
    SetCursorPos(x, y);
    var a = new INPUT[] { Abs(x + 4, y), Abs(x, y) };
    SendInput(2, a, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void Click(int x, int y) {
    PointTo(x, y); System.Threading.Thread.Sleep(30);
    Mouse(0x0002); System.Threading.Thread.Sleep(30); Mouse(0x0004);
  }
  public static void Drag(int x0, int y0, int x1, int y1, int steps) {
    PointTo(x0, y0); System.Threading.Thread.Sleep(40);
    Mouse(0x0002);
    for (int i = 1; i <= steps; i++) {
      SetCursorPos(x0 + (x1 - x0) * i / steps, y0 + (y1 - y0) * i / steps);
      Mouse(0x0001);
      System.Threading.Thread.Sleep(25);
    }
    Mouse(0x0004);
  }
  public static void Wheel(int x, int y, int delta) {
    PointTo(x, y);
    var a = new INPUT[1];
    a[0].type = 0; a[0].mi.flags = 0x0800; a[0].mi.data = (uint)delta;
    SendInput(1, a, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void Key(ushort vk) {
    var a = new INPUT[2];
    a[0].type = 1; a[0].ki.vk = vk;
    a[1].type = 1; a[1].ki.vk = vk; a[1].ki.flags = 0x0002;
    SendInput(2, a, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void KeyCombo(ushort mod, ushort vk) {
    var a = new INPUT[4];
    a[0].type = 1; a[0].ki.vk = mod;
    a[1].type = 1; a[1].ki.vk = vk;
    a[2].type = 1; a[2].ki.vk = vk; a[2].ki.flags = 0x0002;
    a[3].type = 1; a[3].ki.vk = mod; a[3].ki.flags = 0x0002;
    SendInput(4, a, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void Text(string s) {
    foreach (char c in s) {
      var a = new INPUT[2];
      a[0].type = 1; a[0].ki.scan = c; a[0].ki.flags = 0x0004;
      a[1].type = 1; a[1].ki.scan = c; a[1].ki.flags = 0x0004 | 0x0002;
      SendInput(2, a, Marshal.SizeOf(typeof(INPUT)));
      System.Threading.Thread.Sleep(15);
    }
  }
}
"@
}
[AW]::SetProcessDPIAware() | Out-Null

$script:failed = 0
$script:passed = 0
function Log($m) { $m | Out-File -FilePath $script:log -Append -Encoding utf8; $m | Out-Host }
function Check($label, $cond) {
    if ($cond) { Log "PASS $label"; $script:passed++ } else { Log "FAIL $label"; $script:failed++ }
}

# ---- one GUI test at a time (lock file) -------------------------------------
. (Join-Path $PSScriptRoot "harness-paths.ps1")
$script:lockPath = $script:KivactGuiLock
function Enter-GuiLock($owner) {
    if ($env:KIVACT_GUI_LOCK_HELD) { return }  # a wrapper (rerun-round.ps1) holds the lock for the whole round
    for ($i = 0; $i -lt 120; $i++) {
        try {
            $fs = [System.IO.File]::Open($script:lockPath, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write)
            $b = [System.Text.Encoding]::UTF8.GetBytes("$owner pid=$PID $(Get-Date -Format o)")
            $fs.Write($b, 0, $b.Length); $fs.Close()
            return
        } catch {
            Write-Host "gui.lock busy: $(Get-Content $script:lockPath -ErrorAction SilentlyContinue); waiting 30s"
            Start-Sleep -Seconds 30
        }
    }
    throw "could not take gui.lock"
}
function Exit-GuiLock { if ($env:KIVACT_GUI_LOCK_HELD) { return }; Remove-Item -Force $script:lockPath -ErrorAction SilentlyContinue }

# ---- process ------------------------------------------------------------------
$script:runIndex = 0
function Start-App($exe, $title) {
    $script:runIndex++
    $script:stderr = Join-Path $script:out ("stderr-{0}.txt" -f $script:runIndex)
    $script:proc = Start-Process -FilePath $exe -PassThru -RedirectStandardError $script:stderr
    $null = $script:proc.Handle  # keep the handle so ExitCode is available later
    $script:hwnd = [IntPtr]::Zero
    for ($i = 0; $i -lt 200; $i++) {
        Start-Sleep -Milliseconds 100
        $script:proc.Refresh()
        if ($script:proc.MainWindowHandle -ne [IntPtr]::Zero -and $script:proc.MainWindowTitle -eq $title) { $script:hwnd = $script:proc.MainWindowHandle; break }
    }
    if ($script:hwnd -eq [IntPtr]::Zero) { throw "window '$title' not found" }
    Log "started '$title' pid=$($script:proc.Id) hwnd=$script:hwnd after $($i*100)ms"
    [AW]::SetForegroundWindow($script:hwnd) | Out-Null
    Start-Sleep -Milliseconds 700
}
function Stop-App {
    if ($null -eq $script:proc) { return $null }
    $script:proc.CloseMainWindow() | Out-Null
    $ok = $script:proc.WaitForExit(8000)
    if (-not $ok) { Stop-Process -Id $script:proc.Id -Force; Log "  forced kill"; $code = $null } else { $code = $script:proc.ExitCode }
    Log "closed pid=$($script:proc.Id) exit=$code"
    $script:proc = $null
    return $code
}
function Kill-App {
    if ($null -ne $script:proc) {
        if ($script:proc.HasExited) { Log "process had exited: code=$($script:proc.ExitCode)" }
        Stop-Process -Id $script:proc.Id -Force -ErrorAction SilentlyContinue; $script:proc = $null
    }
    if ($script:stderr -and (Test-Path $script:stderr)) { Log "stderr tail:"; Get-Content $script:stderr -Tail 30 | ForEach-Object { Log "  | $_" } }
}

# ---- UIA ------------------------------------------------------------------------
function Root { [System.Windows.Automation.AutomationElement]::FromHandle($script:hwnd) }
function TryFind($id) {
    $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, $id)
    return (Root).FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
}
function Find($id, $tries = 40) {
    for ($k = 0; $k -lt $tries; $k++) {
        $e = TryFind $id
        if ($e) { return $e }
        Start-Sleep -Milliseconds 100
    }
    throw "UIA element '$id' not found"
}
function NameOf($id) { (Find $id).Current.Name }
function ValueOf($id) {
    try { return (Find $id).GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value } catch { return $null }
}
function Center($id) {
    $r = (Find $id).Current.BoundingRectangle
    return @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
}
function ClickId($id) { $c = Center $id; [AW]::Click($c[0], $c[1]); Start-Sleep -Milliseconds 150 }
function InvokeId($id) {
    (Find $id).GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    Start-Sleep -Milliseconds 150
}
function Focused() { [System.Windows.Automation.AutomationElement]::FocusedElement.Current.AutomationId }
function WaitName($id, $expected, $tries = 50) {
    for ($k = 0; $k -lt $tries; $k++) {
        $e = TryFind $id
        if ($e -and $e.Current.Name -eq $expected) { return $true }
        Start-Sleep -Milliseconds 100
    }
    $n = if ($e) { $e.Current.Name } else { "<missing>" }
    Log "  $id name='$n' expected '$expected'"
    return $false
}
function WaitNameLike($id, $pattern, $tries = 80) {
    for ($k = 0; $k -lt $tries; $k++) {
        $e = TryFind $id
        if ($e -and $e.Current.Name -like $pattern) { return $true }
        Start-Sleep -Milliseconds 100
    }
    $n = if ($e) { $e.Current.Name } else { "<missing>" }
    Log "  $id name='$n' expected like '$pattern'"
    return $false
}
function WaitGone($id, $tries = 30) {
    for ($k = 0; $k -lt $tries; $k++) {
        if (-not (TryFind $id)) { return $true }
        Start-Sleep -Milliseconds 100
    }
    return $false
}
# Replace a text field: click it, select all, delete, type (SendInput unicode).
function TypeInto($id, $text) {
    ClickId $id
    [AW]::KeyCombo(0x11, 0x41)  # Ctrl+A
    [AW]::Key(0x2E)             # Delete
    Start-Sleep -Milliseconds 80
    if ($text.Length -gt 0) { [AW]::Text($text) }
    Start-Sleep -Milliseconds 200
}
function Dump($tag) {
    $els = (Root).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
    Log "--- UIA $tag ($($els.Count) elements)"
    foreach ($e in $els) {
        $c = $e.Current
        $r = $c.BoundingRectangle
        $val = ""
        try { $vp = $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern); $val = " value='" + $vp.Current.Value + "'" } catch {}
        Log ("  {0} name='{1}' id='{2}' enabled={3} rect=({4:n0},{5:n0},{6:n0},{7:n0}){8}" -f $c.ControlType.ProgrammaticName, $c.Name, $c.AutomationId, $c.IsEnabled, $r.X, $r.Y, $r.Width, $r.Height, $val)
    }
}
function Shot($name) {
    $r = New-Object AW+RECT
    [AW]::GetWindowRect($script:hwnd, [ref]$r) | Out-Null
    $w = $r.R - $r.L; $h = $r.B - $r.T
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
    $path = Join-Path $script:out "$name.png"
    $bmp.Save($path)
    $g.Dispose()
    Log "screenshot $path ${w}x${h}"
    return $bmp
}
# Pixel at screen coordinates relative to a fresh screenshot of the window.
function PixelAt($bmp, $sx, $sy) {
    $r = New-Object AW+RECT
    [AW]::GetWindowRect($script:hwnd, [ref]$r) | Out-Null
    return $bmp.GetPixel([int]($sx - $r.L), [int]($sy - $r.T))
}
function FindByName($name, $tries = 30) {
    $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $name)
    for ($k = 0; $k -lt $tries; $k++) {
        $all = (Root).FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
        foreach ($e in $all) {
            $t = $e.Current.ControlType.ProgrammaticName
            if ($t -ne "ControlType.Text") { return $e }
        }
        Start-Sleep -Milliseconds 100
    }
    throw "UIA element named '$name' not found"
}
function ClickEl($e) {
    $r = $e.Current.BoundingRectangle
    [AW]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
    Start-Sleep -Milliseconds 150
}
function FindByNameType($name, $typeName, $tries = 30) {
    $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $name)
    for ($k = 0; $k -lt $tries; $k++) {
        $all = (Root).FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
        foreach ($e in $all) {
            if ($e.Current.ControlType.ProgrammaticName -eq $typeName) { return $e }
        }
        Start-Sleep -Milliseconds 100
    }
    throw "UIA $typeName named '$name' not found"
}
