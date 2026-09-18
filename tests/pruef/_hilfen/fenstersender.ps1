# Sucht ein Fenster ueber seinen Titel und schickt ihm eine echte Nachricht:
# "esc" = WM_KEYDOWN + WM_KEYUP mit VK_ESCAPE, "kreuz" = WM_CLOSE (was das
# Kreuz der Titelleiste und Alt+F4 ausloesen). Aufrufer: fenstersender.dh.
param([string]$Titel, [string]$Nachricht)
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class FensterSender {
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
}
"@
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 150 -and $h -eq [IntPtr]::Zero; $i++) {
    $p = Get-Process | Where-Object { $_.MainWindowTitle -eq $Titel } | Select-Object -First 1
    if ($p -and $p.MainWindowHandle -ne 0) { $h = $p.MainWindowHandle }
    else { Start-Sleep -Milliseconds 100 }
}
if ($h -eq [IntPtr]::Zero) { "kein Fenster"; exit 2 }
# Das Fenster ist da, bevor die Bildschleife laeuft -- etwas warten.
Start-Sleep -Milliseconds 700
if ($Nachricht -eq "esc") {
    [FensterSender]::PostMessage($h, 0x100, [IntPtr]0x1B, [IntPtr]0x00010001) | Out-Null
    Start-Sleep -Milliseconds 80
    [FensterSender]::PostMessage($h, 0x101, [IntPtr]0x1B, [IntPtr]0xC0010001) | Out-Null
} elseif ($Nachricht -eq "kreuz") {
    [FensterSender]::PostMessage($h, 0x10, [IntPtr]0, [IntPtr]0) | Out-Null
} else { "unbekannte Nachricht $Nachricht"; exit 3 }
"gesendet"
