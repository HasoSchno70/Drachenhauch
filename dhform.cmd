@echo off
REM GameBasic-Form-Designer (WYSIWYG, Xojo-Stil) starten - nutzt den .venv-Python.
REM Verwendung: dhform [datei.gbform]
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --form %*
