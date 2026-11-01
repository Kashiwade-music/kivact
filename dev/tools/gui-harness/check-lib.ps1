# Extra-check helpers shared by the apps-lib and gui.ps1 based scenarios.
# Requires: $script:hwnd (window handle), Log, Check, Root, Shot.
# Keep this file UTF-8 with BOM.
if (-not ("FX" -as [type])) {
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class FX {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
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
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", SetLastError=true)] public static extern IntPtr SendMessageTimeout(IntPtr h, uint msg, IntPtr w, IntPtr l, uint flags, uint timeout, out IntPtr result);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern uint RegisterWindowMessageW(string name);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
  [DllImport("kernel32.dll")] public static extern IntPtr OpenProcess(uint access, bool inherit, uint pid);
  [DllImport("kernel32.dll")] public static extern IntPtr VirtualAllocEx(IntPtr p, IntPtr addr, UIntPtr size, uint type, uint protect);
  [DllImport("kernel32.dll")] public static extern bool VirtualFreeEx(IntPtr p, IntPtr addr, UIntPtr size, uint type);
  [DllImport("kernel32.dll")] public static extern bool WriteProcessMemory(IntPtr p, IntPtr addr, byte[] buf, UIntPtr size, out UIntPtr written);
  [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr h);
  static void Mouse(uint flags) {
    var a = new INPUT[1]; a[0].type = 0; a[0].mi.flags = flags;
    SendInput(1, a, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void Down() { Mouse(0x0002); }
  public static void Up() { Mouse(0x0004); }
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
  public static void MoveTo(int x, int y) { PointTo(x, y); }
  public static void KeyDown(ushort vk) { var a = new INPUT[1]; a[0].type = 1; a[0].ki.vk = vk; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
  public static void KeyUp(ushort vk) { var a = new INPUT[1]; a[0].type = 1; a[0].ki.vk = vk; a[0].ki.flags = 0x0002; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
  public static void Key(ushort vk) { KeyDown(vk); System.Threading.Thread.Sleep(10); KeyUp(vk); }
  // Synthetic WM_DPICHANGED: the suggested RECT must live in the target
  // process (lParam is a pointer), so it is written there first.
  public static int LastError;
  public static bool SendDpiChanged(IntPtr hwnd, int dpi, int l, int t, int r, int b) {
    uint pid; GetWindowThreadProcessId(hwnd, out pid);
    IntPtr proc = OpenProcess(0x0008 | 0x0020 | 0x0010, false, pid);
    if (proc == IntPtr.Zero) return false;
    var bytes = new byte[16];
    BitConverter.GetBytes(l).CopyTo(bytes, 0); BitConverter.GetBytes(t).CopyTo(bytes, 4);
    BitConverter.GetBytes(r).CopyTo(bytes, 8); BitConverter.GetBytes(b).CopyTo(bytes, 12);
    IntPtr mem = VirtualAllocEx(proc, IntPtr.Zero, (UIntPtr)16, 0x3000, 0x04);
    UIntPtr w; WriteProcessMemory(proc, mem, bytes, (UIntPtr)16, out w);
    IntPtr res;
    IntPtr ok = SendMessageTimeout(hwnd, 0x02E0, (IntPtr)((dpi << 16) | dpi), mem, 0x0002, 5000, out res);
    LastError = Marshal.GetLastWin32Error();
    bool sent = ok != IntPtr.Zero;
    if (!sent) {
      // Cross-process SendMessage of WM_DPICHANGED is refused: post it and
      // leave the RECT allocated (a few bytes in the test app).
      sent = PostMessage(hwnd, 0x02E0, (IntPtr)((dpi << 16) | dpi), mem);
      LastError = Marshal.GetLastWin32Error();
      CloseHandle(proc);
      return sent;
    }
    VirtualFreeEx(proc, mem, UIntPtr.Zero, 0x8000);
    CloseHandle(proc);
    return sent;
  }
}
"@
}

function Fx-WinRect { $r = New-Object FX+RECT; [FX]::GetWindowRect($script:hwnd, [ref]$r) | Out-Null; return $r }

# Sum of pixel values in a screen rectangle (detects redraws).
function Fx-RegionSum($x, $y, $w, $h) {
    $w = [Math]::Max(1, [int]$w); $h = [Math]::Max(1, [int]$h)
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$x, [int]$y, 0, 0, $bmp.Size)
    $g.Dispose()
    $sum = 0
    for ($i = 0; $i -lt $w; $i += 2) { for ($j = 0; $j -lt $h; $j += 2) { $p = $bmp.GetPixel($i, $j); $sum += $p.R + 2 * $p.G + 3 * $p.B } }
    $bmp.Dispose()
    return $sum
}
function Fx-ElSum($e) { $r = $e.Current.BoundingRectangle; return Fx-RegionSum $r.X $r.Y $r.Width $r.Height }
function Fx-ElValue($e) { try { $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value } catch { $null } }
function Fx-ElCenter($e) { $r = $e.Current.BoundingRectangle; @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)) }
function Fx-Click($e) { $c = Fx-ElCenter $e; [FX]::MoveTo($c[0], $c[1]); Start-Sleep -Milliseconds 30; [FX]::Down(); Start-Sleep -Milliseconds 30; [FX]::Up(); Start-Sleep -Milliseconds 200 }
function Fx-FocusedEl { [System.Windows.Automation.AutomationElement]::FocusedElement }

# ---- IME (Microsoft IME, Japanese) -------------------------------------------------
function Fx-ImeOn { [FX]::Key(0x16); Start-Sleep -Milliseconds 300 }   # VK_IME_ON
function Fx-ImeOff { [FX]::Key(0x1A); Start-Sleep -Milliseconds 200 }  # VK_IME_OFF
function Fx-Romaji([string]$s) { foreach ($ch in $s.ToCharArray()) { [FX]::Key([uint16][char]::ToUpper($ch)); Start-Sleep -Milliseconds 60 }; Start-Sleep -Milliseconds 400 }

# Composition redraw + commit on a focused text field element $e.
# Returns the committed value.
function Fin-Ime($e, [string]$romaji, [string]$expect, [string]$tag) {
    Fx-Click $e
    $before = Fx-ElSum $e
    $v0 = Fx-ElValue $e
    Fx-ImeOn
    Fx-Romaji $romaji
    $during = Fx-ElSum $e
    $vd = Fx-ElValue $e
    Shot "$tag-ime-composing" | Out-Null
    Log "  IME composing: region sum $before -> $during, UIA value '$v0' -> '$vd'"
    Check "IME: preedit is drawn while composing (field redrawn)" ($during -ne $before)
    [FX]::Key(0x0D); Start-Sleep -Milliseconds 500   # Enter commits the hiragana
    Fx-ImeOff
    $v1 = Fx-ElValue $e
    Log "  IME committed value: '$v1'"
    Check "IME: commit writes '$expect'" ($v1 -like "*$expect*")
    Shot "$tag-ime-committed" | Out-Null
    return $v1
}

# Reconversion: select the committed text and press VK_CONVERT. With support
# the IME reopens the composition (candidates) for the selection and Enter
# commits a converted text; without, the selection is untouched.
function Fin-Reconvert($e, [string]$committed, [string]$tag) {
    Fx-Click $e
    [FX]::KeyDown(0x11); [FX]::Key(0x41); [FX]::KeyUp(0x11); Start-Sleep -Milliseconds 200   # Ctrl+A
    Fx-ImeOn
    $before = Fx-ElSum $e
    [FX]::Key(0x1C); Start-Sleep -Milliseconds 700   # VK_CONVERT (変換) = reconversion
    $during = Fx-ElSum $e
    Shot "$tag-ime-reconvert" | Out-Null
    [FX]::Key(0x1B); Start-Sleep -Milliseconds 300   # Escape: leave any composition
    Fx-ImeOff
    $v = Fx-ElValue $e
    Log "  reconversion: region sum $before -> $during, value after Escape '$v'"
    Check "IME: reconversion (変換 on a committed selection) reopens the composition" ($during -ne $before)
}

# Clipboard both ways on a text field element.
function Fin-Clipboard($e, [string]$text, [string]$tag) {
    Set-Clipboard -Value $text
    Fx-Click $e
    [FX]::KeyDown(0x11); [FX]::Key(0x41); [FX]::KeyUp(0x11); Start-Sleep -Milliseconds 100
    [FX]::KeyDown(0x11); [FX]::Key(0x56); [FX]::KeyUp(0x11); Start-Sleep -Milliseconds 400   # Ctrl+V
    $v = Fx-ElValue $e
    Log "  pasted value: '$v'"
    Check "clipboard: Ctrl+V pastes the OS clipboard" ($v -eq $text)
    Set-Clipboard -Value "(cleared)"
    [FX]::KeyDown(0x11); [FX]::Key(0x41); [FX]::KeyUp(0x11); Start-Sleep -Milliseconds 100
    [FX]::KeyDown(0x11); [FX]::Key(0x43); [FX]::KeyUp(0x11); Start-Sleep -Milliseconds 400   # Ctrl+C
    $c = Get-Clipboard -Raw
    Check "clipboard: Ctrl+C copies the selection to the OS" ($c -eq $text)
}

# DPI change. Every monitor here is 96 DPI, Windows refuses a cross-process
# WM_DPICHANGED (SendMessage fails, PostMessage returns ERROR_MESSAGE_SYNC_ONLY
# 1159) and the display's scale is a system setting. A debug build of the
# Arvane host answers the registered message ArvaneDebugDpiChange (wParam: the
# new DPI) by sending its own window the WM_DPICHANGED of a move to a monitor
# of that DPI, through winit's real handling. Counted when the host answers
# (the window grows); logged, not counted, otherwise (a release build, which
# does not register the message).
function Fin-Dpi($probe, [string]$tag) {
    $msg = [FX]::RegisterWindowMessageW("ArvaneDebugDpiChange")
    $r = Fx-WinRect
    $p = $probe.Current.BoundingRectangle
    [FX]::PostMessage($script:hwnd, $msg, [IntPtr]144, [IntPtr]::Zero) | Out-Null
    Start-Sleep -Milliseconds 1200
    $r2 = Fx-WinRect
    $p2 = $probe.Current.BoundingRectangle
    $w = $r.R - $r.L; $w2 = $r2.R - $r2.L
    if ($w2 -eq $w) {
        Log "  NOT-COUNTED DPI change: the host ignores ArvaneDebugDpiChange (window ${w}px wide)"
        return
    }
    Shot "$tag-dpi-144" | Out-Null
    Log ("  DPI 96 -> 144: window {0}x{1} -> {2}x{3}; probe {4:n1}x{5:n1} -> {6:n1}x{7:n1}" -f $w, ($r.B - $r.T), $w2, ($r2.B - $r2.T), $p.Width, $p.Height, $p2.Width, $p2.Height)
    Check "DPI change to 144: the probe's UIA box is 1.5 times as large" ([Math]::Abs($p2.Width - 1.5 * $p.Width) -le 2 -and [Math]::Abs($p2.Height - 1.5 * $p.Height) -le 2)
    [FX]::PostMessage($script:hwnd, $msg, [IntPtr]96, [IntPtr]::Zero) | Out-Null
    Start-Sleep -Milliseconds 1200
    $p3 = $probe.Current.BoundingRectangle
    Shot "$tag-dpi-96" | Out-Null
    Check "DPI change back to 96: the probe's UIA box is as before" ([Math]::Abs($p3.Width - $p.Width) -le 1 -and [Math]::Abs($p3.Height - $p.Height) -le 1)
    [FX]::SetWindowPos($script:hwnd, [IntPtr]::Zero, $r.L, $r.T, $w, $r.B - $r.T, 0x0004) | Out-Null
    Start-Sleep -Milliseconds 600
}

# Move to the second monitor and back (same DPI here); the probe stays clickable.
function Fin-Monitor($probe, [string]$tag) {
    $r = Fx-WinRect
    [FX]::SetWindowPos($script:hwnd, [IntPtr]::Zero, 2600, 10, 0, 0, 0x0001 -bor 0x0004) | Out-Null
    Start-Sleep -Milliseconds 800
    $p = $probe.Current.BoundingRectangle
    Log ("  on monitor 2: probe at ({0:n0},{1:n0})" -f $p.X, $p.Y)
    Check "moved to monitor 2: UIA rects follow" ($p.X -ge 2560)
    [FX]::SetWindowPos($script:hwnd, [IntPtr]::Zero, $r.L, $r.T, 0, 0, 0x0001 -bor 0x0004) | Out-Null
    Start-Sleep -Milliseconds 800
}

# Resize the window; the probe's width follows.
function Fin-Resize($probe, [int]$w, [int]$h) {
    $r = Fx-WinRect
    $w0 = $probe.Current.BoundingRectangle.Width
    [FX]::SetWindowPos($script:hwnd, [IntPtr]::Zero, 0, 0, $w, $h, 0x0002 -bor 0x0004) | Out-Null
    Start-Sleep -Milliseconds 800
    $w1 = $probe.Current.BoundingRectangle.Width
    Log ("  resize to {0}x{1}: probe width {2:n0} -> {3:n0}" -f $w, $h, $w0, $w1)
    Check "resize re-lays out ($w x $h)" ($w1 -ne $w0)
    [FX]::SetWindowPos($script:hwnd, [IntPtr]::Zero, 0, 0, $r.R - $r.L, $r.B - $r.T, 0x0002 -bor 0x0004) | Out-Null
    Start-Sleep -Milliseconds 800
}

# Press inside at (x0,y0), drag out of the window to the left and release there.
function Fx-DragOutLeft([int]$x0, [int]$y0, [int]$steps = 10) {
    $wr = Fx-WinRect
    $x1 = $wr.L - 120
    [FX]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 60
    [FX]::Down(); Start-Sleep -Milliseconds 60
    for ($i = 1; $i -le $steps; $i++) { [FX]::MoveTo($x0 + ($x1 - $x0) * $i / $steps, $y0); Start-Sleep -Milliseconds 30 }
    Start-Sleep -Milliseconds 100
    [FX]::Up(); Start-Sleep -Milliseconds 200
}
