@echo off
REM GameBasic-Partikel-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: dhparticles
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --particles %*
