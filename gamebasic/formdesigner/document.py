"""Qt-freies Datenmodell des Form-Designers.

`.gbform` = JSON im Runtime-GUI-Format (Window + widgets[]). Zusatzfelder, die
nur der Designer braucht (`name`), ignoriert die Runtime beim Laden.
"""
from __future__ import annotations

import json
import math
from dataclasses import dataclass, field, replace
from pathlib import Path


# --- Control-Palette --------------------------------------------------------
# Pro Widget-Art: Anzeigename, Default-Groesse, unterstuetzte Events.
@dataclass(frozen=True)
class PaletteSpec:
    kind: str
    label: str
    w: int
    h: int
    events: tuple = ()          # ("on_click",) / ("on_change",) / ...
    has_text: bool = False      # Standard-Text editierbar?
    has_items: bool = False     # items-Liste (dropdown/listbox)?


PALETTE: list[PaletteSpec] = [
    PaletteSpec("button",    "Button",       100, 28, ("on_click",), has_text=True),
    PaletteSpec("label",     "Label",         80, 16, (), has_text=True),
    PaletteSpec("checkbox",  "Checkbox",      16, 16, ("on_click", "on_change"), has_text=True),
    PaletteSpec("radio",     "RadioButton",   16, 16, ("on_click", "on_change"), has_text=True),
    PaletteSpec("slider",    "Slider",       160, 14, ("on_change",)),
    PaletteSpec("textinput", "TextInput",    180, 26, ("on_change",)),
    PaletteSpec("dropdown",  "Dropdown",     160, 24, ("on_change",), has_items=True),
    PaletteSpec("listbox",   "ListBox",      160, 96, ("on_change",), has_items=True),
    PaletteSpec("progress",  "ProgressBar",  180, 18, ()),
    PaletteSpec("image",     "Image",         96, 96, ()),
    PaletteSpec("canvas",    "Canvas",       200, 150, ()),
    PaletteSpec("panel",     "Panel",        160, 100, (), has_text=True),
]

_SPEC_BY_KIND = {p.kind: p for p in PALETTE}


def palette_spec(kind: str) -> PaletteSpec | None:
    return _SPEC_BY_KIND.get(kind)


# --- Geometrie: Snap-to-Grid + Resize -------------------------------------
# Qt-frei, damit headless testbar (die Canvas ruft das nur auf).
GRID = 8                                                  # Raster-Schrittweite (px)
HANDLES = ("nw", "n", "ne", "e", "se", "s", "sw", "w")   # 8 Resize-Griffe


def snap(v: int, grid: int = GRID) -> int:
    """Rundet `v` auf das naechste Vielfache von `grid` (round-half-up, damit das
    Ergebnis symmetrisch ist -- Pythons `round` waere banker's rounding)."""
    if grid <= 1:
        return int(v)
    return int(math.floor(v / grid + 0.5)) * grid


def resize_rect(x: int, y: int, w: int, h: int, handle: str, nx: int, ny: int,
                min_w: int = 8, min_h: int = 8) -> tuple[int, int, int, int]:
    """Neues (x, y, w, h), wenn am `handle` zur Zeiger-Position (nx, ny) gezogen
    wird. Gegenueberliegende Kante bleibt fix; Mindestgroesse wird gewahrt.

    `handle` ist einer von HANDLES; die Buchstaben n/s/e/w steuern, welche
    Kanten der Zeiger bewegt (z.B. "ne" = Nord- + Ost-Kante)."""
    right, bottom = x + w, y + h
    if "e" in handle:
        w = max(min_w, nx - x)
    if "w" in handle:
        nx = min(nx, right - min_w)
        x, w = nx, right - nx
    if "s" in handle:
        h = max(min_h, ny - y)
    if "n" in handle:
        ny = min(ny, bottom - min_h)
        y, h = ny, bottom - ny
    return x, y, w, h


# --- Control ----------------------------------------------------------------
@dataclass
class Control:
    kind: str
    name: str = ""              # Designer-Bezeichner (nur Metadaten, z.B. "btnSave")
    x: int = 0
    y: int = 0
    w: int = 100
    h: int = 24
    text: str = ""
    color: int = 0xFFFFFF
    value: float = 0.0
    min: float = 0.0
    max: float = 1.0
    checked: bool = False
    placeholder: str = ""
    group: str = ""
    items: list = field(default_factory=list)
    sel: int = -1
    enabled: bool = True
    visible: bool = True
    font_size: int = 0
    on_click: str = ""
    on_change: str = ""
    ov: dict = field(default_factory=dict)   # Farb-Overrides: bg/fg/border/accent -> int

    def to_dict(self) -> dict:
        """Runtime-kompatibles widget-Dict (+ Designer-`name`)."""
        d: dict = {
            "kind": self.kind, "x": self.x, "y": self.y, "w": self.w, "h": self.h,
            "text": self.text, "color": self.color,
            "value": self.value, "min": self.min, "max": self.max,
            "checked": self.checked, "placeholder": self.placeholder,
            "visible": self.visible,
        }
        if self.name:
            d["name"] = self.name
        if self.on_click:
            d["on_click"] = self.on_click
        if self.on_change:
            d["on_change"] = self.on_change
        if self.ov:
            d["ov"] = dict(self.ov)
        if self.group:
            d["group"] = self.group
        if self.items:
            d["items"] = list(self.items)
        if self.sel != -1:
            d["sel"] = self.sel
        if not self.enabled:
            d["enabled"] = False
        if self.font_size:
            d["font_size"] = self.font_size
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Control":
        return cls(
            kind=str(d.get("kind", "label")),
            name=str(d.get("name", "")),
            x=int(d.get("x", 0)), y=int(d.get("y", 0)),
            w=int(d.get("w", 100)), h=int(d.get("h", 24)),
            text=str(d.get("text", "")),
            color=int(d.get("color", 0xFFFFFF)),
            value=float(d.get("value", 0.0)),
            min=float(d.get("min", 0.0)), max=float(d.get("max", 1.0)),
            checked=bool(d.get("checked", False)),
            placeholder=str(d.get("placeholder", "")),
            group=str(d.get("group", "")),
            items=list(d.get("items", [])),
            sel=int(d.get("sel", -1)),
            enabled=bool(d.get("enabled", True)),
            visible=bool(d.get("visible", True)),
            font_size=int(d.get("font_size", 0)),
            on_click=str(d.get("on_click", "")),
            on_change=str(d.get("on_change", "")),
            ov={str(k): int(v) for k, v in dict(d.get("ov", {})).items()},
        )

    def clone(self) -> "Control":
        return replace(self, items=list(self.items), ov=dict(self.ov))


# --- FormDoc ----------------------------------------------------------------
@dataclass
class FormDoc:
    title: str = "Form1"
    x: int = 200
    y: int = 120
    w: int = 360
    h: int = 260
    movable: bool = True
    closable: bool = True
    visible: bool = True
    controls: list = field(default_factory=list)   # list[Control]

    # ---- Bearbeiten ----
    def add(self, kind: str, x: int, y: int) -> Control:
        sp = palette_spec(kind) or PaletteSpec(kind, kind, 100, 24)
        c = Control(kind=kind, x=x, y=y, w=sp.w, h=sp.h, name=self._unique_name(kind))
        if sp.has_text:
            c.text = sp.label
        if sp.has_items:
            c.items = ["Eintrag 1", "Eintrag 2", "Eintrag 3"]
            c.sel = 0 if kind == "dropdown" else -1
        if kind == "slider":
            c.min, c.max, c.value = 0.0, 100.0, 50.0
        self.controls.append(c)
        return c

    def remove(self, c: Control) -> None:
        if c in self.controls:
            self.controls.remove(c)

    def _unique_name(self, kind: str) -> str:
        base = {"textinput": "txt", "button": "btn", "checkbox": "chk", "radio": "rad",
                "dropdown": "dd", "listbox": "lst", "slider": "sld", "label": "lbl",
                "progress": "prg", "image": "img", "canvas": "cnv", "panel": "pnl"}.get(kind, kind)
        existing = {c.name for c in self.controls}
        i = 1
        while f"{base}{i}" in existing:
            i += 1
        return f"{base}{i}"

    def control_at(self, x: int, y: int) -> Control | None:
        """Oberstes Control (zuletzt hinzugefuegt = oben) am Punkt; None sonst."""
        for c in reversed(self.controls):
            if c.x <= x < c.x + c.w and c.y <= y < c.y + c.h:
                return c
        return None

    # ---- .gbform IO (Runtime-Format) ----
    def to_dict(self) -> dict:
        return {
            "title": self.title, "x": self.x, "y": self.y, "w": self.w, "h": self.h,
            "movable": self.movable, "closable": self.closable, "visible": self.visible,
            "widgets": [c.to_dict() for c in self.controls],
        }

    @classmethod
    def from_dict(cls, d: dict) -> "FormDoc":
        doc = cls(
            title=str(d.get("title", "Form1")),
            x=int(d.get("x", 200)), y=int(d.get("y", 120)),
            w=int(d.get("w", 360)), h=int(d.get("h", 260)),
            movable=bool(d.get("movable", True)),
            closable=bool(d.get("closable", True)),
            visible=bool(d.get("visible", True)),
        )
        doc.controls = [Control.from_dict(w) for w in d.get("widgets", [])]
        return doc

    def save(self, path: str) -> None:
        Path(path).write_text(json.dumps(self.to_dict(), indent=2, ensure_ascii=False),
                              encoding="utf-8")

    @classmethod
    def load(cls, path: str) -> "FormDoc":
        return cls.from_dict(json.loads(Path(path).read_text(encoding="utf-8")))

    # ---- Code-Generierung ----
    def handler_names(self) -> list[str]:
        """Alle eindeutigen Event-Handler-Namen (Reihenfolge der Controls)."""
        seen: list[str] = []
        for c in self.controls:
            for h in (c.on_click, c.on_change):
                if h and h not in seen:
                    seen.append(h)
        return seen

    def generate_runner(self, form_filename: str, screen_w: int = 800,
                        screen_h: int = 480, screen_title: str | None = None,
                        handler_bodies: dict | None = None) -> str:
        """Lauffaehiges GameBasic-Programm: laedt das `.gbform`, definiert die
        Event-Handler (Stubs oder uebergebene Koerper) und treibt die GUI-Schleife
        -- der Xojo-Lauf. `handler_bodies`: optional {name: code-zeilen}."""
        bodies = handler_bodies or {}
        title = screen_title or self.title
        lines = [
            f"' Auto-generiert vom Form-Designer -- Layout: {form_filename}",
            'IMPORT "gui"',
            f'SCREEN({screen_w}, {screen_h}, "{title}", 1)',
            "",
            "DIM frm AS GUI_WINDOW",
            f'frm = GUI_LOAD("{form_filename}")',
            "",
        ]
        for name in self.handler_names():
            lines.append(f"SUB {name}()")
            body = bodies.get(name)
            if body:
                lines.extend("    " + ln for ln in body.splitlines())
            else:
                lines.append(f'    ' + f"' TODO: {name}")
            lines.append("END SUB")
            lines.append("")
        lines += [
            "WHILE NOT QUITREQUESTED()",
            "    GUI_UPDATE()",
            "    CLS(&H0E1014)",
            "    GUI_DRAW()",
            "    FLIP()",
            "WEND",
            "",
        ]
        return "\n".join(lines)
