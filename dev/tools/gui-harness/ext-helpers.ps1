# Extra helpers for the ecosystem (external-component apps / workbench) scenarios.
# Dot-sourced by scenario-*.ps1 files that run inside gui.ps1 (which defines
# $hwnd, Root/All/Find/NameOf/Center/Shot/Check/Log/Dump and the [W] type).
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class W2 {
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint data, flags, time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort vk, scan; public uint flags, time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct INPUT {
    [FieldOffset(0)] public uint type;
    [FieldOffset(8)] public MOUSEINPUT mi;
    [FieldOffset(8)] public KEYBDINPUT ki;
  }
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll", SetLastError=true)] public static extern uint SendInput(uint n, INPUT[] i, int size);
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
  public static void Drag(int x0, int y0, int x1, int y1, int steps) {
    PointTo(x0, y0); System.Threading.Thread.Sleep(60);
    Mouse(0x0002); System.Threading.Thread.Sleep(60);
    for (int i = 1; i <= steps; i++) {
      SetCursorPos(x0 + (x1 - x0) * i / steps, y0 + (y1 - y0) * i / steps);
      Mouse(0x0001);
      System.Threading.Thread.Sleep(25);
    }
    System.Threading.Thread.Sleep(60);
    Mouse(0x0004);
  }
  public static void KeyDown(ushort vk) { var a = new INPUT[1]; a[0].type = 1; a[0].ki.vk = vk; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
  public static void KeyUp(ushort vk) { var a = new INPUT[1]; a[0].type = 1; a[0].ki.vk = vk; a[0].ki.flags = 0x0002; SendInput(1, a, Marshal.SizeOf(typeof(INPUT))); }
}
"@ -ErrorAction SilentlyContinue

function El-All { (Root).FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition) }

# Find elements by optional regexes on AutomationId / Name / ControlType / Value.
function El-Where([string]$Id, [string]$Name, [string]$Control, [string]$Value, $Scope) {
    $base = if ($Scope) { $Scope } else { Root }
    $all = $base.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
    $out = @()
    foreach ($e in $all) {
        try {
            $c = $e.Current
            if ($Id -and ($c.AutomationId -notmatch $Id)) { continue }
            if ($Name -and ($c.Name -notmatch $Name)) { continue }
            if ($Control -and ($c.ControlType.ProgrammaticName -notmatch $Control)) { continue }
            if ($Value) {
                $v = $null
                try { $v = $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value } catch {}
                if ($v -notmatch $Value) { continue }
            }
            $out += $e
        } catch {}
    }
    # Unrolled on purpose: pipelines (Sort-Object, Where-Object) see elements.
    return $out
}

function El-One([string]$Id, [string]$Name, [string]$Control, [string]$Value, $Scope, [int]$Tries = 40) {
    for ($k = 0; $k -lt $Tries; $k++) {
        $m = El-Where $Id $Name $Control $Value $Scope
        if ($m.Count -ge 1) { return $m[0] }
        Start-Sleep -Milliseconds 100
    }
    throw "no element id='$Id' name='$Name' control='$Control' value='$Value'"
}

function El-Wait([string]$Id, [string]$Name, [string]$Control, [string]$Value, [int]$Tries = 60) {
    for ($k = 0; $k -lt $Tries; $k++) {
        $m = El-Where $Id $Name $Control $Value
        if ($m.Count -ge 1) { return $true }
        Start-Sleep -Milliseconds 100
    }
    return $false
}

function El-WaitAbsent([string]$Id, [string]$Name, [string]$Control, [int]$Tries = 60) {
    for ($k = 0; $k -lt $Tries; $k++) {
        $m = El-Where $Id $Name $Control
        if ($m.Count -eq 0) { return $true }
        Start-Sleep -Milliseconds 100
    }
    return $false
}

function El-Rect($e) { $e.Current.BoundingRectangle }
function El-Center($e) { $r = El-Rect $e; @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)) }
function El-Click($e) { $c = El-Center $e; [W]::Click($c[0], $c[1]); Start-Sleep -Milliseconds 150 }
function El-Value($e) { try { $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value } catch { $null } }
function El-Invoke($e) { $e.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke(); Start-Sleep -Milliseconds 150 }
function Focused-Id { [System.Windows.Automation.AutomationElement]::FocusedElement.Current.AutomationId }
function Focused-Name { [System.Windows.Automation.AutomationElement]::FocusedElement.Current.Name }

# Drag across an element horizontally between two fractions of its width.
function El-DragX($e, [double]$from, [double]$to, [double]$yFrac = 0.5) {
    $r = El-Rect $e
    $y = [int]($r.Y + $r.Height * $yFrac)
    [W2]::Drag([int]($r.X + $r.Width * $from), $y, [int]($r.X + $r.Width * $to), $y, 12)
    Start-Sleep -Milliseconds 250
}

function Pixel-At([int]$x, [int]$y) {
    $bmp = New-Object System.Drawing.Bitmap 1, 1
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($x, $y, 0, 0, $bmp.Size)
    $g.Dispose()
    $c = $bmp.GetPixel(0, 0); $bmp.Dispose(); return $c
}

function Count-Where([string]$Control, [string]$Name) { (El-Where -Control $Control -Name $Name).Count }
