@echo off
REM GameBasic-Tilemap-/Level-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: gbtilemap [datei.json]
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --tilemap %*
