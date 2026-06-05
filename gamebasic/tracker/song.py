"""Song-/Pattern-Datenmodell des Trackers.

Ein **Song** hat ein Tempo (BPM), die Wellenformen der drei Ton-Kanaele,
eine Liste von **Patterns** und eine **Order** (Song-Arrangement: die
Reihenfolge, in der Patterns abgespielt werden -- ein Pattern darf mehrfach
vorkommen).

Ein **Pattern** ist ein Gitter aus `CHANNELS` Kanaelen x `rows` Reihen.
Kanaele 0..2 sind tonal (speichern eine MIDI-Note oder None), Kanal 3 ist
Drum/Noise (speichert einen Hit als beliebige Nicht-None-Note).

Serialisierung als JSON (Editor-eigenes Projektformat). GB-Code-Export
expandiert die Order zu einer flachen Timeline und schreibt einen frame-
basierten Player (`TRACKER_UPDATE`), kompatibel zum bisherigen Single-
Pattern-Player.
"""
from __future__ import annotations

import json

NOTE_NAMES = ("C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B")
WAVEFORMS = ("square", "saw", "sine", "triangle")
TONAL = 3                  # Kanaele 0..2 tonal, Kanal 3 = Noise/Drum
CHANNELS = TONAL + 1
DEFAULT_ROWS = 16
MIN_ROWS = 1
MAX_ROWS = 64

# Lautstaerke-Effekt pro Note: 1..15 (None = Standard-Lautstaerke).
VOL_MAX = 15
DEFAULT_AMP_PCT = 50       # Standard-Amplitude in Prozent (= bisherige 0.5)


def vol_to_pct(v: int) -> int:
    """Mappt eine Noten-Lautstaerke 1..15 auf einen Amplituden-Prozentwert
    1..100 (nie 0 -- 0 ist im Export reserviert fuer „Standard")."""
    return max(1, min(100, round(int(v) / VOL_MAX * 100)))


def midi_to_freq(m: int) -> float:
    return 440.0 * (2.0 ** ((m - 69) / 12.0))


def note_name(m: int) -> str:
    return f"{NOTE_NAMES[m % 12]}{m // 12 - 1}"


class Pattern:
    """Ein Pattern: CHANNELS x rows Gitter aus MIDI-Noten (oder None)."""

    __slots__ = ("name", "rows", "data", "vol")

    def __init__(self, name: str, rows: int = DEFAULT_ROWS):
        self.name = name
        self.rows = max(MIN_ROWS, min(MAX_ROWS, int(rows)))
        # data[channel][row] = MIDI-Note (int) oder None
        self.data: list[list[int | None]] = [
            [None] * self.rows for _ in range(CHANNELS)]
        # vol[channel][row] = Lautstaerke 1..15 oder None (= Standard).
        # Nur sinnvoll, wo auch eine Note steht.
        self.vol: list[list[int | None]] = [
            [None] * self.rows for _ in range(CHANNELS)]

    def get(self, channel: int, row: int) -> int | None:
        return self.data[channel][row]

    def set(self, channel: int, row: int, note: int | None) -> None:
        self.data[channel][row] = note
        if note is None:                     # Note geloescht -> Lautstaerke mit
            self.vol[channel][row] = None

    def get_vol(self, channel: int, row: int) -> int | None:
        return self.vol[channel][row]

    def set_vol(self, channel: int, row: int, v: int | None) -> None:
        """Setzt die Noten-Lautstaerke 1..15 (None/<=0 = Standard). Wirkt nur,
        wenn an der Stelle eine Note steht."""
        if self.data[channel][row] is None:
            return
        if v is None or int(v) <= 0:
            self.vol[channel][row] = None
        else:
            self.vol[channel][row] = max(1, min(VOL_MAX, int(v)))

    def set_rows(self, rows: int) -> None:
        """Aendert die Reihenzahl; bestehende Noten oben bleiben erhalten."""
        rows = max(MIN_ROWS, min(MAX_ROWS, int(rows)))
        for grid in (self.data, self.vol):
            for c in range(CHANNELS):
                col = grid[c]
                if rows < len(col):
                    del col[rows:]
                else:
                    col.extend([None] * (rows - len(col)))
        self.rows = rows

    def clear(self) -> None:
        self.data = [[None] * self.rows for _ in range(CHANNELS)]
        self.vol = [[None] * self.rows for _ in range(CHANNELS)]

    def copy(self, name: str | None = None) -> "Pattern":
        p = Pattern(name if name is not None else self.name, self.rows)
        p.data = [list(col) for col in self.data]
        p.vol = [list(col) for col in self.vol]
        return p

    def _has_vol(self) -> bool:
        return any(v is not None for col in self.vol for v in col)

    def to_dict(self) -> dict:
        d = {"name": self.name, "rows": self.rows, "data": self.data}
        # vol nur schreiben, wenn gesetzt -> alte Dateien/leere Patterns bleiben
        # schlank und abwaerts-kompatibel.
        if self._has_vol():
            d["vol"] = self.vol
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Pattern":
        p = cls(str(d.get("name", "Pattern")), int(d.get("rows", DEFAULT_ROWS)))
        raw = d.get("data") or []
        for c in range(CHANNELS):
            if c < len(raw):
                col = [int(v) if isinstance(v, (int, float)) else None
                       for v in raw[c]][:p.rows]
                col += [None] * (p.rows - len(col))
                p.data[c] = col
        rawv = d.get("vol") or []
        for c in range(CHANNELS):
            if c < len(rawv):
                col = [max(1, min(VOL_MAX, int(v)))
                       if isinstance(v, (int, float)) else None
                       for v in rawv[c]][:p.rows]
                col += [None] * (p.rows - len(col))
                p.vol[c] = col
        return p


class Song:
    """Tempo + Wellenformen + Patterns + Order (Arrangement)."""

    FORMAT = "gbtracker-song"
    VERSION = 1

    def __init__(self):
        self.bpm = 120
        self.waves = ["square", "saw", "triangle"]   # Kanal 0..2
        self.patterns: list[Pattern] = [Pattern("P1")]
        self.order: list[int] = [0]                  # Indizes in patterns
        self.path = ""

    # ------------------------------------------------------ Tempo
    def row_ms(self) -> int:
        """Millisekunden pro Reihe (16tel-Schritte)."""
        return int(60000 / max(1, self.bpm) / 4)

    # ------------------------------------------------------ Pattern-Ops
    def add_pattern(self, name: str | None = None,
                    rows: int = DEFAULT_ROWS) -> int:
        n = name or f"P{len(self.patterns) + 1}"
        self.patterns.append(Pattern(n, rows))
        return len(self.patterns) - 1

    def duplicate_pattern(self, idx: int) -> int:
        src = self.patterns[idx]
        self.patterns.append(src.copy(f"{src.name}*"))
        return len(self.patterns) - 1

    def remove_pattern(self, idx: int) -> None:
        """Loescht Pattern `idx` (mind. eines bleibt). Order wird angepasst:
        Verweise auf das geloeschte Pattern fallen raus, hoehere Indizes
        ruecken nach. Wird die Order leer, zeigt sie auf Pattern 0."""
        if len(self.patterns) <= 1 or not (0 <= idx < len(self.patterns)):
            return
        del self.patterns[idx]
        new_order = []
        for p in self.order:
            if p == idx:
                continue
            new_order.append(p - 1 if p > idx else p)
        self.order = new_order or [0]

    # ------------------------------------------------------ Order-Ops
    def order_add(self, pattern_idx: int) -> None:
        if 0 <= pattern_idx < len(self.patterns):
            self.order.append(pattern_idx)

    def order_remove(self, pos: int) -> None:
        if 0 <= pos < len(self.order) and len(self.order) > 1:
            del self.order[pos]

    def order_move(self, pos: int, delta: int) -> int:
        j = pos + delta
        if 0 <= pos < len(self.order) and 0 <= j < len(self.order):
            self.order[pos], self.order[j] = self.order[j], self.order[pos]
            return j
        return pos

    # ------------------------------------------------------ Flatten
    def _flatten(self) -> tuple[int, list[list[int]], list[list[int]]]:
        """Wie `flatten`, liefert zusaetzlich die Lautstaerke-Spuren `vols`:
        `vols[c][i]` = Amplituden-Prozent (1..100) bei explizit gesetzter
        Noten-Lautstaerke, sonst 0 (= Standard). Nur dort relevant, wo auch
        `channels[c][i] != 0` ist."""
        order = self.order or [0]
        total = sum(self.patterns[p].rows for p in order
                    if 0 <= p < len(self.patterns))
        total = max(1, total)
        channels = [[0] * total for _ in range(CHANNELS)]
        vols = [[0] * total for _ in range(CHANNELS)]
        i = 0
        for p in order:
            if not (0 <= p < len(self.patterns)):
                continue
            pat = self.patterns[p]
            for r in range(pat.rows):
                for c in range(CHANNELS):
                    note = pat.data[c][r]
                    if note is None:
                        continue
                    channels[c][i + r] = (
                        1 if c == TONAL else int(round(midi_to_freq(note))))
                    v = pat.vol[c][r]
                    if v is not None:
                        vols[c][i + r] = vol_to_pct(v)
            i += pat.rows
        return total, channels, vols

    def flatten(self) -> tuple[int, list[list[int]]]:
        """Expandiert die Order zu einer flachen Timeline.

        Liefert `(total_rows, channels)` mit `channels[c]` = Liste von
        Werten ueber den ganzen Song: 0 = nichts, sonst Frequenz in Hz
        (tonal) bzw. 1 (Drum-Hit)."""
        total, channels, _ = self._flatten()
        return total, channels

    # ------------------------------------------------------ JSON-I/O
    def to_dict(self) -> dict:
        return {
            "format": self.FORMAT,
            "version": self.VERSION,
            "bpm": self.bpm,
            "waves": list(self.waves),
            "patterns": [p.to_dict() for p in self.patterns],
            "order": list(self.order),
        }

    @classmethod
    def from_dict(cls, d: dict) -> "Song":
        s = cls()
        s.bpm = int(d.get("bpm", 120))
        waves = d.get("waves") or s.waves
        s.waves = [str(w) for w in waves][:TONAL]
        while len(s.waves) < TONAL:
            s.waves.append("square")
        pats = d.get("patterns") or []
        s.patterns = [Pattern.from_dict(p) for p in pats] or [Pattern("P1")]
        order = [int(p) for p in (d.get("order") or [0])
                 if 0 <= int(p) < len(s.patterns)]
        s.order = order or [0]
        return s

    def save_json(self, path: str) -> None:
        with open(path, "w", encoding="utf-8") as f:
            json.dump(self.to_dict(), f, indent=1)
        self.path = path

    @classmethod
    def load_json(cls, path: str) -> "Song":
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        s = cls.from_dict(data)
        s.path = path
        return s

    # ------------------------------------------------------ GB-Code-Export
    def gb_code(self) -> str:
        """Selbststaendiger frame-basierter Player. Die Order wird zu einer
        flachen Timeline expandiert (wiederholte Patterns werden dupliziert)
        -- so bleibt der Player simpel und identisch zum bisherigen Schema."""
        total, channels, vols = self._flatten()
        rowms = self.row_ms()
        n_patterns = len(self.patterns)
        has_vol = any(any(col) for col in vols)
        lines = [
            'IMPORT "audio"', "",
            f"' === Tracker-Song (gbtracker) -- {n_patterns} Pattern(s), "
            f"Order-Laenge {len(self.order)}, {total} Reihen, BPM {self.bpm} ===",
            f"CONST TRK_ROWS = {total}",
            f"CONST TRK_ROWMS = {rowms}", "",
        ]
        for c in range(CHANNELS):
            lines.append(f"DIM trk{c}[TRK_ROWS] AS INTEGER")
            for r, val in enumerate(channels[c]):
                if val:
                    lines.append(f"trk{c}[{r}] = {val}")
        if has_vol:
            # Lautstaerke-Spuren (Amplitude in Prozent, 0 = Standard).
            for c in range(CHANNELS):
                if not any(vols[c]):
                    continue
                lines.append(f"DIM trkV{c}[TRK_ROWS] AS INTEGER")
                for r, val in enumerate(vols[c]):
                    if val:
                        lines.append(f"trkV{c}[{r}] = {val}")
        lines += [
            "",
            "' --- Player-Status + Update (im Game-Loop aufrufen) ---",
            "DIM trkRow AS INTEGER",
            "DIM trkAcc AS FLOAT",
            "trkRow = 0",
            "trkAcc = 0.0",
            "",
        ]
        if has_vol:
            lines += [
                "' Lautstaerke-Spur -> Amplitude (0 = Standard 0.5)",
                "FUNCTION TRACKER_AMP(pct AS INTEGER) AS FLOAT",
                "    IF pct <= 0 THEN RETURN 0.5",
                "    RETURN pct / 100.0",
                "END FUNCTION",
                "",
            ]
        lines.append("SUB TRACKER_PLAY_ROW(r AS INTEGER)")
        for c in range(TONAL):
            amp = (f"TRACKER_AMP(trkV{c}[r])"
                   if has_vol and any(vols[c]) else "0.5")
            lines.append(
                f"    IF trk{c}[r] > 0 THEN PLAYSOUND("
                f'AUDIO_TONE(trk{c}[r], TRK_ROWMS, "{self.waves[c]}", {amp}))')
        damp = (f"TRACKER_AMP(trkV{TONAL}[r])"
                if has_vol and any(vols[TONAL]) else "0.5")
        lines.append(
            f"    IF trk{TONAL}[r] > 0 THEN PLAYSOUND(AUDIO_NOISE(120, {damp}))")
        lines += [
            "END SUB",
            "",
            "SUB TRACKER_UPDATE(dt_ms AS FLOAT)",
            "    trkAcc = trkAcc + dt_ms",
            "    WHILE trkAcc >= TRK_ROWMS",
            "        trkAcc = trkAcc - TRK_ROWMS",
            "        TRACKER_PLAY_ROW(trkRow)",
            "        trkRow = (trkRow + 1) MOD TRK_ROWS",
            "    WEND",
            "END SUB",
            "",
            "' Im Game-Loop:  TRACKER_UPDATE(DELTA() * 1000.0)",
        ]
        return "\n".join(lines) + "\n"
