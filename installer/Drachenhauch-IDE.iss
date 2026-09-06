; Inno-Setup-Skript fuer Drachenhauch OHNE Python (Weg C, Stufe 3):
; die Runtime dhrt.exe, die IDE in Drachenhauch (ide\ide.dh), das Handbuch
; (docs\*.md), die Beispiele samt Begleit-Editoren -- kein PyInstaller, kein
; Qt, keine Python-Laufzeit. Gemessen 33 MB statt 92 (davon 20 MB dhrt.exe).
;
;   ISCC.exe /DAppVersion=2026.14 installer\Drachenhauch-IDE.iss
;
; Voraussetzung: rust\build_runtime.py --hardware hat dhrt.exe gebaut.
; Eigene AppId und eigener Ordner: das hier ersetzt die Qt-Fassung NICHT,
; beide lassen sich nebeneinander installieren, solange die Qt-IDE die
; Referenz bleibt (docs/entwurf-python-abbau.md, Weg C).

#ifndef AppVersion
  #define AppVersion "0.0"
#endif
#define AppName "Drachenhauch IDE"
#define AppPublisher "Hans Schnorrenberger"

[Setup]
AppId={{5D2B9C7E-0F6A-4C3B-8E1D-2A7F4B9C6E13}}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\Drachenhauch-IDE
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\dhrt.exe
OutputDir=output
OutputBaseFilename=Drachenhauch-IDE-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
ChangesEnvironment=yes
ChangesAssociations=yes
SetupIconFile=Drachenhauch.ico
LicenseFile=EULA.txt

[Languages]
Name: "de"; MessagesFile: "compiler:Languages\German.isl"

[Tasks]
Name: "desktopicon"; Description: "Desktop-Verknuepfung anlegen"; GroupDescription: "Zusaetzliche Symbole:"
Name: "addtopath"; Description: "Installationsordner zum PATH hinzufuegen (dhrt im Terminal nutzbar)"; GroupDescription: "Optionen:"
Name: "assocdh"; Description: ".dh-Dateien mit der Drachenhauch IDE verknuepfen"; GroupDescription: "Optionen:"

[Files]
Source: "..\rust\drachenhauch_runtime\target\release\dhrt.exe"; DestDir: "{app}"; Flags: ignoreversion
; Die IDE selbst -- Quelltext, den der Nutzer lesen und aendern kann.
Source: "..\ide\*.dh"; DestDir: "{app}\ide"; Flags: ignoreversion
; Das Handbuch: die IDE liest docs\ neben ide\ (F1 schlaegt dort nach).
Source: "..\docs\*.md"; DestDir: "{app}\docs"; Flags: ignoreversion
; Beispiele und Begleit-Editoren in die oeffentlichen Dokumente -- dort sind
; sie beschreibbar, und die IDE findet sie ueber %PUBLIC%, wenn neben ide\
; keine liegen. `uninsneveruninstall`: bearbeitete Beispiele ueberleben.
Source: "..\examples\*"; DestDir: "{commondocs}\Drachenhauch\examples"; \
    Flags: recursesubdirs createallsubdirs uninsneveruninstall
Source: "EULA.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "THIRD-PARTY-NOTICES.txt"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
; Die IDE startet aus dem Beispielordner -- so zeigt der Projektbaum beim
; ersten Start etwas (dhrt hinterlegt den Ort als DHRT_START_DIR).
Name: "{group}\Drachenhauch IDE"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{app}\ide\ide.dh"""; WorkingDir: "{commondocs}\Drachenhauch\examples"; IconFilename: "{app}\dhrt.exe"; Comment: "Die IDE in Drachenhauch"
Name: "{group}\SFX-Generator"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\183_sfx_generator.dh"""; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Partikel-Editor"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\185_partikel_editor.dh"""; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Tilemap-Editor"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\187_tilemap_editor.dh"""; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Sprite-Editor"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\189_sprite_editor.dh"""; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Tracker"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\190_tracker.dh"""; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Beispiele"; Filename: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Handbuch (Markdown)"; Filename: "{app}\docs"
Name: "{group}\Lizenzen\Lizenzvertrag (EULA)"; Filename: "{app}\EULA.txt"
Name: "{group}\Lizenzen\Drittanbieter-Lizenzen"; Filename: "{app}\THIRD-PARTY-NOTICES.txt"; Flags: createonlyiffileexists
Name: "{group}\{cm:UninstallProgram,Drachenhauch IDE}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Drachenhauch IDE"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{app}\ide\ide.dh"""; WorkingDir: "{commondocs}\Drachenhauch\examples"; Tasks: desktopicon

[Registry]
; .dh-Dateiverknuepfung: Oeffnen = in der IDE (Argument nach --), Ausfuehren = dhrt run.
Root: HKA; Subkey: "Software\Classes\.dh"; ValueType: string; ValueName: ""; ValueData: "DrachenhauchIDE.Source"; Tasks: assocdh; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source"; ValueType: string; ValueName: ""; ValueData: "Drachenhauch-Quelltext"; Tasks: assocdh; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\dhrt.exe,0"; Tasks: assocdh
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\dhrt.exe"" run ""{app}\ide\ide.dh"" -- ""%1"""; Tasks: assocdh
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\shell\run"; ValueType: string; ValueName: ""; ValueData: "Mit Drachenhauch ausfuehren"; Tasks: assocdh
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\shell\run\command"; ValueType: string; ValueName: ""; ValueData: """{app}\dhrt.exe"" run ""%1"""; Tasks: assocdh

[Code]
// PATH-Eintrag fuer den Installationsordner (optionaler Task) -- wie in
// Drachenhauch.iss.
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
