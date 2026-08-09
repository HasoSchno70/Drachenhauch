@echo off
REM GameBasic-Tilemap-/Level-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: dhtilemap [datei.json]
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --tilemap %*
