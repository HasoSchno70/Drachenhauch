@echo off
REM GameBasic-SFX-Generator starten - nutzt automatisch den .venv-Python.
REM Verwendung: dhsfx
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --sfx %*
