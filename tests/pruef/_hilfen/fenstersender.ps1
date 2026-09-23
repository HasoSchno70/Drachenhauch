# Sucht ein Fenster ueber seinen Titel und schickt ihm eine echte Nachricht:
# "esc" = WM_KEYDOWN + WM_KEYUP mit VK_ESCAPE, "kreuz" = WM_CLOSE (was das
# Kreuz der Titelleiste und Alt+F4 ausloesen), "mausweg" = WM_MOUSEMOVE auf
# (200,150), zweieinhalb Sekunden lang alle 15 ms -- genau das schickt Windows
# von selbst, wenn ein Fenster unter dem Zeiger auftaucht oder verschwindet.
# Der Zeiger des Nutzers wird dabei NICHT bewegt. Aufrufer: fenstersender.dh.
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
} elseif ($Nachricht -eq "mausweg") {
    # lParam = (y << 16) | x, in Fensterkoordinaten.
    $l = [IntPtr]((150 -shl 16) -bor 200)
    $ende = (Get-Date).AddMilliseconds(2500)
    while ((Get-Date) -lt $ende) {
        [FensterSender]::PostMessage($h, 0x200, [IntPtr]0, $l) | Out-Null
        Start-Sleep -Milliseconds 15
    }
} else {
    # Eine Folge, getrennt mit "|": "tippe:abc" = je Zeichen WM_CHAR (dieser
    # Weg fuellt raylibs Zeichenwarteschlange, eine Aufnahme tut es nicht),
    # "anf" = ein Anfuehrungszeichen (auf der Befehlszeile schwer zu
    # uebergeben), "rueck"/"links"/"rechts" = die Taste gedrueckt und los.
    foreach ($teil in $Nachricht.Split("|")) {
        if ($teil.StartsWith("tippe:")) {
            foreach ($z in $teil.Substring(6).ToCharArray()) {
                [FensterSender]::PostMessage($h, 0x102, [IntPtr][int]$z, [IntPtr]1) | Out-Null
                Start-Sleep -Milliseconds 60
            }
        } elseif ($teil -eq "anf") {
            [FensterSender]::PostMessage($h, 0x102, [IntPtr]0x22, [IntPtr]1) | Out-Null
        } elseif ($teil -eq "rueck" -or $teil -eq "links" -or $teil -eq "rechts") {
            $vk = @{ "rueck" = 0x08; "links" = 0x25; "rechts" = 0x27 }[$teil]
            # GLFW nimmt die Taste aus dem SCANCODE im lParam, nicht aus dem vk.
            $ext = @{ "rueck" = 0x000E0001; "links" = 0x014B0001; "rechts" = 0x014D0001 }[$teil]
            [FensterSender]::PostMessage($h, 0x100, [IntPtr]$vk, [IntPtr]$ext) | Out-Null
            Start-Sleep -Milliseconds 80
            [FensterSender]::PostMessage($h, 0x101, [IntPtr]$vk, [IntPtr]($ext -bor 0xC0000000)) | Out-Null
        } else { "unbekannte Nachricht $teil"; exit 3 }
        Start-Sleep -Milliseconds 150
    }
}
"gesendet"
