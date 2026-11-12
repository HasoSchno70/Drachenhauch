# Sucht ein Fenster ueber seinen Titel und nennt den Titel seines Besitzers
# (GetWindow GW_OWNER) -- "besitzer <titel>" oder "besitzer keiner". Ein
# besessenes Fenster bleibt unter Windows ueber seinem Besitzer und hat keinen
# eigenen Eintrag in der Taskleiste (modaler Dialog, Weg E Stufe 2).
param([string]$Titel)
Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class FBes {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string k, string t);
  [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr h, uint c);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
}
"@
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 150 -and $h -eq [IntPtr]::Zero; $i++) {
    # $null kaeme als "" an -- und sucht dann eine Fensterklasse "".
    $h = [FBes]::FindWindow([NullString]::Value, $Titel)
    if ($h -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 100 }
}
if ($h -eq [IntPtr]::Zero) { "kein Fenster"; exit 2 }
# Ein paar Bilder Zeit: der Besitzer wird im Abgleich der gui gesetzt.
Start-Sleep -Milliseconds 900
$o = [FBes]::GetWindow($h, 4)
if ($o -eq [IntPtr]::Zero) { "besitzer keiner"; exit 0 }
$s = New-Object System.Text.StringBuilder 256
[FBes]::GetWindowText($o, $s, 256) | Out-Null
"besitzer " + $s.ToString()
