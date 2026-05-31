@echo off
REM GameBasic-Starter - nutzt automatisch den .venv-Python.
REM Verwendung: gb examples\10_pong.gb
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" %*
