@echo off
REM GameBasic-Form-Designer (WYSIWYG, Xojo-Stil) starten - nutzt den .venv-Python.
REM Verwendung: gbform [datei.gbform]
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --form %*
