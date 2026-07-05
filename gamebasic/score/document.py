"""Notenblatt-Dokument (Qt-frei): Spuren auf einem 5-Linien-System.

Ein `ScoreDoc` hat ein Tempo (BPM), ein Metrum (`time_sig`, v1 fest 4/4 --
siehe Hinweis bei `time_sig`) und mehrere `Track`s (Notenzeilen). Jede Spur
hat einen Notenschluessel (`clef`, treble/bass), GENAU EIN Instrument (kein
Pattern-Zell-Override wie im Tracker -- v1-Vereinfachung) und eine Liste von
`NoteEvent`s (Noten ODER Pausen, Zeiten in Viertel-Beats).

Konvertierung in ein Tracker-`Song` (Zeilenraster) siehe
`gamebasic.score.convert.to_tracker_song`. Serialisierung als eigenes
JSON-Format (`"format": "gbscore-song"`), analog zu `gamebasic.tracker.song`.
"""
from __future__ import annotations

import json

CLEFS = ("treble", "bass")


class NoteEvent:
    """Eine Note ODER eine Pause auf einer Spur.

    Zeiten in Viertel-Beats (4/4: 1 Beat = 1 Viertelnote), float -- NICHT
    vorquantisiert im Modell selbst (Quantisierung passiert bei der
    UI-Noteneingabe + nochmal beim Tracker-Export, siehe `convert.py`)."""

    __slots__ = ("start_beat", "dur_beat", "pitch", "rest")

    def __init__(self, start_beat: float, dur_beat: float,
                 pitch: int | None = None, rest: bool = False):
        self.start_beat = float(start_beat)
        self.dur_beat = max(0.0, float(dur_beat))
        # rest=True erzwingt pitch=None und umgekehrt -- keine
        # inkonsistenten Zustaende moeglich (single source of truth).
        if rest or pitch is None:
            self.pitch = None
            self.rest = True
        else:
            self.pitch = int(pitch)
            self.rest = False

    @property
    def end_beat(self) -> float:
        return self.start_beat + self.dur_beat

    def to_dict(self) -> dict:
        d = {"start": self.start_beat, "dur": self.dur_beat}
        if self.rest:
            d["rest"] = True
        else:
            d["pitch"] = self.pitch
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "NoteEvent":
        start = float(d.get("start", 0.0))
        dur = float(d.get("dur", 1.0))
        if d.get("rest"):
            return cls(start, dur, rest=True)
        pitch = d.get("pitch")
        return cls(start, dur,
                    pitch=int(pitch) if pitch is not None else None)


class Track:
    """Eine Notenzeile (Stimme): Notenschluessel + EIN Instrument + Noten.

    Anders als der Tracker (Instrument-Pool + optionales Pattern-Zell-
    Ueberschreiben) hat eine Score-Spur GENAU EIN Instrument -- v1-
    Vereinfachung, siehe CLAUDE.md-Abschnitt zum Notenblatt-Editor."""

    __slots__ = ("name", "clef", "instrument", "notes")

    def __init__(self, name: str = "Stimme 1", clef: str = "treble",
                 instrument=None):
        self.name = name
        self.clef = clef if clef in CLEFS else "treble"
        if instrument is None:
            from ..tracker.instrument import Instrument
            instrument = Instrument.synth(name, "triangle")
        self.instrument = instrument
        self.notes: list[NoteEvent] = []

    def add_note(self, start_beat: float, dur_beat: float,
                 pitch: int | None = None, rest: bool = False) -> NoteEvent:
        ev = NoteEvent(start_beat, dur_beat, pitch=pitch, rest=rest)
        self.notes.append(ev)
        self.notes.sort(key=lambda n: n.start_beat)
        return ev

    def remove_note(self, ev: NoteEvent) -> None:
        if ev in self.notes:
            self.notes.remove(ev)

    def length_beats(self) -> float:
        return max((n.end_beat for n in self.notes), default=0.0)

    def to_dict(self) -> dict:
        return {"name": self.name, "clef": self.clef,
                "instrument": self.instrument.to_dict(),
                "notes": [n.to_dict() for n in self.notes]}

    @classmethod
    def from_dict(cls, d: dict) -> "Track":
        from ..tracker.instrument import Instrument
        inst_d = d.get("instrument")
        instrument = Instrument.from_dict(inst_d) if inst_d else None
        t = cls(str(d.get("name", "Stimme")),
                str(d.get("clef", "treble")), instrument=instrument)
        t.notes = [NoteEvent.from_dict(n) for n in (d.get("notes") or [])]
        t.notes.sort(key=lambda n: n.start_beat)
        return t


class ScoreDoc:
    """Tempo + Metrum + Spuren (Notenblatt-Projekt)."""

    FORMAT = "gbscore-song"
    VERSION = 1

    def __init__(self):
        self.bpm = 120
        # v1-Limitation: nur 4/4 wird von der UI unterstuetzt/erzeugt; das
        # Feld rundet aber unveraendert durch JSON, damit eine kuenftige
        # Version andere Metren lesen koennte, ohne alte Dateien zu brechen.
        self.time_sig: tuple[int, int] = (4, 4)
        self.tracks: list[Track] = [Track("Stimme 1", "treble")]
        self.path = ""

    def add_track(self, name: str | None = None, clef: str = "treble") -> int:
        n = name or f"Stimme {len(self.tracks) + 1}"
        self.tracks.append(Track(n, clef))
        return len(self.tracks) - 1

    def remove_track(self, idx: int) -> None:
        """Loescht Spur `idx` (mindestens eine bleibt erhalten -- wie
        `Song.remove_pattern`/`TileMapDoc.remove_layer`)."""
        if 0 <= idx < len(self.tracks) and len(self.tracks) > 1:
            del self.tracks[idx]

    def length_beats(self) -> float:
        return max((t.length_beats() for t in self.tracks), default=0.0)

    def to_dict(self) -> dict:
        return {
            "format": self.FORMAT,
            "version": self.VERSION,
            "bpm": self.bpm,
            "time_sig": list(self.time_sig),
            "tracks": [t.to_dict() for t in self.tracks],
        }

    @classmethod
    def from_dict(cls, d: dict) -> "ScoreDoc":
        """Permissiv wie `Song.from_dict` -- prueft den `format`-Marker
        nicht, toleriert fehlende Felder mit sinnvollen Defaults, damit
        auch eine handbearbeitete/formatlose JSON-Datei laedt."""
        doc = cls()
        doc.bpm = int(d.get("bpm", 120))
        ts = d.get("time_sig") or [4, 4]
        doc.time_sig = (int(ts[0]), int(ts[1])) if len(ts) == 2 else (4, 4)
        tracks = d.get("tracks") or []
        doc.tracks = [Track.from_dict(t) for t in tracks] or [
            Track("Stimme 1", "treble")]
        return doc

    def save_json(self, path: str) -> None:
        with open(path, "w", encoding="utf-8") as f:
            json.dump(self.to_dict(), f, indent=1)
        self.path = path

    @classmethod
    def load_json(cls, path: str) -> "ScoreDoc":
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        doc = cls.from_dict(data)
        doc.path = path
        return doc
