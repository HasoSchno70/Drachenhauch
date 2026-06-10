@echo off
REM GameBasic-Animations-FSM-Editor (Unity-Mecanim-Stil) starten - nutzt den .venv-Python.
REM Verwendung: gbanim [datei.gbanim]
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --anim %*
