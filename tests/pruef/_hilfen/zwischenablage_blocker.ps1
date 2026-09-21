# Die Systemzwischenablage fuer eine Weile FESTHALTEN -- Windows gibt sie
# jeweils nur EINEM Prozess, und wer sie offen hat, sperrt jeden anderen aus.
# Genau das passiert im Alltag staendig (zwei Testlaeufe, ein Passwortmanager,
# ein Zwischenablage-Verlauf) und laesst ein ungeschuetztes Setzen STILL
# scheitern.
#
# Aufruf: zwischenablage_blocker.ps1 <markerpfad> <millisekunden>
# Der Marker sagt dem Wartenden, dass sie WIRKLICH gehalten wird -- ohne ihn
# muesste der Test raten, wann der Blocker so weit ist. Der Zustand steckt im
# DATEINAMEN (<marker>.auf / <marker>.fehl) und nicht im Inhalt: eine Datei,
# die PowerShell gerade geschrieben hat, ist unter Windows noch kurz gesperrt,
# und das Lesen fiel genau da hinein (os error 32).
param([Parameter(Mandatory=$true)][string]$Marker,
      [Parameter(Mandatory=$true)][int]$Ms)

$sig = @'
[DllImport("user32.dll", SetLastError=true)] public static extern bool OpenClipboard(IntPtr hWndNewOwner);
[DllImport("user32.dll", SetLastError=true)] public static extern bool CloseClipboard();
'@
$u = Add-Type -MemberDefinition $sig -Name Zwischenablage -Namespace Win32 -PassThru

# Haelt sie gerade ein anderes Programm, warten wir kurz -- sonst sagt der
# Marker "fehl" und der Fall ueberspringt sich, statt etwas zu behaupten.
for ($i = 0; $i -lt 50; $i++) {
    if ($u::OpenClipboard([IntPtr]::Zero)) {
        Set-Content -Encoding ascii -Path "$Marker.auf" -Value "x"
        Start-Sleep -Milliseconds $Ms
        [void]$u::CloseClipboard()
        exit 0
    }
    Start-Sleep -Milliseconds 20
}
Set-Content -Encoding ascii -Path "$Marker.fehl" -Value "x"
exit 1
