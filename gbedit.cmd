@echo off
REM GameBasic-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: gbedit              (leer starten)
REM             gbedit examples\10_pong.gb   (mit geladener Datei)
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --editor %*
