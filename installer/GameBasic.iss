; Inno-Setup-Skript fuer GameBasic.
; Erzeugt einen Windows-Installer aus der PyInstaller-onedir-Ausgabe
; (..\dist\GameBasic) + der gbrt-Runtime. Aufruf ueber installer\build_installer.py
; (setzt AppVersion). Manuell:  ISCC.exe /DAppVersion=2026.1 installer\GameBasic.iss

#ifndef AppVersion
  #define AppVersion "0.0"
#endif
#define AppName "GameBasic"
#define AppPublisher "Hans Schnorrenberger"
#define AppExe "GameBasic.exe"

[Setup]
AppId={{A3F1C0E2-7B4D-4E89-9A12-6C5D8B0F3E47}}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#AppExe}
OutputDir=output
OutputBaseFilename=GameBasic-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
ChangesEnvironment=yes
ChangesAssociations=yes
SetupIconFile=GameBasic.ico

[Languages]
Name: "de"; MessagesFile: "compiler:Languages\German.isl"

[Tasks]
Name: "desktopicon"; Description: "Desktop-Verknuepfung anlegen"; GroupDescription: "Zusaetzliche Symbole:"
Name: "addtopath"; Description: "Installationsordner zum PATH hinzufuegen (gbrt im Terminal nutzbar)"; GroupDescription: "Optionen:"
Name: "assocgb"; Description: ".gb-Dateien mit GameBasic verknuepfen"; GroupDescription: "Optionen:"

[Files]
; Komplette eingefrorene IDE (PyInstaller onedir).
Source: "..\dist\GameBasic\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; Native Runtime neben die Exe -- die IDE findet sie (_find_gbrt) und sie liegt
; (bei aktivem PATH-Task) fuer das Terminal bereit.
Source: "..\rust\gb_runtime\target\release\gbrt.exe"; DestDir: "{app}"; Flags: ignoreversion
; Beispielprogramme in die OEFFENTLICHEN DOKUMENTE (beschreibbar -- Program Files
; waere schreibgeschuetzt, dann scheitern Demos die Dateien schreiben). Ohne den
; screenshots\-Referenzordner (nur Doku-PNGs, blaehen den Installer auf).
Source: "..\examples\*"; DestDir: "{commondocs}\GameBasic\Beispiele"; \
    Excludes: "screenshots\*"; Flags: recursesubdirs createallsubdirs uninsneveruninstall
; Lehrbuch (falls gebaut).
Source: "..\buch-referenz\buch\GameBasic-Lehrbuch.docx"; DestDir: "{app}\docs"; Flags: skipifsourcedoesntexist

[Icons]
Name: "{group}\GameBasic"; Filename: "{app}\{#AppExe}"; Comment: "GameBasic-Editor / Auswahl"
Name: "{group}\Sprite-Editor"; Filename: "{app}\{#AppExe}"; Parameters: "--sprites"
Name: "{group}\Tilemap-Editor"; Filename: "{app}\{#AppExe}"; Parameters: "--tilemap"
Name: "{group}\Form-Designer"; Filename: "{app}\{#AppExe}"; Parameters: "--form"
Name: "{group}\Audio-Studio"; Filename: "{app}\{#AppExe}"; Parameters: "--audio"
Name: "{group}\Beispiele"; Filename: "{commondocs}\GameBasic\Beispiele"
Name: "{group}\{cm:UninstallProgram,GameBasic}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\GameBasic"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; .gb-Dateiverknuepfung (oeffnet im Editor) -- nur bei aktivem Task.
Root: HKA; Subkey: "Software\Classes\.gb"; ValueType: string; ValueName: ""; ValueData: "GameBasic.Source"; Tasks: assocgb; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\GameBasic.Source"; ValueType: string; ValueName: ""; ValueData: "GameBasic-Quelltext"; Tasks: assocgb; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\GameBasic.Source\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#AppExe},0"; Tasks: assocgb
Root: HKA; Subkey: "Software\Classes\GameBasic.Source\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" --editor ""%1"""; Tasks: assocgb
Root: HKA; Subkey: "Software\Classes\GameBasic.Source\shell\run"; ValueType: string; ValueName: ""; ValueData: "Mit GameBasic ausfuehren"; Tasks: assocgb
Root: HKA; Subkey: "Software\Classes\GameBasic.Source\shell\run\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: assocgb

[Code]
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
