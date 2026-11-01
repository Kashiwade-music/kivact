param(
    [string]$Exe,
    [string]$Title,
    [string]$Out,
    [string]$Scenario = "dump",
    [string]$AppArguments = ""
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int command);
  [DllImport("user32.dll")] static extern bool BringWindowToTop(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] static extern bool AttachThreadInput(uint from, uint to, bool attach);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
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
  public static bool ActivateOwned(IntPtr h) {
    if (!GuardOurs(h)) throw new InvalidOperationException("activation guard: window is not the app under test");
    ShowWindow(h, 9);
    if (SetForegroundWindow(h) && GetForegroundWindow() == h) return true;
    uint ignored;
    uint current = GetCurrentThreadId();
    uint target = GetWindowThreadProcessId(h, out ignored);
    uint foreground = GetWindowThreadProcessId(GetForegroundWindow(), out ignored);
    bool targetAttached = target != 0 && target != current && AttachThreadInput(current, target, true);
    bool foregroundAttached = foreground != 0 && foreground != current && foreground != target && AttachThreadInput(current, foreground, true);
    try {
      BringWindowToTop(h);
      SetForegroundWindow(h);
      return GetForegroundWindow() == h;
    } finally {
      if (foregroundAttached) AttachThreadInput(current, foreground, false);
      if (targetAttached) AttachThreadInput(current, target, false);
    }
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
    PointTo(x, y);
    var a = new INPUT[2];
    a[0].type = 0; a[0].mi.flags = 0x0002;
    a[1].type = 0; a[1].mi.flags = 0x0004;
    SendInput(2, a, Marshal.SizeOf(typeof(INPUT)));
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
[W]::SetProcessDPIAware() | Out-Null
New-Item -ItemType Directory -Force $Out | Out-Null
$log = Join-Path $Out "log.txt"
function Log($m) { $m | Out-File -FilePath $log -Encoding Unicode -Append; $m | Out-Host }
function FileSha256([string]$path) {
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::OpenRead($path)
    try { return [BitConverter]::ToString($hasher.ComputeHash($stream)).Replace("-", "") }
    finally { $stream.Dispose(); $hasher.Dispose() }
}

$p = if ($AppArguments) {
    Start-Process -FilePath $Exe -ArgumentList $AppArguments -PassThru
} else {
    Start-Process -FilePath $Exe -PassThru
}
$hwnd = [IntPtr]::Zero
for ($i = 0; $i -lt 100; $i++) {
    Start-Sleep -Milliseconds 100
    $p.Refresh()
    if ($p.MainWindowHandle -ne [IntPtr]::Zero -and $p.MainWindowTitle -eq $Title) { $hwnd = $p.MainWindowHandle; break }
}
if ($hwnd -eq [IntPtr]::Zero) { Log "FAIL: window '$Title' not found (title='$($p.MainWindowTitle)')"; Stop-Process -Id $p.Id -Force; exit 1 }
Log "window hwnd=$hwnd pid=$($p.Id) after $($i*100)ms"

function Root { [System.Windows.Automation.AutomationElement]::FromHandle($hwnd) }
function All {
    (Root).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
}
function Dump($tag) {
    $els = All
    Log "--- UIA $tag ($($els.Count) elements)"
    foreach ($e in $els) {
        $c = $e.Current
        $r = $c.BoundingRectangle
        $val = ""
        try { $vp = $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern); $val = " value='" + $vp.Current.Value + "'" } catch {}
        Log ("  {0} name='{1}' id='{2}' enabled={3} focus={4} rect=({5:n0},{6:n0},{7:n0},{8:n0}){9}" -f $c.ControlType.ProgrammaticName, $c.Name, $c.AutomationId, $c.IsEnabled, $c.HasKeyboardFocus, $r.X, $r.Y, $r.Width, $r.Height, $val)
    }
}
function Find($id) {
    $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, $id)
    for ($k = 0; $k -lt 30; $k++) {
        $e = (Root).FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($e) { return $e }
        Start-Sleep -Milliseconds 100
    }
    throw "UIA element '$id' not found"
}
function NameOf($id) { (Find $id).Current.Name }
function Center($id) {
    $r = (Find $id).Current.BoundingRectangle
    return @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2))
}
function Shot($name) {
    $r = New-Object W+RECT
    [W]::GetWindowRect($hwnd, [ref]$r) | Out-Null
    $w = $r.R - $r.L; $h = $r.B - $r.T
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
    $path = Join-Path $Out "$name.png"
    $bmp.Save($path)
    $g.Dispose()
    Log "screenshot $path ${w}x${h}"
    return $bmp
}
function Check($label, $cond) {
    if ($cond) { Log "PASS $label" } else { Log "FAIL $label"; $script:failed++ }
}
$script:failed = 0

try {
    if (-not [W]::ActivateOwned($hwnd)) { throw "The owned test window could not take the foreground; no input sent" }
    Start-Sleep -Milliseconds 800
    Dump "initial"
    $bmp = Shot "initial"
    . (Join-Path $PSScriptRoot "scenario-$Scenario.ps1")
} catch {
    Log "ERROR $_"
    $script:failed++
} finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    Log "failed=$script:failed"
}
exit $script:failed
