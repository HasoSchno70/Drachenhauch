#!/usr/bin/env bash
# GameBasic-Starter - nutzt automatisch den .venv-Python.
# Verwendung: ./dh.sh examples/10_pong.gb
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$DIR/.venv/Scripts/python.exe" "$DIR/dhrun.py" "$@"
