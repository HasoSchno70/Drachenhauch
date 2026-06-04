@echo off
REM GameBasic-SFX-Generator starten - nutzt automatisch den .venv-Python.
REM Verwendung: gbsfx
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --sfx %*
