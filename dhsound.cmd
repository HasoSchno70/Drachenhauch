@echo off
REM Drachenhauch Audio Studio (Tracker + SFX vereint) starten - nutzt den .venv-Python.
REM Verwendung: dhsound [song.gbtrk]
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --audio %*
