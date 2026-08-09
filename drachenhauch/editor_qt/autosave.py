"""Auto-Save fuer dirty Tabs + Crash-Recovery beim Start.

Schreibt Snapshots in `%APPDATA%/Drachenhauch/autosave/` plus eine
`manifest.json` mit Mappings auf Original-Pfade. Beim naechsten
Editor-Start liest `recover_unsaved()` das Manifest und liefert die
Eintraege zur Wiederaufnahme. Nach erfolgreichem Save (oder explizitem
"Verwerfen") wird der Snapshot-Eintrag entfernt.

Snapshots ueberschreiben sich pro Tab: ein Tab fuer dieselbe Datei
benutzt immer denselben Slug (sha1 des Pfads), neue unbenannte Tabs
nutzen `unnamed_<index>.dh`.
"""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path

from .settings import config_dir


AUTOSAVE_INTERVAL_MS = 30_000   # alle 30 Sekunden


def autosave_dir() -> Path:
    folder = config_dir() / "autosave"
    try:
        folder.mkdir(parents=True, exist_ok=True)
    except OSError:
        pass
    return folder


def manifest_path() -> Path:
    return autosave_dir() / "manifest.json"


def slug_for(file_path: Path | None, tab_idx: int) -> str:
    """Stabiler Snapshot-Dateiname pro Tab.

    - Existierende Datei: sha1 des absoluten Pfads (gleicher Slug bei
      jedem Save -> wir ueberschreiben).
    - Unbenannter Tab: `unnamed_<idx>.dh`.
    """
    if file_path is not None:
        h = hashlib.sha1(str(file_path).encode("utf-8")).hexdigest()[:12]
        return f"saved_{h}.dh"
    return f"unnamed_{tab_idx}.dh"


@dataclass
class RecoveryEntry:
    autosave_file: str           # Slug-Filename relativ zu autosave_dir()
    original_path: str | None    # Original-Pfad (kann None sein bei "(neu)")
    label: str                   # Anzeigename ("01_hello.dh" oder "(neu)")

    @property
    def autosave_full_path(self) -> Path:
        return autosave_dir() / self.autosave_file


def write_manifest(entries: list[RecoveryEntry]) -> bool:
    """Liefert False bei Schreibfehler (z.B. Ordner nicht beschreibbar/voll)
    -- der Aufrufer kann das nutzen, um dem User EINMALIG einen Hinweis zu
    geben, statt dass das Autosave-Sicherheitsnetz unbemerkt ausfaellt."""
    # Review-Fund: ein direktes write_text() ueber die lebende Manifest-Datei
    # ist nicht atomar -- ein Crash/Stromausfall MITTEN im Schreiben (das
    # Manifest wird per 30s-Timer und bei jedem Tab-Close neu geschrieben)
    # haette genau das Sicherheitsnetz kaputt hinterlassen, das fuer den
    # Crash-Fall gedacht ist. Gleiches Muster wie bereits in
    # preset_library.py gefixt: erst in eine Temp-Datei schreiben, dann
    # per Path.replace() (auf demselben Dateisystem atomar) ersetzen.
    path = manifest_path()
    tmp = path.with_suffix(path.suffix + ".tmp")
    try:
        tmp.write_text(
            json.dumps(
                [{
                    "autosave_file": e.autosave_file,
                    "original_path": e.original_path,
                    "label": e.label,
                } for e in entries],
                ensure_ascii=False, indent=2,
            ),
            encoding="utf-8",
        )
        tmp.replace(path)
        return True
    except OSError:
        return False


def read_manifest() -> list[RecoveryEntry]:
    p = manifest_path()
    if not p.exists():
        return []
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return []
    if not isinstance(data, list):
        return []
    out: list[RecoveryEntry] = []
    for raw in data:
        if not isinstance(raw, dict):
            continue
        af = raw.get("autosave_file")
        op = raw.get("original_path")
        lb = raw.get("label")
        if not isinstance(af, str) or not isinstance(lb, str):
            continue
        if op is not None and not isinstance(op, str):
            continue
        out.append(RecoveryEntry(autosave_file=af, original_path=op, label=lb))
    return out


def clear_autosaves() -> None:
    """Loescht alle Snapshot-Dateien + Manifest. Fail-silent."""
    folder = autosave_dir()
    try:
        for f in folder.iterdir():
            try:
                f.unlink()
            except OSError:
                continue
    except OSError:
        return
