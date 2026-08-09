# -*- mode: python ; coding: utf-8 -*-
# PyInstaller-Spec fuer die GameBasic-IDE (onedir, windowed).
# Friert dhrun.py samt drachenhauch-Paket, PySide6, numpy und Pillow ein, sodass
# GameBasic OHNE installiertes Python laeuft. dhrt(.exe) wird NICHT hier
# gebuendelt -- build_installer.py legt es NACH dem PyInstaller-Lauf neben
# die eingefrorene Exe (findet _find_dhrt via sys.executable-Verzeichnis,
# das ist bei einem macOS .app-Bundle Contents/MacOS/).
#
# Aufruf ueber installer/build_installer.py (setzt SPECPATH/Pfade); manuell:
#   .venv\Scripts\pyinstaller installer\GameBasic.spec --noconfirm
import platform
from pathlib import Path
from PyInstaller.utils.hooks import collect_submodules, collect_data_files

ROOT = Path(SPECPATH).resolve().parent           # installer/ -> Repo-Wurzel
ENTRY = str(ROOT / "dhrun.py")
IS_WINDOWS = platform.system() == "Windows"
IS_MACOS = platform.system() == "Darwin"
if IS_WINDOWS:
    ICON = ROOT / "installer" / "GameBasic.ico"
elif IS_MACOS:
    ICON = ROOT / "installer" / "GameBasic.icns"
else:
    ICON = None

# Daten (logo.png, editor_qt/builtin_index.json, ...) + alle Submodule
# (die Editoren werden lazy importiert -> als hiddenimports sicherstellen).
datas = collect_data_files("drachenhauch")
hiddenimports = collect_submodules("drachenhauch")

if not IS_WINDOWS:
    # macOS/Linux haben keinen Installer-Skript-Schritt wie Inno Setup (der
    # unter Windows examples/ separat ins {commondocs}-Verzeichnis kopiert)
    # -- die Beispiele werden hier stattdessen mit ins Bundle gepackt und
    # von dhrun._seed_examples_if_missing() beim allerersten Start in einen
    # beschreibbaren Ort (~/Documents/GameBasic) kopiert.
    examples_dir = ROOT / "examples"
    if examples_dir.is_dir():
        datas.append((str(examples_dir), "examples"))

a = Analysis(
    [ENTRY],
    pathex=[str(ROOT)],
    binaries=[],
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    runtime_hooks=[],
    # Test-/Dev-Ballast raus (kleinere Installation).
    excludes=["tkinter", "pytest", "_pytest", "PySide6.QtQuick",
              "PySide6.QtQml", "PySide6.Qt3DCore", "PySide6.QtWebEngineCore",
              "PySide6.QtWebEngineWidgets", "PySide6.QtWebChannel"],
    noarchive=False,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name="GameBasic",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    console=False,                 # GUI-IDE -> kein Konsolenfenster
    icon=str(ICON) if ICON is not None and ICON.exists() else None,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=False,
    name="GameBasic",
)

if IS_MACOS:
    # Ohne BUNDLE() waere die COLLECT-Ausgabe auf macOS nur ein rohes
    # Unix-Executable in einem Ordner -- kein echtes .app-Paket (kein
    # Info.plist, kein Doppelklick-Start im Finder, kein Dock-Icon).
    app = BUNDLE(
        coll,
        name="GameBasic.app",
        icon=str(ICON) if ICON is not None and ICON.exists() else None,
        bundle_identifier="de.hansschnorrenberger.drachenhauch",
        info_plist={
            "CFBundleName": "GameBasic",
            "CFBundleDisplayName": "GameBasic",
            "CFBundleShortVersionString": "1.0",
            "NSHighResolutionCapable": True,
            # .dh-Dateien im Finder mit GameBasic verknuepfen (Doppelklick
            # startet den Editor mit der Datei) -- Pendant zur Windows-
            # Registry-Dateiverknuepfung in GameBasic.iss.
            "CFBundleDocumentTypes": [{
                "CFBundleTypeName": "GameBasic-Quelltext",
                "CFBundleTypeExtensions": ["gb"],
                "CFBundleTypeRole": "Editor",
                "LSHandlerRank": "Owner",
            }],
        },
    )
