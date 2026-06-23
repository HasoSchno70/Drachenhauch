"""Beschrifteter Schieberegler (Fader) fuer die Editoren.

Ein `Fader` ist ein `QSlider` mit Beschriftung + Live-Wert daneben, float-faehig
ueber interne Int-Skalierung. Die oeffentliche API (`value()` / `setValue()` /
`valueChanged`) ist **bewusst QSpinBox-kompatibel**, damit bestehender Code
(z. B. `_params()`/`_apply_params()` im SFX-Generator) unveraendert funktioniert
und ein Spinbox 1:1 durch einen Fader ersetzt werden kann.

Optional faerbt eine `accent`-Farbe Griff + gefuellte Spur ein (sfxr-Stil:
farbcodierte Parameter-Gruppen).
"""
from __future__ import annotations

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import QHBoxLayout, QLabel, QSlider, QVBoxLayout, QWidget

from .theme import COLORS


class Fader(QWidget):
    """Label + Live-Wert + horizontaler Slider. `value()`/`setValue()` arbeiten
    in echten (ggf. float-)Werten; intern laeuft ein Int-Slider mit Schrittweite
    `step`. `decimals=0` -> liefert `int`, sonst gerundeten `float`."""

    valueChanged = Signal(float)

    def __init__(self, label: str, lo: float, hi: float, val: float,
                 step: float = 1.0, decimals: int = 0, unit: str = "",
                 accent: str | None = None, parent=None):
        super().__init__(parent)
        self._lo = float(lo)
        self._hi = float(hi)
        self._step = float(step) or 1.0
        self._dec = int(decimals)
        self._unit = unit
        self._n = max(1, round((self._hi - self._lo) / self._step))

        lay = QVBoxLayout(self)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(2)
        head = QHBoxLayout()
        head.setContentsMargins(0, 0, 0, 0)
        self._name = QLabel(label)
        self._name.setStyleSheet(f"color:{COLORS['fg_muted']}; font-size:11px;")
        self._val = QLabel("")
        self._val.setAlignment(Qt.AlignmentFlag.AlignRight
                               | Qt.AlignmentFlag.AlignVCenter)
        self._val.setStyleSheet(f"color:{COLORS['fg']}; font-size:11px; "
                                f"font-weight:600;")
        head.addWidget(self._name)
        head.addStretch(1)
        head.addWidget(self._val)
        lay.addLayout(head)

        self._slider = QSlider(Qt.Orientation.Horizontal)
        self._slider.setRange(0, self._n)
        if accent:
            # Gefuellte Spur als Verlauf dunkel -> Akzentfarbe (statt flaechig
            # grell). Der Startpunkt ist eine abgedunkelte Variante des Akzents.
            dark = QColor(accent).darker(300).name()
            self._slider.setStyleSheet(
                f"QSlider::sub-page:horizontal {{ background: qlineargradient("
                f"x1:0, y1:0, x2:1, y2:0, stop:0 {dark}, stop:1 {accent}); "
                f"border-radius:3px; }}"
                f"QSlider::handle:horizontal {{ background-color:{accent}; "
                f"border:2px solid {COLORS['bg']}; width:14px; height:14px; "
                f"margin:-6px 0; border-radius:9px; }}")
        self._slider.valueChanged.connect(self._on_slider)
        lay.addWidget(self._slider)

        self.setValue(val)

    # ---- QSpinBox-kompatible API ----
    def value(self):
        v = self._lo + self._slider.value() * self._step
        return int(round(v)) if self._dec == 0 else round(v, self._dec)

    def setValue(self, v) -> None:
        # Wie QSpinBox: ein wirklicher Wertwechsel feuert valueChanged (ueber
        # das interne Slider-Signal -> _on_slider). So mark't der Undo-Verlauf
        # auch bei programmatischem setValue. Bei gleichem Wert nur das Label.
        v = max(self._lo, min(self._hi, float(v)))
        s = int(max(0, min(self._n, round((v - self._lo) / self._step))))
        if s == self._slider.value():
            self._update_label()
        else:
            self._slider.setValue(s)        # emittiert -> _on_slider

    def setEnabled(self, on: bool) -> None:  # noqa: N802 (Qt-Stil)
        super().setEnabled(on)

    # ---- intern ----
    def _on_slider(self, _s: int) -> None:
        self._update_label()
        self.valueChanged.emit(float(self.value()))

    def _update_label(self) -> None:
        v = self.value()
        txt = f"{v}" if self._dec == 0 else f"{v:.{self._dec}f}"
        if self._unit:
            txt += f" {self._unit}"
        self._val.setText(txt)
