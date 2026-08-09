@echo off
REM Drachenhauch-Form-Designer (WYSIWYG, Xojo-Stil) starten - nutzt den .venv-Python.
REM Verwendung: dhform [datei.dhform]
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --form %*
