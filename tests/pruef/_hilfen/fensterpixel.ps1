# Sucht ein Fenster ueber seinen Titel, holt es nach vorn und liest Farben
# aus seinem INHALT (Client-Bereich, ohne Rahmen und Titelleiste) vom
# Bildschirm. Punkte als "x,y;x,y" in Client-Koordinaten; Ausgabe je Punkt
# eine Zeile "x,y #RRGGBB". Fuer Fenster, die kein Programm selbst
# fotografieren kann -- etwa das zweite, native Fenster eines Programms
# (GUI_WINDOW_NATIVE, Weg E).
param([string]$Titel, [string]$Punkte)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class FP {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string k, string t);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
"@
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 150 -and $h -eq [IntPtr]::Zero; $i++) {
    # $null kaeme als "" an -- und sucht dann eine Fensterklasse "".
    $h = [FP]::FindWindow([NullString]::Value, $Titel)
    if ($h -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 100 }
}
if ($h -eq [IntPtr]::Zero) { "kein Fenster"; exit 2 }
[FP]::SetForegroundWindow($h) | Out-Null
# Ein paar Bilder Zeit, bis es gezeichnet und vorn ist.
Start-Sleep -Milliseconds 900
$r = New-Object FP+RECT
[FP]::GetClientRect($h, [ref]$r) | Out-Null
$o = New-Object FP+POINT
[FP]::ClientToScreen($h, [ref]$o) | Out-Null
$b = [Math]::Max(1, $r.R); $hh = [Math]::Max(1, $r.B)
$bmp = New-Object System.Drawing.Bitmap $b, $hh
$gr = [System.Drawing.Graphics]::FromImage($bmp)
$gr.CopyFromScreen($o.X, $o.Y, 0, 0, $bmp.Size)
foreach ($p in $Punkte.Split(";")) {
    $xy = $p.Split(",")
    $c = $bmp.GetPixel([int]$xy[0], [int]$xy[1])
    "{0},{1} #{2:X2}{3:X2}{4:X2}" -f $xy[0], $xy[1], $c.R, $c.G, $c.B
}
