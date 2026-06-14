# -*- mode: python ; coding: utf-8 -*-
# PyInstaller-Spec fuer die GameBasic-IDE (onedir, windowed).
# Friert gbrun.py samt gamebasic-Paket, PySide6, numpy und Pillow ein, sodass
# GameBasic OHNE installiertes Python laeuft. gbrt.exe wird NICHT hier gebuendelt
# -- der Inno-Installer legt es neben GameBasic.exe (findet _find_gbrt via
# sys.executable-Verzeichnis) und auf den PATH.
#
# Aufruf ueber installer/build_installer.py (setzt SPECPATH/Pfade); manuell:
#   .venv\Scripts\pyinstaller installer\GameBasic.spec --noconfirm
from pathlib import Path
from PyInstaller.utils.hooks import collect_submodules, collect_data_files

ROOT = Path(SPECPATH).resolve().parent           # installer/ -> Repo-Wurzel
ENTRY = str(ROOT / "gbrun.py")
ICON = ROOT / "installer" / "GameBasic.ico"

# Daten (logo.png, editor_qt/builtin_index.json, ...) + alle Submodule
# (die Editoren werden lazy importiert -> als hiddenimports sicherstellen).
datas = collect_data_files("gamebasic")
hiddenimports = collect_submodules("gamebasic")

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
    icon=str(ICON) if ICON.exists() else None,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=False,
    name="GameBasic",
)
