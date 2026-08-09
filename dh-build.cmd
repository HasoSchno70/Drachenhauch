@echo off
REM Native-VM neu kompilieren (nach Aenderungen in drachenhauch\vm_native.pyx).
REM Verwendung: dh-build
echo Building native VM (Cython + MSVC)...
"%~dp0.venv\Scripts\python.exe" "%~dp0setup.py" build_ext --inplace
if errorlevel 1 (
    echo BUILD FEHLGESCHLAGEN
    exit /b 1
)
echo.
echo Native VM gebaut. dhrun.py erkennt sie automatisch.
