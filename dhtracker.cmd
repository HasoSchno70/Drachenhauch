@echo off
REM Drachenhauch-Tracker (Musik-Editor) starten - nutzt automatisch den .venv-Python.
REM Verwendung: dhtracker
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --tracker %*
