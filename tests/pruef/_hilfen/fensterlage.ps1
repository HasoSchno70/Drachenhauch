# Sucht ein Fenster ueber seinen Titel und sagt, ob es vorn liegt (bekommt
# es die Tastatur?), ob ein Punkt in seiner Mitte es trifft (bekommt es
# die Maus?) und ob es fuer die Maus durchlaessig ist. Aufrufer: stillfenster.dhtest.
param([string]$Titel)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class FensterLage {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint art);
  [DllImport("user32.dll")] public static extern IntPtr GetWindowLongPtr(IntPtr h, int nr);
}
"@
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 150 -and $h -eq [IntPtr]::Zero; $i++) {
    $p = Get-Process | Where-Object { $_.MainWindowTitle -eq $Titel } | Select-Object -First 1
    if ($p -and $p.MainWindowHandle -ne 0) { $h = $p.MainWindowHandle }
    else { Start-Sleep -Milliseconds 100 }
}
if ($h -eq [IntPtr]::Zero) { "kein Fenster"; exit 2 }
Start-Sleep -Milliseconds 700
$r = New-Object FensterLage+RECT
[FensterLage]::GetWindowRect($h, [ref]$r) | Out-Null
$pt = New-Object FensterLage+POINT
$pt.X = [int](($r.L + $r.R) / 2); $pt.Y = [int](($r.T + $r.B) / 2)
$w = [FensterLage]::WindowFromPoint($pt)
if ($w -ne [IntPtr]::Zero) { $w = [FensterLage]::GetAncestor($w, 2) }
"vorn " + ([FensterLage]::GetForegroundWindow() -eq $h)
"treffer " + ($w -eq $h)
# WS_EX_TRANSPARENT (0x20): haengt allein am Schalter, waehrend "vorn"
# auch ohne ihn schwankt -- ein neues Fenster bekommt den Vordergrund nicht immer.
"durchlaessig " + (([int64][FensterLage]::GetWindowLongPtr($h, -20) -band 0x20) -ne 0)
