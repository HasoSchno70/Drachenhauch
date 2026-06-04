@echo off
REM GameBasic-Partikel-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: gbparticles
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --particles %*
