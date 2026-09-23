; Inno-Setup-Skript fuer Drachenhauch OHNE Python (Weg C, Stufe 3):
; die Runtime dhrt.exe, die IDE in Drachenhauch (ide\ide.dh), das Handbuch
; (docs\*.md), die Beispiele samt Begleit-Editoren -- kein PyInstaller, kein
; Qt, keine Python-Laufzeit. Seit 2026-09-21 auch die Buecher, die
; ESP32-Sketche, das Aufraeumen von GameBasic und (ueber bauen.dh) die
; Signierung -- alles, was bis dahin nur Drachenhauch.iss mitbrachte.
;
;   ISCC.exe /DAppVersion=2026.14 installer\Drachenhauch-IDE.iss
;
; Voraussetzung: rust\build_runtime.py --hardware hat dhrt.exe gebaut.
; Eigene AppId und eigener Ordner, nicht die der frueheren Qt-Fassung
; (Drachenhauch.iss, 2026-09-21 geloescht): eine noch installierte
; Qt-Fassung bleibt stehen, bis man sie ueber "Programme entfernen" loescht.

#ifndef AppVersion
  #define AppVersion "0.0"
#endif
; Welche dhrt.exe hineinkommt. bauen.dh gibt eine KOPIE an -- dort ist sie
; schon signiert, und die gebaute Datei bleibt unberuehrt (eine laufende
; .exe laesst sich unter Windows ohnehin nicht signieren).
#ifndef DhrtQuelle
  #define DhrtQuelle "..\rust\drachenhauch_runtime\target\release\dhrt.exe"
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
UninstallDisplayIcon={app}\drachenhauch.ico
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

[InstallDelete]
; --- Reste der GameBasic-Installation --- (dieselben wie in Drachenhauch.iss:
; wer von GameBasic gleich auf die Fassung ohne Python wechselt, soll nicht
; rund 80 MB und 225 verwaiste Beispieldateien behalten)
Type: filesandordirs; Name: "{autopf}\GameBasic"
Type: filesandordirs; Name: "{commondocs}\GameBasic"
Type: filesandordirs; Name: "{autoprograms}\GameBasic"
Type: files; Name: "{autodesktop}\GameBasic.lnk"
; Vorschaubilder der eigenen Beispiele: reine Erzeugung, dort legt niemand
; etwas ab -- ohne das blieben umbenannte Bilder fuer immer liegen. Nur
; dieser Unterordner; der Beispielordner selbst ist `uninsneveruninstall`.
Type: filesandordirs; Name: "{commondocs}\Drachenhauch\examples\screenshots"

[Files]
Source: "{#DhrtQuelle}"; DestDir: "{app}"; DestName: "dhrt.exe"; Flags: ignoreversion
; Das Drachen-Symbol fuer Verknuepfungen, .dh-Dateien und die Deinstallation --
; dhrt.exe selbst traegt keins (ein exportiertes Spiel ist eine Kopie davon).
Source: "Drachenhauch.ico"; DestDir: "{app}"; DestName: "drachenhauch.ico"; Flags: ignoreversion
; Logo und Schriftzug: die IDE sucht sie unter daten\bilder neben ide\
; (Fenstersymbol, Schriftzug auf der Willkommensseite).
Source: "..\daten\bilder\*.png"; DestDir: "{app}\daten\bilder"; Flags: ignoreversion
; Der Vorspann beim Start der IDE (daten\video\vorspann.mp4, H.264).
Source: "..\daten\video\*.mp4"; DestDir: "{app}\daten\video"; Flags: ignoreversion
; Die IDE selbst -- Quelltext, den der Nutzer lesen und aendern kann.
Source: "..\ide\*.dh"; DestDir: "{app}\ide"; Flags: ignoreversion
; Das Handbuch: die IDE liest docs\ neben ide\ (F1 schlaegt dort nach).
Source: "..\docs\*.md"; DestDir: "{app}\docs"; Flags: ignoreversion
; Beispiele und Begleit-Editoren in die oeffentlichen Dokumente -- dort sind
; sie beschreibbar, und die IDE findet sie ueber %PUBLIC%, wenn neben ide\
; keine liegen. `uninsneveruninstall`: bearbeitete Beispiele ueberleben.
Source: "..\examples\*"; DestDir: "{commondocs}\Drachenhauch\examples"; \
    Flags: recursesubdirs createallsubdirs uninsneveruninstall
; Sketch-Grundgeruest fuer ESP32/ESP8266 -- neben die Beispiele, weil
; examples\159_esp32_bruecke.dh im Kopfkommentar darauf verweist.
Source: "..\esp32\*"; DestDir: "{commondocs}\Drachenhauch\esp32"; \
    Flags: recursesubdirs createallsubdirs uninsneveruninstall skipifsourcedoesntexist
; Die Buecher, falls gebaut (tools/buch_bauen.dh) -- .docx zum Drucken,
; .epub zum Lesen am Geraet; der Einstieg ist der Band fuer Anfaenger.
Source: "..\buch-referenz\buch\Drachenhauch-Lehrbuch.docx"; DestDir: "{app}\buecher"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\buch-referenz\buch\Drachenhauch-Lehrbuch.epub"; DestDir: "{app}\buecher"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\buch-referenz\buch\Drachenhauch-Handbook.docx"; DestDir: "{app}\buecher"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\buch-referenz\buch\Drachenhauch-Handbook.epub"; DestDir: "{app}\buecher"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\buch-einstieg\buch\Drachenhauch-Einstieg.docx"; DestDir: "{app}\buecher"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\buch-einstieg\buch\Drachenhauch-Einstieg.epub"; DestDir: "{app}\buecher"; Flags: ignoreversion skipifsourcedoesntexist
Source: "EULA.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "THIRD-PARTY-NOTICES-IDE.txt"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
; Die IDE startet aus dem Beispielordner -- so zeigt der Projektbaum beim
; ersten Start etwas (dhrt hinterlegt den Ort als DHRT_START_DIR).
Name: "{group}\Drachenhauch IDE"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{app}\ide\ide.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"; Comment: "Die IDE in Drachenhauch"
Name: "{group}\SFX-Generator"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\183_sfx_generator.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Partikel-Editor"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\185_partikel_editor.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Tilemap-Editor"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\187_tilemap_editor.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Sprite-Editor"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\189_sprite_editor.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Tracker"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\190_tracker.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Form-Designer"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\197_form_designer.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Anim-FSM-Editor"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\198_anim_fsm_editor.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Notenblatt"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{commondocs}\Drachenhauch\examples\199_notenblatt.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Beispiele"; Filename: "{commondocs}\Drachenhauch\examples"
Name: "{group}\Handbuch (Markdown)"; Filename: "{app}\docs"
Name: "{group}\Einstieg (fuer Anfaenger)"; Filename: "{app}\buecher\Drachenhauch-Einstieg.docx"; Flags: createonlyiffileexists
Name: "{group}\Lehrbuch"; Filename: "{app}\buecher\Drachenhauch-Lehrbuch.docx"; Flags: createonlyiffileexists
Name: "{group}\Handbook (English)"; Filename: "{app}\buecher\Drachenhauch-Handbook.docx"; Flags: createonlyiffileexists
Name: "{group}\ESP32-Sketche"; Filename: "{commondocs}\Drachenhauch\esp32"
Name: "{group}\Lizenzen\Lizenzvertrag (EULA)"; Filename: "{app}\EULA.txt"
Name: "{group}\Lizenzen\Drittanbieter-Lizenzen"; Filename: "{app}\THIRD-PARTY-NOTICES-IDE.txt"; Flags: createonlyiffileexists
Name: "{group}\{cm:UninstallProgram,Drachenhauch IDE}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Drachenhauch IDE"; Filename: "{app}\dhrt.exe"; Parameters: "run ""{app}\ide\ide.dh"""; IconFilename: "{app}\drachenhauch.ico"; WorkingDir: "{commondocs}\Drachenhauch\examples"; Tasks: desktopicon

[Registry]
; .dh-Dateiverknuepfung: Oeffnen = in der IDE (Argument nach --), Ausfuehren = dhrt run.
Root: HKA; Subkey: "Software\Classes\.dh"; ValueType: string; ValueName: ""; ValueData: "DrachenhauchIDE.Source"; Tasks: assocdh; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source"; ValueType: string; ValueName: ""; ValueData: "Drachenhauch-Quelltext"; Tasks: assocdh; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\drachenhauch.ico"; Tasks: assocdh
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\dhrt.exe"" run ""{app}\ide\ide.dh"" -- ""%1"""; Tasks: assocdh
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\shell\run"; ValueType: string; ValueName: ""; ValueData: "Mit Drachenhauch ausfuehren"; Tasks: assocdh
Root: HKA; Subkey: "Software\Classes\DrachenhauchIDE.Source\shell\run\command"; ValueType: string; ValueName: ""; ValueData: """{app}\dhrt.exe"" run ""%1"""; Tasks: assocdh

[Code]
// --- Registry-Reste der GameBasic-Installation --- (wie in Drachenhauch.iss)
// Die ProgID `GameBasic.Source` gehoert uns. `.gb` NUR, wenn es noch auf uns
// zeigt: `.gb` ist auch die Endung fuer Game-Boy-ROMs, und ein Emulator, der
// sie inzwischen haelt, geht uns nichts an.
procedure LoescheAlteVerknuepfung(Wurzel: Integer);
var
  Wert: string;
begin
  if RegQueryStringValue(Wurzel, 'Software\Classes\.gb', '', Wert)
     and (Wert = 'GameBasic.Source') then
    RegDeleteKeyIncludingSubkeys(Wurzel, 'Software\Classes\.gb');
  RegDeleteKeyIncludingSubkeys(Wurzel, 'Software\Classes\GameBasic.Source');
end;

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
  if CurStep = ssPostInstall then
  begin
    // Beide Wurzeln -- ob GameBasic fuer alle oder nur fuer den Nutzer
    // installiert war, weiss hier niemand mehr.
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
