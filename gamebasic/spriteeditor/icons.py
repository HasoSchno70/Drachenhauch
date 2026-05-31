"""QPainter-basierte Icon-Generatoren fuer den Sprite-Editor.

Zwei oeffentliche Funktionen:
  - make_tool_icon(name, size=28) -- Icons fuer die linke Tool-Sidebar
    (pencil, eraser, bucket, line, rect, ellipse, eyedropper, select,
    move, magic_wand, spray, ...).
  - make_action_icon(name, size=22) -- Icons fuer die obere Toolbar
    (new, open, save, undo, redo, flip, rotate, animation, ...).

Beide nutzen QPainter mit Antialiasing -- die Icons skalieren bei
Hi-DPI-Displays sauber. Die Implementierung war urspruenglich Teil des
Sprite-Editor-Hauptmoduls (zusammen ~540 Zeilen Painter-Code) und wurde
hier herausgezogen, weil sie keinerlei State und keinerlei Bezug zum
Editor-Workflow hat.
"""
from __future__ import annotations

from PySide6.QtCore import Qt, QPoint, QPointF, QRect, QRectF
from PySide6.QtGui import (
    QBrush, QColor, QIcon, QPainter, QPen, QPixmap, QPolygon, QPolygonF,
)

from .. spriteeditor_qt import COLORS  # Theme-Farben (zirkulaer-OK: dieses
# Modul wird vom Hauptmodul importiert, dann wieder; COLORS ist eine
# einfache dict-Konstante und ist beim Import bereits gesetzt)


# ============================================================
# Tool-Icons (QPainter mit Antialiasing -- gleicher Stil wie Action-Icons)
# ============================================================
#
# Vektor-artige Symbole, scharf bei jeder Aufloesung. Konsistent mit der
# oberen Toolbar (make_action_icon) -- wir mischen NICHT Pixel-Art-Tools
# mit Vektor-Toolbar-Aktionen, sonst wirkt das Layout uneinheitlich.

_OUTLINE_RGB = (32, 32, 36)


def make_tool_icon(name: str, size: int = 28) -> QIcon:
    """QPainter-basierte Tool-Icons mit Antialiasing.

    Designziel: gleicher Look wie make_action_icon -- 2 px Pen,
    RoundCap+RoundJoin, gefuellte Silhouetten wo es die Lesbarkeit
    verbessert (Stift-Schaft, Eraser-Body, Bucket, Eyedrop-Reservoir).
    Akzentfarbe nur als Highlight (Bucket-Tropfen, Eyedrop-Reservoir,
    Eck-Marker bei Select).
    """
    pix = QPixmap(size, size)
    pix.fill(Qt.transparent)
    p = QPainter(pix)
    p.setRenderHint(QPainter.Antialiasing, True)
    p.setRenderHint(QPainter.SmoothPixmapTransform, True)

    fg = QColor(212, 212, 212)
    accent = QColor(96, 165, 250)
    OUT = QColor(*_OUTLINE_RGB)
    pen = QPen(OUT, 1.6, Qt.SolidLine, Qt.RoundCap, Qt.RoundJoin)
    p.setPen(pen)
    s = size

    def f(v):
        return v * s

    def rectF(x, y, w, h):
        return QRectF(f(x), f(y), f(w), f(h))

    def pt(x, y):
        return QPointF(f(x), f(y))

    if name == "pencil":
        # Klassischer Stift schraeg von links-unten nach rechts-oben.
        # Drei Segmente: Holz-Schaft, Metall-Huelse, schwarze Mine.
        wood       = QColor(232, 178, 80)
        wood_high  = QColor(255, 220, 140)
        metal      = QColor(190, 195, 205)
        # Schaft (gefuelltes Polygon)
        p.setBrush(QBrush(wood))
        shaft = QPolygonF([pt(0.10, 0.78), pt(0.62, 0.26),
                            pt(0.74, 0.38), pt(0.22, 0.90)])
        p.drawPolygon(shaft)
        # Highlight-Streifen oben am Schaft
        p.setPen(QPen(wood_high, 1.5, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.14, 0.74), pt(0.66, 0.22))
        # Metall-Huelse
        p.setPen(pen)
        p.setBrush(QBrush(metal))
        ferrule = QPolygonF([pt(0.62, 0.26), pt(0.78, 0.10),
                              pt(0.90, 0.22), pt(0.74, 0.38)])
        p.drawPolygon(ferrule)
        # Mine (schwarze Spitze)
        p.setBrush(QBrush(QColor(45, 45, 50)))
        tip = QPolygonF([pt(0.78, 0.10), pt(0.94, 0.06),
                          pt(0.90, 0.22)])
        p.drawPolygon(tip)

    elif name == "eraser":
        # Klassischer zweifarbiger Eraser, leicht in Perspektive.
        pink = QColor(240, 100, 130)
        pink_high = QColor(255, 165, 185)
        cream = QColor(245, 230, 210)
        cream_dark = QColor(210, 195, 170)
        p.setBrush(QBrush(pink))
        p.drawRect(rectF(0.20, 0.30, 0.60, 0.18))
        p.setBrush(QBrush(cream))
        p.drawRect(rectF(0.20, 0.48, 0.60, 0.30))
        p.setBrush(Qt.NoBrush)
        p.drawRect(rectF(0.20, 0.30, 0.60, 0.48))
        p.drawLine(pt(0.20, 0.48), pt(0.80, 0.48))
        # Highlight oben links
        p.setPen(QPen(pink_high, 1.6, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.26, 0.36), pt(0.50, 0.36))
        # Schatten unten rechts
        p.setPen(QPen(cream_dark, 1.6, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.30, 0.72), pt(0.74, 0.72))

    elif name == "spray":
        # Spraydose: Zylinder + Druckkopf + Spruehwolke (Punkte)
        body = QColor(70, 130, 200)
        body_dark = QColor(40, 90, 160)
        cap = QColor(220, 220, 230)
        # Druckkopf oben (Quader)
        p.setBrush(QBrush(cap))
        p.drawRect(rectF(0.18, 0.10, 0.16, 0.10))
        # Hauptbody (gefuelltes Rechteck)
        p.setBrush(QBrush(body))
        p.drawRect(rectF(0.14, 0.20, 0.24, 0.62))
        # Vertikaler Highlight-Streifen
        p.setPen(QPen(QColor(120, 180, 240), 1.6, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.20, 0.24), pt(0.20, 0.78))
        # Outline
        p.setPen(pen)
        p.setBrush(Qt.NoBrush)
        p.drawRect(rectF(0.14, 0.20, 0.24, 0.62))
        # Spruehwolke (Punkte rechts oben raus)
        p.setBrush(QBrush(accent))
        p.setPen(Qt.NoPen)
        for cx, cy, rr in [(0.50, 0.28, 0.025), (0.62, 0.18, 0.030),
                            (0.60, 0.36, 0.025), (0.74, 0.28, 0.030),
                            (0.78, 0.46, 0.020), (0.84, 0.18, 0.020),
                            (0.86, 0.36, 0.025), (0.92, 0.28, 0.020),
                            (0.50, 0.46, 0.020)]:
            p.drawEllipse(pt(cx, cy), f(rr), f(rr))

    elif name == "bucket":
        # Trapez-Eimer mit Henkel + austretendem Tropfen.
        body = QColor(200, 210, 222)
        body_dark = QColor(150, 160, 175)
        p.setBrush(QBrush(body))
        bucket = QPolygonF([pt(0.22, 0.30), pt(0.78, 0.30),
                             pt(0.72, 0.86), pt(0.28, 0.86)])
        p.drawPolygon(bucket)
        # Inhalt (dunklerer Streifen oben)
        p.setBrush(QBrush(body_dark))
        p.drawEllipse(rectF(0.22, 0.26, 0.56, 0.10))
        p.setBrush(Qt.NoBrush)
        p.drawEllipse(rectF(0.22, 0.26, 0.56, 0.10))
        # Henkel
        p.setPen(QPen(OUT, 1.8, Qt.SolidLine, Qt.RoundCap))
        p.drawArc(rectF(0.30, 0.06, 0.40, 0.32),
                  10 * 16, 160 * 16)
        # Tropfen unten (Akzent)
        p.setPen(QPen(accent.darker(140), 1.4))
        p.setBrush(QBrush(accent))
        drop = QPolygonF([pt(0.50, 0.96), pt(0.40, 0.78),
                           pt(0.60, 0.78)])
        p.drawPolygon(drop)

    elif name == "line":
        # Diagonal mit dickem Akzent + dunklem Schatten + zwei Endpunkten
        p.setPen(QPen(OUT, 4.0, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.18, 0.82), pt(0.82, 0.18))
        p.setPen(QPen(accent, 2.4, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.18, 0.82), pt(0.82, 0.18))
        p.setBrush(QBrush(OUT))
        p.setPen(Qt.NoPen)
        p.drawEllipse(pt(0.18, 0.82), f(0.06), f(0.06))
        p.drawEllipse(pt(0.82, 0.18), f(0.06), f(0.06))

    elif name == "rect":
        # 3-Schicht-Sandwich: aussen schwarz, mitte fg, innen schwarz
        p.setBrush(Qt.NoBrush)
        p.setPen(QPen(OUT, 2.0))
        p.drawRect(rectF(0.16, 0.20, 0.68, 0.60))
        p.setPen(QPen(fg, 2.5))
        p.drawRect(rectF(0.20, 0.24, 0.60, 0.52))
        p.setPen(QPen(OUT, 1.5))
        p.drawRect(rectF(0.26, 0.30, 0.48, 0.40))

    elif name == "rect_fill":
        p.setPen(QPen(OUT, 1.5))
        p.setBrush(QBrush(fg))
        p.drawRect(rectF(0.16, 0.20, 0.68, 0.60))
        p.setBrush(QBrush(QColor(255, 255, 255, 90)))
        p.setPen(Qt.NoPen)
        p.drawRect(rectF(0.20, 0.24, 0.10, 0.06))

    elif name == "ellipse":
        p.setBrush(Qt.NoBrush)
        p.setPen(QPen(OUT, 2.0))
        p.drawEllipse(rectF(0.14, 0.18, 0.72, 0.64))
        p.setPen(QPen(fg, 2.5))
        p.drawEllipse(rectF(0.18, 0.22, 0.64, 0.56))
        p.setPen(QPen(OUT, 1.5))
        p.drawEllipse(rectF(0.26, 0.30, 0.48, 0.40))

    elif name == "ellipse_fill":
        p.setPen(QPen(OUT, 1.5))
        p.setBrush(QBrush(fg))
        p.drawEllipse(rectF(0.14, 0.18, 0.72, 0.64))
        p.setBrush(QBrush(QColor(255, 255, 255, 100)))
        p.setPen(Qt.NoPen)
        p.drawEllipse(rectF(0.24, 0.26, 0.16, 0.10))

    elif name == "eyedrop":
        # Pipette von links-unten zu rechts-oben mit blauem Reservoir.
        p.setPen(QPen(OUT, 1.4))
        p.setBrush(QBrush(OUT))
        p.drawEllipse(pt(0.12, 0.86), f(0.06), f(0.06))
        # Grauer Schaft
        p.setPen(QPen(QColor(140, 145, 155), 4.0,
                       Qt.SolidLine, Qt.FlatCap))
        p.drawLine(pt(0.16, 0.82), pt(0.50, 0.46))
        # Metall-Mittelteil
        p.setPen(QPen(OUT, 1.0))
        p.setBrush(QBrush(QColor(190, 195, 205)))
        ferrule = QPolygonF([pt(0.46, 0.42), pt(0.62, 0.26),
                              pt(0.70, 0.34), pt(0.54, 0.50)])
        p.drawPolygon(ferrule)
        # Reservoir (Akzent gefuellt)
        p.setPen(QPen(accent.darker(150), 1.5))
        p.setBrush(QBrush(accent))
        reservoir = QPolygonF([pt(0.62, 0.26), pt(0.78, 0.10),
                                pt(0.92, 0.24), pt(0.76, 0.40)])
        p.drawPolygon(reservoir)
        # Highlight am Reservoir
        p.setBrush(QBrush(QColor(255, 255, 255, 200)))
        p.setPen(Qt.NoPen)
        p.drawEllipse(pt(0.74, 0.20), f(0.04), f(0.04))

    elif name == "select":
        # Marquee + Eck-Marker
        p.setPen(QPen(fg, 1.6, Qt.DashLine, Qt.FlatCap))
        p.drawRect(rectF(0.16, 0.16, 0.68, 0.68))
        p.setPen(Qt.NoPen)
        p.setBrush(QBrush(accent))
        for cx, cy in [(0.16, 0.16), (0.84, 0.16),
                        (0.16, 0.84), (0.84, 0.84)]:
            p.drawEllipse(pt(cx, cy), 2.0, 2.0)

    elif name == "magic_wand":
        # Zauberstab: schraeger Stab + Stern an der Spitze
        # Stab (graue dicke Linie)
        p.setPen(QPen(QColor(140, 145, 155), 4.0,
                       Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.16, 0.84), pt(0.56, 0.44))
        # Stab-Outline (dünn)
        p.setPen(QPen(OUT, 1.0))
        p.setBrush(Qt.NoBrush)
        # Stern an der Spitze (Funken-Strahlen)
        p.setPen(QPen(accent, 2.0, Qt.SolidLine, Qt.RoundCap))
        cx, cy = f(0.66), f(0.32)
        # 4 Strahlen
        for dx, dy in [(-0.20, 0.0), (0.20, 0.0),
                        (0.0, -0.20), (0.0, 0.20)]:
            p.drawLine(QPointF(cx, cy),
                       QPointF(cx + f(dx), cy + f(dy)))
        # 4 diagonale (kuerzer)
        for dx, dy in [(-0.12, -0.12), (0.12, -0.12),
                        (-0.12, 0.12), (0.12, 0.12)]:
            p.drawLine(QPointF(cx, cy),
                       QPointF(cx + f(dx), cy + f(dy)))
        # Zentrum (heller Punkt)
        p.setPen(Qt.NoPen)
        p.setBrush(QBrush(QColor(255, 255, 255)))
        p.drawEllipse(QPointF(cx, cy), 2.5, 2.5)
        p.setBrush(QBrush(accent))
        p.drawEllipse(QPointF(cx, cy), 1.5, 1.5)

    elif name == "move":
        # 4-Pfeil-Kreuz mit klaren gefuellten Spitzen
        p.setPen(QPen(fg, 2.4, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(pt(0.50, 0.16), pt(0.50, 0.84))
        p.drawLine(pt(0.12, 0.50), pt(0.88, 0.50))
        p.setBrush(QBrush(fg))
        p.setPen(QPen(fg, 1.0))
        p.drawPolygon(QPolygonF([pt(0.50, 0.06), pt(0.40, 0.20),
                                   pt(0.60, 0.20)]))
        p.drawPolygon(QPolygonF([pt(0.50, 0.94), pt(0.40, 0.80),
                                   pt(0.60, 0.80)]))
        p.drawPolygon(QPolygonF([pt(0.06, 0.50), pt(0.20, 0.40),
                                   pt(0.20, 0.60)]))
        p.drawPolygon(QPolygonF([pt(0.94, 0.50), pt(0.80, 0.40),
                                   pt(0.80, 0.60)]))
    else:
        p.drawRect(rectF(0.15, 0.15, 0.70, 0.70))
        p.drawLine(pt(0.15, 0.15), pt(0.85, 0.85))

    p.end()
    return QIcon(pix)


def make_action_icon(name: str, size: int = 22) -> QIcon:
    """Toolbar-Aktion-Icons via QPainter mit Antialiasing.

    Designprinzipien:
    - 2.0 px Pen, RoundCap+RoundJoin -> sanftere Linien, weniger Aliasing
    - Filled Shapes wo's die Silhouette klarer macht (Diskette, Schere,
      Play, Klemmbrett-Kopf, Frame-Buttons-Plus)
    - Akzentfarbe nur bei aktiven Markierungen (Plus, Symmetrie-Achse)
    - Konsistente Padding (~13% am Rand) damit Icons nebeneinander harmonieren
    """
    pix = QPixmap(size, size)
    pix.fill(Qt.transparent)
    p = QPainter(pix)
    p.setRenderHint(QPainter.Antialiasing, True)
    p.setRenderHint(QPainter.SmoothPixmapTransform, True)
    fg = QColor(COLORS["fg"])
    accent = QColor(COLORS["accent"])
    danger = QColor(COLORS["danger"])
    pen = QPen(fg, 2.0)
    pen.setCapStyle(Qt.RoundCap)
    pen.setJoinStyle(Qt.RoundJoin)
    p.setPen(pen)
    s = size

    def f(v):
        return v * s

    if name == "new":
        # Sauberes Blatt mit umgeknickter Ecke
        p.drawLine(f(0.30), f(0.10), f(0.66), f(0.10))
        p.drawLine(f(0.30), f(0.10), f(0.30), f(0.86))
        p.drawLine(f(0.30), f(0.86), f(0.78), f(0.86))
        p.drawLine(f(0.78), f(0.86), f(0.78), f(0.26))
        p.drawLine(f(0.66), f(0.10), f(0.78), f(0.26))
        p.drawLine(f(0.66), f(0.10), f(0.66), f(0.26))
        p.drawLine(f(0.66), f(0.26), f(0.78), f(0.26))
    elif name == "open":
        # Ordner mit hoch gezogener Klappe (zwei Lagen)
        p.setBrush(QBrush(fg.darker(140)))
        # Tab oben links
        p.drawLine(f(0.10), f(0.30), f(0.40), f(0.30))
        p.drawLine(f(0.40), f(0.30), f(0.48), f(0.40))
        p.drawLine(f(0.48), f(0.40), f(0.90), f(0.40))
        p.drawLine(f(0.10), f(0.30), f(0.10), f(0.82))
        p.drawLine(f(0.90), f(0.40), f(0.90), f(0.82))
        p.drawLine(f(0.10), f(0.82), f(0.90), f(0.82))
    elif name == "save":
        # Diskette: aussen Outline, oben heller Streifen, unten Label
        p.setBrush(QBrush(accent))
        p.drawRoundedRect(f(0.15), f(0.15), f(0.70), f(0.70), 2, 2)
        p.setBrush(QBrush(QColor(255, 255, 255, 220)))
        p.setPen(Qt.NoPen)
        p.drawRect(f(0.27), f(0.15), f(0.46), f(0.20))   # Schreibschutz-Schieber-Bereich
        p.drawRect(f(0.27), f(0.55), f(0.46), f(0.30))   # Label
        p.setPen(pen)
        p.drawRect(f(0.27), f(0.55), f(0.46), f(0.30))
    elif name == "export":
        # Pfeil nach unten in eine offene Schale
        p.drawLine(f(0.50), f(0.10), f(0.50), f(0.62))
        p.drawLine(f(0.50), f(0.62), f(0.32), f(0.46))
        p.drawLine(f(0.50), f(0.62), f(0.68), f(0.46))
        # Schale (U-Form)
        p.drawLine(f(0.18), f(0.70), f(0.18), f(0.86))
        p.drawLine(f(0.18), f(0.86), f(0.82), f(0.86))
        p.drawLine(f(0.82), f(0.86), f(0.82), f(0.70))
    elif name == "undo":
        # Klassischer "Curl-Arrow" -- Bogen + Pfeilspitze unten links
        p.drawArc(f(0.20), f(0.22), f(0.60), f(0.50),
                  30 * 16, 200 * 16)
        # Pfeilspitze
        p.drawLine(f(0.20), f(0.46), f(0.18), f(0.30))
        p.drawLine(f(0.20), f(0.46), f(0.34), f(0.40))
    elif name == "redo":
        p.drawArc(f(0.20), f(0.22), f(0.60), f(0.50),
                  -50 * 16, -200 * 16)
        p.drawLine(f(0.80), f(0.46), f(0.82), f(0.30))
        p.drawLine(f(0.80), f(0.46), f(0.66), f(0.40))
    elif name == "zoom_in" or name == "zoom_out":
        # Lupe mit Glas + Griff
        p.drawEllipse(f(0.14), f(0.14), f(0.50), f(0.50))
        p.drawLine(f(0.58), f(0.58), f(0.86), f(0.86))
        # Plus / Minus im Glas
        cx, cy = f(0.39), f(0.39)
        l = f(0.10)
        p.drawLine(cx - l, cy, cx + l, cy)
        if name == "zoom_in":
            p.drawLine(cx, cy - l, cx, cy + l)
    elif name == "play":
        # Gefuelltes Dreieck
        poly = QPolygon([
            QPoint(int(f(0.28)), int(f(0.18))),
            QPoint(int(f(0.28)), int(f(0.82))),
            QPoint(int(f(0.84)), int(f(0.50))),
        ])
        p.setBrush(QBrush(QColor(COLORS["success"])))
        p.setPen(QPen(QColor(COLORS["success"]).darker(140), 1.5))
        p.drawPolygon(poly)
    elif name == "flip_h":
        # Achse + zwei Pfeile, beide Seiten gleich gross (klassisch)
        p.setPen(QPen(accent, 1.6, Qt.DashLine))
        p.drawLine(f(0.50), f(0.12), f(0.50), f(0.88))
        p.setPen(pen)
        # Linker Pfeil
        p.drawLine(f(0.42), f(0.50), f(0.18), f(0.50))
        p.drawLine(f(0.18), f(0.50), f(0.28), f(0.40))
        p.drawLine(f(0.18), f(0.50), f(0.28), f(0.60))
        # Rechter Pfeil
        p.drawLine(f(0.58), f(0.50), f(0.82), f(0.50))
        p.drawLine(f(0.82), f(0.50), f(0.72), f(0.40))
        p.drawLine(f(0.82), f(0.50), f(0.72), f(0.60))
    elif name == "flip_v":
        p.setPen(QPen(accent, 1.6, Qt.DashLine))
        p.drawLine(f(0.12), f(0.50), f(0.88), f(0.50))
        p.setPen(pen)
        p.drawLine(f(0.50), f(0.42), f(0.50), f(0.18))
        p.drawLine(f(0.50), f(0.18), f(0.40), f(0.28))
        p.drawLine(f(0.50), f(0.18), f(0.60), f(0.28))
        p.drawLine(f(0.50), f(0.58), f(0.50), f(0.82))
        p.drawLine(f(0.50), f(0.82), f(0.40), f(0.72))
        p.drawLine(f(0.50), f(0.82), f(0.60), f(0.72))
    elif name == "rotate_cw" or name == "rotate_ccw":
        sweep = 270 if name == "rotate_cw" else -270
        p.drawArc(f(0.18), f(0.18), f(0.60), f(0.60),
                  90 * 16, sweep * 16)
        if name == "rotate_cw":
            tip_x, tip_y = f(0.78), f(0.32)
            p.drawLine(tip_x, tip_y, tip_x + f(0.10), tip_y - f(0.02))
            p.drawLine(tip_x, tip_y, tip_x - f(0.04), tip_y + f(0.12))
        else:
            tip_x, tip_y = f(0.22), f(0.32)
            p.drawLine(tip_x, tip_y, tip_x - f(0.10), tip_y - f(0.02))
            p.drawLine(tip_x, tip_y, tip_x + f(0.04), tip_y + f(0.12))
    elif name == "frame_add":
        # Frame-Outline + dezenter Plus-Indikator unten rechts (Akzent)
        p.drawRoundedRect(f(0.16), f(0.20), f(0.50), f(0.55), 2, 2)
        # Plus-Akzent
        p.setPen(QPen(accent, 2.5, Qt.SolidLine, Qt.RoundCap))
        cx, cy = f(0.78), f(0.62)
        l = f(0.09)
        p.drawLine(cx - l, cy, cx + l, cy)
        p.drawLine(cx, cy - l, cx, cy + l)
    elif name == "frame_dup":
        # Zwei ueberlappende Frames
        p.drawRoundedRect(f(0.14), f(0.30), f(0.46), f(0.50), 2, 2)
        p.setBrush(QBrush(QColor(COLORS["bg"])))
        p.drawRoundedRect(f(0.34), f(0.16), f(0.46), f(0.50), 2, 2)
    elif name == "frame_del":
        # Muelleimer mit Deckel + Streifen
        p.drawLine(f(0.16), f(0.22), f(0.84), f(0.22))
        p.drawLine(f(0.36), f(0.22), f(0.36), f(0.16))
        p.drawLine(f(0.64), f(0.22), f(0.64), f(0.16))
        p.drawLine(f(0.36), f(0.16), f(0.64), f(0.16))
        # Body
        p.setPen(QPen(danger, 2.0, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(f(0.24), f(0.30), f(0.30), f(0.84))
        p.drawLine(f(0.30), f(0.84), f(0.70), f(0.84))
        p.drawLine(f(0.70), f(0.84), f(0.76), f(0.30))
        # Streifen innen
        p.drawLine(f(0.40), f(0.34), f(0.40), f(0.78))
        p.drawLine(f(0.50), f(0.34), f(0.50), f(0.78))
        p.drawLine(f(0.60), f(0.34), f(0.60), f(0.78))
    elif name == "swap":
        # Versetzte Doppelpfeile (klassisches Swap-Symbol)
        p.drawLine(f(0.22), f(0.36), f(0.78), f(0.36))
        p.drawLine(f(0.78), f(0.36), f(0.66), f(0.26))
        p.drawLine(f(0.78), f(0.36), f(0.66), f(0.46))
        p.drawLine(f(0.78), f(0.64), f(0.22), f(0.64))
        p.drawLine(f(0.22), f(0.64), f(0.34), f(0.54))
        p.drawLine(f(0.22), f(0.64), f(0.34), f(0.74))
    elif name == "sym_x":
        # Vertikale Achse + symmetrische Halbkreise (klarer als 2 Pfeile)
        p.setPen(QPen(accent, 1.8, Qt.DashLine))
        p.drawLine(f(0.50), f(0.12), f(0.50), f(0.88))
        p.setPen(QPen(fg, 2.0, Qt.SolidLine, Qt.RoundCap, Qt.RoundJoin))
        # Linker Halbmond
        p.drawArc(f(0.16), f(0.30), f(0.34), f(0.40),
                  90 * 16, 180 * 16)
        # Rechter Halbmond
        p.drawArc(f(0.50), f(0.30), f(0.34), f(0.40),
                  90 * 16, -180 * 16)
    elif name == "sym_y":
        # Horizontale Achse + zwei Halbmonde oben/unten
        p.setPen(QPen(accent, 1.8, Qt.DashLine))
        p.drawLine(f(0.12), f(0.50), f(0.88), f(0.50))
        p.setPen(QPen(fg, 2.0, Qt.SolidLine, Qt.RoundCap, Qt.RoundJoin))
        p.drawArc(f(0.30), f(0.16), f(0.40), f(0.34),
                  0 * 16, 180 * 16)
        p.drawArc(f(0.30), f(0.50), f(0.40), f(0.34),
                  0 * 16, -180 * 16)
    elif name == "tile":
        # 3x3 Grid mit hervorgehobener Mitte
        cell = f(0.22)
        margin = f(0.13)
        for row in range(3):
            for col in range(3):
                x = margin + col * cell
                y = margin + row * cell
                if row == 1 and col == 1:
                    p.setBrush(QBrush(accent))
                    p.setPen(QPen(accent.darker(140), 1.2))
                    p.drawRect(x, y, cell - 1, cell - 1)
                    p.setPen(pen)
                else:
                    p.setBrush(Qt.NoBrush)
                    p.drawRect(x, y, cell - 1, cell - 1)
    elif name == "cut":
        # Klassische Schere: zwei Klingen + zwei Augen
        # Klingen (zwei sich kreuzende dicke Linien)
        p.setPen(QPen(fg, 2.4, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(f(0.34), f(0.50), f(0.86), f(0.18))
        p.drawLine(f(0.34), f(0.50), f(0.86), f(0.82))
        # Augen (Kreise)
        p.setPen(QPen(fg, 2.0))
        p.setBrush(Qt.NoBrush)
        p.drawEllipse(f(0.10), f(0.32), f(0.24), f(0.20))
        p.drawEllipse(f(0.10), f(0.48), f(0.24), f(0.20))
    elif name == "copy":
        # Zwei Blaetter, zweites mit gefuelltem Hintergrund
        p.setBrush(QBrush(QColor(COLORS["bg_panel"])))
        p.drawRoundedRect(f(0.14), f(0.30), f(0.46), f(0.54), 2, 2)
        p.setBrush(QBrush(QColor(COLORS["bg_alt"])))
        p.drawRoundedRect(f(0.32), f(0.14), f(0.50), f(0.54), 2, 2)
        # Text-Linien im vorderen Blatt
        p.setPen(QPen(fg.darker(140), 1.2))
        p.drawLine(f(0.40), f(0.28), f(0.74), f(0.28))
        p.drawLine(f(0.40), f(0.38), f(0.74), f(0.38))
        p.drawLine(f(0.40), f(0.48), f(0.66), f(0.48))
    elif name == "paste":
        # Klemmbrett: oben Klippe (akzent gefuellt), unten Blatt mit
        # Linien
        p.setBrush(QBrush(QColor(COLORS["bg_panel"])))
        p.drawRoundedRect(f(0.18), f(0.20), f(0.60), f(0.66), 3, 3)
        # Klippe oben
        p.setBrush(QBrush(accent))
        p.setPen(QPen(accent.darker(140), 1.5))
        p.drawRoundedRect(f(0.34), f(0.10), f(0.28), f(0.16), 2, 2)
        # Text-Linien
        p.setPen(QPen(fg.darker(140), 1.2))
        p.drawLine(f(0.28), f(0.42), f(0.68), f(0.42))
        p.drawLine(f(0.28), f(0.52), f(0.68), f(0.52))
        p.drawLine(f(0.28), f(0.62), f(0.58), f(0.62))
    elif name == "test_sprite":
        # Frame mit gefuelltem Play-Dreieck darin -- "Run im Spiel"-Symbol
        p.setBrush(Qt.NoBrush)
        p.setPen(QPen(fg, 1.8))
        p.drawRoundedRect(f(0.10), f(0.18), f(0.80), f(0.64), 3, 3)
        # Play-Dreieck (Akzent-gruen)
        play_color = QColor(COLORS["success"])
        p.setBrush(QBrush(play_color))
        p.setPen(QPen(play_color.darker(140), 1.5))
        poly = QPolygon([
            QPoint(int(f(0.36)), int(f(0.32))),
            QPoint(int(f(0.36)), int(f(0.68))),
            QPoint(int(f(0.68)), int(f(0.50))),
        ])
        p.drawPolygon(poly)

    elif name == "code":
        # < / > mit Slash dazwischen
        p.setPen(QPen(fg, 2.4, Qt.SolidLine, Qt.RoundCap, Qt.RoundJoin))
        p.drawLine(f(0.30), f(0.30), f(0.16), f(0.50))
        p.drawLine(f(0.16), f(0.50), f(0.30), f(0.70))
        p.drawLine(f(0.70), f(0.30), f(0.84), f(0.50))
        p.drawLine(f(0.84), f(0.50), f(0.70), f(0.70))
        p.setPen(QPen(accent, 2.4, Qt.SolidLine, Qt.RoundCap))
        p.drawLine(f(0.58), f(0.22), f(0.42), f(0.78))

    p.end()
    return QIcon(pix)


