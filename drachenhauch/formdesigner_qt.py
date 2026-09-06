"""Form-Designer (WYSIWYG, Xojo-Stil) -- PySide6-UI.

Eigenstaendiges Qt-Programm: links die Control-Palette, in der Mitte die
Design-Flaeche (Controls platzieren/selektieren/verschieben/loeschen), rechts
der Inspector (Eigenschaften + Events). Speichert/laedt `.dhform` (Runtime-
Format) und kann das Formular per "Run" mit `dhrt` starten (laedt das Layout +
generierte Event-Handler-Stubs -- der Xojo-Lauf).

Datenmodell + Code-Generierung liegen Qt-frei in `drachenhauch/formdesigner/`.
Start: `dhform [datei.dhform]` bzw. `dhrun.py --form`.
"""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from PySide6.QtCore import Qt, QRect, QRectF, QPoint, QSize, QMimeData, Signal
from PySide6.QtGui import (
    QAction, QColor, QPainter, QPen, QBrush, QKeySequence, QFont, QPixmap, QIcon,
    QShortcut,
    QLinearGradient, QPolygon,
)
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QListWidget, QListWidgetItem, QDockWidget,
    QScrollArea, QFormLayout, QLineEdit, QSpinBox, QCheckBox, QPlainTextEdit,
    QFileDialog, QMessageBox, QLabel, QVBoxLayout, QHBoxLayout, QDoubleSpinBox,
    QComboBox, QAbstractItemView, QMenu, QStackedWidget, QToolBar, QPushButton,
    QColorDialog,
)

from .formdesigner import (
    FormDoc, FormProject, Control, History, PALETTE, palette_spec, GRID, HANDLES,
    snap, resize_rect, FORM_THEMES, theme_colors, EVENTS, hex_zu_int,
    regeln_parsen, regeln_text,
)

try:
    from .editor_qt.theme import global_qss
except Exception:  # pragma: no cover - Theme optional
    def global_qss() -> str:
        return ""

try:
    from .editor_qt.highlighter import DHHighlighter
except Exception:  # pragma: no cover - Highlighter optional
    DHHighlighter = None

PAD = 24          # Rand um das Fenster auf der Canvas
TITLE_H = 22      # Titelleisten-Hoehe (wie im gui-Modul)
HANDLE = 8        # Kantenlaenge eines Resize-Griffs (px)
RULER = 18        # Breite/Hoehe der Lineale am Rand
TBL_FILTER_H = 22  # Hoehe der Tabellen-Filterzeile (wie TBL_FILTER_H in gui.rs)
SHADOW = 7        # Staffelungen des Schlagschattens unter dem Formular
GRID_MIN_ZOOM = 0.5   # darunter wird das Raster zu Rauschen -> aus

# Resize-Griff -> Maus-Cursor (diagonal/horizontal/vertikal)
_HANDLE_CURSORS = {
    "nw": Qt.CursorShape.SizeFDiagCursor, "se": Qt.CursorShape.SizeFDiagCursor,
    "ne": Qt.CursorShape.SizeBDiagCursor, "sw": Qt.CursorShape.SizeBDiagCursor,
    "n": Qt.CursorShape.SizeVerCursor, "s": Qt.CursorShape.SizeVerCursor,
    "e": Qt.CursorShape.SizeHorCursor, "w": Qt.CursorShape.SizeHorCursor,
}


# Geteilte Fundstelle statt einer eigenen Kopie: die lokale Variante suchte NUR
# im Dev-Baum (`rust/drachenhauch_runtime/target/...` relativ zu dieser Datei) und kannte
# den PyInstaller-Fall nicht -- in der installierten IDE liegt `dhrt.exe` neben
# `Drachenhauch.exe`, F5 meldete dort also "Runtime nicht gebaut: python
# rust/build_runtime.py", einen Rat, den ein Installer-Nutzer nicht befolgen
# kann. Der Alias haelt den Namen fuer Tests patchbar (wie in output_console).
from .editor_qt.dhrt_locate import (NO_WINDOW, find_dhrt as _find_dhrt,
                                    dhrt_spawn_semaphore)


def _dhrt_diagnostics(dhrt, dh_path: Path) -> list:
    """`dhrt --check` auf die erzeugte Datei. Liefert die Diagnose-Liste
    (leer = sauber). ACHTUNG: `--check` endet IMMER mit Code 0, die Fehler
    stehen als JSON auf stdout -- wer nur den Rueckgabewert prueft, sieht
    nichts."""
    try:
        with dhrt_spawn_semaphore:
            r = subprocess.run([str(dhrt), "--check", str(dh_path)],
                               capture_output=True, text=True,
                               encoding="utf-8", timeout=30,
                               creationflags=NO_WINDOW)
        return json.loads((r.stdout or "").strip() or "[]")
    except (OSError, ValueError, subprocess.SubprocessError):
        return []          # Pruefung nicht moeglich -> Lauf trotzdem versuchen


def _col(i: int) -> QColor:
    return QColor((i >> 16) & 0xFF, (i >> 8) & 0xFF, i & 0xFF)


def _mix(a: QColor, b: QColor, t: float) -> QColor:
    """Zwei Farben mischen (t = 0 ganz a, 1 ganz b)."""
    t = min(1.0, max(0.0, t))
    return QColor(int(a.red() + (b.red() - a.red()) * t),
                  int(a.green() + (b.green() - a.green()) * t),
                  int(a.blue() + (b.blue() - a.blue()) * t))


def _shade(c: QColor, d: int) -> QColor:
    return QColor(min(255, max(0, c.red() + d)),
                  min(255, max(0, c.green() + d)),
                  min(255, max(0, c.blue() + d)))


def _fill_surface(qp: QPainter, r: QRect, face: QColor, border: QColor,
                  rad: int, grad: int, gloss: int):
    """Flaeche wie die Laufzeit: flach, gerundet oder gewoelbt mit Glanzkante.

    Der Designer zeichnet mit Qt, kann also nicht dieselben Befehle nutzen wie
    dhrt -- er baut den Eindruck nach. Genau deshalb sind die Werte
    (`radius`/`gradient`/`gloss`) dieselben Zahlen wie in den Metriken der
    Laufzeit, damit beide Seiten am selben Regler haengen."""
    if grad > 0:
        gradient = QLinearGradient(r.topLeft(), r.bottomLeft())
        gradient.setColorAt(0.0, _shade(face, grad))
        gradient.setColorAt(1.0, _shade(face, -grad))
        qp.setBrush(QBrush(gradient))
    else:
        qp.setBrush(face)
    qp.setPen(QPen(border, 1))
    if rad > 0:
        qp.drawRoundedRect(r, rad, rad)
    else:
        qp.drawRect(r)
    if grad > 0 and gloss > 0 and r.height() >= 6:
        g2 = QLinearGradient(r.topLeft(), QPoint(r.left(), r.center().y()))
        g2.setColorAt(0.0, QColor(255, 255, 255, int(gloss * 255 / 100)))
        g2.setColorAt(1.0, QColor(255, 255, 255, 0))
        qp.setBrush(QBrush(g2)); qp.setPen(Qt.PenStyle.NoPen)
        top = QRect(r.left(), r.top(), r.width(), max(2, r.height() // 2))
        if rad > 0:
            qp.drawRoundedRect(top, rad, rad)
        else:
            qp.drawRect(top)


def _progress_frac(c) -> float:
    """Fuellstand eines ProgressBar-Controls -- exakt wie die Laufzeit
    (`gui.rs`, `Kind::Progress`): Anteil an [min, max]. Die Canvas las `value`
    vorher roh als 0..1, ein Balken mit max=100/value=25 sah dadurch randvoll
    aus und lief zur Laufzeit dann bei 25 %."""
    span = c.max - c.min
    if span == 0:
        return 0.0
    return min(1.0, max(0.0, (c.value - c.min) / span))


# MIME-Typ fuer Drag&Drop eines Palette-Eintrags (Control-Art).
_CONTROL_MIME = "application/x-gbcontrol-kind"

_PAL_BG = QColor(34, 46, 58)
_PAL_FG = QColor(222, 232, 240)
_PAL_ACCENT = QColor(43, 196, 232)
_PAL_BORDER = QColor(78, 100, 122)


def _preview_neu(qp: QPainter, kind: str, r: QRect, *, fg, muted, accent, border,
                 face, sunk, title_bg, win_bg, rad: int, grad: float, gloss: float,
                 text: str = "", value: float = 0.0, mn: float = 0.0, mx: float = 1.0,
                 checked: bool = False, items=(), farbe: int = 0xE84B4B,
                 datum: str = "") -> bool:
    """Vorschau der neun Arten, die der Designer seit 2026-08-31 kennt.

    Liefert False, wenn `kind` nicht dazugehoert -- der Aufrufer zeichnet dann
    seinen eigenen Rueckfall. EINE Quelle fuer Palettensymbol UND
    Entwurfsflaeche: zwei Fassungen liefen frueher oder spaeter auseinander.
    """
    al = Qt.AlignmentFlag
    x, y, w, h = r.left(), r.top(), r.width(), r.height()
    cy = r.center().y()

    def flaeche(rect, farbe_=None, tief=False):
        _fill_surface(qp, rect, sunk if tief else (farbe_ or face), border,
                      rad, -grad if tief else grad, 0 if tief else gloss)

    if kind == "textarea":
        flaeche(r, tief=True)
        qp.setPen(muted)
        for i in range(max(1, min(4, (h - 8) // 8))):
            yy = y + 7 + i * 8
            if yy > r.bottom() - 4:
                break
            qp.drawLine(x + 6, yy, r.right() - (6 if i < 2 else max(8, w // 3)), yy)

    elif kind == "spinner":
        flaeche(r, tief=True)
        bx = QRect(r.right() - 14, y + 2, 12, max(4, h - 4))
        _fill_surface(qp, bx, face, border, max(0, rad - 1), grad, gloss)
        qp.setPen(fg)
        qp.drawText(QRect(bx.left(), bx.top(), bx.width(), bx.height() // 2), al.AlignCenter, "▲")
        qp.drawText(QRect(bx.left(), cy, bx.width(), bx.height() // 2), al.AlignCenter, "▼")
        qp.drawText(r.adjusted(6, 0, -18, 0), al.AlignVCenter, _zahl(value))

    elif kind == "knob":
        d = max(8, min(w, h))
        kr = QRect(x + (w - d) // 2, y + (h - d) // 2, d, d)
        qp.setPen(QPen(_mix(accent, win_bg, 0.5), 2))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        # Skalenbogen: 270 Grad, unten offen -- so hat die Laufzeit den Knopf.
        qp.drawArc(kr.adjusted(1, 1, -1, -1), -45 * 16, 270 * 16)
        _fill_surface(qp, kr.adjusted(3, 3, -3, -3), face, border, d, grad, gloss)
        # Zeiger auf den Anteil zwischen min und max.
        spanne = (mx - mn) or 1.0
        t = min(1.0, max(0.0, (value - mn) / spanne))
        import math
        winkel = math.radians(135 - t * 270)
        cx2, cy2 = kr.center().x(), kr.center().y()
        qp.setPen(QPen(accent, 2))
        qp.drawLine(cx2, cy2,
                    int(cx2 + math.cos(winkel) * d * 0.32),
                    int(cy2 - math.sin(winkel) * d * 0.32))

    elif kind == "toggle":
        pw = min(w, 34)
        pill = QRect(x, cy - 8, pw, 16)
        _fill_surface(qp, pill, _mix(face, accent, 0.55) if checked else sunk,
                      border, 8, grad, gloss)
        kx = pill.right() - 13 if checked else pill.left() + 1
        qp.setBrush(_col(0xE8ECF0) if checked else _mix(face, win_bg, 0.2))
        qp.setPen(QPen(border, 1))
        qp.drawEllipse(QRect(kx, pill.top() + 1, 14, 14))
        if text and w > pw + 8:
            qp.setPen(fg)
            qp.drawText(QRect(x + pw + 6, y, w - pw - 6, h), al.AlignVCenter, text)

    elif kind == "tree":
        flaeche(r, tief=True)
        qp.setPen(fg)
        for i, it in enumerate(list(items)[:max(0, (h - 8) // 14)] or ["Knoten"]):
            yy = y + 4 + i * 14
            qp.drawText(QRect(x + 6, yy, 8, 12), al.AlignCenter, "▸")
            qp.drawText(QRect(x + 16, yy, max(4, w - 20), 12), al.AlignVCenter, str(it))

    elif kind == "toolbar":
        _fill_surface(qp, r, _mix(face, win_bg, 0.25), border, rad, grad, gloss)
        for i in range(min(3, max(1, (w - 8) // 30))):
            _fill_surface(qp, QRect(x + 4 + i * 30, y + 3, 26, max(4, h - 6)),
                          face, border, max(0, rad - 1), grad, gloss)

    elif kind == "splitter":
        senkrecht = h > w
        qp.setPen(QPen(border, 2))
        if senkrecht:
            qp.drawLine(r.center().x(), y + 2, r.center().x(), r.bottom() - 2)
        else:
            qp.drawLine(x + 2, cy, r.right() - 2, cy)
        # Griff: drei Punkte in der Mitte -- daran erkennt man, dass es ziehbar
        # ist und nicht bloss eine Linie (Unterschied zum Separator).
        qp.setBrush(muted); qp.setPen(Qt.PenStyle.NoPen)
        for k in (-1, 0, 1):
            if senkrecht:
                qp.drawEllipse(QRect(r.center().x() - 1, cy + k * 5 - 1, 3, 3))
            else:
                qp.drawEllipse(QRect(r.center().x() + k * 5 - 1, cy - 1, 3, 3))

    elif kind == "colorpicker":
        strip = 12 if w > 40 else 6
        feld = QRect(x, y, max(4, w - strip - 4), h)
        g = QLinearGradient(feld.left(), 0, feld.right(), 0)
        g.setColorAt(0.0, QColor(255, 255, 255)); g.setColorAt(1.0, _col(farbe))
        qp.setPen(Qt.PenStyle.NoPen); qp.setBrush(g); qp.drawRect(feld)
        g2 = QLinearGradient(0, feld.top(), 0, feld.bottom())
        g2.setColorAt(0.0, QColor(0, 0, 0, 0)); g2.setColorAt(1.0, QColor(0, 0, 0, 255))
        qp.setBrush(g2); qp.drawRect(feld)
        hs = QRect(r.right() - strip, y, strip, h)
        gh = QLinearGradient(0, hs.top(), 0, hs.bottom())
        for i, col in enumerate((0xFF0000, 0xFFFF00, 0x00FF00, 0x00FFFF,
                                 0x0000FF, 0xFF00FF, 0xFF0000)):
            gh.setColorAt(i / 6.0, _col(col))
        qp.setBrush(gh); qp.drawRect(hs)
        qp.setPen(QPen(QColor(255, 255, 255), 1)); qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawEllipse(QRect(feld.right() - 8, feld.top() + 4, 6, 6))

    elif kind == "datepicker":
        flaeche(r, tief=True)
        kopf = QRect(x, y, w, min(16, h))
        qp.fillRect(kopf, title_bg)
        qp.setPen(fg)
        qp.drawText(kopf, al.AlignCenter, _monat_kopf(datum))
        if h > 34:
            qp.setPen(muted)
            sw = max(1, w // 7)
            for i, t in enumerate(("Mo", "Di", "Mi", "Do", "Fr", "Sa", "So")):
                qp.drawText(QRect(x + i * sw, y + 17, sw, 11), al.AlignCenter, t)
            zeilen = max(1, min(6, (h - 30) // 12))
            tag, gewaehlt = 1, _tag_aus(datum)
            qp.setPen(fg)
            for zi in range(zeilen):
                for si in range(7):
                    zelle = QRect(x + si * sw, y + 29 + zi * 12, sw, 12)
                    if tag == gewaehlt:
                        qp.fillRect(zelle.adjusted(1, 0, -1, 0), accent)
                        qp.setPen(win_bg); qp.drawText(zelle, al.AlignCenter, str(tag))
                        qp.setPen(fg)
                    else:
                        qp.drawText(zelle, al.AlignCenter, str(tag))
                    tag += 1
    else:
        return False
    return True


def _zahl(v: float) -> str:
    """Kommazahl ohne unnoetige Nullen -- `3` statt `3.0`."""
    return str(int(v)) if float(v).is_integer() else f"{v:g}"


_MONATE = ("Januar", "Februar", "Maerz", "April", "Mai", "Juni", "Juli",
           "August", "September", "Oktober", "November", "Dezember")


def _monat_kopf(datum: str) -> str:
    """`"2026-08-31"` -> `"August 2026"`. Ohne Datum der laufende Monat --
    genauso, wie ein frischer Waehler zur Laufzeit HEUTE zeigt."""
    import datetime
    try:
        j, m, _ = (int(t) for t in str(datum).split("-"))
        if not 1 <= m <= 12:
            raise ValueError
    except (ValueError, TypeError):
        heute = datetime.date.today()
        j, m = heute.year, heute.month
    return f"{_MONATE[m - 1]} {j}"


def _tag_aus(datum: str) -> int:
    try:
        return int(str(datum).split("-")[2])
    except (ValueError, IndexError, TypeError):
        return 0


def _paint_glyph(qp: QPainter, kind: str, r: QRect, theme: str = "glas_dunkel"):
    """Vorschau eines Controls in `r` -- fuer Palette-Icon + Drag-Bild.

    Zeichnet ueber dieselben Flaechen-Hilfen wie die Entwurfsflaeche und in den
    Farben eines Themas. Vorher waren hier eigene, fest verdrahtete Farben und
    grobe Rechtecke: die Palette zeigte etwas anderes als das, was nach dem
    Ablegen erschien.
    """
    th = theme_colors(theme)
    fg = _col(th["text_fg"])
    muted = _col(th["muted_fg"])
    accent = _col(th["accent"])
    border = _col(th["widget_border"])
    face = _col(th["widget_bg"])
    sunk = _mix(face, _col(th["win_bg"]), 0.55)
    rad, grad, gloss = th["radius"], th["gradient"], th["gloss"]
    al = Qt.AlignmentFlag

    qp.setRenderHint(QPainter.RenderHint.Antialiasing, True)
    f = QFont("Segoe UI")
    f.setPixelSize(9)
    qp.setFont(f)
    cy = r.center().y()

    def flaeche(rect, farbe=None, tief=False):
        _fill_surface(qp, rect, sunk if tief else (farbe or face), border,
                      rad, -grad if tief else grad, 0 if tief else gloss)

    if kind == "button":
        flaeche(r)
        qp.setPen(fg); qp.drawText(r, al.AlignCenter, "Button")
    elif kind == "label":
        qp.setPen(fg); qp.drawText(r, al.AlignCenter, "Label")
    elif kind in ("checkbox", "radio"):
        bs = 13
        box = QRect(r.left() + 2, cy - bs // 2, bs, bs)
        qp.setPen(QPen(border, 1)); qp.setBrush(sunk)
        if kind == "checkbox":
            qp.drawRoundedRect(box, 3, 3)
            qp.setPen(QPen(accent, 2))
            qp.drawLine(box.left() + 3, cy, box.center().x(), box.bottom() - 3)
            qp.drawLine(box.center().x(), box.bottom() - 3, box.right() - 2, box.top() + 2)
        else:
            qp.drawEllipse(box)
            qp.setBrush(accent); qp.setPen(Qt.PenStyle.NoPen)
            qp.drawEllipse(box.adjusted(4, 4, -4, -4))
        qp.setPen(fg)
        qp.drawText(QRect(box.right() + 5, r.top(), r.right() - box.right() - 5, r.height()),
                    al.AlignVCenter, "Check" if kind == "checkbox" else "Option")
    elif kind == "slider":
        flaeche(QRect(r.left() + 2, cy - 3, r.width() - 4, 6), tief=True)
        qp.setBrush(accent); qp.setPen(Qt.PenStyle.NoPen)
        qp.drawRect(QRect(r.left() + 3, cy - 2, r.width() // 2 - 3, 4))
        _fill_surface(qp, QRect(r.center().x() - 6, cy - 7, 12, 14), face, border, 6, grad, gloss)
    elif kind == "textinput":
        flaeche(r, tief=True)
        qp.setPen(QPen(accent, 1)); qp.drawLine(r.left() + 6, r.top() + 5, r.left() + 6, r.bottom() - 5)
        qp.setPen(muted)
        qp.drawText(r.adjusted(11, 0, -2, 0), al.AlignVCenter, "Text…")
    elif kind == "dropdown":
        flaeche(r)
        qp.setPen(fg)
        qp.drawText(r.adjusted(6, 0, -16, 0), al.AlignVCenter, "Auswahl")
        qp.drawText(QRect(r.right() - 14, r.top(), 12, r.height()), al.AlignCenter, "▾")
    elif kind == "listbox":
        flaeche(r, tief=True)
        qp.fillRect(QRect(r.left() + 2, r.top() + 3, r.width() - 4, 9),
                    _mix(_col(th["win_bg"]), accent, 0.45))
        qp.setPen(muted)
        for i in range(2):
            yy = r.top() + 17 + i * 8
            qp.drawLine(r.left() + 6, yy, r.right() - 6, yy)
    elif kind == "progress":
        flaeche(r, tief=True)
        inner = QRect(r.left() + 2, r.top() + 2, int((r.width() - 4) * 0.6), r.height() - 4)
        _fill_surface(qp, inner, accent, accent, max(0, rad - 1), grad, gloss)
    elif kind == "image":
        flaeche(r)
        qp.setPen(QPen(accent, 1))
        qp.drawLine(r.left() + 5, r.bottom() - 5, r.center().x() - 2, cy)
        qp.drawLine(r.center().x() - 2, cy, r.right() - 6, r.bottom() - 5)
        qp.setBrush(QColor(240, 220, 120)); qp.setPen(Qt.PenStyle.NoPen)
        qp.drawEllipse(QRect(r.right() - 16, r.top() + 5, 7, 7))
    elif kind == "table":
        flaeche(r)
        # Kopfzeile plus zwei Gitterlinien -- klein genug fuers Palettensymbol,
        # aber sofort als Tabelle zu erkennen.
        qp.fillRect(QRect(r.left() + 1, r.top() + 1, r.width() - 2, 9), _col(th["title_bg"]))
        qp.setPen(QPen(_mix(face, border, 0.55), 1))
        for i in (1, 2):
            gy = r.top() + 10 + i * ((r.height() - 12) // 3)
            qp.drawLine(r.left() + 1, gy, r.right() - 1, gy)
        for i in (1, 2):
            gx = r.left() + i * (r.width() // 3)
            qp.drawLine(gx, r.top() + 1, gx, r.bottom() - 1)
    elif kind == "canvas":
        qp.setBrush(sunk); qp.setPen(QPen(border, 1, Qt.PenStyle.DashLine)); qp.drawRect(r)
        qp.setPen(muted); qp.drawText(r, al.AlignCenter, "Canvas")
    elif kind == "panel":
        flaeche(r, _mix(face, _col(th["win_bg"]), 0.5))
        qp.fillRect(QRect(r.left() + 1, r.top() + 1, r.width() - 2, 9), _col(th["title_bg"]))
    elif kind == "groupbox":
        qp.setPen(QPen(border, 1)); qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRoundedRect(r.adjusted(1, 5, -1, -1), max(2, rad), max(2, rad))
        qp.fillRect(QRect(r.left() + 8, r.top() + 1, 34, 9), _col(th["win_bg"]))
        qp.setPen(muted); qp.drawText(QRect(r.left() + 9, r.top(), 34, 11), al.AlignLeft, "Gruppe")
    elif kind == "separator":
        qp.setPen(QPen(border, 2)); qp.drawLine(r.left() + 3, cy, r.right() - 3, cy)
    elif _preview_neu(qp, kind, r, fg=fg, muted=muted, accent=accent, border=border,
                      face=face, sunk=sunk, title_bg=_col(th["title_bg"]),
                      win_bg=_col(th["win_bg"]), rad=rad, grad=grad, gloss=gloss,
                      text="An/Aus", value=3, mn=0, mx=10, checked=True,
                      items=("Ordner", "Datei"), datum=""):
        pass
    else:
        flaeche(r)
        qp.setPen(fg); qp.drawText(r, al.AlignCenter, kind[:8])


def _palette_icon(kind: str, w: int = 92, h: int = 46, theme: str = "glas_dunkel") -> QIcon:
    pm = QPixmap(w, h)
    pm.fill(QColor(0, 0, 0, 0))
    qp = QPainter(pm)
    _paint_glyph(qp, kind, QRect(4, 6, w - 8, h - 12), theme)
    qp.end()
    return QIcon(pm)


def _tool_icon(kind: str, sz: int = 26) -> QIcon:
    """Symbol fuer die Haupt-Werkzeugleiste, programmatisch gezeichnet.

    Keine Bilddateien: so skalieren sie mit der Leistengroesse, folgen dem
    Thema und muessen nicht mit ausgeliefert werden -- dieselbe Ueberlegung
    wie bei den Control-Vorschauen der Palette.
    """
    pm = QPixmap(sz, sz)
    pm.fill(QColor(0, 0, 0, 0))
    qp = QPainter(pm)
    qp.setRenderHint(QPainter.RenderHint.Antialiasing, True)
    hell = QColor(198, 216, 230)
    akzent = QColor(43, 196, 232)
    gruen = QColor(90, 200, 120)
    m = sz // 2
    r = QRect(3, 3, sz - 6, sz - 6)

    if kind == "new":                       # leeres Blatt mit Eselsohr
        qp.setPen(QPen(hell, 2)); qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(r.left() + 2, r.top(), r.width() - 5, r.height()))
        qp.setPen(QPen(akzent, 2))
        qp.drawLine(r.right() - 6, r.top(), r.right() - 3, r.top() + 4)
    elif kind == "open":                    # Ordner
        qp.setPen(QPen(hell, 2)); qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawLine(r.left(), r.bottom(), r.right(), r.bottom())
        qp.drawLine(r.left(), r.top() + 4, r.left(), r.bottom())
        qp.drawLine(r.left(), r.top() + 4, r.left() + 5, r.top() + 4)
        qp.drawLine(r.left() + 5, r.top() + 4, r.left() + 7, r.top() + 7)
        qp.drawLine(r.left() + 7, r.top() + 7, r.right(), r.top() + 7)
        qp.drawLine(r.right(), r.top() + 7, r.right(), r.bottom())
    elif kind == "save":                    # Diskette
        qp.setPen(QPen(hell, 2)); qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(r)
        qp.setBrush(akzent); qp.setPen(Qt.PenStyle.NoPen)
        qp.drawRect(QRect(r.left() + 4, r.top() + 1, r.width() - 8, 5))
        qp.setPen(QPen(hell, 1)); qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(r.left() + 3, r.bottom() - 7, r.width() - 6, 7))
    elif kind in ("undo", "redo"):          # gebogener Pfeil
        qp.setPen(QPen(hell, 2)); qp.setBrush(Qt.BrushStyle.NoBrush)
        bogen = QRect(r.left(), r.top() + 3, r.width(), r.height() - 3)
        # Halbkreis oben, Spitze links bzw. rechts.
        qp.drawArc(bogen, 0 * 16, 180 * 16)
        qp.setBrush(hell); qp.setPen(Qt.PenStyle.NoPen)
        x = r.left() if kind == "undo" else r.right()
        d = 4 if kind == "undo" else -4
        qp.drawPolygon(QPolygon([QPoint(x, m + 1), QPoint(x + d, m - 3),
                                 QPoint(x + d, m + 5)]))
    elif kind == "run":                     # Wiedergabe-Dreieck
        qp.setBrush(gruen); qp.setPen(Qt.PenStyle.NoPen)
        qp.drawPolygon(QPolygon([QPoint(r.left() + 2, r.top()), QPoint(r.right(), m),
                                 QPoint(r.left() + 2, r.bottom())]))
    elif kind == "code":                    # spitze Klammern
        qp.setPen(QPen(akzent, 2)); qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawLine(r.left() + 5, m - 4, r.left() + 1, m)
        qp.drawLine(r.left() + 1, m, r.left() + 5, m + 4)
        qp.drawLine(r.right() - 5, m - 4, r.right() - 1, m)
        qp.drawLine(r.right() - 1, m, r.right() - 5, m + 4)
    qp.end()
    return QIcon(pm)


def _arrange_icon(kind: str, sz: int = 22) -> QIcon:
    """Kleines Icon fuer einen Anordnen-Befehl (Balken + Hilfslinie)."""
    pm = QPixmap(sz, sz); pm.fill(QColor(0, 0, 0, 0))
    qp = QPainter(pm)
    bar = QColor(150, 200, 220); guide = QColor(43, 196, 232)
    qp.setPen(Qt.PenStyle.NoPen); qp.setBrush(bar)
    def hbar(x, y, w): qp.drawRect(x, y, w, 3)
    def vbar(x, y, h): qp.drawRect(x, y, 3, h)
    def gv(x): qp.setPen(QPen(guide, 1)); qp.drawLine(x, 2, x, sz - 2); qp.setPen(Qt.PenStyle.NoPen)
    def gh(y): qp.setPen(QPen(guide, 1)); qp.drawLine(2, y, sz - 2, y); qp.setPen(Qt.PenStyle.NoPen)
    if kind == "left":
        gv(3); hbar(4, 4, 14); hbar(4, 10, 8); hbar(4, 16, 12)
    elif kind == "right":
        gv(sz - 3); hbar(sz - 18, 4, 14); hbar(sz - 12, 10, 8); hbar(sz - 16, 16, 12)
    elif kind == "center_h":
        c = sz // 2; gv(c)
        for w, y in ((14, 4), (8, 10), (12, 16)): hbar(c - w // 2, y, w)
    elif kind == "top":
        gh(3); vbar(4, 4, 14); vbar(10, 4, 8); vbar(16, 4, 12)
    elif kind == "bottom":
        gh(sz - 3); vbar(4, sz - 18, 14); vbar(10, sz - 12, 8); vbar(16, sz - 16, 12)
    elif kind == "center_v":
        c = sz // 2; gh(c)
        for h, x in ((14, 4), (8, 10), (12, 16)): vbar(x, c - h // 2, h)
    elif kind == "dist_h":
        vbar(3, 5, 12); vbar(sz // 2 - 1, 5, 12); vbar(sz - 4, 5, 12)
    elif kind == "dist_v":
        hbar(5, 3, 12); hbar(5, sz // 2 - 1, 12); hbar(5, sz - 4, 12)
    elif kind == "same_w":
        qp.drawRect(4, 5, 14, 4); qp.drawRect(4, 13, 14, 4)
    elif kind == "same_h":
        qp.drawRect(5, 4, 4, 14); qp.drawRect(13, 4, 4, 14)
    elif kind == "same_both":
        qp.setBrush(Qt.BrushStyle.NoBrush); qp.setPen(QPen(bar, 2)); qp.drawRect(4, 4, 14, 14)
    qp.end()
    return QIcon(pm)


class _PaletteList(QListWidget):
    """Palette mit grafischen Control-Icons; Eintraege sind per Drag&Drop auf die
    Canvas ziehbar (MIME `_CONTROL_MIME` = Control-Art)."""
    def __init__(self, parent=None):
        super().__init__(parent)
        # Raster mit Beschriftung UNTER der Vorschau (statt einspaltiger
        # Liste): so passen doppelt so viele Controls ins Bild und die
        # Vorschau bekommt Platz, gross genug um erkennbar zu sein.
        self.setViewMode(QListWidget.ViewMode.IconMode)
        self.setIconSize(QSize(92, 46))
        self.setGridSize(QSize(104, 74))
        self.setSpacing(4)
        self.setMovement(QListWidget.Movement.Static)   # Eintraege nicht verschiebbar
        self.setResizeMode(QListWidget.ResizeMode.Adjust)
        self.setWordWrap(True)
        self.setUniformItemSizes(True)
        self.setDragEnabled(True)
        self.setDragDropMode(QAbstractItemView.DragDropMode.DragOnly)
        self.setSelectionMode(QAbstractItemView.SelectionMode.SingleSelection)

    def mimeData(self, items):
        md = QMimeData()
        if items:
            kind = items[0].data(Qt.ItemDataRole.UserRole)
            md.setData(_CONTROL_MIME, str(kind).encode("utf-8"))
            md.setText(str(kind))
        return md


class _Canvas(QWidget):
    """Zeichnet das Formular + Controls und behandelt Platzieren/Selektieren/Ziehen."""
    selection_changed = Signal(object)   # Control | None
    doc_changed = Signal()
    doc_replaced = Signal(object)        # FormDoc (komplett ersetzt: set_doc/Undo)
    handler_requested = Signal(object)   # Control (Doppelklick -> Code-Editor)
    context_menu = Signal(object)        # QPoint (global) -- Rechtsklick auf Control
    zoom_changed = Signal(float)         # neue Zoom-Stufe
    form_resized = Signal()              # Formular per Griff in der Groesse geaendert

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc = FormDoc()
        self.selected: Control | None = None     # primaeres Control (Inspector/Griffe)
        self.selection: list[Control] = []       # vollstaendige Mehrfach-Selektion
        self.place_kind: str | None = None      # aus der Palette "scharf geschaltet"
        self.snap_grid = True                    # Snap-to-Grid aktiv?
        # commit_history(pre_snapshot): vom Fenster gesetzt, legt einen
        # Undo-Checkpoint an. None = kein Undo (z.B. Standalone-Canvas).
        self.commit_history = None
        self._drag = False
        self._drag_off = QPoint(0, 0)
        self._resize_handle: str | None = None   # aktiver Resize-Griff beim Ziehen
        self._pending: dict | None = None        # Pre-Gesten-Snapshot (Drag/Resize)
        self._nudge_active = False               # laufende Pfeiltasten-Verschiebung?
        self.zoom = 1.0                          # Zoom-Faktor der Design-Flaeche
        self.show_rulers = True                  # Lineale am Rand?
        self._guides_v: list[int] = []           # aktive vertikale Ausrichtlinien (ctrl-x)
        self._guides_h: list[int] = []           # aktive horizontale Ausrichtlinien (ctrl-y)
        self._multi = False                      # Gruppen-Drag (mehrere) aktiv?
        self._drag_origin = (0, 0)               # Maus-Start (ctrl-Raum) fuer Gruppen-Drag
        self._drag_starts: list = []             # [(control, x0, y0)] beim Gruppen-Drag
        self._band = False                       # Auswahlrahmen (Rubber-Band) aktiv?
        self._band_start = (0, 0)
        self._band_now = (0, 0)
        self._band_additive = False
        self._form_resize: str | None = None     # Formular-Resize-Griff (e/s/se)
        self.setMinimumSize(640, 480)
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setMouseTracking(True)              # Hover-Cursor ueber den Griffen
        self.setAcceptDrops(True)                # Drag&Drop aus der Palette

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self.selected = None
        self.selection = []
        # Alle Gesten-Flags loeschen: das Dokument kann mitten im Ziehen
        # getauscht werden (Strg+Z, Formularwechsel). Blieben sie stehen,
        # verschob die naechste Mausbewegung das neue Dokument.
        self._pending = None
        self._band = False
        self._drag = False
        self._multi = False
        self._drag_starts = []
        self._resize_handle = None
        self._form_resize = None
        self.place_kind = None
        self._guides_v = []; self._guides_h = []
        self.selection_changed.emit(None)
        self.doc_replaced.emit(doc)
        self._resize_to_doc()
        self.update()

    def _resize_to_doc(self):
        z = self.zoom
        self.setMinimumSize(int((self.doc.w + 2 * PAD + 40) * z),
                            int((self.doc.h + 2 * PAD + 40) * z))

    def set_zoom(self, z: float):
        z = max(0.25, min(4.0, z))
        if abs(z - self.zoom) < 1e-6:
            return
        self.zoom = z
        self._resize_to_doc()
        self.update()
        self.zoom_changed.emit(z)

    def wheelEvent(self, ev):
        if ev.modifiers() & Qt.KeyboardModifier.ControlModifier:
            self.set_zoom(self.zoom * (1.1 if ev.angleDelta().y() > 0 else 1 / 1.1))
            ev.accept()
        else:
            super().wheelEvent(ev)

    # -- Koordinaten --
    # Widget-Pixel -> Zeichen-Raum (durch den Zoom geteilt); Zeichen-Raum ->
    # Control-relativ (Fenster-Inhalt). Mausereignisse erst durch _to_draw.
    def _to_draw(self, pt) -> QPoint:
        z = self.zoom or 1.0
        return QPoint(int(pt.x() / z), int(pt.y() / z))

    def _to_ctrl(self, p: QPoint) -> tuple[int, int]:
        return (p.x() - PAD, p.y() - PAD - TITLE_H)

    def _snap(self, v: int) -> int:
        return snap(v) if self.snap_grid else int(v)

    _ALIGN_THRESH = 6        # Fangabstand (Zeichen-Pixel) fuer Ausrichtlinien

    def _align_axis(self, lo: float, size: float, targets: set) -> tuple:
        """Eine Achse fangen: `lo`/`size` = Kante+Laenge des gezogenen Controls.
        Liefert (neue_lo | None, guide-Linie | None) -- gefangen wird die Kante
        (Anfang/Mitte/Ende) mit der kleinsten Distanz zu einem Ziel."""
        best = None
        for off in (0.0, size / 2.0, size):       # left/center/right bzw. top/mid/bottom
            edge = lo + off
            for t in targets:
                d = t - edge
                if abs(d) <= self._ALIGN_THRESH and (best is None or abs(d) < abs(best[0])):
                    best = (d, t)
        if best is None:
            return None, None
        return lo + best[0], best[1]

    def _x_targets(self, c: Control) -> set:
        xs = {0.0, self.doc.w / 2.0, float(self.doc.w)}
        for o in self.doc.controls:
            if o is not c:
                xs.update((float(o.x), o.x + o.w / 2.0, float(o.x + o.w)))
        return xs

    def _y_targets(self, c: Control) -> set:
        bottom = self.doc.h - TITLE_H
        ys = {0.0, bottom / 2.0, float(bottom)}
        for o in self.doc.controls:
            if o is not c:
                ys.update((float(o.y), o.y + o.h / 2.0, float(o.y + o.h)))
        return ys

    def _move_to(self, px: int, py: int):
        """Selektiertes Control nach (px, py) (Ziel-Ecke) verschieben -- mit
        Ausrichtungs-Fang an andere Controls/Formularraender, sonst Raster-Fang.
        Setzt die aktiven Hilfslinien (`_guides_*`)."""
        c = self.selected
        px = max(0, px); py = max(0, py)
        nx, gx = self._align_axis(px, c.w, self._x_targets(c))
        ny, gy = self._align_axis(py, c.h, self._y_targets(c))
        c.x = int(nx) if nx is not None else self._snap(px)
        c.y = int(ny) if ny is not None else self._snap(py)
        # Im Formular halten: ein weit nach rechts/unten gezogenes Control lag
        # sonst ausserhalb der Zeichenflaeche -- unsichtbar, nicht anklickbar,
        # in keiner Liste, und so auch ins .dhform gespeichert. Zurueckholen
        # ging nur per Undo oder Hand-Edit im JSON.
        c.x = min(max(0, c.x), self._max_x(c))
        c.y = min(max(0, c.y), self._max_y(c))
        self._guides_v = [int(gx)] if gx is not None else []
        self._guides_h = [int(gy)] if gy is not None else []

    def _clear_guides(self):
        if self._guides_v or self._guides_h:
            self._guides_v = []; self._guides_h = []
            self.update()

    def _handle_points(self, c: Control) -> dict[str, QPoint]:
        """Screen-Mittelpunkte der 8 Resize-Griffe des Controls."""
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        mx, my = x + c.w // 2, y + c.h // 2
        rx, by = x + c.w, y + c.h
        return {
            "nw": QPoint(x, y), "n": QPoint(mx, y), "ne": QPoint(rx, y),
            "e": QPoint(rx, my), "se": QPoint(rx, by), "s": QPoint(mx, by),
            "sw": QPoint(x, by), "w": QPoint(x, my),
        }

    def _handle_tol(self, c: Control | None = None) -> float:
        """Trefferradius eines Resize-Griffs im Zeichen-Raum.

        `/zoom`, damit der Griff auf dem BILDSCHIRM immer gleich gross ist (bei
        0,25x war er sonst 2 px gross und praktisch nicht treffbar). Zusaetzlich
        nie mehr als ein Viertel des Controls: die 8 Zonen ueberdeckten ein
        16x16-Control (Checkbox/Radio = Palette-Default) sonst zu 100 %, es gab
        keinen Pixel mehr zum Verschieben -- jeder Zieh-Versuch hat die Checkbox
        stattdessen auf 8x8 geschrumpft."""
        t = (HANDLE / 2.0) / (self.zoom or 1.0)
        if c is not None:
            t = min(t, c.w / 4.0, c.h / 4.0)
        return max(2.0, t)

    def _handle_at(self, p: QPoint) -> str | None:
        """Welcher Resize-Griff des selektierten Controls liegt unter `p`?"""
        c = self.selected
        if c is None:
            return None
        tol = self._handle_tol(c)
        # Innenbereich gehoert immer dem Verschieben (durch die Viertel-Grenze
        # oben bleibt er mindestens halb so gross wie das Control).
        x, y = PAD + c.x, PAD + TITLE_H + c.y
        if (x + tol < p.x() < x + c.w - tol) and (y + tol < p.y() < y + c.h - tol):
            return None
        for name, hp in self._handle_points(c).items():
            if abs(p.x() - hp.x()) <= tol and abs(p.y() - hp.y()) <= tol:
                return name
        return None

    def _form_handle_points(self) -> dict[str, QPoint]:
        """Resize-Griffe des Formulars (rechts/unten/Ecke), Zeichen-Raum."""
        w, h = self.doc.w, self.doc.h
        return {
            "e": QPoint(PAD + w, PAD + h // 2),
            "s": QPoint(PAD + w // 2, PAD + h),
            "se": QPoint(PAD + w, PAD + h),
        }

    def _form_handle_at(self, p: QPoint) -> str | None:
        if self.selection:                       # nur wenn das Fenster "selektiert" ist
            return None
        tol = self._handle_tol()
        for name, hp in self._form_handle_points().items():
            if abs(p.x() - hp.x()) <= tol and abs(p.y() - hp.y()) <= tol:
                return name
        return None

    def _clamp_fw(self, v: int) -> int:
        v = max(120, int(v))
        if self.doc.min_w:
            v = max(v, self.doc.min_w)
        if self.doc.max_w:
            v = min(v, self.doc.max_w)
        return v

    def _clamp_fh(self, v: int) -> int:
        v = max(80, int(v))
        if self.doc.min_h:
            v = max(v, self.doc.min_h)
        if self.doc.max_h:
            v = min(v, self.doc.max_h)
        return v

    def paintEvent(self, _ev):
        qp = QPainter(self)
        qp.setRenderHint(QPainter.RenderHint.Antialiasing, True)
        self._paint_backdrop(qp)                       # Hintergrund (Widget-Raum)
        qp.save()
        qp.scale(self.zoom, self.zoom)                 # ab hier alles im Zeichen-Raum
        d = self.doc
        th = theme_colors(d.theme)
        win = QRect(PAD, PAD, d.w, d.h)
        # Formularflaeche + Titelleiste im gewaehlten Thema -- vorher fest
        # verdrahtet, sodass der Entwurf immer cyan aussah, egal was das
        # Formular spaeter benutzt.
        self._paint_form_shadow(qp, win)
        qp.fillRect(win, _col(th["win_bg"]))
        qp.setPen(QPen(_col(th["win_border"]), 1))
        qp.drawRect(win)
        if self.snap_grid and self.zoom >= GRID_MIN_ZOOM:
            self._paint_grid(qp, d, th)
        # Titelleiste
        qp.fillRect(QRect(PAD, PAD, d.w, TITLE_H), _col(th["title_bg"]))
        qp.setPen(_col(th["title_fg"]))
        qp.drawText(PAD + 6, PAD + 15, d.title)
        # Controls
        for c in d.controls:
            self._paint_control(qp, c)
        self._paint_guides(qp, d)
        if not self.selection:
            self._paint_form_handles(qp, d)
        self._paint_selection(qp)
        if self._band:
            self._paint_band(qp)
        qp.restore()                                   # zurueck in den Widget-Raum
        if self.show_rulers:
            self._paint_rulers(qp)

    def _paint_rulers(self, qp: QPainter):
        """Lineale am oberen + linken Rand (Widget-Raum). Zeigen Formular-
        Koordinaten (Inhalt: x ab Form-Links, y ab unter der Titelleiste),
        skaliert mit dem Zoom; markieren den Bereich der Selektion cyan."""
        z = self.zoom
        # Die Lineale liegen im Widget-Raum und schrumpfen daher NICHT mit dem
        # Zoom -- der Formularrand (PAD) schon. Ab etwa 0,7x deckten sie sonst
        # den Anfang des Formulars zu: Controls bis x=48 waren bei 0,25x
        # unsichtbar (aber weiter anklickbar), man suchte ein "verschwundenes"
        # Control. Darum die Leiste auf den verfuegbaren Rand begrenzen.
        R = max(6, min(RULER, int(PAD * z)))
        bg = QColor(28, 34, 42); tickc = QColor(96, 112, 128); txt = QColor(150, 165, 180)
        ox = PAD * z                       # Widget-x von Form-Inhalt-x = 0
        oy = (PAD + TITLE_H) * z            # Widget-y von Form-Inhalt-y = 0
        W, H = self.width(), self.height()
        qp.fillRect(0, 0, W, R, bg)
        qp.fillRect(0, 0, R, H, bg)
        qp.fillRect(0, 0, R, R, QColor(20, 26, 32))
        # Selektions-Bereich hervorheben
        hi = QColor(43, 196, 232, 70)
        for c in self.selection:
            qp.fillRect(int(ox + c.x * z), 0, max(1, int(c.w * z)), R, hi)
            qp.fillRect(0, int(oy + c.y * z), R, max(1, int(c.h * z)), hi)
        step = 50
        qp.setFont(QFont("Segoe UI", 6))
        # Horizontal
        i = 0
        while True:
            wx = ox + i * z
            if wx > W:
                break
            if wx >= R:
                qp.setPen(QPen(tickc, 1)); qp.drawLine(int(wx), R - 5, int(wx), R)
                qp.setPen(txt); qp.drawText(int(wx) + 2, R - 6, str(i))
            i += step
        # Vertikal
        j = 0
        while True:
            wy = oy + j * z
            if wy > H:
                break
            if wy >= R:
                qp.setPen(QPen(tickc, 1)); qp.drawLine(R - 5, int(wy), R, int(wy))
                qp.save(); qp.translate(R - 6, int(wy) - 2); qp.rotate(-90)
                qp.setPen(txt); qp.drawText(0, 0, str(j)); qp.restore()
            j += step

    def _paint_form_handles(self, qp: QPainter, d: FormDoc):
        """Das Formular ist 'selektiert' (kein Control): Accent-Rahmen + 3 Griffe."""
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(PAD - 1, PAD - 1, d.w + 2, d.h + 2))
        qp.setPen(QPen(QColor(12, 18, 24), 1))
        qp.setBrush(QBrush(QColor(43, 196, 232)))
        s = HANDLE / (self.zoom or 1.0)      # auf dem Bildschirm konstant gross
        for hp in self._form_handle_points().values():
            qp.drawRect(QRectF(hp.x() - s / 2, hp.y() - s / 2, s, s))

    def _paint_selection(self, qp: QPainter):
        """Mehrfach-Selektion: alle bekommen einen Rahmen, das primaere Control
        zusaetzlich die 8 Resize-Griffe (Einzel-Resize)."""
        if len(self.selection) == 1:
            self._paint_handles(qp, self.selection[0])
            return
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        for c in self.selection:
            x = PAD + c.x; y = PAD + TITLE_H + c.y
            qp.drawRect(QRect(x - 1, y - 1, c.w + 2, c.h + 2))

    def _paint_band(self, qp: QPainter):
        x0, y0 = self._band_start
        x1, y1 = self._band_now
        r = QRect(PAD + min(x0, x1), PAD + TITLE_H + min(y0, y1), abs(x1 - x0), abs(y1 - y0))
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(QColor(43, 196, 232, 40))
        qp.drawRect(r)

    def _paint_guides(self, qp: QPainter, d: FormDoc):
        """Ausrichtungs-Hilfslinien waehrend des Ziehens (pink, ueber dem Form)."""
        if not (self._guides_v or self._guides_h):
            return
        qp.setPen(QPen(QColor(255, 92, 162), 1, Qt.PenStyle.DashLine))
        for gx in self._guides_v:
            X = PAD + gx
            qp.drawLine(X, PAD, X, PAD + d.h)
        for gy in self._guides_h:
            Y = PAD + TITLE_H + gy
            qp.drawLine(PAD, Y, PAD + d.w, Y)

    def _paint_backdrop(self, qp: QPainter):
        """Flaeche um das Formular: leichter Verlauf statt einer flachen Farbe.

        Eine einzige Farbe liess das Formular wie einen Ausschnitt derselben
        Flaeche wirken; der Verlauf gibt der Arbeitsflaeche eine Richtung,
        ohne von den Controls abzulenken. Bleibt im Widget-Raum -- er soll
        beim Zoomen stehen, nicht mitwandern.

        Bewusst HELLER als die Docks ringsum und heller als beide
        Formular-Themen. Vorher war die Flaeche fast schwarz, und damit hatte
        ein Schlagschatten keinen Spielraum mehr nach unten: gemessene 11
        Stufen Unterschied, praktisch unsichtbar. Erst ein angehobener Grund
        gibt dem Schatten Platz -- dieselbe Ueberlegung wie bei der
        Arbeitsflaeche in Figma oder Xojo."""
        g = QLinearGradient(0, 0, 0, self.height() or 1)
        g.setColorAt(0.0, QColor(52, 60, 70))
        g.setColorAt(1.0, QColor(36, 42, 50))
        qp.fillRect(self.rect(), QBrush(g))

    def _paint_form_shadow(self, qp: QPainter, win: QRect):
        """Weicher Schlagschatten unter dem Formular.

        Er hebt die Form von der Arbeitsflaeche ab -- ohne ihn steht sie als
        randlose Farbflaeche darin. Qt hat keinen Formen-Weichzeichner ohne
        Umweg ueber ein QGraphicsEffect, deshalb gestaffelte Kopien mit
        geringer Deckkraft (dasselbe Rezept wie `schatten_weich` im
        chart-Modul): von aussen nach innen, die Ueberlagerung ergibt den
        Verlauf. Leicht nach unten versetzt -- Licht kommt von oben."""
        qp.save()
        qp.setPen(Qt.PenStyle.NoPen)
        qp.setBrush(QColor(0, 0, 0, 38))
        for i in range(SHADOW, 0, -1):
            qp.drawRoundedRect(win.adjusted(-i, -i + 4, i, i + 4), i + 1, i + 1)
        qp.restore()

    def _paint_grid(self, qp: QPainter, d: FormDoc, th: dict):
        """Dezente Raster-Punkte im Fenster-Inhaltsbereich (unter der Titelleiste).

        Die Farbe kommt aus dem Formular-Thema: ein fester blaugrauer Ton war
        auf einem hellen Formular ein dunkles Punktmuster und stach mehr
        hervor als die Controls.

        Gemischt wird zur SCHRIFTFARBE, nicht zur Rahmenfarbe -- die liegt in
        beiden Themen dicht an der Fensterfarbe, das Raster verschwand damit
        vollstaendig. Die Schriftfarbe steht per Definition im Kontrast zum
        Hintergrund, also ergibt ein kleiner Anteil davon in JEDEM Thema eine
        sichtbare Andeutung: hell auf dunkel, dunkel auf hell."""
        qp.setPen(QPen(_mix(_col(th["win_bg"]), _col(th["text_fg"]), 0.25), 1))
        x0 = PAD
        y0 = PAD + TITLE_H
        gx = GRID
        while gx <= d.w:
            gy = GRID
            while gy <= d.h - TITLE_H:
                qp.drawPoint(x0 + gx, y0 + gy)
                gy += GRID
            gx += GRID

    def _paint_control(self, qp: QPainter, c: Control):
        """Rendert ein Control moeglichst so, wie es zur Laufzeit (gui-Modul,
        Cyan-Theme) aussieht: gefuellte Flaechen, Haken/Knopf/Fortschritt,
        Dropdown-Pfeil, ListBox-Eintraege, disabled gedimmt, unsichtbar getoent."""
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        w, h = max(c.w, 4), max(c.h, 4)
        r = QRect(x, y, w, h)
        k = c.kind
        en = c.enabled
        th = theme_colors(self.doc.theme)
        fg = _col(th["text_fg"]) if en else _col(th["muted_fg"])
        accent = _col(th["accent"]) if en else _mix(_col(th["accent"]), _col(th["win_bg"]), 0.55)
        border = _col(th["widget_border"])
        # Flaechenfarbe + Plastik aus dem Thema. `rad` bestimmt, ob eckig oder
        # gerundet gezeichnet wird -- so entspricht der Entwurf dem, was die
        # Laufzeit spaeter zeigt.
        face = _col(th["widget_bg"]) if en else _mix(_col(th["widget_bg"]), _col(th["win_bg"]), 0.5)
        # Versenkte Flaeche (Eingabefeld, Liste, Fortschritts-Trog): dunkler
        # als die erhabene, damit man sieht, was man ausfuellt und was man
        # anklickt -- dieselbe Unterscheidung wie in der Laufzeit.
        sunk = _mix(face, _col(th["win_bg"]), 0.55)
        rad = th["radius"]
        grad = th["gradient"]
        al = Qt.AlignmentFlag
        font = QFont("Segoe UI")
        font.setPixelSize(max(7, c.font_size)) if c.font_size else font.setPointSize(8)
        qp.setFont(font)
        qp.setRenderHint(QPainter.RenderHint.Antialiasing, True)

        if k == "button":
            _fill_surface(qp, r, face, border, rad, grad, th["gloss"])
            qp.setPen(fg); qp.drawText(r, al.AlignCenter, c.text or "Button")
        elif k == "label":
            col = _col(c.color) if (en and c.color != 0xFFFFFF) else fg
            qp.setPen(col); qp.drawText(r, al.AlignVCenter | al.AlignLeft, c.text or "Label")
        elif k in ("checkbox", "radio"):
            bs = min(h, 16)
            box = QRect(x, y + (h - bs) // 2, bs, bs)
            qp.setPen(QPen(border, 1))
            qp.setBrush(_mix(face, _col(th["win_bg"]), 0.45))   # versenkte Mulde
            if k == "checkbox":
                qp.drawRoundedRect(box, 3, 3)
                if c.checked:
                    qp.setPen(QPen(accent, 2))
                    qp.drawLine(box.left() + 3, box.center().y(), box.center().x(), box.bottom() - 3)
                    qp.drawLine(box.center().x(), box.bottom() - 3, box.right() - 2, box.top() + 2)
            else:
                qp.drawEllipse(box)
                if c.checked:
                    qp.setBrush(accent); qp.setPen(Qt.PenStyle.NoPen)
                    qp.drawEllipse(box.adjusted(4, 4, -4, -4))
            qp.setPen(fg)
            qp.drawText(QRect(box.right() + 6, y, w - bs - 6, h), al.AlignVCenter, c.text or k.capitalize())
        elif k == "slider":
            midy = y + h // 2
            frac = (c.value - c.min) / (c.max - c.min) if c.max > c.min else 0.0
            frac = min(1.0, max(0.0, frac))
            kx = int(x + 5 + frac * (w - 10))
            qp.setPen(QPen(border, 3)); qp.drawLine(x + 5, midy, x + w - 5, midy)
            qp.setPen(QPen(accent, 3)); qp.drawLine(x + 5, midy, kx, midy)
            qp.setBrush(accent); qp.setPen(Qt.PenStyle.NoPen)
            qp.drawEllipse(QPoint(kx, midy), 6, 6)
        elif k == "textinput":
            _fill_surface(qp, r, sunk, border, rad, -grad, 0)
            if c.text:
                qp.setPen(fg); txt = c.text
            else:
                qp.setPen(_col(th["muted_fg"])); txt = c.placeholder or ""
            qp.drawText(r.adjusted(6, 0, -4, 0), al.AlignVCenter, txt)
        elif k == "dropdown":
            _fill_surface(qp, r, face, border, rad, grad, th["gloss"])
            sel = c.items[c.sel] if 0 <= c.sel < len(c.items) else ""
            qp.setPen(fg); qp.drawText(r.adjusted(6, 0, -18, 0), al.AlignVCenter, sel)
            qp.drawText(QRect(x + w - 16, y, 14, h), al.AlignCenter, "▾")
        elif k == "listbox":
            _fill_surface(qp, r, sunk, border, rad, -grad, 0)
            qp.save(); qp.setClipRect(r)
            lh = 15
            for i, it in enumerate(c.items):
                iy = y + 2 + i * lh
                if iy >= y + h:
                    break
                if i == c.sel:
                    qp.fillRect(QRect(x + 1, iy, w - 2, lh), _mix(_col(th["win_bg"]), accent, 0.45))
                qp.setPen(fg); qp.drawText(QRect(x + 5, iy, w - 8, lh), al.AlignVCenter, str(it))
            qp.restore()
        elif k == "progress":
            _fill_surface(qp, r, sunk, border, rad, -grad, 0)
            frac = _progress_frac(c)
            fillw = int((w - 2) * frac)
            if fillw > 0:
                qp.fillRect(QRect(x + 1, y + 1, fillw, h - 2), accent)
        elif k == "panel":
            _fill_surface(qp, r, _mix(face, _col(th["win_bg"]), 0.5), border, rad, grad, 0)
            qp.fillRect(QRect(x, y, w, 16), _col(th["title_bg"]))
            qp.setPen(fg); qp.drawText(QRect(x + 5, y, w - 8, 16), al.AlignVCenter, c.text or "")
        elif k == "separator":
            my = y + h // 2
            qp.setPen(QPen(border, 1)); qp.drawLine(x, my, x + w - 1, my)
        elif k == "groupbox":
            qp.setPen(QPen(border, 1)); qp.setBrush(Qt.BrushStyle.NoBrush)
            qp.drawRect(QRect(x, y + 7, w - 1, h - 8))
            if c.text:
                qp.fillRect(QRect(x + 8, y, min(w - 16, len(c.text) * 7 + 8), 13), _col(th["win_bg"]))
                qp.setPen(fg); qp.drawText(x + 12, y + 11, c.text)
        elif k == "image":
            qp.setPen(QPen(border, 1)); qp.setBrush(_mix(face, _col(th["win_bg"]), 0.4)); qp.drawRect(r)
            qp.setPen(QPen(accent, 1))
            qp.drawLine(x + 4, y + h - 5, x + w // 2 - 2, y + h // 2)
            qp.drawLine(x + w // 2 - 2, y + h // 2, x + w - 5, y + h - 5)
            qp.setBrush(QColor(240, 220, 120)); qp.setPen(Qt.PenStyle.NoPen)
            qp.drawEllipse(QRect(x + w - 16, y + 6, 7, 7))
        elif k == "table":
            self._paint_table(qp, c, r, th, face, sunk, border, fg, accent)
        elif k == "canvas":
            qp.setBrush(sunk); qp.setPen(QPen(border, 1, Qt.PenStyle.DashLine)); qp.drawRect(r)
            qp.setPen(_col(th["muted_fg"])); qp.drawText(r, al.AlignCenter, "Canvas")
        elif k == "layout":
            # Unsichtbar zur Laufzeit -- im Entwurf gestrichelt mit seiner Art,
            # damit man ihn anfassen kann. Die Kinder liegen im Entwurf, wo
            # sie liegen; verteilt werden sie beim Start.
            yj0 = c.extra.get("layout")
            yj: dict = yj0 if isinstance(yj0, dict) else {}
            qp.setBrush(Qt.BrushStyle.NoBrush); qp.setPen(QPen(accent, 1, Qt.PenStyle.DashLine)); qp.drawRect(r)
            qp.setPen(_col(th["muted_fg"]))
            qp.drawText(r.adjusted(4, 2, -4, -2), al.AlignLeft | al.AlignTop, "Layout: " + str(yj.get("art") or "spalte"))
        elif _preview_neu(qp, k, r, fg=fg, muted=_col(th["muted_fg"]), accent=accent,
                          border=border, face=face, sunk=sunk,
                          title_bg=_col(th["title_bg"]), win_bg=_col(th["win_bg"]),
                          rad=rad, grad=grad, gloss=th["gloss"],
                          text=c.text, value=c.value, mn=c.min, mx=c.max,
                          checked=c.checked, items=c.items,
                          farbe=hex_zu_int(c.extra.get("color_value")) or 0xE84B4B,
                          datum=str(c.extra.get("date") or "")):
            pass
        else:
            _fill_surface(qp, r, face, border, rad, grad, th["gloss"])
            qp.setPen(fg); qp.drawText(r, al.AlignCenter, c.text or k)

        if not c.visible:                       # unsichtbares Control angedeutet toenen
            qp.fillRect(r, QColor(_col(th["win_bg"]).red(), _col(th["win_bg"]).green(),
                                  _col(th["win_bg"]).blue(), 150))

    @staticmethod
    def _table_opt(c: Control, key: str, standard):
        """Einstellung der Tabelle aus `extra["table"]` -- die Laufzeit-Felder
        liegen dort unveraendert, damit ein per GUI_SAVE erzeugtes Formular
        beim Oeffnen nichts verliert."""
        tj = c.extra.get("table") or {}
        v = tj.get(key, standard)
        return v if isinstance(v, type(standard)) or standard is None else standard

    def _paint_table(self, qp: QPainter, c: Control, r: QRect, th: dict,
                     face, sunk, border, fg, accent):
        """Vorschau der Tabelle: Kopfzeile, optional Filterzeile, ein paar
        Gitterlinien. Bewusst nur eine ANDEUTUNG -- die Zeilen kommen im
        Normalfall zur Laufzeit aus dem Programm, hier zu tun als wuesste der
        Designer sie waere gelogen."""
        x, y, w, h = r.x(), r.y(), r.width(), r.height()
        kopf = [str(s) for s in (self._table_opt(c, "headers", []) or [])]
        breiten = [int(v) for v in (self._table_opt(c, "col_widths", []) or [])
                   if isinstance(v, (int, float))]
        kh = int(self._table_opt(c, "header_h", 22))
        zh = max(8, int(self._table_opt(c, "row_h", 20)))
        filterzeile = bool(self._table_opt(c, "filter_row", False))
        gitter = bool(self._table_opt(c, "grid", True))
        zebra = bool(self._table_opt(c, "zebra", False))
        fest = int(self._table_opt(c, "frozen", 0))

        qp.save()
        qp.setClipRect(r)
        qp.fillRect(r, face)
        qp.setPen(QPen(border, 1)); qp.setBrush(Qt.BrushStyle.NoBrush); qp.drawRect(r)

        # Spaltenbreiten: gesetzte nehmen, sonst gleichmaessig verteilen --
        # dieselbe Regel wie in der Laufzeit, sonst saehe der Entwurf anders
        # aus als das Ergebnis.
        n = max(len(kopf), len(breiten))
        if n == 0:
            qp.setPen(_col(th["muted_fg"]))
            qp.drawText(r, Qt.AlignmentFlag.AlignCenter, "Tabelle (ohne Spalten)")
            qp.restore()
            return
        if len(breiten) != n:
            breiten = [max(40, (w - 12) // n)] * n

        if kh > 0:
            qp.fillRect(QRect(x + 1, y + 1, w - 2, kh - 1), _col(th["title_bg"]))
            qp.setPen(QPen(border, 1))
            qp.drawLine(x, y + kh - 1, x + w - 1, y + kh - 1)
        fy = y + kh
        koerper = y + kh + (TBL_FILTER_H if filterzeile else 0)

        cx = x + 1
        for i in range(n):
            cw = breiten[i]
            if cx > x + w:
                break
            if kh > 0:
                qp.setPen(fg)
                qp.drawText(QRect(cx + 5, y, max(1, cw - 8), kh),
                            Qt.AlignmentFlag.AlignVCenter,
                            kopf[i] if i < len(kopf) else "")
            if filterzeile:
                qp.fillRect(QRect(cx + 2, fy + 2, max(1, cw - 5), TBL_FILTER_H - 4), sunk)
                qp.setPen(_col(th["muted_fg"]))
                qp.drawText(QRect(cx + 6, fy, max(1, cw - 10), TBL_FILTER_H),
                            Qt.AlignmentFlag.AlignVCenter, "Filter ...")
            if gitter and i < n - 1:
                qp.setPen(QPen(_mix(face, border, 0.6), 1))
                qp.drawLine(cx + cw, y + 1, cx + cw, y + h - 2)
            cx += cw

        # Angedeutete Datenzeilen -- sie zeigen Zeilenhoehe und Zebra, ohne
        # Inhalte zu erfinden.
        i = 0
        ry = koerper
        while ry < y + h - 2:
            if zebra and i % 2 == 1:
                qp.fillRect(QRect(x + 1, ry, w - 2, min(zh, y + h - 1 - ry)),
                            _mix(face, _col(th["win_bg"]), 0.35))
            if gitter:
                qp.setPen(QPen(_mix(face, border, 0.45), 1))
                qp.drawLine(x + 1, ry + zh - 1, x + w - 2, ry + zh - 1)
            ry += zh
            i += 1

        # Kante des festen Blocks -- im Entwurf immer zeigen, denn hier gibt es
        # kein Scrollen, an dem man sie sonst erkennen wuerde.
        if 0 < fest <= n:
            fx = x + 1 + sum(breiten[:fest])
            qp.setPen(QPen(accent, 1))
            qp.drawLine(fx, y + 1, fx, y + h - 2)
        qp.restore()

    def _paint_handles(self, qp: QPainter, c: Control):
        x = PAD + c.x
        y = PAD + TITLE_H + c.y
        # Selektionsrahmen
        qp.setPen(QPen(QColor(43, 196, 232), 1, Qt.PenStyle.DashLine))
        qp.setBrush(Qt.BrushStyle.NoBrush)
        qp.drawRect(QRect(x - 1, y - 1, c.w + 2, c.h + 2))
        # 8 Resize-Griffe -- so gross wie ihre Trefferzone (siehe _handle_tol)
        qp.setPen(QPen(QColor(12, 18, 24), 1))
        qp.setBrush(QBrush(QColor(43, 196, 232)))
        s = 2 * self._handle_tol(c)
        for hp in self._handle_points(c).values():
            qp.drawRect(QRectF(hp.x() - s / 2, hp.y() - s / 2, s, s))

    # -- Maus --
    def _finish_gesture(self):
        """Laufende Geste beenden: Undo-Checkpoint setzen (falls sich wirklich
        etwas geaendert hat) und ALLE Gesten-Flags loeschen.

        Wird nicht nur beim Loslassen gerufen, sondern auch, wenn sich die Maus
        ohne gedrueckte Taste bewegt. Sonst ueberlebte der Zustand jede
        Unterbrechung, die das Release verschluckt (modales Kontextmenue,
        Fokusverlust) oder das Dokument tauscht (Strg+Z waehrend des Ziehens) --
        danach verschob schon blosses Hovern das Control, ohne Undo."""
        active = (self._drag or self._multi or self._band
                  or self._resize_handle is not None or self._form_resize is not None
                  or self._pending is not None)
        if not active:
            return False
        if self._band:
            self._finish_band()
            self._band = False
        if self._pending is not None:
            if self.commit_history and self._pending != self.doc.to_dict():
                self.commit_history(self._pending)
            self._pending = None
        self._drag = False
        self._multi = False
        self._drag_starts = []
        self._resize_handle = None
        self._form_resize = None
        self._clear_guides()
        return True

    def mousePressEvent(self, ev):
        # Nur die linke Taste bedient Platzieren/Selektieren/Ziehen. Vorher
        # platzierte auch ein Rechtsklick ein scharf geschaltetes Control und
        # startete unsichtbare Auswahlrahmen; ausserdem ueberschrieb eine
        # zweite Taste mitten im Ziehen den Pre-Gesten-Snapshot, womit die
        # ganze Verschiebung nicht mehr rueckgaengig zu machen war.
        if ev.button() != Qt.MouseButton.LeftButton:
            return
        self._nudge_active = False
        p = self._to_draw(ev.position())
        cx, cy = self._to_ctrl(p)
        ctrl = bool(ev.modifiers() & Qt.KeyboardModifier.ControlModifier)
        if self.place_kind:
            self._place_control(self.place_kind, cx, cy)
            self.place_kind = None
            return
        # Resize-Griff -- nur bei genau einem selektierten Control
        if len(self.selection) == 1:
            handle = self._handle_at(p)
            if handle is not None:
                self._resize_handle = handle
                self._pending = self.doc.to_dict()           # Pre-Resize-Snapshot
                return
        # Formular-Resize-Griff (wenn nichts selektiert = Fenster aktiv)
        fh = self._form_handle_at(p)
        if fh is not None:
            self._form_resize = fh
            self._pending = self.doc.to_dict()
            return
        hit = self.doc.control_at(cx, cy)
        if hit is None:                                       # leerer Bereich -> Rubber-Band
            if not ctrl:
                self._select(None)
            self._band = True
            self._band_additive = ctrl
            self._band_start = (cx, cy)
            self._band_now = (cx, cy)
            self.update()
            return
        if ctrl:                                              # Strg+Klick: Toggle
            self._toggle_select(hit)
            self.update()
            return
        if not self._in_selection(hit):                       # neues Einzel-Ziel
            self._select(hit)
        else:                                                 # Teil der Gruppe -> primary
            self.selected = hit
            self.selection_changed.emit(hit)
        self._begin_drag(cx, cy)
        self.update()

    def _begin_drag(self, cx: int, cy: int):
        self._drag = True
        self._pending = self.doc.to_dict()                   # Move = eine Geste = 1 Undo
        if len(self.selection) > 1:
            self._multi = True
            self._drag_origin = (cx, cy)
            self._drag_starts = [(c, c.x, c.y) for c in self.selection]
        else:
            self._multi = False
            self._drag_off = QPoint(cx - self.selected.x, cy - self.selected.y)

    def _multi_move(self, cx: int, cy: int):
        dx = self._snap(cx - self._drag_origin[0])
        dy = self._snap(cy - self._drag_origin[1])
        # Das Delta begrenzen, NICHT jedes Control einzeln: bei einem
        # Einzel-Clamp liefen die Controls am Rand auf, waehrend die anderen
        # weiterwanderten -- eine ausgerichtete Knopfreihe wurde beim Schieben
        # an den linken Rand still zusammengedrueckt.
        dx = self._clamp_delta(dx, [(sx, self._max_x(c)) for c, sx, _ in self._drag_starts])
        dy = self._clamp_delta(dy, [(sy, self._max_y(c)) for c, _, sy in self._drag_starts])
        for c, sx, sy in self._drag_starts:
            c.x = sx + dx; c.y = sy + dy

    @staticmethod
    def _clamp_delta(d: int, starts: list) -> int:
        """Verschiebe-Delta so begrenzen, dass alle Controls im Formular
        bleiben. `starts` = [(startwert, maximum)]. Passt die Gruppe nicht
        vollstaendig hinein, wird nur nach links/oben geklemmt -- lieber ein
        Ueberstand als eine verzerrte Gruppe."""
        lo = -min(s for s, _ in starts)
        hi = min(m - s for s, m in starts)
        return max(lo, min(d, hi)) if hi >= lo else max(lo, d)

    def _max_x(self, c: Control) -> int:
        return max(0, self.doc.w - c.w)

    def _max_y(self, c: Control) -> int:
        return max(0, self.doc.h - TITLE_H - c.h)

    def mouseMoveEvent(self, ev):
        p = self._to_draw(ev.position())
        cx, cy = self._to_ctrl(p)
        if not (ev.buttons() & Qt.MouseButton.LeftButton):
            # Ohne gedrueckte Taste darf keine Geste mehr laufen -- sonst
            # verschiebt/resized blosses Hovern (siehe _finish_gesture).
            if self._finish_gesture():
                self.update()
        if self._band:
            self._band_now = (cx, cy)
            self.update()
            return
        if self._form_resize is not None:
            if "e" in self._form_resize:
                self.doc.w = self._clamp_fw(self._snap(p.x() - PAD))
            if "s" in self._form_resize:
                self.doc.h = self._clamp_fh(self._snap(p.y() - PAD))
            self._resize_to_doc()
            self.form_resized.emit()
            self.doc_changed.emit()
            self.update()
            return
        if self._resize_handle is not None and self.selected is not None:
            c = self.selected
            nx, ny = self._snap(cx), self._snap(cy)
            rx, ry, rw, rh = resize_rect(c.x, c.y, c.w, c.h, self._resize_handle, nx, ny)
            # Der Resize-Pfad klemmte als einziger NICHT auf >= 0: ein Zug am
            # NW-/N-/W-Griff ueber die obere/linke Formularkante hinaus schrieb
            # `x: -56` ins .dhform, zur Laufzeit ragte das Widget aus dem
            # Fenster und wurde weggeclippt. Kante festhalten, Groesse kuerzen.
            if rx < 0:
                rw += rx; rx = 0
            if ry < 0:
                rh += ry; ry = 0
            c.x, c.y, c.w, c.h = rx, ry, max(1, rw), max(1, rh)
            self.selection_changed.emit(c)               # Inspector live aktualisieren
            self.doc_changed.emit()
            self.update()
            return
        if self._drag and self.selected is not None:
            if self._multi:
                self._multi_move(cx, cy)
            else:
                self._move_to(cx - self._drag_off.x(), cy - self._drag_off.y())
            self.selection_changed.emit(self.selected)   # Inspector live aktualisieren
            self.doc_changed.emit()
            self.update()
            return
        # Kein Knopf gedrueckt: Hover-Cursor ueber Resize-Griffen
        if not (ev.buttons() & Qt.MouseButton.LeftButton):
            handle = self._handle_at(p) if len(self.selection) == 1 else None
            if handle is None:
                handle = self._form_handle_at(p)
            self.setCursor(_HANDLE_CURSORS[handle] if handle else Qt.CursorShape.ArrowCursor)

    def mouseReleaseEvent(self, _ev):
        # Gesten-Ende: Rubber-Band auswerten, dann nur EIN Undo-Checkpoint --
        # und auch nur, falls sich wirklich etwas geaendert hat.
        self._finish_gesture()
        self.update()

    def _finish_band(self):
        x0, y0 = self._band_start
        x1, y1 = self._band_now
        rx0, rx1 = sorted((x0, x1))
        ry0, ry1 = sorted((y0, y1))
        hits = [c for c in self.doc.controls
                if c.x < rx1 and c.x + c.w > rx0 and c.y < ry1 and c.y + c.h > ry0]
        if hits:
            self._select_many(hits, additive=self._band_additive)

    def mouseDoubleClickEvent(self, ev):
        if ev.button() != Qt.MouseButton.LeftButton:
            return
        cx, cy = self._to_ctrl(self._to_draw(ev.position()))
        hit = self.doc.control_at(cx, cy)
        if hit is not None:
            # Geste des ersten Klicks sauber abschliessen statt den Snapshot zu
            # verwerfen -- eine Bewegung dazwischen waere sonst nicht undobar.
            self._finish_gesture()
            self._focus_on(hit)         # Mehrfach-Auswahl nicht wegwerfen
            self.update()
            self.handler_requested.emit(hit)

    def _place_control(self, kind: str, cx: int, cy: int):
        """Ein neues Control an (cx, cy) platzieren (von Klick-Platzieren UND
        Drag&Drop genutzt). Setzt einen Undo-Checkpoint + selektiert es."""
        pre = self.doc.to_dict()
        c = self.doc.add(kind, max(self._snap(cx), 0), max(self._snap(cy), 0))
        # Auch ein Klick neben das Formular soll ein erreichbares Control geben.
        c.x = min(c.x, self._max_x(c)); c.y = min(c.y, self._max_y(c))
        if self.commit_history:
            self.commit_history(pre)
        self._select(c)
        self.doc_changed.emit()
        self.update()
        return c

    # -- Drag&Drop aus der Palette --
    def dragEnterEvent(self, ev):
        if ev.mimeData().hasFormat(_CONTROL_MIME):
            ev.acceptProposedAction()

    def dragMoveEvent(self, ev):
        if ev.mimeData().hasFormat(_CONTROL_MIME):
            ev.acceptProposedAction()

    def dropEvent(self, ev):
        md = ev.mimeData()
        if not md.hasFormat(_CONTROL_MIME):
            return
        kind = bytes(md.data(_CONTROL_MIME)).decode("utf-8")
        cx, cy = self._to_ctrl(self._to_draw(ev.position()))
        self.place_kind = None
        self._place_control(kind, cx, cy)
        ev.acceptProposedAction()
        self.setFocus()

    _ARROWS = {
        Qt.Key.Key_Left: (-1, 0), Qt.Key.Key_Right: (1, 0),
        Qt.Key.Key_Up: (0, -1), Qt.Key.Key_Down: (0, 1),
    }

    def keyPressEvent(self, ev):
        if ev.key() in (Qt.Key.Key_Delete, Qt.Key.Key_Backspace) and self.selection:
            # Ein laufender Drag darf hier nicht weiterlaufen -- sein Release
            # haette sonst einen zweiten, leeren Undo-Schritt erzeugt.
            self._drag = False; self._multi = False; self._pending = None
            self._resize_handle = None; self._form_resize = None
            pre = self.doc.to_dict()                         # Undo: Zustand vor dem Loeschen
            for c in list(self.selection):
                self.doc.remove(c)
            if self.commit_history:
                self.commit_history(pre)
            self._select(None)
            self.doc_changed.emit()
            self.update()
        elif ev.key() in self._ARROWS and self.selection:
            dx, dy = self._ARROWS[ev.key()]
            step = GRID if (ev.modifiers() & Qt.KeyboardModifier.ShiftModifier) else 1
            self._nudge(dx * step, dy * step)
        else:
            super().keyPressEvent(ev)

    def _nudge(self, dx: int, dy: int):
        """Selektion per Pfeiltaste verschieben (alle selektierten Controls).
        Eine Tastenfolge (bis Klick/Selektionswechsel) = EIN Undo-Schritt."""
        if not self._nudge_active:
            if self.commit_history:
                self.commit_history(self.doc.to_dict())      # Pre-Nudge-Snapshot
            self._nudge_active = True
        for c in self.selection:
            c.x = min(max(0, c.x + dx), self._max_x(c))
            c.y = min(max(0, c.y + dy), self._max_y(c))
        self.selection_changed.emit(self.selected)
        self.doc_changed.emit()
        self.update()

    def contextMenuEvent(self, ev):
        cx, cy = self._to_ctrl(self._to_draw(ev.pos()))
        hit = self.doc.control_at(cx, cy)
        if hit is not None:
            self._focus_on(hit)
            self.update()
            self.context_menu.emit(ev.globalPos())

    def _in_selection(self, c: Control) -> bool:
        """Identitaets- statt Wertvergleich -- `Control` ist eine dataclass mit
        generiertem `__eq__`, `in` traefe also jedes feldgleiche Control."""
        return any(x is c for x in self.selection)

    def _focus_on(self, c: Control):
        """`c` zum primaeren Control machen. Gehoert es schon zur Auswahl,
        bleibt die Gruppe bestehen -- Rechtsklick/Doppelklick auf ein
        Gruppenmitglied warf sie vorher weg, weshalb das Kontextmenue-Loeschen
        von fuenf markierten Controls nur eines erwischte."""
        if self._in_selection(c):
            self.selected = c
            self._nudge_active = False
            self.selection_changed.emit(c)
        else:
            self._select(c)

    def _select(self, c: Control | None):
        self.selected = c
        self.selection = [c] if c is not None else []
        self._nudge_active = False           # neue Selektion beendet Nudge-Sitzung
        self.selection_changed.emit(c)

    def _toggle_select(self, c: Control):
        """Strg+Klick: Control zur Mehrfach-Selektion hinzu/heraus."""
        if self._in_selection(c):
            self.selection = [x for x in self.selection if x is not c]
            self.selected = self.selection[-1] if self.selection else None
        else:
            self.selection.append(c)
            self.selected = c
        self._nudge_active = False
        self.selection_changed.emit(self.selected)

    def _select_many(self, controls: list, additive: bool = False):
        if additive:
            for c in controls:
                if not self._in_selection(c):
                    self.selection.append(c)
        else:
            self.selection = list(controls)
        self.selected = self.selection[-1] if self.selection else None
        self._nudge_active = False
        self.selection_changed.emit(self.selected)


class _Inspector(QWidget):
    """Eigenschaften + Events des gewaehlten Controls editieren."""
    changed = Signal()
    handler_renamed = Signal(str, str)     # alter, neuer Handler-Name
    control_renamed = Signal(object, str)  # Control, alter Control-Name

    def __init__(self, parent=None):
        super().__init__(parent)
        self._c: Control | None = None
        self._doc: FormDoc | None = None          # fuer die Layout-Zuordnung
        self._loading = False
        self._form = QFormLayout(self)
        self.name = QLineEdit(); self.text = QLineEdit()
        self.sx = QSpinBox(); self.sy = QSpinBox(); self.sw = QSpinBox(); self.sh = QSpinBox()
        for s in (self.sx, self.sy, self.sw, self.sh):
            s.setRange(0, 32000)         # 4000 klemmte grosse Controls still ab
        # Mindestens 1x1: bei Breite oder Hoehe 0 trifft `control_at` das
        # Control nicht mehr -- es war weder selektierbar noch sichtbar und
        # nur per Undo zurueckzuholen.
        self.sw.setMinimum(1); self.sh.setMinimum(1)
        self.enabled = QCheckBox("aktiviert")
        self.visible = QCheckBox("sichtbar")
        self.checked = QCheckBox("angehakt")
        # Ein Feld je Ereignis, aus der zentralen Liste erzeugt -- ein
        # weiteres Ereignis in der Laufzeit braucht hier keine Zeile mehr.
        self.ev_edits = {ev: QLineEdit() for ev in EVENTS}
        self.on_click = self.ev_edits["on_click"]     # Namen bleiben gueltig
        self.on_change = self.ev_edits["on_change"]
        # `group` fehlte komplett -- ohne sie landen ALLE RadioButtons in
        # derselben leeren Gruppe und schliessen sich nicht gegenseitig aus.
        self.group = QLineEdit(); self.group.setPlaceholderText("z.B. schwierigkeit")
        self.placeholder = QLineEdit()
        self.tooltip = QLineEdit(); self.tooltip.setPlaceholderText("Hinweis beim Ueberfahren")
        # Text und Formular: Ausrichtung, Umbruch, Textfeld-Grenzen -- dieselben
        # Schluessel wie die Laufzeit (`align`, `wrap`, `passwort`, ...).
        self.align = QComboBox()
        self.align.addItem("(Vorgabe)", "")
        self.align.addItem("links", "links")
        self.align.addItem("mitte", "mitte")
        self.align.addItem("rechts", "rechts")
        self.wrap = QCheckBox("umbrechen (bei Breite)")
        self.passwort = QCheckBox("Passwort (Punkte)")
        self.nur_lesen = QCheckBox("nur lesen")
        self.maxlaenge = QSpinBox(); self.maxlaenge.setRange(0, 100000)
        self.maxlaenge.setToolTip("Hoechstlaenge in Zeichen (0 = frei)")
        self.zahlen = QComboBox()
        self.zahlen.addItem("frei", 0)
        self.zahlen.addItem("ganze Zahl", 1)
        self.zahlen.addItem("Kommazahl", 2)
        # Liste: die zwei Schalter liegen in `extra["list"]`, dem Schluessel
        # der Laufzeit -- so ueberlebt ein per GUI_SAVE geschriebener Haken
        # den Rundweg durch den Designer.
        self.l_multi = QCheckBox("Mehrfachauswahl")
        self.l_kaestchen = QCheckBox("Kaestchen je Eintrag")
        # Layout-Behaelter: Art und Masse; fuer jedes andere Control der
        # Behaelter, in dem es steckt, und sein Gewicht.
        self.ly_art = QComboBox()
        for name, wert in (("Zeile", "zeile"), ("Spalte", "spalte"), ("Raster", "raster")):
            self.ly_art.addItem(name, wert)
        self.ly_spalten = QSpinBox(); self.ly_spalten.setRange(1, 32); self.ly_spalten.setValue(2)
        self.ly_abstand = QSpinBox(); self.ly_abstand.setRange(0, 200); self.ly_abstand.setValue(6)
        self.ly_rand = QSpinBox(); self.ly_rand.setRange(0, 200)
        self.ly_dehnen = QCheckBox("quer dehnen"); self.ly_dehnen.setChecked(True)
        self.ly_ausr = QComboBox()
        for name, wert in (("Anfang", 0), ("Mitte", 1), ("Ende", 2)):
            self.ly_ausr.addItem(name, wert)
        self.in_layout = QComboBox()
        self.in_gewicht = QSpinBox(); self.in_gewicht.setRange(0, 99)
        self.in_gewicht.setToolTip("0 = eigene Groesse, ab 1 = Anteil am Restplatz")
        # Punkt 6: Bildmodus, unbestimmter Fortschritt, senkrechter Schieber,
        # Ziehen/Ablage, rollendes Panel.
        self.bildmodus = QComboBox()
        for wert in ("strecken", "einpassen", "fuellen", "mitte", "kacheln"):
            self.bildmodus.addItem(wert, wert)
        self.p_unbestimmt = QCheckBox("unbestimmt (laufendes Band)")
        self.s_senkrecht = QCheckBox("senkrecht (GUI_VSLIDER, h = Laenge)")
        self.ziehbar = QCheckBox("ziehbar (Quelle)")
        self.ablage = QCheckBox("Ablage (nimmt Gezogenes an)")
        self.in_panel = QComboBox()
        # Mindestmass und Pruefregeln.
        self.min_w = QSpinBox(); self.min_w.setRange(0, 4000)
        self.min_h = QSpinBox(); self.min_h.setRange(0, 4000)
        self.tab_index = QSpinBox(); self.tab_index.setRange(0, 999); self.tab_index.setToolTip("0 = Anlege-Reihenfolge; groesser 0 kommt zuerst, aufsteigend")
        self.regeln = QLineEdit()
        self.bindung = QLineEdit()
        self.bindung.setPlaceholderText("Spaltenname (GUI_BIND)")
        self.formular = QLineEdit()
        self.formular.setPlaceholderText("Formularname (leer = Vorgabe)")
        self.ta_umbruch = QCheckBox("Zeilenumbruch (Notizen statt Code)")
        self.regeln.setPlaceholderText("pflicht; laenge 2 60; email; bereich 0 100; muster [0-9]{5}")
        self.regeln.setToolTip("Regeln durch ; getrennt: pflicht, zahl, ganz, bereich a b, laenge min max, email, datum, muster regex. "
                               "Eigene Meldung mit = dahinter: pflicht = Der Name fehlt.")
        # Der Trenner traegt seine Richtung im `text`-Feld -- als freie
        # Eingabe waere "h"/"v" nicht zu erraten, und ein Tippfehler faellt
        # erst zur Laufzeit auf (GUI_SPLITTER lehnt alles andere ab).
        self.orient = QComboBox()
        self.orient.addItem("waagerecht", "h")
        self.orient.addItem("senkrecht", "v")
        # Farbwaehler und Datumswaehler tragen ihren Wert in `extra` -- genau
        # unter den Schluesseln, die die Laufzeit in die `.dhform` schreibt
        # (`color_value` / `date`, siehe gui.rs::widget_json). So ueberlebt er
        # den Rundweg Speichern -> Laden, ohne dass das Modell neue Felder
        # braucht.
        self.pick_btn = QPushButton(); self.pick_btn.setFixedHeight(22)
        self.pick_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.pick_btn.clicked.connect(self._pick_startfarbe)
        self.datum = QLineEdit(); self.datum.setPlaceholderText("JJJJ-MM-TT (leer = heute)")
        self.ssel = QSpinBox(); self.ssel.setRange(-1, 9999)
        self.ssel.setToolTip("Ausgewaehlter Eintrag (-1 = keiner)")
        self.items = QPlainTextEdit(); self.items.setMaximumHeight(90)

        # --- Tabelle -------------------------------------------------------
        # Spaltentitel und Breiten als Text, eine Zeile je Spalte: eine echte
        # Spalten-Tabelle im Inspector waere ein Editor im Editor. Die
        # DATENzeilen fehlen bewusst -- eine Tabelle wird zur Laufzeit
        # gefuellt (Datei, Datenbank), nicht im Designer abgetippt.
        self.t_headers = QPlainTextEdit(); self.t_headers.setMaximumHeight(80)
        self.t_widths = QLineEdit(); self.t_widths.setPlaceholderText("z.B. 80, 160, 100 -- leer = gleich verteilt")
        self.t_rowh = QSpinBox(); self.t_rowh.setRange(8, 400); self.t_rowh.setValue(20)
        self.t_headh = QSpinBox(); self.t_headh.setRange(0, 200); self.t_headh.setValue(22)
        self.t_frozen = QSpinBox(); self.t_frozen.setRange(0, 20)
        self.t_zebra = QCheckBox("Zebra-Streifen")
        self.t_grid = QCheckBox("Gitterlinien"); self.t_grid.setChecked(True)
        self.t_filter = QCheckBox("Filterzeile im Kopf")
        self.t_sort = QCheckBox("Sortieren per Kopfklick"); self.t_sort.setChecked(True)
        self.t_resize = QCheckBox("Spaltenbreiten ziehbar"); self.t_resize.setChecked(True)
        self.t_reorder = QCheckBox("Spalten verschiebbar")
        self.t_multi = QCheckBox("Mehrfachauswahl")
        self.t_edit = QLineEdit(); self.t_edit.setPlaceholderText("bearbeitbare Spalten, z.B. 1, 2")
        self._t_felder = (self.t_headers, self.t_widths, self.t_rowh, self.t_headh,
                          self.t_frozen, self.t_zebra, self.t_grid, self.t_filter,
                          self.t_sort, self.t_resize, self.t_reorder, self.t_multi,
                          self.t_edit)
        self.vmin = QDoubleSpinBox(); self.vmax = QDoubleSpinBox(); self.vval = QDoubleSpinBox()
        for s in (self.vmin, self.vmax, self.vval):
            s.setRange(-1e6, 1e6)
        # Farbe (Swatch-Button statt Hex) + Schriftgroesse
        self.color_btn = QPushButton(); self.color_btn.setFixedHeight(22)
        self.color_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        self.color_btn.clicked.connect(self._pick_color)
        self.sfont = QSpinBox(); self.sfont.setRange(0, 96)
        self.sfont.setToolTip("Schriftgroesse in px (0 = Standard)")
        # Anker (Reflow beim Fenster-Resize): an welchen Kanten klebt das Control?
        self.a_l = QCheckBox("L"); self.a_r = QCheckBox("R")
        self.a_t = QCheckBox("O"); self.a_b = QCheckBox("U")
        for cb, tip in ((self.a_l, "Links"), (self.a_r, "Rechts"),
                        (self.a_t, "Oben"), (self.a_b, "Unten")):
            cb.setToolTip(f"Anker {tip}")
        self._anchor_box = QWidget()
        _ah = QHBoxLayout(self._anchor_box); _ah.setContentsMargins(0, 0, 0, 0)
        for cb in (self.a_l, self.a_r, self.a_t, self.a_b):
            _ah.addWidget(cb)
        self._rows = []
        self._sections = []          # (Ueberschrift-Label, [Widgets darunter])
        # Nach Themen gruppiert statt einer langen Zeilenliste: bei einem
        # Control mit vielen Eigenschaften suchte man sonst die gewuenschte
        # Zeile. Die Ueberschriften blenden sich mit ihrem Inhalt aus -- sonst
        # staende bei einem Label ein leeres "Werte" im Inspector.
        self._section("Allgemein")
        self._add("Name", self.name)
        self._add("Text", self.text)
        self._add("Gruppe", self.group)
        self._add("Platzhalter", self.placeholder)
        self._add("Tooltip", self.tooltip)
        self._add("Ausrichtung", self.align)
        self._add("", self.wrap)
        self._add("", self.passwort)
        self._add("", self.nur_lesen)
        self._add("Hoechstlaenge", self.maxlaenge)
        self._add("Zahlen", self.zahlen)
        self._add("", self.l_multi)
        self._add("", self.l_kaestchen)
        self._add("Layout-Art", self.ly_art)
        self._add("Spalten", self.ly_spalten)
        self._add("Abstand", self.ly_abstand)
        self._add("Rand", self.ly_rand)
        self._add("Quer", self.ly_ausr)
        self._add("", self.ly_dehnen)
        self._add("Layout", self.in_layout)
        self._add("Gewicht", self.in_gewicht)
        self._add("Bildmodus", self.bildmodus)
        self._add("", self.p_unbestimmt)
        self._add("", self.s_senkrecht)
        self._add("", self.ziehbar)
        self._add("", self.ablage)
        self._add("Panel", self.in_panel)
        self._add("Min-Breite", self.min_w)
        self._add("Min-Hoehe", self.min_h)
        self._add("Tab-Index", self.tab_index)
        self._add("Regeln", self.regeln)
        self._add("Bindung", self.bindung)
        self._add("Formular", self.formular)
        self._add("", self.ta_umbruch)
        self._add("Richtung", self.orient)
        self._add("Startfarbe", self.pick_btn)
        self._add("Startdatum", self.datum)

        self._section("Darstellung")
        self._add("Farbe", self.color_btn)
        self._add("Schriftgroesse", self.sfont)

        self._section("Position und Groesse")
        self._add("X", self.sx); self._add("Y", self.sy)
        self._add("Breite", self.sw); self._add("Hoehe", self.sh)
        self._add("Anker", self._anchor_box)

        self._section("Werte")
        self._add("Items (1/Zeile)", self.items)
        self._add("Auswahl", self.ssel)
        self._add("Min", self.vmin); self._add("Max", self.vmax); self._add("Wert", self.vval)

        self._section("Tabelle")
        self._add("Spalten (1/Zeile)", self.t_headers)
        self._add("Breiten (px)", self.t_widths)
        self._add("Zeilenhoehe", self.t_rowh)
        self._add("Kopfhoehe", self.t_headh)
        self._add("Feste Spalten", self.t_frozen)
        self._add("Bearbeitbar", self.t_edit)
        for _cb in (self.t_zebra, self.t_grid, self.t_filter, self.t_sort,
                    self.t_resize, self.t_reorder, self.t_multi):
            self._add("", _cb)

        self._section("Zustand")
        self._add("", self.enabled)
        self._add("", self.visible)
        self._add("", self.checked)

        self._section("Ereignisse")
        for ev in EVENTS:
            self._add(ev, self.ev_edits[ev])
        # Signale
        self.name.editingFinished.connect(self._apply)
        self.text.editingFinished.connect(self._apply)
        self.group.editingFinished.connect(self._apply)
        self.placeholder.editingFinished.connect(self._apply)
        self.tooltip.editingFinished.connect(self._apply)
        self.align.currentIndexChanged.connect(self._apply)
        self.zahlen.currentIndexChanged.connect(self._apply)
        self.maxlaenge.valueChanged.connect(self._apply)
        for _w in (self.wrap, self.passwort, self.nur_lesen, self.l_multi, self.l_kaestchen, self.ly_dehnen,
                   self.p_unbestimmt, self.s_senkrecht, self.ziehbar, self.ablage):
            _w.toggled.connect(self._apply)
        for _w in (self.ly_art, self.ly_ausr, self.in_layout, self.bildmodus, self.in_panel):
            _w.currentIndexChanged.connect(self._apply)
        for _w in (self.ly_spalten, self.ly_abstand, self.ly_rand, self.in_gewicht, self.min_w, self.min_h, self.tab_index):
            _w.valueChanged.connect(self._apply)
        self.regeln.editingFinished.connect(self._apply)
        self.bindung.editingFinished.connect(self._apply)
        self.formular.editingFinished.connect(self._apply)
        self.ta_umbruch.toggled.connect(self._apply)
        self.datum.editingFinished.connect(self._apply)
        self.orient.currentIndexChanged.connect(self._apply)
        self.ssel.valueChanged.connect(self._apply)
        self.t_headers.textChanged.connect(self._apply)
        self.t_widths.editingFinished.connect(self._apply)
        self.t_edit.editingFinished.connect(self._apply)
        for _w in (self.t_rowh, self.t_headh, self.t_frozen):
            _w.valueChanged.connect(self._apply)
        for _w in (self.t_zebra, self.t_grid, self.t_filter, self.t_sort,
                   self.t_resize, self.t_reorder, self.t_multi):
            _w.toggled.connect(self._apply)
        self.visible.toggled.connect(self._apply)
        for s in (self.sx, self.sy, self.sw, self.sh, self.vmin, self.vmax, self.vval):
            s.valueChanged.connect(self._apply)
        for _e in self.ev_edits.values():
            _e.editingFinished.connect(self._apply)
        self.items.textChanged.connect(self._apply)
        self.enabled.toggled.connect(self._apply)
        self.checked.toggled.connect(self._apply)
        self.sfont.valueChanged.connect(self._apply)
        for cb in (self.a_l, self.a_r, self.a_t, self.a_b):
            cb.toggled.connect(self._apply)
        self.set_control(None)

    def _update_color_btn(self):
        if self._c is not None:
            c = _col(self._c.color)
            self.color_btn.setText(f"#{self._c.color & 0xFFFFFF:06X}")
            fg = "#000" if (c.red() + c.green() + c.blue()) > 360 else "#fff"
            self.color_btn.setStyleSheet(
                f"background:{c.name()}; color:{fg}; border:1px solid #555;")

    def _update_pick_btn(self):
        """Der Knopf zeigt die Startfarbe des Farbwaehlers."""
        if self._c is None:
            return
        w = hex_zu_int(self._c.extra.get("color_value"))
        self.pick_btn.setText("(Vorgabe)" if w is None else f"#{w:06X}")
        c = _col(w if w is not None else 0xE84B4B)
        fg = "#000" if (c.red() + c.green() + c.blue()) > 360 else "#fff"
        self.pick_btn.setStyleSheet(f"background:{c.name()}; color:{fg}; border:1px solid #555;")

    def _pick_startfarbe(self):
        if self._c is None:
            return
        alt = hex_zu_int(self._c.extra.get("color_value"))
        col = QColorDialog.getColor(_col(alt if alt is not None else 0xE84B4B),
                                    self, "Startfarbe waehlen")
        if col.isValid():
            self._c.extra["color_value"] = \
                f"#{(col.red() << 16) | (col.green() << 8) | col.blue():06X}"
            self._update_pick_btn()
            self.changed.emit()

    def _pick_color(self):
        if self._c is None:
            return
        col = QColorDialog.getColor(_col(self._c.color), self, "Farbe waehlen")
        if col.isValid():
            self._c.color = (col.red() << 16) | (col.green() << 8) | col.blue()
            self._update_color_btn()
            self.changed.emit()

    def _section(self, titel: str):
        """Abschnitts-Ueberschrift, die die ganze Zeile einnimmt."""
        lbl = QLabel(titel.upper())
        f = lbl.font(); f.setBold(True); f.setPointSizeF(max(7.0, f.pointSizeF() - 1.0))
        lbl.setFont(f)
        lbl.setStyleSheet("color:#5fb6d6; margin-top:8px; border-bottom:1px solid #2e4356;")
        self._form.addRow(lbl)
        self._sections.append((lbl, []))

    def _add(self, label, widget):
        self._form.addRow(label, widget)
        self._rows.append((label, widget))
        if self._sections:
            self._sections[-1][1].append(widget)

    def _sync_sections(self):
        """Ueberschrift nur zeigen, wenn darunter ueberhaupt etwas steht.

        Geprueft wird `isHidden()`, NICHT `isVisible()`: letzteres ist auch
        dann False, wenn blosss das Fenster noch nicht angezeigt wurde -- beim
        Aufbau waeren so alle Ueberschriften verschwunden, obwohl ihre Zeilen
        da sind.
        """
        for lbl, widgets in self._sections:
            lbl.setVisible(any(not w.isHidden() for w in widgets))

    def _show(self, widget, on: bool):
        widget.setVisible(on)
        lbl = self._form.labelForField(widget)
        if lbl is not None:
            lbl.setVisible(on)

    @staticmethod
    def _zahlen(text: str) -> list:
        """Komma- oder leerzeichengetrennte Zahlen aus einem Eingabefeld.
        Alles, was keine Zahl ist, wird still verworfen -- ein Tippfehler
        soll nicht die ganze Zeile unbrauchbar machen."""
        out = []
        for teil in text.replace(";", ",").replace(" ", ",").split(","):
            teil = teil.strip()
            if teil.lstrip("-").isdigit():
                out.append(int(teil))
        return out

    def _tabelle_laden(self, c: Control):
        """Tabellen-Einstellungen aus `extra["table"]` in die Felder."""
        tj = (c.extra.get("table") or {}) if c.kind == "table" else {}
        self.t_headers.setPlainText("\n".join(str(s) for s in (tj.get("headers") or [])))
        self.t_widths.setText(", ".join(str(int(v)) for v in (tj.get("col_widths") or [])
                                        if isinstance(v, (int, float))))
        self.t_rowh.setValue(int(tj.get("row_h", 20)))
        self.t_headh.setValue(int(tj.get("header_h", 22)))
        self.t_frozen.setValue(int(tj.get("frozen", 0)))
        self.t_zebra.setChecked(bool(tj.get("zebra", False)))
        self.t_grid.setChecked(bool(tj.get("grid", True)))
        self.t_filter.setChecked(bool(tj.get("filter_row", False)))
        self.t_sort.setChecked(bool(tj.get("sortable", True)))
        self.t_resize.setChecked(bool(tj.get("resizable_cols", True)))
        self.t_reorder.setChecked(bool(tj.get("reorderable", False)))
        self.t_multi.setChecked(bool(tj.get("multi", False)))
        self.t_edit.setText(", ".join(str(i) for i, an in enumerate(tj.get("col_edit") or []) if an))

    def _tabelle_schreiben(self, c: Control):
        """Felder zurueck nach `extra["table"]`. Vorhandene Schluessel (z.B.
        `rows` aus einem geladenen Formular) bleiben erhalten -- der Designer
        stellt sie nicht dar, darf sie aber auch nicht wegwerfen."""
        if c.kind != "table":
            return
        tj = dict(c.extra.get("table") or {})
        tj["headers"] = [ln for ln in self.t_headers.toPlainText().splitlines() if ln != ""]
        br = self._zahlen(self.t_widths.text())
        if br:
            tj["col_widths"] = br
        else:
            tj.pop("col_widths", None)
        tj["row_h"] = self.t_rowh.value()
        tj["header_h"] = self.t_headh.value()
        tj["frozen"] = self.t_frozen.value()
        tj["zebra"] = self.t_zebra.isChecked()
        tj["grid"] = self.t_grid.isChecked()
        tj["filter_row"] = self.t_filter.isChecked()
        tj["sortable"] = self.t_sort.isChecked()
        tj["resizable_cols"] = self.t_resize.isChecked()
        tj["reorderable"] = self.t_reorder.isChecked()
        tj["multi"] = self.t_multi.isChecked()
        spalten = self._zahlen(self.t_edit.text())
        if spalten:
            n = max(spalten) + 1
            tj["col_edit"] = [i in spalten for i in range(n)]
        else:
            tj.pop("col_edit", None)
        tj.setdefault("rows", [])
        c.extra["table"] = tj

    def set_doc(self, doc: FormDoc | None):
        self._doc = doc

    def set_control(self, c: Control | None):
        self._c = c
        self._loading = True
        if c is None:
            for _, w in self._rows:
                self._show(w, False)
            self._sync_sections()
            self._loading = False
            return
        sp = palette_spec(c.kind)
        self.name.setText(c.name); self.text.setText(c.text)
        self.sx.setValue(c.x); self.sy.setValue(c.y); self.sw.setValue(c.w); self.sh.setValue(c.h)
        for ev, feld in self.ev_edits.items():
            feld.setText(getattr(c, ev))
        self.group.setText(c.group); self.placeholder.setText(c.placeholder)
        self.tooltip.setText(c.tooltip)
        ai = self.align.findData(c.align or "")
        self.align.setCurrentIndex(ai if ai >= 0 else 0)
        self.wrap.setChecked(c.wrap); self.passwort.setChecked(c.passwort)
        self.nur_lesen.setChecked(c.nur_lesen); self.maxlaenge.setValue(c.maxlaenge)
        zi = self.zahlen.findData(int(c.zahlen))
        self.zahlen.setCurrentIndex(zi if zi >= 0 else 0)
        lj0 = c.extra.get("list")
        lj: dict = lj0 if isinstance(lj0, dict) else {}
        self.l_multi.setChecked(bool(lj.get("multi")))
        self.l_kaestchen.setChecked(bool(lj.get("kaestchen")))
        yj0 = c.extra.get("layout")
        yj: dict = yj0 if isinstance(yj0, dict) else {}
        ai = self.ly_art.findData(str(yj.get("art") or "spalte"))
        self.ly_art.setCurrentIndex(ai if ai >= 0 else 1)
        self.ly_spalten.setValue(int(yj.get("spalten") or 2))
        self.ly_abstand.setValue(int(yj.get("abstand", 6)))
        self.ly_rand.setValue(int(yj.get("rand", 0)))
        self.ly_dehnen.setChecked(yj.get("dehnen", True) is not False)
        qi = self.ly_ausr.findData(int(yj.get("ausrichtung", 0)))
        self.ly_ausr.setCurrentIndex(qi if qi >= 0 else 0)
        self.in_layout.clear()
        self.in_layout.addItem("(keins)", "")
        if self._doc is not None:
            for l in self._doc.controls:
                if l.kind == "layout" and l is not c:
                    self.in_layout.addItem(l.name, l.name)
            eltern, gew = self._doc.layout_von(c)
            li = self.in_layout.findData(eltern.name if eltern else "")
            self.in_layout.setCurrentIndex(li if li >= 0 else 0)
            self.in_gewicht.setValue(gew)
        self.in_panel.clear()
        self.in_panel.addItem("(keins)", "")
        if self._doc is not None:
            for p in self._doc.controls:
                if p.kind == "panel" and p is not c:
                    self.in_panel.addItem(p.name, p.name)
            pe, _ = self._doc.layout_von(c, "panel")
            pi = self.in_panel.findData(pe.name if pe else "")
            self.in_panel.setCurrentIndex(pi if pi >= 0 else 0)
        mi = self.bildmodus.findData(str(c.extra.get("mode") or "strecken"))
        self.bildmodus.setCurrentIndex(mi if mi >= 0 else 0)
        self.p_unbestimmt.setChecked(bool(c.extra.get("indeterminate")))
        self.s_senkrecht.setChecked(bool(c.extra.get("vertical")))
        self.ziehbar.setChecked(bool(c.extra.get("draggable")))
        self.ablage.setChecked(bool(c.extra.get("drop_target")))
        self.min_w.setValue(int(c.extra.get("min_w") or 0))
        self.min_h.setValue(int(c.extra.get("min_h") or 0))
        self.tab_index.setValue(int(c.extra.get("tab_index") or 0))
        self.regeln.setText(regeln_text(c.extra.get("rules")))
        self.bindung.setText(str(c.extra.get("bind") or ""))
        self.formular.setText(str(c.extra.get("form") or ""))
        self.ta_umbruch.setChecked(bool(c.extra.get("wrap_text")))
        self.items.setPlainText("\n".join(c.items))
        self.ssel.setValue(c.sel)
        self._tabelle_laden(c)
        self.vmin.setValue(c.min); self.vmax.setValue(c.max); self.vval.setValue(c.value)
        self.enabled.setChecked(c.enabled); self.checked.setChecked(c.checked)
        self.visible.setChecked(c.visible)
        self.sfont.setValue(c.font_size); self._update_color_btn()
        self.datum.setText(str(c.extra.get("date") or ""))
        i = self.orient.findData((c.text or "h").lower())
        self.orient.setCurrentIndex(i if i >= 0 else 0)
        self._update_pick_btn()
        a = c.anchor or "lt"
        self.a_l.setChecked("l" in a); self.a_r.setChecked("r" in a)
        self.a_t.setChecked("t" in a); self.a_b.setChecked("b" in a)
        has_text = bool(sp and sp.has_text)
        has_items = bool(sp and sp.has_items)
        events = sp.events if sp else ()
        # Zahlenfeld und Drehknopf haben min/max/wert genauso wie der Schieber;
        # beim Trenner sind min/max die Grenzen, in die er sich schieben laesst.
        is_range = c.kind in ("slider", "progress", "spinner", "knob", "splitter")
        is_check = c.kind in ("checkbox", "radio", "toggle")
        for w in (self.name, self.color_btn, self.sfont, self.sx, self.sy,
                  self.sw, self.sh, self._anchor_box, self.enabled, self.visible):
            self._show(w, True)
        self._show(self.text, has_text)
        self._show(self.items, has_items)
        self._show(self.ssel, has_items)
        ist_tabelle = c.kind == "table"
        for _w in self._t_felder:
            self._show(_w, ist_tabelle)
        self._show(self.group, c.kind == "radio")
        self._show(self.placeholder, c.kind in ("textinput", "textarea"))
        self._show(self.tooltip, True)
        self._show(self.align, c.kind in ("label", "button", "textinput"))
        self._show(self.wrap, c.kind == "label")
        for _w in (self.passwort, self.nur_lesen, self.maxlaenge, self.zahlen):
            self._show(_w, c.kind == "textinput")
        self._show(self.l_multi, c.kind == "listbox")
        self._show(self.l_kaestchen, c.kind == "listbox")
        ist_layout = c.kind == "layout"
        for _w in (self.ly_art, self.ly_abstand, self.ly_rand, self.ly_dehnen, self.ly_ausr):
            self._show(_w, ist_layout)
        self._show(self.ly_spalten, ist_layout and self.ly_art.currentData() == "raster")
        hat_layouts = self.in_layout.count() > 1
        self._show(self.in_layout, hat_layouts)
        self._show(self.in_gewicht, hat_layouts)
        self._show(self.bildmodus, c.kind == "image")
        self._show(self.p_unbestimmt, c.kind == "progress")
        self._show(self.s_senkrecht, c.kind == "slider")
        self._show(self.ziehbar, not ist_layout)
        self._show(self.ablage, not ist_layout)
        self._show(self.in_panel, self.in_panel.count() > 1 and c.kind not in ("panel", "layout"))
        self._show(self.min_w, True)
        self._show(self.min_h, True)
        self._show(self.tab_index, c.kind not in ("label", "panel", "separator", "groupbox", "toolbar", "progress", "image", "canvas", "layout"))
        self._show(self.regeln, c.kind in ("textinput", "textarea", "dropdown", "checkbox", "toggle"))
        bindbar = c.kind in ("textinput", "textarea", "label", "checkbox", "toggle", "dropdown",
                             "slider", "spinner", "knob", "datepicker", "colorpicker")
        self._show(self.bindung, bindbar)
        self._show(self.formular, bindbar)
        self._show(self.ta_umbruch, c.kind == "textarea")
        self._show(self.orient, c.kind == "splitter")
        self._show(self.pick_btn, c.kind == "colorpicker")
        self._show(self.datum, c.kind == "datepicker")
        # Der Baum zeigt `items` als oberste Ebene, hat aber keinen
        # Auswahl-Index im Designer -- welcher Knoten gewaehlt ist, entscheidet
        # das Programm.
        if c.kind == "tree":
            self._show(self.ssel, False)
        # Nur die Ereignisse zeigen, die diese Art wirklich ausloest -- ein
        # angebotenes Ereignis, das nie feuert, laesst einen den Fehler im
        # eigenen Programm suchen.
        for ev, feld in self.ev_edits.items():
            self._show(feld, ev in events)
        self._show(self.vmin, is_range); self._show(self.vmax, is_range); self._show(self.vval, is_range)
        self._show(self.checked, is_check)
        self._sync_sections()
        self._loading = False

    def _apply(self):
        if self._loading or self._c is None:
            return
        c = self._c
        alte_handler = {ev: getattr(c, ev) for ev in EVENTS}
        old_name = c.name
        c.name = self.name.text().strip()
        c.text = self.text.text()
        c.x, c.y, c.w, c.h = self.sx.value(), self.sy.value(), self.sw.value(), self.sh.value()
        for ev, feld in self.ev_edits.items():
            setattr(c, ev, feld.text().strip())
        c.group = self.group.text().strip()
        c.placeholder = self.placeholder.text()
        c.tooltip = self.tooltip.text()
        c.align = self.align.currentData() or ""
        c.wrap = self.wrap.isChecked()
        c.passwort = self.passwort.isChecked()
        c.nur_lesen = self.nur_lesen.isChecked()
        c.maxlaenge = self.maxlaenge.value()
        c.zahlen = int(self.zahlen.currentData() or 0)
        if c.kind == "layout":
            yj = dict(c.extra.get("layout") or {}) if isinstance(c.extra.get("layout"), dict) else {}
            yj["art"] = self.ly_art.currentData() or "spalte"
            if yj["art"] == "raster":
                yj["spalten"] = self.ly_spalten.value()
            else:
                yj.pop("spalten", None)
            for schluessel, feld, standard in (("abstand", self.ly_abstand, 6), ("rand", self.ly_rand, 0)):
                if feld.value() != standard:
                    yj[schluessel] = feld.value()
                else:
                    yj.pop(schluessel, None)
            if self.ly_ausr.currentData():
                yj["ausrichtung"] = int(self.ly_ausr.currentData())
            else:
                yj.pop("ausrichtung", None)
            if self.ly_dehnen.isChecked():
                yj.pop("dehnen", None)
            else:
                yj["dehnen"] = False
            yj.setdefault("kinder", [])
            c.extra["layout"] = yj
            self._show(self.ly_spalten, yj["art"] == "raster")
        b = self.bindung.text().strip()
        if b:
            c.extra["bind"] = b
            if self.formular.text().strip():
                c.extra["form"] = self.formular.text().strip()
            else:
                c.extra.pop("form", None)
        else:
            c.extra.pop("bind", None); c.extra.pop("form", None)
        if c.kind == "textarea":
            if self.ta_umbruch.isChecked():
                c.extra["wrap_text"] = True
            else:
                c.extra.pop("wrap_text", None)
        for schluessel, feld in (("min_w", self.min_w), ("min_h", self.min_h), ("tab_index", self.tab_index)):
            if feld.value() > 0:
                c.extra[schluessel] = feld.value()
            else:
                c.extra.pop(schluessel, None)
        if c.kind in ("textinput", "textarea", "dropdown", "checkbox", "toggle"):
            rl = regeln_parsen(self.regeln.text())
            if rl:
                c.extra["rules"] = rl
            else:
                c.extra.pop("rules", None)
        for schluessel, feld in (("indeterminate", self.p_unbestimmt), ("vertical", self.s_senkrecht),
                                 ("draggable", self.ziehbar), ("drop_target", self.ablage)):
            if feld.isChecked():
                c.extra[schluessel] = True
            else:
                c.extra.pop(schluessel, None)
        if c.kind == "image":
            modus = self.bildmodus.currentData() or "strecken"
            if modus == "strecken":
                c.extra.pop("mode", None)
            else:
                c.extra["mode"] = modus
        if self._doc is not None and self.in_panel.count() > 1:
            ziel = self.in_panel.currentData() or ""
            eltern = next((p for p in self._doc.controls if p.kind == "panel" and p.name == ziel), None)
            if eltern is not self._doc.layout_von(c, "panel")[0]:
                self._doc.layout_zuordnen(c, eltern, 0, "panel")
        if self._doc is not None and self.in_layout.count() > 1:
            ziel = self.in_layout.currentData() or ""
            eltern = next((l for l in self._doc.controls if l.kind == "layout" and l.name == ziel), None)
            alt_eltern, alt_gew = self._doc.layout_von(c)
            if eltern is not alt_eltern or alt_gew != self.in_gewicht.value():
                self._doc.layout_zuordnen(c, eltern, self.in_gewicht.value())
        if c.kind == "listbox":
            lj = dict(c.extra.get("list") or {}) if isinstance(c.extra.get("list"), dict) else {}
            if self.l_multi.isChecked():
                lj["multi"] = True
            else:
                lj.pop("multi", None)
            if self.l_kaestchen.isChecked():
                lj["kaestchen"] = True
            else:
                lj.pop("kaestchen", None)
            if lj:
                c.extra["list"] = lj
            else:
                c.extra.pop("list", None)
        if c.kind == "splitter":
            # Der Trenner hat kein Text-Feld im Inspektor -- `text` traegt bei
            # ihm die Richtung, die aus dem Auswahlfeld kommt.
            c.text = self.orient.currentData() or "h"
        if c.kind == "datepicker":
            d = self.datum.text().strip()
            # Leer heisst HEUTE (so legt die Laufzeit einen frischen Waehler
            # an) -- dann darf gar kein Datum in der Datei stehen.
            if d:
                c.extra["date"] = d
            else:
                c.extra.pop("date", None)
        self._tabelle_schreiben(c)
        items = [ln for ln in self.items.toPlainText().splitlines() if ln != ""]
        c.items = items
        if c.kind == "dropdown" and items and c.sel < 0:
            c.sel = 0
        c.sel = min(self.ssel.value(), len(items) - 1) if items else min(c.sel, -1)
        c.min, c.max, c.value = self.vmin.value(), self.vmax.value(), self.vval.value()
        if c.max < c.min:                      # sonst clampt GUI_SET_VALUE ins Leere
            c.max = c.min
        c.value = min(max(c.value, c.min), c.max)
        c.enabled = self.enabled.isChecked()
        c.visible = self.visible.isChecked()
        c.checked = self.checked.isChecked()
        c.font_size = self.sfont.value()
        a = ("l" if self.a_l.isChecked() else "") + ("r" if self.a_r.isChecked() else "") \
            + ("t" if self.a_t.isChecked() else "") + ("b" if self.a_b.isChecked() else "")
        c.anchor = a or "lt"
        # Handler-Umbenennung melden, BEVOR `changed` das Code-Panel neu
        # befuellt: `doc.code` ist nach Namen geschluesselt, der alte Rumpf
        # blieb sonst als unerreichbare Leiche zurueck und der Export
        # emittierte fuer den neuen Namen nur ein `' TODO`.
        for ev, old in alte_handler.items():
            new = getattr(c, ev)
            if old and new and old != new:
                self.handler_renamed.emit(old, new)
        # Control umbenannt -> abgeleitete Handler-Namen ziehen mit. Das
        # Dokument entscheidet das (dort wohnen die Namensregeln); gemeldet
        # wird es, damit Code-Panel und Inspector den neuen Namen zeigen.
        if old_name != c.name:
            self.control_renamed.emit(c, old_name)
        self.changed.emit()


class _WindowInspector(QWidget):
    """Eigenschaften des Formulars selbst (wie Xojos Fenster-Inspector). Wird
    angezeigt, wenn KEIN Control selektiert ist."""
    changed = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc: FormDoc | None = None
        self._loading = False
        f = QFormLayout(self)
        self.title = QLineEdit()
        self.sw = QSpinBox(); self.sh = QSpinBox()
        self.minw = QSpinBox(); self.minh = QSpinBox()
        self.maxw = QSpinBox(); self.maxh = QSpinBox()
        for s in (self.sw, self.sh, self.minw, self.minh, self.maxw, self.maxh):
            s.setRange(0, 32000)
        self.sw.setMinimum(40); self.sh.setMinimum(40)
        self.movable = QCheckBox("beweglich")
        self.closable = QCheckBox("schliessbar")
        self.resizable = QCheckBox("groessenveraenderbar")
        self.visible = QCheckBox("sichtbar")
        self.theme = QComboBox()
        for name in FORM_THEMES:
            self.theme.addItem(name or "(Vorgabe)", name)
        # Standard- (Enter) und Abbrechen-Knopf (ESC): Auswahl unter den
        # Knoepfen des Formulars, gefuehrt als Widget-Index wie in der Datei.
        self.default_btn = QComboBox()
        self.cancel_btn = QComboBox()
        f.addRow("Titel", self.title)
        f.addRow("Thema", self.theme)
        f.addRow("Enter-Knopf", self.default_btn)
        f.addRow("ESC-Knopf", self.cancel_btn)
        f.addRow("Breite", self.sw); f.addRow("Hoehe", self.sh)
        f.addRow("Min. Breite", self.minw); f.addRow("Min. Hoehe", self.minh)
        f.addRow("Max. Breite", self.maxw); f.addRow("Max. Hoehe", self.maxh)
        f.addRow("", self.movable); f.addRow("", self.closable)
        f.addRow("", self.resizable); f.addRow("", self.visible)
        self.title.editingFinished.connect(self._apply)
        for s in (self.sw, self.sh, self.minw, self.minh, self.maxw, self.maxh):
            s.valueChanged.connect(self._apply)
        for c in (self.movable, self.closable, self.resizable, self.visible):
            c.toggled.connect(self._apply)
        self.theme.currentIndexChanged.connect(self._apply)
        self.default_btn.currentIndexChanged.connect(self._apply)
        self.cancel_btn.currentIndexChanged.connect(self._apply)

    def _knopf_listen(self, doc: FormDoc) -> None:
        for combo, feld in ((self.default_btn, "default_button"), (self.cancel_btn, "cancel_button")):
            combo.clear()
            combo.addItem("(keiner)", -1)
            for i, c in enumerate(doc.controls):
                if c.kind == "button":
                    combo.addItem(c.name or c.text or f"Knopf {i}", i)
            k = combo.findData(getattr(doc, feld))
            combo.setCurrentIndex(k if k >= 0 else 0)

    def set_doc(self, doc: FormDoc):
        self.doc = doc
        self._loading = True
        if doc is not None:
            self._knopf_listen(doc)
            self.title.setText(doc.title)
            self.sw.setValue(doc.w); self.sh.setValue(doc.h)
            self.minw.setValue(doc.min_w); self.minh.setValue(doc.min_h)
            self.maxw.setValue(doc.max_w); self.maxh.setValue(doc.max_h)
            self.movable.setChecked(doc.movable); self.closable.setChecked(doc.closable)
            self.resizable.setChecked(doc.resizable); self.visible.setChecked(doc.visible)
            i = self.theme.findData(doc.theme or "")
            self.theme.setCurrentIndex(i if i >= 0 else 0)
        self._loading = False

    def _apply(self):
        if self._loading or self.doc is None:
            return
        d = self.doc
        d.title = self.title.text()
        d.w, d.h = self.sw.value(), self.sh.value()
        d.min_w, d.min_h = self.minw.value(), self.minh.value()
        d.max_w, d.max_h = self.maxw.value(), self.maxh.value()
        # min > max ist widerspruechlich: `_clamp_fw`/`_clamp_fh` wenden erst
        # min, dann max an -- das Formular landete unter seinem eigenen Minimum.
        if d.max_w and d.max_w < d.min_w:
            d.max_w = d.min_w; self.maxw.setValue(d.max_w)
        if d.max_h and d.max_h < d.min_h:
            d.max_h = d.min_h; self.maxh.setValue(d.max_h)
        d.movable, d.closable = self.movable.isChecked(), self.closable.isChecked()
        d.resizable, d.visible = self.resizable.isChecked(), self.visible.isChecked()
        d.theme = self.theme.currentData() or ""
        d.default_button = int(self.default_btn.currentData() if self.default_btn.currentData() is not None else -1)
        d.cancel_button = int(self.cancel_btn.currentData() if self.cancel_btn.currentData() is not None else -1)
        self.changed.emit()


class _CodePanel(QWidget):
    """Integrierter Code-Editor: pro Event-Handler ein Drachenhauch-Body. Die
    Combo listet die Handler des Formulars, der Editor zeigt/aendert den Body
    des gewaehlten (gespeichert in `doc.code[name]`)."""
    edited = Signal()             # Body geaendert
    session_started = Signal()    # ein Handler wurde frisch geladen (Undo-Basis)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.doc: FormDoc | None = None
        self.current: str | None = None
        self._loading = False

        lay = QVBoxLayout(self)
        top = QHBoxLayout()
        top.addWidget(QLabel("Handler:"))
        self.combo = QComboBox(); self.combo.setMinimumWidth(200)
        top.addWidget(self.combo, 1)
        lay.addLayout(top)
        self.sig = QLabel("")
        self.sig.setStyleSheet("color:#5fb6d6;")
        lay.addWidget(self.sig)
        self.editor = QPlainTextEdit()
        self.editor.setLineWrapMode(QPlainTextEdit.LineWrapMode.NoWrap)
        f = QFont("Consolas"); f.setStyleHint(QFont.StyleHint.Monospace); f.setPointSize(10)
        self.editor.setFont(f)
        self.editor.setTabStopDistance(4 * self.editor.fontMetrics().horizontalAdvance(" "))
        self._hl = None
        if DHHighlighter is not None:
            self._hl = DHHighlighter(self.editor.document())
        lay.addWidget(self.editor, 1)

        self.combo.currentIndexChanged.connect(self._on_combo)
        self.editor.textChanged.connect(self._on_text)
        self._show_empty()

    def set_doc(self, doc: FormDoc):
        # `current` NICHT zuruecksetzen: nach jedem Undo tauscht die Canvas das
        # Dokument, und `refresh()` waehlte dann wieder den ERSTEN Handler --
        # beim Code-Schreiben wurde man staendig in einen fremden geworfen.
        self.doc = doc
        self.refresh()

    def refresh(self):
        """Combo neu aus den Handler-Namen des Formulars fuellen (Auswahl halten)."""
        names = self.doc.handler_names() if self.doc else []
        self._loading = True
        self.combo.clear()
        self.combo.addItems(names)
        self._loading = False
        if self.current in names:
            self.show_handler(self.current)
        elif names:
            self.show_handler(names[0])
        else:
            self._show_empty()

    def show_handler(self, name: str):
        if not self.doc or name not in self.doc.handler_names():
            return
        self._loading = True
        idx = self.combo.findText(name)
        if idx >= 0:
            self.combo.setCurrentIndex(idx)
        self.current = name
        self.sig.setText(f"SUB {name}()      …      END SUB")
        self.editor.setReadOnly(False)
        self.editor.setPlainText(self.doc.code.get(name, ""))
        self._loading = False
        self.session_started.emit()

    def focus_editor(self):
        self.editor.setFocus()

    def detach_highlighter(self):
        """Highlighter vom Dokument loesen -- verhindert einen Use-after-free
        beim Interpreter-Shutdown (QSyntaxHighlighter ueberlebt sonst die
        Teardown-Race von Dokument + QApplication)."""
        if self._hl is not None:
            self._hl.setDocument(None)
            self._hl = None

    def _show_empty(self):
        self._loading = True
        self.current = None
        self.editor.clear()
        self.editor.setReadOnly(True)
        self.sig.setText("(kein Handler — Doppelklick auf ein Control)")
        self._loading = False

    def _on_combo(self):
        if self._loading or not self.doc:
            return
        name = self.combo.currentText()
        if name:
            self.show_handler(name)

    def _on_text(self):
        if self._loading or not self.doc or self.current is None:
            return
        self.doc.code[self.current] = self.editor.toPlainText()
        self.edited.emit()


class _OpenForm:
    """Ein im Designer geoeffnetes Formular mit eigenem Zustand: Dokument, Pfad,
    Undo-Historie und Dirty-Flag. (Undo ist pro Formular -- nicht uebergreifend.)"""
    def __init__(self, doc: FormDoc, path: Path | None = None):
        self.doc = doc
        self.path = path
        self.history = History()
        self.dirty = False
        # Stand beim letzten Speichern/Laden -- damit ein Undo zurueck auf die
        # Datei den Stern wieder loescht, statt ihn fuer immer stehen zu lassen.
        self.saved: dict | None = doc.to_dict() if path is not None else None


class FormDesigner(QMainWindow):
    def __init__(self, project_root: Path):
        super().__init__()
        self.project_root = Path(project_root)
        # Multi-Form-Zustand
        self.forms: list[_OpenForm] = []
        self.active_index: int = -1
        self.project = FormProject()
        self.project_path: Path | None = None
        self._main_form: _OpenForm | None = None   # Startformular (Objekt, nicht Pfad)
        self.unresolved: list[str] = []            # beim Laden fehlende .dhform
        self._suppress_row = False
        self.setWindowTitle("Drachenhauch Form-Designer")
        self.resize(1500, 950)   # Groesse fuer den Fall, dass jemand
                                 # aus dem Vollbild zurueckschaltet
        QShortcut(QKeySequence("F11"), self, activated=self._toggle_fullscreen)

        self.canvas = _Canvas()
        scroll = QScrollArea(); scroll.setWidget(self.canvas); scroll.setWidgetResizable(False)
        self.setCentralWidget(scroll)

        # Formular-Liste (Multi-Form-Navigator)
        self.form_list = QListWidget()
        self.form_list.currentRowChanged.connect(self._on_form_row)
        self._dock("Formulare", self.form_list, Qt.DockWidgetArea.LeftDockWidgetArea)

        # Palette (grafische Icons + Drag&Drop)
        self.palette = _PaletteList()
        for sp in PALETTE:
            it = QListWidgetItem(_palette_icon(sp.kind), sp.label)
            it.setData(Qt.ItemDataRole.UserRole, sp.kind)
            it.setToolTip(f"{sp.label} — ziehen oder klicken+platzieren")
            self.palette.addItem(it)
        self.palette.itemClicked.connect(self._arm_place)
        pdock = self._dock("Controls", self.palette, Qt.DockWidgetArea.LeftDockWidgetArea)
        pdock.setMinimumWidth(226)   # zwei Rasterspalten

        # Inspector -- Stack: Control-Eigenschaften ODER Fenster-Eigenschaften
        self.inspector = _Inspector()
        self.win_inspector = _WindowInspector()
        self._insp_stack = QStackedWidget()
        self._insp_stack.addWidget(self.inspector)       # 0: Control selektiert
        self._insp_stack.addWidget(self.win_inspector)   # 1: Fenster (nichts selektiert)
        self._dock("Inspector", self._insp_stack, Qt.DockWidgetArea.RightDockWidgetArea)

        # Code-Editor (integriert)
        self.code_panel = _CodePanel()
        self.code_dock = self._dock("Code", self.code_panel, Qt.DockWidgetArea.BottomDockWidgetArea)

        # Undo/Redo-Edit-Sessions (transient, pro Selektion/Handler)
        self.canvas.commit_history = self._commit_history
        self._insp_baseline: dict | None = None
        self._insp_dirty = False
        self._code_baseline: dict | None = None
        self._code_dirty = False
        self._clip: dict | None = None            # Clipboard fuer Kopieren/Einfuegen
        self._proc = None                         # laufender F5-Testlauf
        self._run_dir: Path | None = None         # dessen Temp-Verzeichnis

        self.canvas.selection_changed.connect(self.inspector.set_control)
        self.canvas.selection_changed.connect(self._on_selection_changed)
        self.canvas.selection_changed.connect(self._update_status)
        self.canvas.doc_changed.connect(self._on_doc_changed)
        self.canvas.doc_replaced.connect(self.code_panel.set_doc)
        self.canvas.doc_replaced.connect(self.inspector.set_doc)
        self.inspector.set_doc(self.canvas.doc)
        self.canvas.form_resized.connect(self._on_form_resized)
        self.canvas.handler_requested.connect(self._open_handler)
        self.canvas.context_menu.connect(self._show_context_menu)
        self.inspector.changed.connect(self._on_inspector_changed)
        self.inspector.handler_renamed.connect(self._on_handler_renamed)
        self.inspector.control_renamed.connect(self._on_control_renamed)
        self.win_inspector.changed.connect(self._on_window_changed)
        self.code_panel.session_started.connect(self._on_code_session)
        self.code_panel.edited.connect(self._on_code_edited)

        self._status = QLabel("")                 # Live-Anzeige der Selektion
        self.statusBar().addPermanentWidget(self._status)
        self._zoom_lbl = QLabel("100 %")
        self.statusBar().addPermanentWidget(self._zoom_lbl)
        self.canvas.zoom_changed.connect(
            lambda z: self._zoom_lbl.setText(f"{round(z * 100)} %"))

        self._build_menu()
        # NACH dem Menue: die Leiste benutzt dieselben QActions fuer
        # Rueckgaengig/Wiederholen, damit sie automatisch mit ausgrauen.
        self._build_main_toolbar()
        self.addToolBarBreak()             # Anordnen-Befehle in eigene Zeile
        self._build_arrange_toolbar()
        self._add_open_form(FormDoc())     # ein leeres Start-Formular
        # Zusaetzlich zum closeEvent: laeuft der Designer in-process (dhrun.py
        # --form) und die Host-App ruft `QApplication.quit()`, gibt es kein
        # closeEvent -- der lebende DHHighlighter segfaultet dann im Teardown.
        app = QApplication.instance()
        if app is not None:
            app.aboutToQuit.connect(self.code_panel.detach_highlighter)

    # -- Ungespeichert-Schutz ------------------------------------------------
    def _confirm_dirty(self) -> bool:
        """True = fortfahren. Fragt fuer ALLE ungespeicherten Formulare, nicht
        nur das aktive -- vorher gingen beim Schliessen des Fensters oder beim
        Oeffnen eines Projekts saemtliche Aenderungen kommentarlos verloren
        (samt ihrer Undo-Historien)."""
        dirty = [of for of in self.forms if of.dirty]
        if not dirty:
            return True
        names = ", ".join(of.path.name if of.path else "(unbenannt)" for of in dirty)
        r = QMessageBox.question(
            self, "Ungespeicherte Aenderungen",
            f"Nicht gespeichert: {names}\n\nAenderungen speichern?",
            QMessageBox.StandardButton.Save | QMessageBox.StandardButton.Discard
            | QMessageBox.StandardButton.Cancel)
        if r == QMessageBox.StandardButton.Cancel:
            return False
        if r == QMessageBox.StandardButton.Save:
            return self.save_all()
        return True

    def closeEvent(self, ev):
        if not self._confirm_dirty():
            ev.ignore()
            return
        self.code_panel.detach_highlighter()
        self._stop_run()          # Testlauf + Temp-Ordner nicht ueberleben lassen
        super().closeEvent(ev)

    def _dock(self, title, widget, area):
        d = QDockWidget(title, self)
        d.setWidget(widget)
        self.addDockWidget(area, d)
        return d

    def _build_menu(self):
        m = self.menuBar().addMenu("&Datei")
        def act(name, shortcut, fn, menu=m):
            a = QAction(name, self)
            if shortcut:
                a.setShortcut(QKeySequence(shortcut))
            a.triggered.connect(fn); menu.addAction(a); return a
        act("Neues Formular", "Ctrl+N", self.new_form)
        act("Formular oeffnen...", "Ctrl+O", self.open_form)
        act("Formular schliessen", "Ctrl+W", self.close_form)
        m.addSeparator()
        act("Speichern", "Ctrl+S", self.save_form)
        act("Speichern unter...", "Ctrl+Shift+S", self.save_form_as)
        act("Alle speichern", "Ctrl+Alt+S", self.save_all)
        m.addSeparator()
        act("Projekt oeffnen...", None, self.open_project)
        act("Projekt speichern...", None, self.save_project)
        act("Als Startformular setzen", None, self.set_main_form)
        m.addSeparator()
        act("GB-Code exportieren...", None, self.export_gb_code)
        act("Ausfuehren (dhrt)", "F5", self.run_form)

        e = self.menuBar().addMenu("&Bearbeiten")
        self.act_undo = act("Rueckgaengig", "Ctrl+Z", self.undo, menu=e)
        self.act_redo = act("Wiederholen", "Ctrl+Y", self.redo, menu=e)
        # Zweites, uebliches Redo-Kuerzel (Strg+Umschalt+Z)
        redo2 = QAction(self)
        redo2.setShortcut(QKeySequence("Ctrl+Shift+Z"))
        redo2.triggered.connect(self.redo)
        self.addAction(redo2)
        e.addSeparator()
        # Edit-Ops: Shortcut NUR wenn die Canvas fokussiert ist -- sonst wuerde
        # z.B. Strg+C/V die Textbearbeitung im Code-/Inspector-Panel kapern.
        def edit_act(label, key, fn):
            a = QAction(label, self)
            a.setShortcut(QKeySequence(key))
            a.setShortcutContext(Qt.ShortcutContext.WidgetWithChildrenShortcut)
            a.triggered.connect(fn)
            e.addAction(a); self.canvas.addAction(a)
            return a
        edit_act("Duplizieren", "Ctrl+D", self.duplicate_selected)
        edit_act("Kopieren", "Ctrl+C", self.copy_selected)
        edit_act("Einfuegen", "Ctrl+V", self.paste_clip)
        act("Loeschen", None, self.delete_selected, menu=e)   # Del macht die Canvas
        e.addSeparator()
        edit_act("Nach vorne", "Ctrl+]", self.raise_selected)
        edit_act("Nach hinten", "Ctrl+[", self.lower_selected)

        v = self.menuBar().addMenu("&Ansicht")
        self.act_snap = QAction("Am Raster ausrichten", self, checkable=True)
        self.act_snap.setChecked(self.canvas.snap_grid)
        self.act_snap.setShortcut(QKeySequence("Ctrl+G"))
        self.act_snap.toggled.connect(self._toggle_snap)
        v.addAction(self.act_snap)
        self.act_rulers = QAction("Lineale", self, checkable=True)
        self.act_rulers.setChecked(self.canvas.show_rulers)
        self.act_rulers.toggled.connect(self._toggle_rulers)
        v.addAction(self.act_rulers)
        v.addSeparator()
        act("Vergroessern", "Ctrl+=", lambda: self.canvas.set_zoom(self.canvas.zoom * 1.25), menu=v)
        act("Verkleinern", "Ctrl+-", lambda: self.canvas.set_zoom(self.canvas.zoom / 1.25), menu=v)
        act("Zoom 100%", "Ctrl+0", lambda: self.canvas.set_zoom(1.0), menu=v)

        # Anordnen (Mehrfach-Auswahl): Ausrichten / Gleiche Groesse / Verteilen
        ar = self.menuBar().addMenu("&Anordnen")
        for label, edge in (("Linksbuendig", "left"), ("Rechtsbuendig", "right"),
                            ("Oben buendig", "top"), ("Unten buendig", "bottom"),
                            ("Zentriert horizontal", "center_h"),
                            ("Zentriert vertikal", "center_v")):
            act(label, None, (lambda e: lambda: self._align(e))(edge), menu=ar)
        ar.addSeparator()
        act("Gleiche Breite", None, lambda: self._same_size("w"), menu=ar)
        act("Gleiche Hoehe", None, lambda: self._same_size("h"), menu=ar)
        act("Gleiche Groesse", None, lambda: self._same_size("both"), menu=ar)
        ar.addSeparator()
        act("Horizontal verteilen", None, lambda: self._distribute("h"), menu=ar)
        act("Vertikal verteilen", None, lambda: self._distribute("v"), menu=ar)

    def _build_main_toolbar(self):
        """Die staendig gebrauchten Befehle als Leiste. Sie lagen bisher nur
        im Menue -- besonders "Ausfuehren" war ohne F5 nicht zu finden."""
        tb = QToolBar("Datei")
        tb.setIconSize(QSize(26, 26))
        tb.setToolButtonStyle(Qt.ToolButtonStyle.ToolButtonIconOnly)
        self.addToolBar(tb)
        self.main_bar = tb

        def add(kind, tip, fn, shortcut=None):
            a = QAction(_tool_icon(kind), tip, self)
            a.setToolTip(f"{tip} ({shortcut})" if shortcut else tip)
            a.triggered.connect(fn)
            tb.addAction(a)
            return a

        add("new", "Neues Formular", self.new_form, "Strg+N")
        add("open", "Formular oeffnen", self.open_form, "Strg+O")
        add("save", "Speichern", self.save_form, "Strg+S")
        tb.addSeparator()
        # Dieselben QActions wie im Menue -- so folgen sie automatisch dem
        # Aktiv/Inaktiv-Zustand, den _refresh_history_actions setzt.
        self.act_undo.setIcon(_tool_icon("undo"))
        self.act_redo.setIcon(_tool_icon("redo"))
        tb.addAction(self.act_undo)
        tb.addAction(self.act_redo)
        tb.addSeparator()
        add("code", "Code-Fenster zeigen", self._show_code_dock)
        run = add("run", "Ausfuehren", self.run_form, "F5")
        # Das gemeinsame Thema faerbt genau diesen Objektnamen gruen -- so
        # sticht der wichtigste Knopf hervor, ohne eine eigene Regel.
        btn = tb.widgetForAction(run)
        if btn is not None:
            btn.setObjectName("RunButton")

    def _show_code_dock(self):
        self.code_dock.show()
        self.code_dock.raise_()

    def _build_arrange_toolbar(self):
        """Anordnen-Befehle als grafische Toolbar (Xojo-Stil), schneller als das Menue."""
        tb = QToolBar("Anordnen")
        tb.setIconSize(QSize(26, 26))
        self.addToolBar(tb)
        self.arrange_bar = tb
        # Waagerecht und senkrecht getrennt: sechs Ausrichtungs-Symbole in
        # einer Reihe liest man sonst als eine Wand, obwohl es zwei Achsen
        # sind. Die Zahl ist die Mindest-Auswahl -- Verteilen braucht drei.
        groups = [
            [("left", "Linksbuendig", lambda: self._align("left"), 2),
             ("center_h", "Zentriert waagerecht", lambda: self._align("center_h"), 2),
             ("right", "Rechtsbuendig", lambda: self._align("right"), 2)],
            [("top", "Oben buendig", lambda: self._align("top"), 2),
             ("center_v", "Zentriert senkrecht", lambda: self._align("center_v"), 2),
             ("bottom", "Unten buendig", lambda: self._align("bottom"), 2)],
            [("same_w", "Gleiche Breite", lambda: self._same_size("w"), 2),
             ("same_h", "Gleiche Hoehe", lambda: self._same_size("h"), 2),
             ("same_both", "Gleiche Groesse", lambda: self._same_size("both"), 2)],
            [("dist_h", "Waagerecht verteilen", lambda: self._distribute("h"), 3),
             ("dist_v", "Senkrecht verteilen", lambda: self._distribute("v"), 3)],
        ]
        self._arrange_actions = []         # (QAction, Mindest-Auswahl)
        for gi, group in enumerate(groups):
            if gi:
                tb.addSeparator()
            for kind, tip, fn, min_n in group:
                a = QAction(_arrange_icon(kind, 26), tip, self)
                a.setToolTip(f"{tip} (ab {min_n} Controls)")
                a.triggered.connect(fn)
                tb.addAction(a)
                self._arrange_actions.append((a, min_n))
        self._refresh_arrange_actions()

    def _refresh_arrange_actions(self):
        """Anordnen-Befehle ausgrauen, solange zu wenig ausgewaehlt ist.

        Vorher sahen alle elf Knoepfe immer klickbar aus und antworteten dann
        mit einer Zeile in der Statusleiste -- ein grauer Knopf sagt dasselbe,
        bevor man klickt."""
        n = len(self.canvas.selection)
        for a, min_n in getattr(self, "_arrange_actions", []):
            a.setEnabled(n >= min_n)

    def _toggle_snap(self, on: bool):
        self.canvas.snap_grid = on
        self.canvas.update()

    def _toggle_rulers(self, on: bool):
        self.canvas.show_rulers = on
        self.canvas.update()

    # -- Anordnen (auf die Mehrfach-Auswahl, je mit Undo-Checkpoint) --
    def _arrange(self, op, min_n: int = 2):
        sel = list(self.canvas.selection)
        if len(sel) < min_n:
            self.statusBar().showMessage(f"Mindestens {min_n} Controls auswaehlen.", 2500)
            return
        pre = self.canvas.doc.to_dict()
        op(sel)
        if pre == self.canvas.doc.to_dict():
            return                       # nichts geaendert -> kein leerer Undo-Schritt
        self._commit_history(pre)
        self.canvas.update()
        self._mark_dirty()

    def _align(self, edge: str):
        self._arrange(lambda s: self.canvas.doc.align(s, edge))

    def _same_size(self, dim: str):
        # Referenz = primaeres (zuletzt geklicktes) Control
        self._arrange(lambda s: self.canvas.doc.same_size(s, self.canvas.selected, dim))

    def _distribute(self, axis: str):
        self._arrange(lambda s: self.canvas.doc.distribute(s, axis), min_n=3)

    # -- Aktive Form + Navigator ------------------------------------------
    @property
    def active(self) -> _OpenForm:
        return self.forms[self.active_index]

    @property
    def history(self) -> History:
        return self.active.history

    @property
    def path(self) -> Path | None:
        return self.active.path if 0 <= self.active_index < len(self.forms) else None

    @path.setter
    def path(self, v):
        self.active.path = v

    def _add_open_form(self, doc: FormDoc, path: Path | None = None, switch: bool = True):
        of = _OpenForm(doc, path)
        self.forms.append(of)
        if switch:
            self._switch_to(len(self.forms) - 1)
        else:
            self._refresh_form_list()
        return of

    def _switch_to(self, index: int):
        if not (0 <= index < len(self.forms)):
            return
        self.active_index = index
        self._insp_baseline = None; self._insp_dirty = False
        self._code_baseline = None; self._code_dirty = False
        self.canvas.set_doc(self.active.doc)   # doc_replaced -> Code-Panel
        self._refresh_history_actions()
        self._refresh_form_list()
        self._update_title()

    def _set_active_doc(self, doc: FormDoc):
        """Aktives Dokument ersetzen (Undo/Redo) -- haelt forms + canvas synchron."""
        self.active.doc = doc
        self.canvas.set_doc(doc)

    def _refresh_form_list(self):
        self._suppress_row = True
        self.form_list.clear()
        for of in self.forms:
            title = of.doc.title or "(Formular)"
            star = " *" if of.dirty else ""
            # Primaer das gemerkte Objekt, sonst der Manifest-Pfad. Der reine
            # String-Vergleich verfehlte ein von Hand mit Backslashes
            # geschriebenes Manifest und liess die Krone verschwinden.
            crown = "★ " if (of is self._main_form or
                             (self._main_form is None and of.path and self.project.main
                              and self._rel(of.path) == self.project.main)) else ""
            self.form_list.addItem(f"{crown}{title}{star}")
        if 0 <= self.active_index < len(self.forms):
            self.form_list.setCurrentRow(self.active_index)
        self._suppress_row = False

    def _on_form_row(self, row: int):
        if self._suppress_row or row < 0 or row == self.active_index:
            return
        self._switch_to(row)

    def _rel(self, p: Path) -> str:
        """Pfad relativ zum Projekt-Verzeichnis (fuer das `.dhproj`-Manifest).

        `os.path.relpath` statt `Path.relative_to`: letzteres kann nicht nach
        oben laufen, und der Fallback schrieb dann den BLOSSEN DATEINAMEN ins
        Manifest -- eine Form in `../shared/` war beim naechsten Oeffnen des
        Projekts spurlos verschwunden. Auf einem anderen Laufwerk (relpath
        wirft) bleibt der Absolutpfad stehen, der wenigstens auffindbar ist."""
        if self.project_path is not None:
            try:
                return Path(os.path.relpath(Path(p), self.project_path.parent)).as_posix()
            except ValueError:
                return Path(p).as_posix()
        return Path(p).name

    # -- Undo/Redo --
    def _commit_history(self, snapshot: dict):
        """Vom Canvas gerufen (Platzieren/Loeschen/Drag/Resize): Checkpoint setzen."""
        self.history.push(snapshot)
        self._refresh_history_actions()

    def _on_selection_changed(self, c):
        """Neue Selektion -> Inspector-Stack umschalten (Control vs. Fenster) +
        Basis fuer eine evtl. folgende Inspector-Edit-Session."""
        self._insp_baseline = self.canvas.doc.to_dict()
        self._insp_dirty = False
        self._refresh_arrange_actions()
        if c is None:                       # nichts selektiert -> Fenster-Inspector
            self.win_inspector.set_doc(self.canvas.doc)
            self._insp_stack.setCurrentWidget(self.win_inspector)
        else:
            self._insp_stack.setCurrentWidget(self.inspector)

    def _on_window_changed(self):
        """Fenster-Inspector-Aenderung: Edits einer Sitzung = EIN Undo-Schritt."""
        if not self._insp_dirty and self._insp_baseline is not None:
            self.history.push(self._insp_baseline)
            self._insp_dirty = True
            self._refresh_history_actions()
        self.canvas._resize_to_doc()
        self.canvas.update()
        self._refresh_form_list()           # Titel evtl. geaendert
        self._mark_dirty()

    def _on_form_resized(self):
        """Formular per Griff vergroessert/verkleinert -> Fenster-Inspector live."""
        self.win_inspector.set_doc(self.canvas.doc)
        self._mark_dirty()

    def _on_handler_renamed(self, old: str, new: str):
        """Handler im Inspector umbenannt -> den Code-Rumpf mitnehmen. Sonst
        blieb er unter dem alten Schluessel liegen: das Code-Panel zeigte
        schlagartig leer und der Export erzeugte nur ein `' TODO`."""
        doc = self.canvas.doc
        if old not in doc.code:
            return
        if old in doc.handler_names():
            return                       # ein anderes Control nutzt ihn noch
        if doc.code.get(new):
            return                       # dort steht schon Code -- nicht ueberschreiben
        doc.code[new] = doc.code.pop(old)

    def _toggle_fullscreen(self) -> None:
        """F11: echtes Vollbild an/aus. Zurueck geht es auf MAXIMIERT, nicht
        auf die alte Fenstergroesse -- sonst schrumpft der Designer beim
        Verlassen des Vollbilds auf ein kleines Fenster mitten im Bild."""
        if self.isFullScreen():
            self.showMaximized()
        else:
            self.showFullScreen()

    def _on_control_renamed(self, c, old_name: str):
        """Control umbenannt: abgeleitete Handler mitnehmen und die Anzeige
        nachziehen. Ohne das blieb im Code-Fenster der alte Name stehen --
        man aenderte den Namen und nichts passierte."""
        if not self.canvas.doc.rename_control_handlers(c, old_name):
            return
        self.code_panel.refresh()
        # Inspector neu befuellen, damit die Ereignis-Zeile den neuen Namen
        # zeigt (sie haelt den alten sonst bis zum naechsten Auswaehlen).
        self.inspector.set_control(c)

    def _on_inspector_changed(self):
        """Inspector-Aenderung. Alle Edits einer Selektion = EIN Undo-Schritt."""
        if not self._insp_dirty and self._insp_baseline is not None:
            self.history.push(self._insp_baseline)
            self._insp_dirty = True
            self._refresh_history_actions()
        self.canvas.update()
        self.code_panel.refresh()      # evtl. umbenannte Handler in die Combo uebernehmen
        self._mark_dirty()

    def _refresh_history_actions(self):
        self.act_undo.setEnabled(self.history.can_undo)
        self.act_redo.setEnabled(self.history.can_redo)

    def undo(self):
        if not self.history.can_undo:
            return
        prev = self.history.undo(self.canvas.doc.to_dict())
        self._set_active_doc(FormDoc.from_dict(prev))
        self._refresh_history_actions()
        self._resync_dirty()

    def redo(self):
        if not self.history.can_redo:
            return
        nxt = self.history.redo(self.canvas.doc.to_dict())
        self._set_active_doc(FormDoc.from_dict(nxt))
        self._refresh_history_actions()
        self._resync_dirty()

    # -- Edit-Ops (Control-bezogen, je mit Undo-Checkpoint) --
    def _control_op(self, mutate, select=None):
        """`mutate(doc, c)` mit Undo-Checkpoint ausfuehren. `select`: None = nichts
        aendern, 'result' = Rueckgabe selektieren, 'none' = abwaehlen."""
        c = self.canvas.selected
        if c is None:
            return
        pre = self.canvas.doc.to_dict()
        res = mutate(self.canvas.doc, c)
        if pre == self.canvas.doc.to_dict():
            return                       # z.B. "nach vorne" auf dem vordersten
        self._commit_history(pre)
        if select == "result":
            self.canvas._select(res)
        elif select == "none":
            self.canvas._select(None)
        self.canvas.update()
        self._mark_dirty()

    def duplicate_selected(self):
        """Ganze Auswahl duplizieren -- Loeschen/Ziehen/Nudge bedienen sie
        ebenfalls, Strg+D erwischte vorher nur das primaere Control."""
        sel = list(self.canvas.selection)
        if not sel:
            return
        pre = self.canvas.doc.to_dict()
        copies = [self.canvas.doc.duplicate(c) for c in sel]
        self._commit_history(pre)
        self.canvas._select_many(copies)
        self.canvas.update()
        self._mark_dirty()

    def delete_selected(self):
        sel = list(self.canvas.selection)
        if not sel:
            return
        pre = self.canvas.doc.to_dict()
        for c in sel:
            self.canvas.doc.remove(c)
        self._commit_history(pre)
        self.canvas._select(None)
        self.canvas.update()
        self._mark_dirty()

    def raise_selected(self):
        self._control_op(lambda d, c: d.to_front(c))

    def lower_selected(self):
        self._control_op(lambda d, c: d.to_back(c))

    def copy_selected(self):
        sel = list(self.canvas.selection)
        if not sel:
            return
        self._clip = [c.to_dict() for c in sel]     # ganze Auswahl, nicht nur eines
        self.statusBar().showMessage(f"Kopiert: {len(sel)} Control(s)", 2000)

    def paste_clip(self):
        if not self._clip:
            return
        pre = self.canvas.doc.to_dict()
        new = [self.canvas.doc.clone_from_dict(d) for d in self._clip]
        self._commit_history(pre)
        self.canvas._select_many(new)
        self.canvas.update()
        self._mark_dirty()

    def _show_context_menu(self, gpos):
        if self.canvas.selected is None:
            return
        menu = QMenu(self)
        # Ohne das sammelt das Hauptfenster pro Rechtsklick ein totes
        # QMenu-Kind an -- in diesem Projekt die typische Vorstufe eines
        # Teardown-Race beim Schliessen.
        menu.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose)
        menu.addAction("Duplizieren", self.duplicate_selected)
        menu.addAction("Kopieren", self.copy_selected)
        menu.addAction("Loeschen", self.delete_selected)
        menu.addSeparator()
        menu.addAction("Nach vorne", self.raise_selected)
        menu.addAction("Nach hinten", self.lower_selected)
        menu.exec(gpos)

    def _update_status(self, c):
        n = len(self.canvas.selection)
        if n > 1:
            self._status.setText(f"{n} Controls ausgewaehlt")
        elif c is not None:
            self._status.setText(f"{c.name}    x={c.x} y={c.y}    {c.w}×{c.h}")
        else:
            self._status.setText("")

    # -- Code-Editor --
    def _open_handler(self, c: Control):
        """Doppelklick auf ein Control: Handler anlegen/anspringen + fokussieren."""
        if self.canvas.doc.primary_event(c) is None:
            self.statusBar().showMessage(f"{c.kind}: kein Event-Handler moeglich.", 3000)
            return
        pre = self.canvas.doc.to_dict()
        name = self.canvas.doc.ensure_handler(c)
        changed = pre != self.canvas.doc.to_dict()
        if changed:
            self._commit_history(pre)             # Handler-Erzeugung = Undo-Schritt
        self.code_panel.refresh()
        self.code_panel.show_handler(name)
        self.inspector.set_control(c)             # neuer Handler-Name im Inspector
        self.canvas.update()
        self.code_dock.show(); self.code_dock.raise_()
        self.code_panel.focus_editor()
        if changed:                               # Doppelklick auf einen schon
            self._mark_dirty()                    # vorhandenen Handler aendert nichts

    def _on_code_session(self):
        self._code_baseline = self.canvas.doc.to_dict()
        self._code_dirty = False

    def _on_code_edited(self):
        """Code-Edits einer Handler-Sitzung = EIN Undo-Schritt."""
        if not self._code_dirty and self._code_baseline is not None:
            self.history.push(self._code_baseline)
            self._code_dirty = True
            self._refresh_history_actions()
        self._mark_dirty()

    # -- Aktionen --
    def _on_doc_changed(self):
        """Canvas hat das Dokument geaendert. Die Handler-Combo muss folgen,
        wenn sich die Handler-Liste geaendert hat -- nach dem Loeschen eines
        Controls auf der Canvas zeigte sie weiter dessen Handler an und
        schrieb Edits in einen `code`-Eintrag, den es nicht mehr gibt. Der
        Listenvergleich ist billig genug fuer den Drag-Pfad (dort aendert er
        sich nie), ein blindes `refresh()` waere es nicht. Verglichen wird
        gegen den TATSAECHLICHEN Combo-Inhalt, nicht gegen eine mitgefuehrte
        Kopie -- die liefe bei jedem anderen Weg, der `refresh()` ruft, aus
        dem Tritt."""
        self._mark_dirty()
        combo = self.code_panel.combo
        shown = [combo.itemText(i) for i in range(combo.count())]
        if self.canvas.doc.handler_names() != shown:
            self.code_panel.refresh()

    def _mark_dirty(self):
        was = self.active.dirty
        self.active.dirty = True
        self._update_title()
        if not was:
            self._refresh_form_list()    # nur bei Uebergang -> Stern erscheint

    def _resync_dirty(self):
        """Dirty gegen den gespeicherten Stand neu bestimmen (nach Undo/Redo).
        Ein Undo bis zurueck zur Datei liess den Stern sonst fuer immer
        stehen -- er wurde bedeutungslos und verdeckte echte Aenderungen."""
        of = self.active
        of.dirty = of.saved is None or self.canvas.doc.to_dict() != of.saved
        self._update_title()
        self._refresh_form_list()        # Navigator lief nach Undo sonst nach

    def _update_title(self):
        of = self.active
        name = of.path.name if of.path else "(unbenannt)"
        star = "*" if of.dirty else ""
        proj = f"  [{self.project_path.name}]" if self.project_path else ""
        self.setWindowTitle(f"Drachenhauch Form-Designer{proj}  --  {name}{star}")

    def new_form(self):
        self._add_open_form(FormDoc())

    def open_form(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Formular oeffnen", str(self.project_root),
                                            "Drachenhauch-Form (*.dhform *.gbform);;Alle (*.*)")
        if not fn:
            return
        # Schon offen? Dann dorthin wechseln statt einen zweiten Puffer
        # anzulegen -- sonst arbeitet man abwechselnd in beiden und der
        # zuletzt gespeicherte ueberschreibt den anderen ersatzlos.
        target = Path(fn).resolve()
        for i, of in enumerate(self.forms):
            if of.path is not None and Path(of.path).resolve() == target:
                self._switch_to(i)
                self.statusBar().showMessage(f"{target.name} ist bereits geoeffnet.", 3000)
                return
        try:
            self._add_open_form(FormDoc.load(fn), Path(fn))
        except Exception as e:  # noqa: BLE001
            QMessageBox.critical(self, "Fehler", f"Konnte nicht laden:\n{e}")

    def close_form(self):
        if self.active.dirty:
            r = QMessageBox.question(
                self, "Formular schliessen",
                "Das Formular hat ungespeicherte Aenderungen.",
                QMessageBox.StandardButton.Save | QMessageBox.StandardButton.Discard
                | QMessageBox.StandardButton.Cancel)
            if r == QMessageBox.StandardButton.Cancel:
                return
            # Vorher gab es nur Ja/Nein -- kein Weg, beim Schliessen zu sichern.
            if r == QMessageBox.StandardButton.Save and not self.save_form():
                return
        i = self.active_index
        if self.forms[i] is self._main_form:
            self._main_form = None
        del self.forms[i]
        if not self.forms:
            self._add_open_form(FormDoc())        # nie ganz leer
        else:
            self._switch_to(min(i, len(self.forms) - 1))

    def _write(self, fn: str, write) -> bool:
        """Schreibvorgang mit Fehlerdialog. Vorher hatte KEIN Speicherpfad eine
        Fehlerbehandlung: auf ein volles/schreibgeschuetztes Ziel oder einen
        getrennten Netzpfad flog ein roher Traceback, im UI passierte nichts --
        der Nutzer hielt die Datei fuer gespeichert."""
        try:
            write()
            return True
        except OSError as e:
            QMessageBox.critical(self, "Speichern fehlgeschlagen",
                                 f"{fn}\n\n{e}")
            return False

    def save_form(self):
        if self.path is None:
            return self.save_form_as()
        if not self._write(str(self.path), lambda: self.canvas.doc.save(str(self.path))):
            return False
        self._mark_saved()
        return True

    def save_form_as(self):
        fn, _ = QFileDialog.getSaveFileName(self, "Formular speichern", str(self.project_root),
                                            "Drachenhauch-Form (*.dhform)")
        if not fn:
            return False
        if not fn.endswith(".dhform"):
            fn += ".dhform"
        # Pfad ERST nach erfolgreichem Schreiben uebernehmen -- sonst zeigte der
        # Titel nach einem Fehlschlag eine Datei an, die nie existiert hat, und
        # ein spaeteres Strg+S schrieb still an denselben kaputten Ort.
        if not self._write(fn, lambda: self.canvas.doc.save(fn)):
            return False
        self.path = Path(fn)
        self._mark_saved()
        return True

    def _mark_saved(self):
        """Formular als gespeichert markieren + den Stand merken, damit ein
        Undo zurueck zur Datei den Stern wieder verschwinden laesst."""
        of = self.active
        of.dirty = False
        of.saved = self.canvas.doc.to_dict()
        self._update_title(); self._refresh_form_list()

    def save_all(self) -> bool:
        start = self.active_index
        ok = True
        for i in range(len(self.forms)):
            if self.forms[i].dirty or self.forms[i].path is None:
                self._switch_to(i)
                if not self.save_form():
                    ok = False
                    break                          # Abbruch im Speichern-Dialog
        self._switch_to(min(start, len(self.forms) - 1))
        if not ok:
            self.statusBar().showMessage("Speichern abgebrochen -- nicht alle "
                                         "Formulare wurden gesichert.", 5000)
        return ok

    def set_main_form(self):
        if self.path is None:
            self.statusBar().showMessage("Formular zuerst speichern.", 3000)
            return
        # Das Formular selbst merken, nicht den relativen Pfad: solange kein
        # Projektpfad feststeht, liefert `_rel` nur den Dateinamen. Nach dem
        # Speichern in ein Unterverzeichnis passte der nicht mehr zu den
        # Manifest-Eintraegen und `main` fiel still auf forms[0] zurueck.
        self._main_form = self.active
        self.project.main = self._rel(self.path)
        self._refresh_form_list()
        self.statusBar().showMessage(f"Startformular: {self.path.name}", 3000)

    # -- Projekt (.dhproj) --
    def open_project(self):
        fn, _ = QFileDialog.getOpenFileName(self, "Projekt oeffnen", str(self.project_root),
                                            "Drachenhauch-Projekt (*.dhproj *.gbproj)")
        if fn:
            self.load_project_file(fn)

    def load_project_file(self, fn: str):
        if not self._confirm_dirty():        # sonst waren alle offenen Formulare weg
            return
        try:
            proj = FormProject.load(fn)
        except Exception as e:  # noqa: BLE001
            QMessageBox.critical(self, "Fehler", f"Projekt nicht ladbar:\n{e}")
            return
        base = Path(fn).parent
        self.project = proj
        self.project_path = Path(fn)
        self.forms = []
        self.active_index = -1
        # Nicht ladbare Formulare merken statt sie stumm zu schlucken: sie
        # bleiben im Manifest (siehe save_project) und der Nutzer erfaehrt davon.
        self.unresolved: list[str] = []
        main_idx = 0
        for rel in proj.forms:
            try:
                doc = FormDoc.load(str(base / rel))
            except Exception:  # noqa: BLE001
                self.unresolved.append(rel)
                continue
            of = self._add_open_form(doc, base / rel, switch=False)
            if rel == proj.main:
                main_idx = len(self.forms) - 1
                self._main_form = of
        if not self.forms:
            self._add_open_form(FormDoc(), switch=False)
        self._switch_to(min(main_idx, len(self.forms) - 1))
        if self.unresolved:
            QMessageBox.warning(
                self, "Formulare fehlen",
                "Diese Formulare konnten nicht geladen werden und bleiben "
                "unveraendert im Projekt:\n\n" + "\n".join(self.unresolved))

    def save_project(self):
        # Erst sicherstellen, dass jede Form einen Pfad hat (+ gespeichert ist).
        start = self.active_index
        for i in range(len(self.forms)):
            if self.forms[i].path is None:
                self._switch_to(i)
                if not self.save_form_as():
                    self._switch_to(start)
                    return
        if self.project_path is None:
            fn, _ = QFileDialog.getSaveFileName(self, "Projekt speichern", str(self.project_root),
                                                "Drachenhauch-Projekt (*.dhproj)")
            if not fn:
                self._switch_to(start)
                return
            if not fn.endswith(".dhproj"):
                fn += ".dhproj"
            self.project_path = Path(fn)
        # Alle Forms speichern + Manifest aufbauen (relativ zum Projektpfad).
        self.project.forms = []
        main_rel = ""
        for of in self.forms:
            if not self._write(str(of.path), lambda of=of: of.doc.save(str(of.path))):
                self._switch_to(start)
                return
            rel = self._rel(of.path)
            self.project.add(rel)
            if of is getattr(self, "_main_form", None):
                main_rel = rel
        # Beim Laden uebersprungene Formulare bleiben im Manifest -- vorher hat
        # der Neuaufbau aus den OFFENEN Formularen sie dauerhaft geloescht,
        # obwohl die Datei nur kurzzeitig nicht erreichbar war.
        for rel in getattr(self, "unresolved", []):
            self.project.add(rel)
        if main_rel:
            self.project.main = main_rel
        if self.project.main not in self.project.forms:
            self.project.main = self.project.forms[0] if self.project.forms else ""
        if not self._write(str(self.project_path),
                           lambda: self.project.save(str(self.project_path))):
            self._switch_to(start)
            return
        for of in self.forms:                       # erst jetzt als sauber markieren
            of.dirty = False
            of.saved = of.doc.to_dict()
        self._switch_to(min(start, len(self.forms) - 1))
        self.statusBar().showMessage(f"Projekt gespeichert: {self.project_path.name}", 3000)

    def export_gb_code(self):
        """Aktives Formular als eigenstaendiges Drachenhauch-Programm (explizite
        GUI_*-Konstruktion, kein GUI_LOAD) speichern."""
        default = str(self.path.with_suffix(".dh")) if self.path else str(self.project_root)
        fn, _ = QFileDialog.getSaveFileName(self, "GB-Code exportieren", default,
                                            "Drachenhauch (*.dh)")
        if not fn:
            return
        if not fn.endswith(".dh"):
            fn += ".dh"
        code = self.canvas.doc.generate_gb_code(screen_title=self.canvas.doc.title)
        if not self._write(fn, lambda: Path(fn).write_text(code, encoding="utf-8")):
            return
        self.statusBar().showMessage(f"GB-Code exportiert: {Path(fn).name}", 4000)

    def _stop_run(self):
        """Laufenden Testlauf beenden + sein Temp-Verzeichnis raeumen. Vorher
        blieb pro F5 ein `gbform_*`-Ordner in %TEMP% liegen und die
        dhrt-Fenster sammelten sich an -- sie ueberlebten sogar den Designer."""
        p = getattr(self, "_proc", None)
        if p is not None:
            try:
                # `Vorschau` (QProcess) kennt stoppen(); die Tests reichen
                # weiterhin ein Popen-artiges Objekt herein -> beide Formen
                # bedienen, statt den Test-Doppelgaenger umzubauen.
                if hasattr(p, "stoppen"):
                    p.stoppen()
                elif p.poll() is None:
                    p.terminate()
            except OSError:
                pass
        self._proc = None
        d = getattr(self, "_run_dir", None)
        if d:
            shutil.rmtree(d, ignore_errors=True)
        self._run_dir = None

    def run_form(self):
        dhrt = _find_dhrt(self.project_root)
        if dhrt is None:
            QMessageBox.warning(self, "dhrt fehlt", "Native Runtime nicht gefunden.\n"
                                "Im Entwicklungsbaum: python rust/build_runtime.py")
            return
        self._stop_run()
        tmp = Path(tempfile.mkdtemp(prefix="gbform_"))
        self._run_dir = tmp
        gb = tmp / "run.dh"
        runner = self.canvas.doc.generate_runner("form.dhform",
                                                 screen_title=self.canvas.doc.title)
        if not self._write(str(tmp), lambda: (
                self.canvas.doc.save(str(tmp / "form.dhform")),
                gb.write_text(runner, encoding="utf-8"))):
            return
        # Vorab pruefen: `run` scheiterte sonst STUMM. Weder stdout/stderr noch
        # der Exit-Code wurden ausgewertet, es gab also keinerlei Rueckmeldung
        # -- der Nutzer druckte mehrfach F5 und hielt den Designer fuer kaputt.
        diags = _dhrt_diagnostics(dhrt, gb)
        errs = [d for d in diags if d.get("severity") != "warning"]
        if errs:
            lines = runner.splitlines()
            def _at(d):
                n = int(d.get("line", 0))
                src = lines[n - 1].strip() if 1 <= n <= len(lines) else ""
                return f"Zeile {n}: {d.get('message', '')}" + (f"\n    {src}" if src else "")
            QMessageBox.critical(
                self, "Formular laeuft nicht",
                "Das erzeugte Programm hat Fehler:\n\n"
                + "\n".join(_at(d) for d in errs[:5])
                + ("\n\n(weitere ausgelassen)" if len(errs) > 5 else ""))
            return
        try:
            self._proc = self._spawn([str(dhrt), "run", str(gb)], str(tmp))
        except OSError as e:
            QMessageBox.critical(self, "Start fehlgeschlagen", str(e))

    def _spawn(self, cmd: list, cwd: str):
        """Eigene Methode, damit Tests den Start ersetzen koennen, ohne
        `subprocess.Popen` global zu patchen (das zerlegt sonst das
        `subprocess.run` der Vorab-Pruefung).

        Der Semaphor schuetzt die Prozess-ERSTELLUNG gegen gleichzeitig
        startende `dhrt`-Subprozesse aus anderen Editor-Threads (verifizierter
        Windows-Crash, siehe Kommentar in `dhrt_locate`).

        Gestartet wird ueber `QProcess` (siehe editor_qt/vorschau_start.py):
        kein Konsolenfenster, und ein Absturz der Form landet als Meldung im
        Designer statt in einem Fenster, das sich sofort wieder schliesst."""
        from .editor_qt.vorschau_start import starte_vorschau
        with dhrt_spawn_semaphore:
            return starte_vorschau(self, cmd, cwd, titel="Form-Vorschau")

    def _arm_place(self, item: QListWidgetItem):
        self.canvas.place_kind = item.data(Qt.ItemDataRole.UserRole)
        self.statusBar().showMessage(f"Platzieren: {item.text()} -- auf die Flaeche klicken", 4000)


def launch(project_root: Path, initial_file: Path | None = None) -> int:
    app = QApplication.instance()
    if app is None:
        # Fusion-Style NUR bei frischer QApplication erzwingen -- siehe
        # trackereditor_qt.launch() fuer die volle Begruendung.
        app = QApplication([])
        app.setStyle("Fusion")
    app.setStyleSheet(global_qss())
    win = FormDesigner(project_root)
    if initial_file:
        open_initial(win, Path(initial_file))
    win.showMaximized()      # "ordentlich Platz" -- wie Audio Studio und
                             # der Notenblatt-Editor; F11 = echtes Vollbild
    return app.exec()


def open_initial(win: FormDesigner, p: Path) -> bool:
    """Start-Argument oeffnen (`.dhform` oder `.dhproj`). True bei Erfolg.
    Eigene Funktion, damit der Zweig ohne `app.exec()` testbar ist."""
    try:
        if not p.is_file():
            raise OSError("Datei nicht gefunden" if not p.exists()
                          else "ist ein Verzeichnis")
        # `.lower()`: auf Windows ist `Projekt.GBPROJ` dieselbe Datei. Der
        # case-sensitive Vergleich schickte sie in den Formular-Zweig, wo das
        # Manifest als leeres Formular durchging -- das naechste Strg+S hat
        # dann die Projektdatei ueberschrieben.
        # `.gbproj` ist der Name aus der GameBasic-Zeit -- gespeichert wird nur
        # noch `.dhproj`, aber ein vorhandenes Projekt muss sich oeffnen lassen
        # (der Oeffnen-Dialog zeigt es ebenfalls weiter an).
        if p.suffix.lower() in (".dhproj", ".gbproj"):
            win.load_project_file(str(p))
        else:
            # leeres Start-Formular durch die geladene Datei ersetzen
            doc = FormDoc.load(str(p))
            of = win.forms[0]
            of.doc = doc; of.path = p; of.saved = doc.to_dict()
            win._switch_to(0)
        return True
    except Exception as e:  # noqa: BLE001
        # Vorher still verschluckt: ein Tippfehler im Dateinamen oeffnete
        # kommentarlos ein leeres Formular, der Nutzer baute es neu.
        QMessageBox.warning(win, "Nicht ladbar", f"{p}\n\n{e}")
        return False
