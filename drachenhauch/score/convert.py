"""Konvertiert ein `ScoreDoc` (Notenblatt) in einen Tracker-`Song` (Zeilenraster).

Qt-frei, headless testbar. Nie eine Exception fuer Rundungs-/Reduktions-
Faelle (best effort + gesammelte Warnungen); `ValueError` NUR wenn die
Spurzahl den Tracker-Kanal-Maximalwert sprengt.

**V1-Limitationen** (siehe auch CLAUDE.md/docs/score-editor.md):
- Akkorde (mehrere Noten mit identischem Start-Beat auf einer Spur) werden
  auf die hoechste Note reduziert -- ein Tracker-Kanal ist einstimmig.
- Noten, die ueber eine 64-Zeilen-Tracker-Pattern-Grenze hinaus klingen
  wuerden, werden dort gekappt (Tracker-Patterns koennen nicht "binden").
- Zeiten, die kein exaktes Vielfaches einer Sechzehntelnote sind (z.B. aus
  einer handbearbeiteten Datei oder kuenftigen Triolen), werden auf die
  naechste Tracker-Zeile gerundet.
- Bindeboegen (`Track.slurs`) und Fingersaetze (`NoteEvent.fingering`) sind
  reine Notationszusaetze ohne Tracker-Entsprechung -- sie werden beim
  Export ignoriert. `staccato` dagegen wirkt (siehe `STACCATO_FACTOR`).
"""
from __future__ import annotations

import math

from ..tracker.instrument import Instrument
from ..tracker.song import MAX_CHANNELS, MAX_ROWS, MIN_CHANNELS, NOTE_OFF, Song

ROWS_PER_BEAT = 4    # Tracker-Zeile = 1/16-Note (Song.row_ms() = 60000/bpm/4)
_ROUND_EPS = 1e-6
# Staccato verkuerzt die Tracker-Klangdauer auf diesen Anteil der notierten
# Dauer (Rest wird stumm, wie beim Zeichnen einer klassischen Staccato-
# Interpretation) -- mindestens STACCATO_MIN_ROWS, damit auch sehr kurze
# Noten noch hoerbar bleiben.
STACCATO_FACTOR = 0.5
STACCATO_MIN_ROWS = 1


def _beat_to_row(beat: float) -> int:
    return int(round(beat * ROWS_PER_BEAT))


def _row_exact(beat: float) -> bool:
    row_f = beat * ROWS_PER_BEAT
    return abs(row_f - round(row_f)) <= _ROUND_EPS


def to_tracker_song(doc) -> tuple[Song, list[str]]:
    """`doc`: `drachenhauch.score.document.ScoreDoc`. Liefert `(song, warnings)`;
    mutiert `doc` niemals."""
    warnings: list[str] = []
    n_tracks = len(doc.tracks)
    channel_count = n_tracks + 1     # + Pflicht-Drum-Kanal (Song.tonal)
    if channel_count > MAX_CHANNELS:
        raise ValueError(
            f"dhscore: zu viele Spuren fuer den Tracker (max "
            f"{MAX_CHANNELS - 1} + Drum-Kanal, gegeben: {n_tracks})")
    channel_count = max(MIN_CHANNELS, channel_count)

    song = Song(channels=channel_count)
    song.bpm = int(doc.bpm)

    for i, track in enumerate(doc.tracks):
        idx = song.add_instrument(Instrument.from_dict(track.instrument.to_dict()))
        song.channel_inst[i] = idx
    # Drum-Kanal (letzter Kanal) bleibt bewusst unbelegt -- v1 hat kein
    # Perkussions-Spurkonzept, faellt auf den Tracker-Noise-Default zurueck.

    total_rows = max(1, _beat_to_row(doc.length_beats()))

    if total_rows <= MAX_ROWS:
        song.patterns[0].set_rows(total_rows)
        song.order = [0]
        chunks = [(0, total_rows, song.patterns[0])]
    else:
        n_chunks = math.ceil(total_rows / MAX_ROWS)
        song.patterns = []
        song.order = []
        chunks = []
        for k in range(n_chunks):
            row_lo = k * MAX_ROWS
            row_hi = min(total_rows, row_lo + MAX_ROWS)
            pat_idx = song.add_pattern(name=f"P{k + 1}", rows=row_hi - row_lo)
            song.order_add(pat_idx)
            chunks.append((row_lo, row_hi, song.patterns[pat_idx]))

    def _find_chunk(row: int):
        for row_lo, row_hi, pat in chunks:
            if row_lo <= row < row_hi:
                return row_lo, row_hi, pat
        return None

    for i, track in enumerate(doc.tracks):
        # Akkord-Reduktion: mehrere Noten, die auf dieselbe Tracker-Zeile
        # runden -> nur die hoechste behalten (Tracker-Kanal einstimmig).
        # Gruppierung ueber die GERUNDETE Zeile statt des rohen start_beat --
        # sonst wuerden zwei Noten mit knapp unterschiedlichem start_beat,
        # die auf dieselbe Zeile runden, NICHT als Akkord erkannt und die
        # zweite ueberschreibt die erste in pat.set() lautlos (keine
        # Warnung), statt regulaer per Akkord-Reduktion ersetzt zu werden.
        by_row: dict[int, list] = {}
        for note in track.notes:
            if note.rest:
                continue
            by_row.setdefault(_beat_to_row(note.start_beat), []).append(note)
        reduced = []
        for row, group in by_row.items():
            if len(group) > 1:
                top = max(group, key=lambda n: n.pitch)
                beats = ", ".join(f"{n.start_beat:g}" for n in group)
                warnings.append(
                    f"Spur {i + 1} ({track.name}), Zeile {row} (Beats "
                    f"{beats}): Akkord auf hoechste Note reduziert "
                    f"(Tracker-Kanal ist einstimmig)")
                reduced.append(top)
            else:
                reduced.append(group[0])
        reduced.sort(key=lambda n: n.start_beat)

        for note in reduced:
            if not _row_exact(note.start_beat):
                warnings.append(
                    f"Spur {i + 1} ({track.name}), Note bei Beat "
                    f"{note.start_beat:g}: auf die naechste Tracker-Zeile "
                    f"gerundet")
            start_row = _beat_to_row(note.start_beat)
            end_row = _beat_to_row(note.end_beat)
            if note.staccato:
                shortened = note.start_beat + note.dur_beat * STACCATO_FACTOR
                end_row = max(start_row + STACCATO_MIN_ROWS, _beat_to_row(shortened))
            chunk = _find_chunk(start_row)
            if chunk is None:
                warnings.append(
                    f"Spur {i + 1} ({track.name}), Note bei Beat "
                    f"{note.start_beat:g}: fiel aus dem Song heraus, "
                    f"uebersprungen")
                continue
            row_lo, row_hi, pat = chunk
            local_row = start_row - row_lo
            # local_row < pat.rows ist hier per Konstruktion garantiert:
            # `_find_chunk` liefert nur Chunks mit row_lo <= start_row <
            # row_hi, und jedes chunk-Tupel wird mit pat.rows == row_hi -
            # row_lo gebaut (s.o., sowohl im Single-Pattern- als auch im
            # Multi-Chunk-Zweig) -- kein zusaetzlicher Bounds-Check noetig.
            pat.set(i, local_row, note.pitch)
            if end_row > row_hi:
                warnings.append(
                    f"Spur {i + 1} ({track.name}), Note bei Beat "
                    f"{note.start_beat:g}: an Pattern-Grenze gekuerzt "
                    f"(Reihe {row_hi})")
                continue
            # NOTE_OFF an der Stelle setzen, wo die Note endet -- schneidet
            # ein nachklingendes Sample ab, falls dort keine neue Note
            # ohnehin schon den Kanal ueberschreibt (Legato-Fall heilt sich
            # von selbst: die naechste Note wird im naechsten Schleifen-
            # durchlauf an derselben Zeile geschrieben und ersetzt es).
            if end_row < total_rows:
                off_chunk = _find_chunk(end_row)
                if off_chunk is not None:
                    o_lo, o_hi, o_pat = off_chunk
                    o_local = end_row - o_lo
                    if 0 <= o_local < o_pat.rows and o_pat.data[i][o_local] is None:
                        o_pat.set(i, o_local, NOTE_OFF)

    return song, warnings
