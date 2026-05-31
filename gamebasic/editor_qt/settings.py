"""Persistenz fuer Recent-Files und Editor-Settings.

Liest/Schreibt aus %APPDATA%/GameBasic (Windows) bzw. ~/.GameBasic
(POSIX). Identisches Format wie der alte CTk-Editor, sodass sich
Einstellungen ueber die Migration tragen.
"""
from __future__ import annotations

import json
import os
from pathlib import Path


RECENT_FILES_MAX = 10


def config_dir() -> Path:
    base = Path(os.environ.get("APPDATA", str(Path.home())))
    folder = base / "GameBasic"
    try:
        folder.mkdir(parents=True, exist_ok=True)
    except OSError:
        return Path.home()
    return folder


def _recent_path() -> Path:
    return config_dir() / "recent.json"


def _settings_path() -> Path:
    return config_dir() / "settings.json"


def load_recent() -> list[str]:
    p = _recent_path()
    if not p.exists():
        return []
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
        if isinstance(data, list):
            return [s for s in data if isinstance(s, str)]
    except Exception:
        return []
    return []


def save_recent(items: list[str]) -> None:
    try:
        _recent_path().write_text(
            json.dumps(items[:RECENT_FILES_MAX], ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
    except OSError:
        pass


def load_settings() -> dict:
    p = _settings_path()
    if not p.exists():
        return {}
    try:
        d = json.loads(p.read_text(encoding="utf-8"))
        return d if isinstance(d, dict) else {}
    except Exception:
        return {}


def save_settings(data: dict) -> None:
    try:
        _settings_path().write_text(
            json.dumps(data, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
    except OSError:
        pass
