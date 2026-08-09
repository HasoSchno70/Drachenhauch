"""Wiederverwendbare Preset-Bibliothek fuer die Tool-Editoren (Partikel/SFX).

Qt-frei -> headless testbar. Ein Preset ist ein JSON-serialisierbares
Parameter-Dict (genau das, was die Editoren via `_capture_state`/`_params`
liefern). Zwei Quellen:

- **builtins**: schreibgeschuetzte Werks-Presets (im Code definiert).
- **user**: vom Nutzer gespeicherte Presets, persistiert als JSON-Datei.

Ein User-Preset mit gleichem Namen ueberschreibt ein gleichnamiges Builtin
(in der Anzeige), Builtins selbst sind nicht loeschbar.
"""
from __future__ import annotations

import json
from pathlib import Path


def default_dir() -> Path:
    """Persistentes Nutzer-Verzeichnis fuer Presets (ueber Projekte hinweg)."""
    return Path.home() / ".drachenhauch" / "presets"


class PresetLibrary:
    def __init__(self, path, builtins: dict | None = None):
        self.path = Path(path)
        self.builtins: dict[str, dict] = {
            str(k): dict(v) for k, v in (builtins or {}).items()}
        self.user: dict[str, dict] = {}
        self.load()

    # ---------------------------------------------------------- I/O
    def load(self) -> None:
        try:
            data = json.loads(self.path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            self.user = {}
            return
        if isinstance(data, dict):
            self.user = {str(k): dict(v) for k, v in data.items()
                         if isinstance(v, dict)}
        else:
            self.user = {}

    def save(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        # Review-Fund: ein direktes write_text() ueber die lebende Datei ist
        # NICHT atomar -- ein Fehler/Absturz mitten im Schreiben (voller
        # Datentraeger etc.) haette die gesamte Preset-Bibliothek des Nutzers
        # korrumpiert. Erst in eine Temp-Datei schreiben, dann per Rename
        # ersetzen (Path.replace() ist auf demselben Dateisystem atomar).
        tmp = self.path.with_suffix(self.path.suffix + ".tmp")
        tmp.write_text(json.dumps(self.user, indent=2), encoding="utf-8")
        tmp.replace(self.path)

    # ---------------------------------------------------------- Query
    def names(self) -> list[str]:
        """Alle Preset-Namen, alphabetisch -- Builtins + User vereinigt."""
        return sorted(set(self.builtins) | set(self.user))

    def get(self, name: str) -> dict | None:
        """Parameter-Dict zu `name` (User hat Vorrang vor Builtin)."""
        if name in self.user:
            return dict(self.user[name])
        if name in self.builtins:
            return dict(self.builtins[name])
        return None

    def is_builtin(self, name: str) -> bool:
        """True, wenn `name` NUR als Builtin existiert (also nicht loeschbar)."""
        return name in self.builtins and name not in self.user

    # ---------------------------------------------------------- Mutate
    def add(self, name: str, state: dict) -> None:
        name = str(name).strip()
        if not name:
            raise ValueError("Preset-Name darf nicht leer sein")
        self.user[name] = dict(state)
        self.save()

    def remove(self, name: str) -> bool:
        """Loescht ein User-Preset. Builtins bleiben unangetastet (False)."""
        if name in self.user:
            del self.user[name]
            self.save()
            return True
        return False
