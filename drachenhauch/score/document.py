"""Notenblatt-Dokument (Qt-frei): Spuren auf einem 5-Linien-System.

Ein `ScoreDoc` hat ein Tempo (BPM), ein Metrum (`time_sig`, v1 fest 4/4 --
siehe Hinweis bei `time_sig`) und mehrere `Track`s (Notenzeilen). Jede Spur
hat einen Notenschluessel (`clef`, treble/bass), GENAU EIN Instrument (kein
Pattern-Zell-Override wie im Tracker -- v1-Vereinfachung) und eine Liste von
`NoteEvent`s (Noten ODER Pausen, Zeiten in Viertel-Beats).

Konvertierung in ein Tracker-`Song` (Zeilenraster) siehe
`drachenhauch.score.convert.to_tracker_song`. Serialisierung als eigenes
JSON-Format (`"format": "dhscore-song"`), analog zu `drachenhauch.tracker.song`.
"""
from __future__ import annotations

import json
from typing import Any

CLEFS = ("treble", "bass")
# Toleranz fuer Beat-Identitaets-Vergleiche (Float-Rundungsfehler) -- gleicher
# Wert wie `convert._ROUND_EPS`, hier aber fuer Notengleichheit/Slur-Anker
# statt Tracker-Zeilen-Rundung.
_BEAT_EPS = 1e-6


class NoteEvent:
    """Eine Note ODER eine Pause auf einer Spur.

    Zeiten in Viertel-Beats (4/4: 1 Beat = 1 Viertelnote), float -- NICHT
    vorquantisiert im Modell selbst (Quantisierung passiert bei der
    UI-Noteneingabe + nochmal beim Tracker-Export, siehe `convert.py`).

    `staccato`/`fingering` sind reine Notationszusaetze (wie `Track.slurs`).
    `staccato` wirkt zusaetzlich auf die Wiedergabe/den Tracker-Export
    (verkuerzte Klangdauer, siehe `_apply_envelope`-Analogon in
    `scoreeditor_qt._trigger_note` und `convert.STACCATO_FACTOR`);
    `fingering` (1..5, Klavier-Fingersatz) ist rein informativ.

    `accidental` (-1/0/+1 = B/natuerlich/Kreuz) haelt fest, WELCHES
    Vorzeichen beim Setzen der Note in der Toolbar aktiv war -- rein
    fuer die Anzeige (`♯`/`♭`), der Klang haengt nur an `pitch` (MIDI).
    Ohne das koennte die Darstellung nicht zwischen enharmonisch
    gleichen Toenen unterscheiden (z.B. Db vs. C#, beide MIDI-pitch=61)
    -- Default 1 (Kreuz) reproduziert die alte, feste "immer als Kreuz"-
    Darstellung fuer aeltere/ohne Toolbar erzeugte Noten."""

    __slots__ = ("start_beat", "dur_beat", "pitch", "rest", "staccato",
                 "fingering", "accidental")

    def __init__(self, start_beat: float, dur_beat: float,
                 pitch: int | None = None, rest: bool = False,
                 staccato: bool = False, fingering: int | None = None,
                 accidental: int = 1):
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
        self.staccato = bool(staccato) and not self.rest
        self.fingering = (max(1, min(5, int(fingering)))
                          if fingering is not None and not self.rest else None)
        self.accidental = max(-1, min(1, int(accidental))) if not self.rest else 0

    @property
    def end_beat(self) -> float:
        return self.start_beat + self.dur_beat

    def to_dict(self) -> dict:
        # explizit dict[str, Any] annotiert -- sonst leitet mypy den
        # Werttyp allein aus den ersten beiden (float-)Eintraegen ab und
        # meckert bei den spaeteren bool/int/None-Werten.
        d: dict[str, Any] = {"start": self.start_beat, "dur": self.dur_beat}
        if self.rest:
            d["rest"] = True
        else:
            d["pitch"] = self.pitch
        if self.staccato:
            d["staccato"] = True
        if self.fingering is not None:
            d["fingering"] = self.fingering
        if self.accidental != 1:
            d["accidental"] = self.accidental
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "NoteEvent":
        start = float(d.get("start", 0.0))
        dur = float(d.get("dur", 1.0))
        staccato = bool(d.get("staccato", False))
        fingering = d.get("fingering")
        accidental = int(d.get("accidental", 1))
        if d.get("rest"):
            return cls(start, dur, rest=True, staccato=staccato)
        pitch = d.get("pitch")
        return cls(start, dur,
                    pitch=int(pitch) if pitch is not None else None,
                    staccato=staccato, fingering=fingering,
                    accidental=accidental)


class Track:
    """Eine Notenzeile (Stimme): Notenschluessel + EIN Instrument + Noten.

    Anders als der Tracker (Instrument-Pool + optionales Pattern-Zell-
    Ueberschreiben) hat eine Score-Spur GENAU EIN Instrument -- v1-
    Vereinfachung, siehe CLAUDE.md-Abschnitt zum Notenblatt-Editor.

    `slurs` sind rein visuelle Phrasierungsboegen -- Paare von Beat-
    Positionen (nicht Objekt-Referenzen, damit sie JSON-serialisierbar
    bleiben und Undo/Redo-Snapshots sie automatisch mitnehmen)."""

    __slots__ = ("name", "clef", "instrument", "notes", "slurs")

    def __init__(self, name: str = "Stimme 1", clef: str = "treble",
                 instrument=None):
        self.name = name
        self.clef = clef if clef in CLEFS else "treble"
        if instrument is None:
            from ..tracker.instrument import Instrument
            instrument = Instrument.synth(name, "triangle")
        self.instrument = instrument
        self.notes: list[NoteEvent] = []
        self.slurs: list[tuple[float, float]] = []

    def add_note(self, start_beat: float, dur_beat: float,
                 pitch: int | None = None, rest: bool = False,
                 staccato: bool = False, fingering: int | None = None,
                 accidental: int = 1) -> NoteEvent:
        ev = NoteEvent(start_beat, dur_beat, pitch=pitch, rest=rest,
                      staccato=staccato, fingering=fingering,
                      accidental=accidental)
        self.notes.append(ev)
        self.notes.sort(key=lambda n: n.start_beat)
        return ev

    def remove_note(self, ev: NoteEvent) -> None:
        if ev in self.notes:
            self.notes.remove(ev)
            # Nur aufraeumen, wenn keine andere Note (Akkord-Nachbar mit
            # identischem start_beat) noch dort sitzt -- sonst wuerde ein
            # Bogen geloescht, der eigentlich der verbliebenen Note gehoert
            # (slurs sind nur an Beat-Positionen gebunden, nicht an Noten,
            # siehe Klassendocstring).
            if not any(abs(n.start_beat - ev.start_beat) < _BEAT_EPS
                      for n in self.notes):
                self.remove_slurs_at(ev.start_beat)

    def add_slur(self, start_beat: float, end_beat: float) -> None:
        """Fuegt einen Bindebogen zwischen zwei Beat-Positionen hinzu (rein
        visuell -- keine Wirkung auf Wiedergabe/Tracker-Export)."""
        lo, hi = (start_beat, end_beat) if start_beat <= end_beat else (end_beat, start_beat)
        if abs(hi - lo) < _BEAT_EPS:
            return
        pair = (float(lo), float(hi))
        if not any(abs(a - pair[0]) < _BEAT_EPS and abs(b - pair[1]) < _BEAT_EPS
                  for a, b in self.slurs):
            self.slurs.append(pair)

    def remove_slurs_at(self, beat: float) -> None:
        """Entfernt alle Bindeboegen, die an dieser Beat-Position beginnen
        oder enden (z.B. wenn die zugehoerige Note geloescht wird)."""
        self.slurs = [(a, b) for a, b in self.slurs
                     if abs(a - beat) > _BEAT_EPS and abs(b - beat) > _BEAT_EPS]

    def relocate_slurs(self, old_beat: float, new_beat: float) -> None:
        """Verschiebt Bindebogen-Anker von `old_beat` nach `new_beat` (wenn
        die zugehoerige Note per Ziehen verschoben wird). Re-sortiert das
        Paar zu (lo, hi), falls der verschobene Anker die andere Seite
        ueberholt hat.

        Erwartet, dass die gezogene Note SELBST bereits auf `new_beat`
        steht, wenn dies aufgerufen wird (wie beim Drag in
        `scoreeditor_qt.py`, wo `note.start_beat` schon waehrend
        `mouseMoveEvent` mutiert wird) -- sonst kann ein noch auf
        `old_beat` sitzender Akkord-Nachbar nicht von der bereits
        verschobenen Note unterschieden werden (siehe Guard unten)."""
        if abs(old_beat - new_beat) < _BEAT_EPS:
            return
        if any(abs(n.start_beat - old_beat) < _BEAT_EPS for n in self.notes):
            # Eine andere Note (Akkord-Nachbar) sitzt noch auf old_beat --
            # ein dort verankerter Bogen gehoert wahrscheinlich ihr, nicht
            # der verschobenen Note. Nicht mitziehen (gleiche Vorsicht wie
            # remove_note).
            return
        relocated = [
            (new_beat if abs(a - old_beat) < _BEAT_EPS else a,
             new_beat if abs(b - old_beat) < _BEAT_EPS else b)
            for a, b in self.slurs]
        self.slurs = [(min(a, b), max(a, b)) for a, b in relocated]

    def length_beats(self) -> float:
        return max((n.end_beat for n in self.notes), default=0.0)

    def to_dict(self) -> dict:
        d = {"name": self.name, "clef": self.clef,
             "instrument": self.instrument.to_dict(),
             "notes": [n.to_dict() for n in self.notes]}
        if self.slurs:
            d["slurs"] = [[a, b] for a, b in self.slurs]
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Track":
        from ..tracker.instrument import Instrument
        inst_d = d.get("instrument")
        instrument = Instrument.from_dict(inst_d) if inst_d else None
        t = cls(str(d.get("name", "Stimme")),
                str(d.get("clef", "treble")), instrument=instrument)
        t.notes = [NoteEvent.from_dict(n) for n in (d.get("notes") or [])]
        t.slurs = [(float(a), float(b)) for a, b in (d.get("slurs") or [])]
        t.notes.sort(key=lambda n: n.start_beat)
        return t


class ScoreDoc:
    """Tempo + Metrum + Spuren (Notenblatt-Projekt)."""

    FORMAT = "dhscore-song"
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
