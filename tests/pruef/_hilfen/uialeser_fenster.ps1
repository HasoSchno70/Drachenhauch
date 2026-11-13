# Liest den UIA-Baum EINES Fensters, gefunden ueber seinen Titel (FindWindow)
# -- auch das zweite, native Fenster eines Programms (GUI_WINDOW_NATIVE,
# Weg E), das kein "Hauptfenster" seines Prozesses ist. Gibt je Knoten
# "N: <ControlType> [<Name>]" aus; mit -Klick drueckt es zuletzt den Knopf
# dieses Namens (InvokePattern), mit -Wert setzt es das erste Textfeld.
param([string]$Titel, [string]$Klick = "", [string]$Wert = "")
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class UF {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string k, string t);
}
"@
$A = [System.Windows.Automation.AutomationElement]
$Alle = [System.Windows.Automation.TreeScope]::Descendants
$Wahr = [System.Windows.Automation.Condition]::TrueCondition
$win = $null
$kids = $null
for ($i = 0; $i -lt 150; $i++) {
    # $null kaeme als "" an -- und sucht dann eine Fensterklasse "".
    $h = [UF]::FindWindow([NullString]::Value, $Titel)
    if ($h -ne [IntPtr]::Zero) {
        $win = $A::FromHandle($h)
        $kids = $win.FindAll($Alle, $Wahr)
        if ($kids.Count -ge 3) { break }
    }
    Start-Sleep -Milliseconds 100
}
if (-not $win) { "kein Fenster"; exit 2 }
foreach ($k in $kids) { "N: " + $k.Current.ControlType.ProgrammaticName + " [" + $k.Current.Name + "]" }
function Finde($typ, $name) {
    foreach ($k in $win.FindAll($Alle, $Wahr)) {
        if ($k.Current.ControlType.ProgrammaticName -eq $typ -and ($name -eq "" -or $k.Current.Name -eq $name)) { return $k }
    }
    return $null
}
if ($Wert -ne "") {
    $edit = Finde "ControlType.Edit" ""
    if (-not $edit) { "Feld fehlt"; exit 3 }
    $edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($Wert); Start-Sleep -Milliseconds 300
    "WERT: " + $edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
}
if ($Klick -ne "") {
    $btn = Finde "ControlType.Button" $Klick
    if (-not $btn) { "Knopf fehlt"; exit 3 }
    $btn.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    "gedrueckt"
}
