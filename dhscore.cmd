@echo off
REM GameBasic-Notenblatt-Editor (Notensatz-Stil) starten - nutzt automatisch den .venv-Python.
REM Verwendung: dhscore
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --score %*
