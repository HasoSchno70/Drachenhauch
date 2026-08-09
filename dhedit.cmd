@echo off
REM GameBasic-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: dhedit              (leer starten)
REM             dhedit examples\10_pong.dh   (mit geladener Datei)
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --editor %*
