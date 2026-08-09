@echo off
REM GameBasic-Spiel als .exe verpacken (PyInstaller).
REM Verwendung:
REM   dh-package examples\10_pong.dh
REM   dh-package examples\17_tilemap.dh --windowed --onefile
REM   dh-package examples\10_pong.dh --windowed --icon=mein.ico
"%~dp0.venv\Scripts\python.exe" "%~dp0gb-package.py" %*
