"""Song-/Pattern-Datenmodell des Trackers.

Ein **Song** hat ein Tempo (BPM), eine konfigurierbare Kanalzahl
(`Song.channels`, 4..32 -- `Song.set_channels()`), die Wellenformen der
tonalen Kanaele, eine Liste von **Patterns** und eine **Order** (Song-
Arrangement: die Reihenfolge, in der Patterns abgespielt werden -- ein
Pattern darf mehrfach vorkommen).

Ein **Pattern** ist ein Gitter aus `Pattern.channels` Kanaelen x `rows`
Reihen (immer identisch zu `Song.channels`, den es erzeugt hat). Kanal
0..`Song.tonal - 1` sind tonal (speichern eine MIDI-Note oder None), der
LETZTE Kanal (`Song.tonal`) ist Drum/Noise (speichert einen Hit als
beliebige Nicht-None-Note) -- unabhaengig von der Gesamt-Kanalzahl.

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
CHANNELS = TONAL + 1       # Default-/Mindest-Kanalzahl neuer Songs
MIN_CHANNELS = CHANNELS
MAX_CHANNELS = 32         # XM-Tracker-Niveau; letzter Kanal bleibt immer Drum
DEFAULT_ROWS = 16
MIN_ROWS = 1
MAX_ROWS = 64

# Note-Off: eine Zelle, die eine klingende Note explizit VOR der naechsten
# echten Note abschneidet (klassisches Tracker-Konzept, XM/IT nennen es
# "Key Off"). Liegt im selben Wertebereich wie `data[c][r]` (int | None),
# damit alle bestehenden "naechstes Event"-Grenzen (Mixer-Sustain, Live-
# Vorhoeren-Laenge) es automatisch als Grenze erkennen -- kein Sonder-Array
# noetig. -1 ist ausserhalb des gueltigen MIDI-Bereichs (0..127).
NOTE_OFF = -1

# Lautstaerke-Effekt pro Note: 1..15 (None = Standard-Lautstaerke).
VOL_MAX = 15
DEFAULT_AMP_PCT = 50       # Standard-Amplitude in Prozent (= bisherige 0.5)
# Pitch-Slide-Effekt pro Note: -SLIDE_MAX..+SLIDE_MAX Halbtoene ueber die
# Reihen-Dauer (0/None = kein Slide). Beim Export zu Hz/s vorberechnet.
SLIDE_MAX = 12

# Effekt-Spalte pro Note (klassische Tracker-Effekte). `fx` = Code, `fxp` =
# Parameter (ein Byte, 0..255 -- bei ARP/VIB als zwei Nibbles x/y gelesen).
FX_NONE = 0
FX_ARP = 1          # Arpeggio: Note, Note+x, Note+y (x,y = fxp-Nibbles, Halbtoene)
FX_VIB = 2          # Vibrato: Speed x (Hz), Tiefe y (Nibble -> y*0.125 Halbtoene)
FX_RET = 3          # Retrigger: Note alle fxp Ticks neu anschlagen
FX_OFF = 4          # Sample-Offset: Start um fxp*512 Frames spaeter
FX_CODES = (FX_NONE, FX_ARP, FX_VIB, FX_RET, FX_OFF)
FX_NAMES = {FX_NONE: "—", FX_ARP: "Arp", FX_VIB: "Vib",
            FX_RET: "Ret", FX_OFF: "Off"}
# Ticks pro Reihe (Granularitaet von Arp/Vibrato/Retrigger im Render).
TICKS_PER_ROW = 6


def slide_hz_per_s(freq: float, semitones: int, row_ms: int) -> int:
    """Pitch-Slide (Halbtoene ueber eine Reihe) -> Hz/s fuer AUDIO_SFX."""
    if not semitones or row_ms <= 0:
        return 0
    target = freq * (2.0 ** (semitones / 12.0))
    return int(round((target - freq) * 1000.0 / row_ms))


def vol_to_pct(v: int) -> int:
    """Mappt eine Noten-Lautstaerke 1..15 auf einen Amplituden-Prozentwert
    1..100 (nie 0 -- 0 ist im Export reserviert fuer „Standard")."""
    return max(1, min(100, round(int(v) / VOL_MAX * 100)))


def midi_to_freq(m: int) -> float:
    return 440.0 * (2.0 ** ((m - 69) / 12.0))


def note_name(m: int) -> str:
    return f"{NOTE_NAMES[m % 12]}{m // 12 - 1}"


class Pattern:
    """Ein Pattern: `channels` x rows Gitter aus MIDI-Noten (oder None)."""

    __slots__ = ("name", "rows", "channels", "data", "vol", "slide", "fx", "fxp")

    def __init__(self, name: str, rows: int = DEFAULT_ROWS,
                 channels: int = CHANNELS):
        self.name = name
        self.rows = max(MIN_ROWS, min(MAX_ROWS, int(rows)))
        self.channels = max(MIN_CHANNELS, min(MAX_CHANNELS, int(channels)))
        # data[channel][row] = MIDI-Note (int) oder None
        self.data: list[list[int | None]] = [
            [None] * self.rows for _ in range(self.channels)]
        # vol[channel][row] = Lautstaerke 1..15 oder None (= Standard).
        # Nur sinnvoll, wo auch eine Note steht.
        self.vol: list[list[int | None]] = [
            [None] * self.rows for _ in range(self.channels)]
        # slide[channel][row] = Pitch-Slide in Halbtoenen (-12..12) oder None.
        self.slide: list[list[int | None]] = [
            [None] * self.rows for _ in range(self.channels)]
        # fx[channel][row] = Effekt-Code (FX_*), fxp = Parameter-Byte. None =
        # kein Effekt. Nur sinnvoll, wo auch eine Note steht.
        self.fx: list[list[int | None]] = [
            [None] * self.rows for _ in range(self.channels)]
        self.fxp: list[list[int | None]] = [
            [None] * self.rows for _ in range(self.channels)]

    def get(self, channel: int, row: int) -> int | None:
        return self.data[channel][row]

    def set(self, channel: int, row: int, note: int | None) -> None:
        self.data[channel][row] = note
        if note is None or note == NOTE_OFF:  # geloescht/Note-Off -> Effekte mit
            self.vol[channel][row] = None
            self.slide[channel][row] = None
            self.fx[channel][row] = None
            self.fxp[channel][row] = None

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

    def get_slide(self, channel: int, row: int) -> int | None:
        return self.slide[channel][row]

    def set_slide(self, channel: int, row: int, s: int | None) -> None:
        """Setzt den Pitch-Slide -12..12 Halbtoene (None/0 = kein Slide).
        Wirkt nur, wenn an der Stelle eine Note steht."""
        if self.data[channel][row] is None:
            return
        if s is None or int(s) == 0:
            self.slide[channel][row] = None
        else:
            self.slide[channel][row] = max(-SLIDE_MAX, min(SLIDE_MAX, int(s)))

    def get_fx(self, channel: int, row: int) -> tuple[int, int]:
        """(Effekt-Code, Parameter) der Zelle; (FX_NONE, 0) wenn keiner."""
        fx = self.fx[channel][row]
        if fx is None:
            return FX_NONE, 0
        return int(fx), int(self.fxp[channel][row] or 0)

    def set_fx(self, channel: int, row: int, fx: int | None,
               param: int = 0) -> None:
        """Setzt Effekt-Code + Parameter (0..255). FX_NONE/None loescht den
        Effekt. Wirkt nur, wenn an der Stelle eine Note steht."""
        if self.data[channel][row] is None:
            return
        if fx is None or int(fx) == FX_NONE or int(fx) not in FX_CODES:
            self.fx[channel][row] = None
            self.fxp[channel][row] = None
        else:
            self.fx[channel][row] = int(fx)
            self.fxp[channel][row] = max(0, min(255, int(param)))

    def set_rows(self, rows: int) -> None:
        """Aendert die Reihenzahl; bestehende Noten oben bleiben erhalten."""
        rows = max(MIN_ROWS, min(MAX_ROWS, int(rows)))
        for grid in (self.data, self.vol, self.slide, self.fx, self.fxp):
            for c in range(self.channels):
                col = grid[c]
                if rows < len(col):
                    del col[rows:]
                else:
                    col.extend([None] * (rows - len(col)))
        self.rows = rows

    def set_channels(self, channels: int) -> None:
        """Aendert die Kanalzahl; bestehende Kanaele/Noten bleiben erhalten,
        neue Kanaele kommen leer dazu (rechts angehaengt)."""
        channels = max(MIN_CHANNELS, min(MAX_CHANNELS, int(channels)))
        for name in ("data", "vol", "slide", "fx", "fxp"):
            grid = getattr(self, name)
            if channels < len(grid):
                del grid[channels:]
            else:
                grid.extend([None] * self.rows for _ in range(channels - len(grid)))
        self.channels = channels

    def clear(self) -> None:
        self.data = [[None] * self.rows for _ in range(self.channels)]
        self.vol = [[None] * self.rows for _ in range(self.channels)]
        self.slide = [[None] * self.rows for _ in range(self.channels)]
        self.fx = [[None] * self.rows for _ in range(self.channels)]
        self.fxp = [[None] * self.rows for _ in range(self.channels)]

    def copy(self, name: str | None = None) -> "Pattern":
        p = Pattern(name if name is not None else self.name, self.rows,
                   channels=self.channels)
        p.data = [list(col) for col in self.data]
        p.vol = [list(col) for col in self.vol]
        p.slide = [list(col) for col in self.slide]
        p.fx = [list(col) for col in self.fx]
        p.fxp = [list(col) for col in self.fxp]
        return p

    def _has_vol(self) -> bool:
        return any(v is not None for col in self.vol for v in col)

    def _has_slide(self) -> bool:
        return any(v is not None for col in self.slide for v in col)

    def _has_fx(self) -> bool:
        return any(v is not None for col in self.fx for v in col)

    def to_dict(self) -> dict:
        d = {"name": self.name, "rows": self.rows, "channels": self.channels,
             "data": self.data}
        # Effekt-Spalten nur schreiben, wenn gesetzt -> alte Dateien/leere
        # Patterns bleiben schlank und abwaerts-kompatibel.
        if self._has_vol():
            d["vol"] = self.vol
        if self._has_slide():
            d["slide"] = self.slide
        if self._has_fx():
            d["fx"] = self.fx
            d["fxp"] = self.fxp
        return d

    @classmethod
    def from_dict(cls, d: dict, channels: int = CHANNELS) -> "Pattern":
        """`channels`: Fallback-Kanalzahl, wenn `d` selbst keine (aeltere
        Dateien) traegt -- ueblicherweise `Song.channels` des ladenden Songs."""
        n = int(d.get("channels", channels))
        p = cls(str(d.get("name", "Pattern")), int(d.get("rows", DEFAULT_ROWS)),
                channels=n)
        raw = d.get("data") or []
        for c in range(p.channels):
            if c < len(raw):
                col = [int(v) if isinstance(v, (int, float)) else None
                       for v in raw[c]][:p.rows]
                col += [None] * (p.rows - len(col))
                p.data[c] = col
        rawv = d.get("vol") or []
        for c in range(p.channels):
            if c < len(rawv):
                col = [max(1, min(VOL_MAX, int(v)))
                       if isinstance(v, (int, float)) else None
                       for v in rawv[c]][:p.rows]
                col += [None] * (p.rows - len(col))
                p.vol[c] = col
        raws = d.get("slide") or []
        for c in range(p.channels):
            if c < len(raws):
                col = [max(-SLIDE_MAX, min(SLIDE_MAX, int(v)))
                       if isinstance(v, (int, float)) and int(v) != 0 else None
                       for v in raws[c]][:p.rows]
                col += [None] * (p.rows - len(col))
                p.slide[c] = col
        rawfx = d.get("fx") or []
        rawfp = d.get("fxp") or []
        for c in range(p.channels):
            if c < len(rawfx):
                col = [int(v) if isinstance(v, (int, float))
                       and int(v) in FX_CODES and int(v) != FX_NONE else None
                       for v in rawfx[c]][:p.rows]
                col += [None] * (p.rows - len(col))
                p.fx[c] = col
            if c < len(rawfp):
                colp = [max(0, min(255, int(v)))
                        if isinstance(v, (int, float)) else None
                        for v in rawfp[c]][:p.rows]
                colp += [None] * (p.rows - len(colp))
                p.fxp[c] = colp
        return p


class Song:
    """Tempo + Wellenformen + Patterns + Order (Arrangement)."""

    FORMAT = "gbtracker-song"
    VERSION = 1

    _DEFAULT_WAVES = ("square", "saw", "triangle")

    def __init__(self, channels: int = CHANNELS):
        self.bpm = 120
        self.channels = max(MIN_CHANNELS, min(MAX_CHANNELS, int(channels)))
        # Kanal 0..tonal-1 tonal (je eigene Wellenform), letzter Kanal = Drum.
        self.waves = [self._DEFAULT_WAVES[i] if i < len(self._DEFAULT_WAVES)
                      else "square" for i in range(self.tonal)]
        self.patterns: list[Pattern] = [Pattern("P1", channels=self.channels)]
        self.order: list[int] = [0]                  # Indizes in patterns
        # Instrument-Pool (Samples + benannte Synth-Klaenge). Optional --
        # ein Kanal ohne zugewiesenes Instrument nutzt weiter seine Wellenform.
        self.instruments: list = []                  # list[Instrument]
        # channel_inst[c] = Index ins instruments-Pool oder None (= Synth/waves)
        self.channel_inst: list[int | None] = [None] * self.channels
        self.path = ""

    @property
    def tonal(self) -> int:
        """Index/Anzahl der tonalen Kanaele -- der LETZTE Kanal ist immer
        Drum/Noise, unabhaengig von der Gesamt-Kanalzahl."""
        return self.channels - 1

    def set_channels(self, channels: int) -> None:
        """Aendert die Kanalzahl des ganzen Songs (alle Patterns + Wellenform-
        /Instrument-Zuweisungen). Neue Kanaele: leer, Wellenform "square",
        kein Instrument. Ueberzaehlige Kanaele werden rechts abgeschnitten."""
        channels = max(MIN_CHANNELS, min(MAX_CHANNELS, int(channels)))
        if channels == self.channels:
            return
        old_tonal = self.tonal
        self.channels = channels
        if self.tonal < old_tonal:
            del self.waves[self.tonal:]
        else:
            self.waves.extend(["square"] * (self.tonal - old_tonal))
        if channels < len(self.channel_inst):
            del self.channel_inst[channels:]
        else:
            self.channel_inst.extend([None] * (channels - len(self.channel_inst)))
        for pat in self.patterns:
            pat.set_channels(channels)

    # ------------------------------------------------------ Instrumente
    def instrument_for_channel(self, c: int):
        """Liefert das effektive Instrument fuer Kanal `c`: das zugewiesene
        Sample/Instrument -- oder ein fluechtiges Synth-Instrument aus der
        Kanal-Wellenform (letzter Kanal = Noise)."""
        from .instrument import Instrument
        idx = self.channel_inst[c] if c < len(self.channel_inst) else None
        if idx is not None and 0 <= idx < len(self.instruments):
            return self.instruments[idx]
        wf = "noise" if c == self.tonal else self.waves[c]
        return Instrument.synth(f"Ch{c}", wf)

    def _channel_wave(self, c: int) -> str | None:
        """Synth-Wellenform fuer Kanal `c` im Live-Export -- oder None, wenn
        der Kanal ein Sample-Instrument nutzt (das der Synth-Export nicht
        darstellen kann)."""
        idx = self.channel_inst[c] if c < len(self.channel_inst) else None
        if idx is not None and 0 <= idx < len(self.instruments):
            inst = self.instruments[idx]
            if getattr(inst, "kind", "synth") in ("sample", "keymap"):
                return None
            return inst.waveform
        return self.waves[c]

    def add_instrument(self, inst) -> int:
        self.instruments.append(inst)
        return len(self.instruments) - 1

    def remove_instrument(self, idx: int) -> bool:
        """Entfernt ein Instrument; Kanal-Zuweisungen werden nachgezogen."""
        if not (0 <= idx < len(self.instruments)):
            return False
        del self.instruments[idx]
        for c in range(len(self.channel_inst)):
            ci = self.channel_inst[c]
            if ci == idx:
                self.channel_inst[c] = None
            elif ci is not None and ci > idx:
                self.channel_inst[c] = ci - 1
        return True

    # ------------------------------------------------------ Tempo
    def row_ms(self) -> int:
        """Millisekunden pro Reihe (16tel-Schritte)."""
        return int(60000 / max(1, self.bpm) / 4)

    # ------------------------------------------------------ Pattern-Ops
    def add_pattern(self, name: str | None = None,
                    rows: int = DEFAULT_ROWS) -> int:
        n = name or f"P{len(self.patterns) + 1}"
        self.patterns.append(Pattern(n, rows, channels=self.channels))
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
    def _flatten(self):
        """Wie `flatten`, liefert zusaetzlich die Effekt-Spuren:
        `vols[c][i]` = Amplituden-Prozent (1..100) bei expliziter Lautstaerke,
        sonst 0; `slides[c][i]` = Pitch-Slide in Hz/s (fuer AUDIO_SFX), sonst 0.
        Beide nur relevant, wo `channels[c][i] != 0` ist."""
        order = self.order or [0]
        total = sum(self.patterns[p].rows for p in order
                    if 0 <= p < len(self.patterns))
        total = max(1, total)
        rowms = self.row_ms()
        channels = [[0] * total for _ in range(self.channels)]
        vols = [[0] * total for _ in range(self.channels)]
        slides = [[0] * total for _ in range(self.channels)]
        i = 0
        for p in order:
            if not (0 <= p < len(self.patterns)):
                continue
            pat = self.patterns[p]
            for r in range(pat.rows):
                for c in range(self.channels):
                    note = pat.data[c][r]
                    # Note-Off hat im Live-GB-Player keine Wirkung -- der
                    # spielt ohnehin nur genau eine Reihe pro Note (kein
                    # Sustain-Konzept dort), also einfach wie leer behandeln.
                    if note is None or note == NOTE_OFF:
                        continue
                    channels[c][i + r] = (
                        1 if c == self.tonal else int(round(midi_to_freq(note))))
                    v = pat.vol[c][r]
                    if v is not None:
                        vols[c][i + r] = vol_to_pct(v)
                    s = pat.slide[c][r]
                    if s is not None and c != self.tonal:   # nur tonale Kanaele
                        slides[c][i + r] = slide_hz_per_s(
                            midi_to_freq(note), s, rowms)
            i += pat.rows
        return total, channels, vols, slides

    def flatten(self) -> tuple[int, list[list[int]]]:
        """Expandiert die Order zu einer flachen Timeline.

        Liefert `(total_rows, channels)` mit `channels[c]` = Liste von
        Werten ueber den ganzen Song: 0 = nichts, sonst Frequenz in Hz
        (tonal) bzw. 1 (Drum-Hit)."""
        total, channels, _, _ = self._flatten()
        return total, channels

    # ------------------------------------------------------ JSON-I/O
    def to_dict(self) -> dict:
        d = {
            "format": self.FORMAT,
            "version": self.VERSION,
            "bpm": self.bpm,
            "channels": self.channels,
            "waves": list(self.waves),
            "patterns": [p.to_dict() for p in self.patterns],
            "order": list(self.order),
        }
        # Instrumente nur schreiben, wenn vorhanden (abwaerts-kompatibel).
        if self.instruments:
            d["instruments"] = [ins.to_dict() for ins in self.instruments]
            d["channel_inst"] = list(self.channel_inst)
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Song":
        channels = max(MIN_CHANNELS, min(MAX_CHANNELS,
                       int(d.get("channels", CHANNELS))))
        s = cls(channels=channels)
        s.bpm = int(d.get("bpm", 120))
        waves = d.get("waves") or s.waves
        s.waves = [str(w) for w in waves][:s.tonal]
        while len(s.waves) < s.tonal:
            s.waves.append("square")
        pats = d.get("patterns") or []
        s.patterns = ([Pattern.from_dict(p, channels=channels) for p in pats]
                      or [Pattern("P1", channels=channels)])
        order = [int(p) for p in (d.get("order") or [0])
                 if 0 <= int(p) < len(s.patterns)]
        s.order = order or [0]
        insts = d.get("instruments") or []
        if insts:
            from .instrument import Instrument
            s.instruments = [Instrument.from_dict(i) for i in insts]
            ci = d.get("channel_inst") or []
            s.channel_inst = [
                (int(ci[c]) if c < len(ci) and ci[c] is not None
                 and 0 <= int(ci[c]) < len(s.instruments) else None)
                for c in range(s.channels)]
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
        total, channels, vols, slides = self._flatten()
        rowms = self.row_ms()
        n_patterns = len(self.patterns)
        has_vol = any(any(col) for col in vols)
        has_slide = any(any(col) for col in slides)
        lines = [
            'IMPORT "audio"', "",
            f"' === Tracker-Song (gbtracker) -- {n_patterns} Pattern(s), "
            f"Order-Laenge {len(self.order)}, {total} Reihen, BPM {self.bpm} ===",
            f"CONST TRK_ROWS = {total}",
            f"CONST TRK_ROWMS = {rowms}", "",
        ]
        for c in range(self.channels):
            lines.append(f"DIM trk{c}[TRK_ROWS] AS INTEGER")
            for r, val in enumerate(channels[c]):
                if val:
                    lines.append(f"trk{c}[{r}] = {val}")
        if has_vol:
            # Lautstaerke-Spuren (Amplitude in Prozent, 0 = Standard).
            for c in range(self.channels):
                if not any(vols[c]):
                    continue
                lines.append(f"DIM trkV{c}[TRK_ROWS] AS INTEGER")
                for r, val in enumerate(vols[c]):
                    if val:
                        lines.append(f"trkV{c}[{r}] = {val}")
        if has_slide:
            # Pitch-Slide-Spuren (Hz/s, vorberechnet; 0 = kein Slide).
            for c in range(self.tonal):
                if not any(slides[c]):
                    continue
                lines.append(f"DIM trkSl{c}[TRK_ROWS] AS INTEGER")
                for r, val in enumerate(slides[c]):
                    if val:
                        lines.append(f"trkSl{c}[{r}] = {val}")
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
        for c in range(self.tonal):
            wave = self._channel_wave(c)
            if wave is None:
                # Kanal nutzt ein Sample-Instrument -> der Live-Synth-Export
                # kann das (noch) nicht. Render-to-File / Sampler-Export folgt.
                lines.append(
                    f"    ' Kanal {c}: Sample-Instrument -- via Audio-Datei "
                    f"exportieren (Render-to-File)")
                continue
            amp = (f"TRACKER_AMP(trkV{c}[r])"
                   if has_vol and any(vols[c]) else "0.5")
            tone = (f'AUDIO_TONE(trk{c}[r], TRK_ROWMS, "{wave}", {amp})')
            if has_slide and any(slides[c]):
                # Mit Slide -> AUDIO_SFX (Pitch-Bend ueber die Reihe), sonst Ton.
                sfx = (f'AUDIO_SFX("{wave}", trk{c}[r], trkSl{c}[r], 4, '
                       f'TRK_ROWMS, 40, 0.0, 0, {amp})')
                lines += [
                    f"    IF trk{c}[r] > 0 THEN",
                    f"        IF trkSl{c}[r] <> 0 THEN",
                    f"            PLAYSOUND({sfx})",
                    "        ELSE",
                    f"            PLAYSOUND({tone})",
                    "        END IF",
                    "    END IF",
                ]
            else:
                lines.append(f"    IF trk{c}[r] > 0 THEN PLAYSOUND({tone})")
        damp = (f"TRACKER_AMP(trkV{self.tonal}[r])"
                if has_vol and any(vols[self.tonal]) else "0.5")
        lines.append(
            f"    IF trk{self.tonal}[r] > 0 THEN PLAYSOUND(AUDIO_NOISE(120, {damp}))")
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
