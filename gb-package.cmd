@echo off
REM GameBasic-Spiel als .exe verpacken (PyInstaller).
REM Verwendung:
REM   gb-package examples\10_pong.gb
REM   gb-package examples\17_tilemap.gb --windowed --onefile
REM   gb-package examples\10_pong.gb --windowed --icon=mein.ico
"%~dp0.venv\Scripts\python.exe" "%~dp0gb-package.py" %*
