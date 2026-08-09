@echo off
REM GameBasic Audio Studio (Tracker + SFX vereint) starten - nutzt den .venv-Python.
REM Verwendung: dhsound [song.gbtrk]
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --audio %*
