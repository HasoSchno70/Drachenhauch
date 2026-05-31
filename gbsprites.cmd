@echo off
REM GameBasic-Sprite-Editor starten - nutzt automatisch den .venv-Python.
REM Verwendung: gbsprites              (leer starten)
REM             gbsprites assets\hero.png   (mit geladenem Sprite)
"%~dp0.venv\Scripts\python.exe" "%~dp0gbrun.py" --sprites %*
