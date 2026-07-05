"""Block-Operationen auf Pattern-Ausschnitten (Copy/Paste/Transpose/Interpolate).

Qt-frei (wie der Rest von `gamebasic.tracker`), damit headless testbar. Der
Editor uebersetzt nur seine QTableWidget-Selektion in ein (c0, r0, c1, r1)-
Rechteck und ruft diese Funktionen direkt auf dem `Pattern`-Modell auf.
"""
from __future__ import annotations

from .song import NOTE_OFF


def _norm_rect(c0: int, r0: int, c1: int, r1: int) -> tuple[int, int, int, int]:
    c0, c1 = sorted((c0, c1))
    r0, r1 = sorted((r0, r1))
    return c0, r0, c1, r1


def block_copy(pat, c0: int, r0: int, c1: int, r1: int) -> list[list[dict]]:
    """Kopiert das Rechteck [c0..c1] x [r0..r1] (inklusiv, unsortiert erlaubt)
    als positions-unabhaengige Zellen-Liste `cells[dc][dr]` -- geeignet zum
    Einfuegen an beliebiger Stelle via `block_paste`."""
    c0, r0, c1, r1 = _norm_rect(c0, r0, c1, r1)
    cells = []
    for c in range(c0, c1 + 1):
        col = []
        for r in range(r0, r1 + 1):
            fx, fxp = pat.get_fx(c, r)
            col.append({
                "note": pat.data[c][r],
                "vol": pat.vol[c][r],
                "slide": pat.slide[c][r],
                "fx": fx or None,
                "fxp": fxp if fx else None,
            })
        cells.append(col)
    return cells


def block_paste(pat, cells: list[list[dict]], c0: int, r0: int) -> tuple[int, int]:
    """Fuegt eine `block_copy()`-Zellen-Liste bei (c0, r0) ein -- Zellen, die
    ausserhalb des Patterns landen, werden stillschweigend uebersprungen
    (Clipping an den Pattern-Graenzen). Liefert die tatsaechlich eingefuegte
    Groesse (n_channels, n_rows) fuer z.B. eine anschliessende Selektion."""
    n_c = n_r = 0
    for dc, col in enumerate(cells):
        c = c0 + dc
        if not (0 <= c < pat.channels):
            continue
        n_c = max(n_c, dc + 1)
        for dr, cell in enumerate(col):
            r = r0 + dr
            if not (0 <= r < pat.rows):
                continue
            n_r = max(n_r, dr + 1)
            pat.set(c, r, cell["note"])
            if cell["note"] is not None:
                pat.set_vol(c, r, cell["vol"])
                pat.set_slide(c, r, cell["slide"])
                pat.set_fx(c, r, cell["fx"], cell["fxp"] or 0)
    return n_c, n_r


def block_transpose(pat, c0: int, r0: int, c1: int, r1: int, semitones: int,
                    skip_channel: int | None = None) -> None:
    """Verschiebt alle Noten im Rechteck um `semitones` Halbtoene (geklemmt
    auf MIDI 0..127). `skip_channel` (typischerweise der Drum-Kanal, dessen
    "Note" nur ein Hit ist) bleibt unveraendert. Note-Off-Zellen sind keine
    Tonhoehen und bleiben ebenfalls unveraendert."""
    c0, r0, c1, r1 = _norm_rect(c0, r0, c1, r1)
    for c in range(c0, c1 + 1):
        if c == skip_channel or not (0 <= c < pat.channels):
            continue
        for r in range(r0, r1 + 1):
            note = pat.data[c][r]
            if note is not None and note != NOTE_OFF:
                pat.data[c][r] = max(0, min(127, note + semitones))


def block_interpolate(pat, c0: int, r0: int, c1: int, r1: int,
                      skip_channel: int | None = None) -> None:
    """Fuellt je Kanal die leeren Reihen zwischen der ERSTEN und LETZTEN
    gesetzten Note im Bereich mit einer linearen Notenrampe (klassisches
    Tracker-Werkzeug fuer Glissandi/Tonhoehen-Uebergaenge, z.B. Renoise/
    OpenMPT "Interpolate"). Bereits gesetzte Zwischen-Noten bleiben stehen;
    Kanaele mit weniger als zwei Noten im Bereich bleiben unveraendert.
    Note-Off-Zellen zaehlen nicht als Rampen-Endpunkt (keine Tonhoehe),
    bleiben aber -- wie jede belegte Zelle -- von der Rampe unangetastet."""
    c0, r0, c1, r1 = _norm_rect(c0, r0, c1, r1)
    for c in range(c0, c1 + 1):
        if c == skip_channel or not (0 <= c < pat.channels):
            continue
        rows_with_notes = [r for r in range(r0, r1 + 1)
                           if pat.data[c][r] is not None
                           and pat.data[c][r] != NOTE_OFF]
        if len(rows_with_notes) < 2:
            continue
        first, last = rows_with_notes[0], rows_with_notes[-1]
        span = last - first
        if span <= 0:
            continue
        n0, n1 = pat.data[c][first], pat.data[c][last]
        for r in range(first + 1, last):
            if pat.data[c][r] is not None:
                continue
            t = (r - first) / span
            note = int(round(n0 + (n1 - n0) * t))
            pat.set(c, r, max(0, min(127, note)))
