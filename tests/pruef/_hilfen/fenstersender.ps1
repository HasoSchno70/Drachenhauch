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
} elseif ($Nachricht.StartsWith("klicks:")) {
    # "klicks:N:MS" = N Linksklicks auf (150,100), zwischen Druecken und
    # Loslassen MS Millisekunden (genau gewartet, Start-Sleep hat ~15 ms
    # Raster). MS = 0 heisst: beide Nachrichten gleich hintereinander -- dann
    # kommen sie im selben Bild an (flanken.rs).
    $teile = $Nachricht.Split(":")
    $n = [int]$teile[1]; $ms = [double]$teile[2]
    $l = [IntPtr]((100 -shl 16) -bor 150)
    for ($k = 0; $k -lt $n; $k++) {
        [FensterSender]::PostMessage($h, 0x200, [IntPtr]0, $l) | Out-Null
        Start-Sleep -Milliseconds 100
        [FensterSender]::PostMessage($h, 0x201, [IntPtr]1, $l) | Out-Null
        $sw = [Diagnostics.Stopwatch]::StartNew()
        while ($sw.Elapsed.TotalMilliseconds -lt $ms) { }
        [FensterSender]::PostMessage($h, 0x202, [IntPtr]0, $l) | Out-Null
        Start-Sleep -Milliseconds 250
    }
} elseif ($Nachricht.StartsWith("tasten:")) {
    # "tasten:N:MS" = N mal die Taste A (WM_KEYDOWN/WM_KEYUP; GLFW nimmt den
    # Scancode 0x1E aus dem lParam), MS Millisekunden gehalten, genau
    # gewartet. MS = 0: beide gleich hintereinander, also im selben Bild.
    $teile = $Nachricht.Split(":")
    $n = [int]$teile[1]; $ms = [double]$teile[2]
    for ($k = 0; $k -lt $n; $k++) {
        [FensterSender]::PostMessage($h, 0x100, [IntPtr]0x41, [IntPtr]0x001E0001) | Out-Null
        $sw = [Diagnostics.Stopwatch]::StartNew()
        while ($sw.Elapsed.TotalMilliseconds -lt $ms) { }
        [FensterSender]::PostMessage($h, 0x101, [IntPtr]0x41, [IntPtr]0xC01E0001) | Out-Null
        Start-Sleep -Milliseconds 250
    }
} else {
    # Eine Folge, getrennt mit "|": "tippe:abc" = je Zeichen WM_CHAR (dieser
    # Weg fuellt raylibs Zeichenwarteschlange, eine Aufnahme tut es nicht),
    # "anf" = ein Anfuehrungszeichen (auf der Befehlszeile schwer zu
    # uebergeben), "rueck"/"links"/"rechts"/"auf"/"ab"/"enter"/"escape"/"f12" = die
    # Taste gedrueckt und los ("esc" allein ist die Nachricht oben).
    foreach ($teil in $Nachricht.Split("|")) {
        if ($teil.StartsWith("tippe:")) {
            foreach ($z in $teil.Substring(6).ToCharArray()) {
                [FensterSender]::PostMessage($h, 0x102, [IntPtr][int]$z, [IntPtr]1) | Out-Null
                Start-Sleep -Milliseconds 60
            }
        } elseif ($teil -eq "anf") {
            [FensterSender]::PostMessage($h, 0x102, [IntPtr]0x22, [IntPtr]1) | Out-Null
        } elseif (@("rueck", "links", "rechts", "auf", "ab", "enter", "escape", "f12") -contains $teil) {
            $vk = @{ "rueck" = 0x08; "links" = 0x25; "rechts" = 0x27; "auf" = 0x26; "ab" = 0x28; "enter" = 0x0D; "escape" = 0x1B; "f12" = 0x7B }[$teil]
            # GLFW nimmt die Taste aus dem SCANCODE im lParam, nicht aus dem vk.
            $ext = @{ "rueck" = 0x000E0001; "links" = 0x014B0001; "rechts" = 0x014D0001; "auf" = 0x01480001; "ab" = 0x01500001; "enter" = 0x001C0001; "escape" = 0x00010001; "f12" = 0x00580001 }[$teil]
            [FensterSender]::PostMessage($h, 0x100, [IntPtr]$vk, [IntPtr]$ext) | Out-Null
            # 160 statt 80 ms: kommen runter UND hoch im selben Bild an (ein
            # langsames Bild unter Last), sieht raylib gar keinen Druck.
            Start-Sleep -Milliseconds 160
            [FensterSender]::PostMessage($h, 0x101, [IntPtr]$vk, [IntPtr]($ext -bor 0xC0000000)) | Out-Null
        } else { "unbekannte Nachricht $teil"; exit 3 }
        Start-Sleep -Milliseconds 150
    }
}
"gesendet"
