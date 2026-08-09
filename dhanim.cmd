@echo off
REM GameBasic-Animations-FSM-Editor (Unity-Mecanim-Stil) starten - nutzt den .venv-Python.
REM Verwendung: dhanim [datei.gbanim]
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --anim %*
