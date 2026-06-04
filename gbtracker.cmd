@echo off
REM GameBasic-Tracker (Musik-Editor) starten - nutzt automatisch den .venv-Python.
REM Verwendung: gbtracker
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --tracker %*
