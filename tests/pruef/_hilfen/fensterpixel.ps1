# Sucht ein Fenster ueber seinen Titel und liest Farben aus seinem INHALT
# (Client-Bereich, ohne Rahmen und Titelleiste). Punkte als "x,y;x,y" in
# Client-Koordinaten; Ausgabe je Punkt eine Zeile "x,y #RRGGBB". Fuer
# Fenster, die kein Programm selbst fotografieren kann -- etwa das zweite,
# native Fenster eines Programms (GUI_WINDOW_NATIVE, Weg E).
#
# Gelesen wird mit PrintWindow(PW_RENDERFULLCONTENT): das Fenster zeichnet
# sich selbst in ein Bild, auch wenn ein anderes davor liegt. Bis 2026-11-12
# kam das Bild vom Bildschirm (CopyFromScreen) -- lief nebenher ein
# Vollbildprogramm, las der Fall dessen Farben.
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
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
}
"@
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 150 -and $h -eq [IntPtr]::Zero; $i++) {
    # $null kaeme als "" an -- und sucht dann eine Fensterklasse "".
    $h = [FP]::FindWindow([NullString]::Value, $Titel)
    if ($h -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 100 }
}
if ($h -eq [IntPtr]::Zero) { "kein Fenster"; exit 2 }
# Ein paar Bilder Zeit, bis es gezeichnet ist.
Start-Sleep -Milliseconds 900
$w = New-Object FP+RECT
[FP]::GetWindowRect($h, [ref]$w) | Out-Null
$o = New-Object FP+POINT
[FP]::ClientToScreen($h, [ref]$o) | Out-Null
$b = [Math]::Max(1, $w.R - $w.L); $hh = [Math]::Max(1, $w.B - $w.T)
$bmp = New-Object System.Drawing.Bitmap $b, $hh
$gr = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $gr.GetHdc()
# 2 = PW_RENDERFULLCONTENT (auch OpenGL-/DirectX-Inhalt, seit Windows 8.1)
[FP]::PrintWindow($h, $dc, 2) | Out-Null
$gr.ReleaseHdc($dc)
# Client-Ursprung innerhalb des ganzen Fensters
$dx = $o.X - $w.L; $dy = $o.Y - $w.T
foreach ($p in $Punkte.Split(";")) {
    $xy = $p.Split(",")
    $c = $bmp.GetPixel([int]$xy[0] + $dx, [int]$xy[1] + $dy)
    "{0},{1} #{2:X2}{3:X2}{4:X2}" -f $xy[0], $xy[1], $c.R, $c.G, $c.B
}
