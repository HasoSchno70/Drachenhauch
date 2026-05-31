@echo off
REM Native-VM neu kompilieren (nach Aenderungen in gamebasic\vm_native.pyx).
REM Verwendung: gb-build
echo Building native VM (Cython + MSVC)...
"%~dp0.venv\Scripts\python.exe" "%~dp0setup.py" build_ext --inplace
if errorlevel 1 (
    echo BUILD FEHLGESCHLAGEN
    exit /b 1
)
echo.
echo Native VM gebaut. gbrun.py erkennt sie automatisch.
