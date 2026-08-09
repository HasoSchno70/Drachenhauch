@echo off
REM GameBasic-Starter - nutzt automatisch den .venv-Python.
REM Verwendung: dh examples\10_pong.gb
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" %*
