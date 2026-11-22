; Kapitel 23 -- ein Installer fuer die gebaute Mediathek. Erst im Studio
; bauen (Strg+B legt alles nach bau/), dann mit Inno Setup uebersetzen:
;   ISCC mediathek.iss
; Heraus kommt installer\Mediathek-1.0-Setup.exe.

#define Fassung "1.0"

[Setup]
AppName=Mediathek
AppVersion={#Fassung}
DefaultDirName={autopf}\Mediathek
DefaultGroupName=Mediathek
UninstallDisplayIcon={app}\Mediathek.exe
OutputDir=installer
OutputBaseFilename=Mediathek-{#Fassung}-Setup
PrivilegesRequiredOverridesAllowed=dialog
WizardStyle=modern
Compression=lzma2
SolidCompression=yes

[Languages]
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktop"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked

[Files]
; Alles aus bau/: das Programm, assets/ und die mitgelieferte Datenbank.
; Die kopiert das Programm beim ersten Start in den Benutzerordner.
Source: "bau\*"; DestDir: "{app}"; Flags: recursesubdirs ignoreversion

[Icons]
Name: "{group}\Mediathek"; Filename: "{app}\Mediathek.exe"
Name: "{autodesktop}\Mediathek"; Filename: "{app}\Mediathek.exe"; Tasks: desktop

[Run]
Filename: "{app}\Mediathek.exe"; Description: "{cm:LaunchProgram,Mediathek}"; Flags: nowait postinstall skipifsilent
