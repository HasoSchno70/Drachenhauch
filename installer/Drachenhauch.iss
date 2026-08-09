; Inno-Setup-Skript fuer Drachenhauch.
; Erzeugt einen Windows-Installer aus der PyInstaller-onedir-Ausgabe
; (..\dist\Drachenhauch) + der dhrt-Runtime. Aufruf ueber installer\build_installer.py
; (setzt AppVersion). Manuell:  ISCC.exe /DAppVersion=2026.1 installer\Drachenhauch.iss

#ifndef AppVersion
  #define AppVersion "0.0"
#endif
#define AppName "Drachenhauch"
#define AppPublisher "Hans Schnorrenberger"
#define AppExe "Drachenhauch.exe"

[Setup]
AppId={{A3F1C0E2-7B4D-4E89-9A12-6C5D8B0F3E47}}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
; Die AppId bleibt die der GameBasic-Installation (siehe oben), damit in
; "Programme entfernen" EIN Eintrag ersetzt wird statt zwei zu entstehen.
; Genau deshalb wuerde Inno aber auch den GEMERKTEN Ordner wiederverwenden --
; also weiter nach `Program Files\GameBasic` installieren. Das hier erzwingt
; den neuen Vorgabe-Ordner; der alte wird unten in [InstallDelete] entfernt.
UsePreviousAppDir=no
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#AppExe}
OutputDir=output
OutputBaseFilename=Drachenhauch-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
ChangesEnvironment=yes
ChangesAssociations=yes
SetupIconFile=Drachenhauch.ico
; EULA-Zustimmungsseite im Setup-Assistenten.
LicenseFile=EULA.txt

; --- Code-Signing (optional) ---
; build_installer.py signiert App-Exe, dhrt.exe UND den fertigen Installer extern
; per signtool (gesteuert ueber GB_SIGN_CERT/GB_SIGN_PASS) -- das ist der
; Standardweg. Wer ZUSAETZLICH den Uninstaller bei der Kompilierung signieren
; lassen will, definiert beim ISCC-Aufruf ein SignTool und aktiviert die zwei
; Zeilen unten:
;   ISCC.exe "/Ssign=signtool sign /fd SHA256 /f C:\keys\cert.pfx /p PASS /tr http://timestamp.digicert.com /td SHA256 $f" /DAppVersion=2026.1 Drachenhauch.iss
; SignTool=sign
; SignedUninstaller=yes

[Languages]
Name: "de"; MessagesFile: "compiler:Languages\German.isl"

[Tasks]
Name: "desktopicon"; Description: "Desktop-Verknuepfung anlegen"; GroupDescription: "Zusaetzliche Symbole:"
Name: "addtopath"; Description: "Installationsordner zum PATH hinzufuegen (dhrt im Terminal nutzbar)"; GroupDescription: "Optionen:"
Name: "assocgb"; Description: ".dh-Dateien mit Drachenhauch verknuepfen"; GroupDescription: "Optionen:"

[InstallDelete]
; --- Reste der GameBasic-Installation ---
;
; Der alte Programmordner. Ohne das bliebe die komplette vorige Fassung
; (rund 80 MB samt eigener dhrt.exe) fuer immer liegen, weil der neue
; Installer woanders hin schreibt.
Type: filesandordirs; Name: "{autopf}\GameBasic"
; Der alte Beispielordner in den oeffentlichen Dokumenten. Er wurde mit
; `uninsneveruninstall` abgelegt, damit selbst bearbeitete Beispiele eine
; Deinstallation ueberleben -- die Kehrseite: NIEMAND raeumt ihn je weg.
; Genau so hat der Juni-Installer 225 verwaiste Dateien hinterlassen.
Type: filesandordirs; Name: "{commondocs}\GameBasic"
; Startmenue-Gruppe des alten Namens (die Verknuepfungen darin zeigen auf
; einen Ordner, den wir gerade geloescht haben).
Type: filesandordirs; Name: "{autoprograms}\GameBasic"
Type: files; Name: "{autodesktop}\GameBasic.lnk"

[Files]
; Komplette eingefrorene IDE (PyInstaller onedir).
Source: "..\dist\Drachenhauch\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; Native Runtime neben die Exe -- die IDE findet sie (_find_dhrt) und sie liegt
; (bei aktivem PATH-Task) fuer das Terminal bereit.
Source: "..\rust\drachenhauch_runtime\target\release\dhrt.exe"; DestDir: "{app}"; Flags: ignoreversion
; Beispielprogramme + Showcase-Thumbnails (screenshots/) in die OEFFENTLICHEN
; DOKUMENTE -> `%PUBLIC%\Documents\Drachenhauch\examples`. Das ist exakt der
; `project_root/examples` der eingefrorenen App (dhrun._project_root): so findet
; der Editor Beispiele UND Showcase-Vorschaubilder, und der Ort ist BESCHREIBBAR
; (Program Files waere schreibgeschuetzt -> Demos die Dateien schreiben + „Neu"
; speichern wuerden scheitern). uninsneveruninstall: vom User editierte Beispiele
; bleiben bei der Deinstallation erhalten.
Source: "..\examples\*"; DestDir: "{commondocs}\Drachenhauch\examples"; \
    Flags: recursesubdirs createallsubdirs uninsneveruninstall
; Sketch-Grundgeruest fuer ESP32/ESP8266 -- NEBEN die Beispiele, weil
; examples\159_esp32_bruecke.dh im Kopfkommentar darauf verweist. Ohne das
; laese der Nutzer dort von einem Gegenstueck, das er nicht hat.
Source: "..\esp32\*"; DestDir: "{commondocs}\Drachenhauch\esp32"; \
    Flags: recursesubdirs createallsubdirs uninsneveruninstall skipifsourcedoesntexist
; Lehrbuch (falls gebaut) -- zum Drucken das .docx, zum Lesen am Geraet das .epub.
; Beide Sprachen: dieselben Kapitelquellen, das englische Handbuch entsteht ueber
; den Katalog buch-referenz\buch\i18n\en.json.
Source: "..\buch-referenz\buch\Drachenhauch-Lehrbuch.docx"; DestDir: "{app}\docs"; Flags: skipifsourcedoesntexist
Source: "..\buch-referenz\buch\Drachenhauch-Lehrbuch.epub"; DestDir: "{app}\docs"; Flags: skipifsourcedoesntexist
Source: "..\buch-referenz\buch\Drachenhauch-Handbook.docx"; DestDir: "{app}\docs"; Flags: skipifsourcedoesntexist
Source: "..\buch-referenz\buch\Drachenhauch-Handbook.epub"; DestDir: "{app}\docs"; Flags: skipifsourcedoesntexist
; Lizenz + Drittanbieter-Lizenzhinweise (Pflicht-Beilage fuer MIT/BSD/Apache/LGPL).
Source: "EULA.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "THIRD-PARTY-NOTICES.txt"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\Drachenhauch"; Filename: "{app}\{#AppExe}"; Comment: "Drachenhauch-Editor / Auswahl"
Name: "{group}\Sprite-Editor"; Filename: "{app}\{#AppExe}"; Parameters: "--sprites"
Name: "{group}\Tilemap-Editor"; Filename: "{app}\{#AppExe}"; Parameters: "--tilemap"
Name: "{group}\Form-Designer"; Filename: "{app}\{#AppExe}"; Parameters: "--form"
Name: "{group}\Audio-Studio"; Filename: "{app}\{#AppExe}"; Parameters: "--audio"
Name: "{group}\Beispiele"; Filename: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Lehrbuch"; Filename: "{app}\docs\Drachenhauch-Lehrbuch.docx"; Flags: createonlyiffileexists
Name: "{group}\Handbook (English)"; Filename: "{app}\docs\Drachenhauch-Handbook.docx"; Flags: createonlyiffileexists
Name: "{group}\Lizenzen\Lizenzvertrag (EULA)"; Filename: "{app}\EULA.txt"
Name: "{group}\Lizenzen\Drittanbieter-Lizenzen"; Filename: "{app}\THIRD-PARTY-NOTICES.txt"
Name: "{group}\{cm:UninstallProgram,Drachenhauch}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Drachenhauch"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; .dh-Dateiverknuepfung (oeffnet im Editor) -- nur bei aktivem Task.
Root: HKA; Subkey: "Software\Classes\.dh"; ValueType: string; ValueName: ""; ValueData: "Drachenhauch.Source"; Tasks: assocgb; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Drachenhauch.Source"; ValueType: string; ValueName: ""; ValueData: "Drachenhauch-Quelltext"; Tasks: assocgb; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Drachenhauch.Source\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#AppExe},0"; Tasks: assocgb
Root: HKA; Subkey: "Software\Classes\Drachenhauch.Source\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" --editor ""%1"""; Tasks: assocgb
Root: HKA; Subkey: "Software\Classes\Drachenhauch.Source\shell\run"; ValueType: string; ValueName: ""; ValueData: "Mit Drachenhauch ausfuehren"; Tasks: assocgb
Root: HKA; Subkey: "Software\Classes\Drachenhauch.Source\shell\run\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: assocgb

[Code]
// --- Registry-Reste der GameBasic-Installation ---
//
// Die vorige Fassung hat die ProgID `GameBasic.Source` angelegt und `.gb`
// darauf verwiesen. Die ProgID gehoert uns, die kann weg.
//
// `.gb` NUR, wenn es noch auf uns zeigt: `.gb` ist zugleich die Endung fuer
// Game-Boy-ROMs (genau der Namenskonflikt, wegen dem wir auf `.dh` gewechselt
// sind). Steht dort inzwischen ein Emulator, waere ein blindes Loeschen ein
// Eingriff in fremde Software -- deshalb wird der Wert vorher gelesen.
procedure LoescheAlteVerknuepfung(Wurzel: Integer);
var
  Wert: string;
begin
  if RegQueryStringValue(Wurzel, 'Software\Classes\.gb', '', Wert)
     and (Wert = 'GameBasic.Source') then
    RegDeleteKeyIncludingSubkeys(Wurzel, 'Software\Classes\.gb');
  RegDeleteKeyIncludingSubkeys(Wurzel, 'Software\Classes\GameBasic.Source');
end;

// --- PATH-Eintrag fuer den Installationsordner (optionaler Task) ---
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Lowercase(Param) + ';', ';' + Lowercase(OrigPath) + ';') = 0;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  OrigPath: string;
begin
  if CurStep = ssPostInstall then
  begin
    // Beide Wurzeln: eine Installation "fuer alle" schreibt nach HKLM, eine
    // fuer den aktuellen Nutzer nach HKCU. Welche es damals war, wissen wir
    // hier nicht mehr -- also in beiden nachsehen.
    LoescheAlteVerknuepfung(HKEY_LOCAL_MACHINE);
    LoescheAlteVerknuepfung(HKEY_CURRENT_USER);
  end;
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
  begin
    if NeedsAddPath(ExpandConstant('{app}')) then
    begin
      if not RegQueryStringValue(HKEY_LOCAL_MACHINE,
        'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
        'Path', OrigPath) then OrigPath := '';
      if (OrigPath <> '') and (OrigPath[Length(OrigPath)] <> ';') then
        OrigPath := OrigPath + ';';
      RegWriteStringValue(HKEY_LOCAL_MACHINE,
        'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
        'Path', OrigPath + ExpandConstant('{app}'));
    end;
  end;
end;
