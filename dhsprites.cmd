@echo off
REM GameBasic-Sprite-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: dhsprites              (leer starten)
REM             dhsprites assets\hero.png   (mit geladenem Sprite)
"%~dp0.venv\Scripts\python.exe" "%~dp0dhrun.py" --sprites %*
