@echo off
REM GameBasic-Notenblatt-Editor (Notensatz-Stil) starten - nutzt automatisch den .venv-Python.
REM Verwendung: gbscore
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --score %*
