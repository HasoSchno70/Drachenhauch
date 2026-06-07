"""Qt-freies Datenmodell des Form-Designers.

`.gbform` = JSON im Runtime-GUI-Format (Window + widgets[]). Zusatzfelder, die
nur der Designer braucht (`name`), ignoriert die Runtime beim Laden.
"""
from __future__ import annotations

import json
import math
import re
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
    PaletteSpec("groupbox",  "GroupBox",     160, 100, (), has_text=True),
    PaletteSpec("separator", "Separator",    160,  8, ()),
]

_SPEC_BY_KIND = {p.kind: p for p in PALETTE}


def palette_spec(kind: str) -> PaletteSpec | None:
    return _SPEC_BY_KIND.get(kind)


# --- GB-Code-Emit-Helfer (fuer generate_gb_code) ----------------------------
def _gb_str(s: str) -> str:
    """GameBasic-String-Literal: `"` wird zu `""` escaped, Zeilenumbrueche raus."""
    t = str(s).replace("\r", " ").replace("\n", " ").replace('"', '""')
    return f'"{t}"'


def _gb_hex(i: int) -> str:
    return f"&H{int(i) & 0xFFFFFF:06X}"


def _gb_bool(b: bool) -> str:
    return "TRUE" if b else "FALSE"


def _gb_num(x) -> str:
    """FLOAT-Literal mit Dezimalpunkt (GB-FLOAT erwartet z.B. `0.0`, nicht `0`)."""
    return repr(float(x))


def _gb_ident(s: str) -> str:
    """Bezeichner aus einem Control-Namen: nur [A-Za-z0-9_], nicht mit Ziffer."""
    t = re.sub(r"[^A-Za-z0-9_]", "_", str(s))
    if not t or t[0].isdigit():
        t = "_" + t
    return t


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


# --- FormProject (Multi-Form-Manifest) --------------------------------------
@dataclass
class FormProject:
    """Manifest eines Multi-Form-Projekts (`.gbproj`, JSON). Verweist auf die
    einzelnen `.gbform`-Dateien (relativ zum Projekt-Verzeichnis) -- jede Form
    bleibt ihre eigene Datei. `main` = Startformular (einer der `forms`-Pfade).
    Qt-frei + headless testbar."""
    forms: list = field(default_factory=list)   # list[str] relative .gbform-Pfade
    main: str = ""

    def add(self, rel: str) -> None:
        if rel not in self.forms:
            self.forms.append(rel)
        if not self.main:
            self.main = rel

    def remove(self, rel: str) -> None:
        if rel in self.forms:
            self.forms.remove(rel)
        if self.main == rel:
            self.main = self.forms[0] if self.forms else ""

    def to_dict(self) -> dict:
        return {"forms": list(self.forms), "main": self.main}

    @classmethod
    def from_dict(cls, d: dict) -> "FormProject":
        forms = [str(x) for x in d.get("forms", [])]
        main = str(d.get("main", ""))
        if main not in forms:                       # defensiv: main muss Mitglied sein
            main = forms[0] if forms else ""
        return cls(forms=forms, main=main)

    def save(self, path: str) -> None:
        Path(path).write_text(json.dumps(self.to_dict(), indent=2, ensure_ascii=False),
                              encoding="utf-8")

    @classmethod
    def load(cls, path: str) -> "FormProject":
        return cls.from_dict(json.loads(Path(path).read_text(encoding="utf-8")))


# --- Undo/Redo-Historie -----------------------------------------------------
# Snapshot-basiert + Qt-frei (headless testbar). Ein Snapshot ist das
# `FormDoc.to_dict()`-Dict; die UI stellt mit `FormDoc.from_dict()` wieder her.
class History:
    """Zwei-Stack-Undo/Redo ueber komplette Doc-Snapshots.

    Aufruf-Konvention: VOR einer Mutation den aktuellen Zustand via `push()`
    sichern. `undo(current)`/`redo(current)` bekommen den jeweils aktuellen
    Zustand (der auf den anderen Stack wandert) und liefern den wiederherzu-
    stellenden Snapshot."""

    def __init__(self, limit: int = 200):
        self._undo: list[dict] = []
        self._redo: list[dict] = []
        self._limit = max(1, limit)

    def clear(self) -> None:
        self._undo.clear()
        self._redo.clear()

    def push(self, snapshot: dict) -> None:
        """Pre-Mutations-Zustand ablegen; loescht den Redo-Stack."""
        self._undo.append(snapshot)
        if len(self._undo) > self._limit:
            self._undo.pop(0)
        self._redo.clear()

    @property
    def can_undo(self) -> bool:
        return bool(self._undo)

    @property
    def can_redo(self) -> bool:
        return bool(self._redo)

    def undo(self, current: dict) -> dict:
        prev = self._undo.pop()
        self._redo.append(current)
        return prev

    def redo(self, current: dict) -> dict:
        nxt = self._redo.pop()
        self._undo.append(current)
        return nxt


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
    anchor: str = "lt"                       # Verankerung: Teilmenge von "lrtb" (Default oben-links)
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
        if self.anchor and self.anchor != "lt":
            d["anchor"] = self.anchor
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
            anchor=str(d.get("anchor", "lt")) or "lt",
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
    resizable: bool = False                        # Fenster zur Laufzeit groessenveraenderbar?
    min_w: int = 0                                 # Mindest-/Maximalgroesse (0 = keine Grenze)
    min_h: int = 0
    max_w: int = 0
    max_h: int = 0
    controls: list = field(default_factory=list)   # list[Control]
    code: dict = field(default_factory=dict)       # Event-Handler-Koerper: name -> GB-Code

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

    def duplicate(self, c: Control, dx: int = GRID, dy: int = GRID) -> Control:
        """Kopie von `c` (versetzt, frischer Name) ans Ende (= oben) anhaengen."""
        nc = c.clone()
        nc.x += dx; nc.y += dy
        nc.name = self._unique_name(nc.kind)
        self.controls.append(nc)
        return nc

    def clone_from_dict(self, d: dict, dx: int = GRID, dy: int = GRID) -> Control:
        """Aus einem widget-Dict (Clipboard) ein neues Control bauen + anhaengen."""
        nc = Control.from_dict(d)
        nc.x += dx; nc.y += dy
        nc.name = self._unique_name(nc.kind)
        self.controls.append(nc)
        return nc

    # ---- Anordnen (Mehrfach-Auswahl): Ausrichten / Gleiche Groesse / Verteilen ----
    def align(self, controls: list, edge: str) -> None:
        """Controls an einer Kante/Mitte des Gesamt-Begrenzungsrahmens ausrichten.
        `edge`: left/right/top/bottom/center_h/center_v."""
        if len(controls) < 2:
            return
        xs = [c.x for c in controls]; rights = [c.x + c.w for c in controls]
        ys = [c.y for c in controls]; bottoms = [c.y + c.h for c in controls]
        if edge == "left":
            m = min(xs)
            for c in controls: c.x = m
        elif edge == "right":
            m = max(rights)
            for c in controls: c.x = max(0, m - c.w)
        elif edge == "top":
            m = min(ys)
            for c in controls: c.y = m
        elif edge == "bottom":
            m = max(bottoms)
            for c in controls: c.y = max(0, m - c.h)
        elif edge == "center_h":
            cx = (min(xs) + max(rights)) // 2
            for c in controls: c.x = max(0, cx - c.w // 2)
        elif edge == "center_v":
            cy = (min(ys) + max(bottoms)) // 2
            for c in controls: c.y = max(0, cy - c.h // 2)

    def same_size(self, controls: list, ref, dim: str) -> None:
        """Alle Controls auf die Groesse des Referenz-Controls `ref` bringen.
        `dim`: w/h/both."""
        if ref is None:
            return
        for c in controls:
            if c is ref:
                continue
            if dim in ("w", "both"): c.w = ref.w
            if dim in ("h", "both"): c.h = ref.h

    def distribute(self, controls: list, axis: str) -> None:
        """Controls entlang einer Achse gleichmaessig verteilen (gleiche Luecken;
        erstes + letztes bleiben fix). `axis`: h/v. Braucht >= 3 Controls."""
        if len(controls) < 3:
            return
        if axis == "h":
            cs = sorted(controls, key=lambda c: c.x)
            span = (cs[-1].x + cs[-1].w) - cs[0].x
            gap = (span - sum(c.w for c in cs)) / (len(cs) - 1)
            pos = float(cs[0].x)
            for c in cs:
                c.x = max(0, int(round(pos))); pos += c.w + gap
        else:
            cs = sorted(controls, key=lambda c: c.y)
            span = (cs[-1].y + cs[-1].h) - cs[0].y
            gap = (span - sum(c.h for c in cs)) / (len(cs) - 1)
            pos = float(cs[0].y)
            for c in cs:
                c.y = max(0, int(round(pos))); pos += c.h + gap

    def to_front(self, c: Control) -> None:
        """Z-Reihenfolge: ans Ende (zuletzt gezeichnet = oben)."""
        if c in self.controls:
            self.controls.remove(c); self.controls.append(c)

    def to_back(self, c: Control) -> None:
        """Z-Reihenfolge: an den Anfang (zuerst gezeichnet = unten)."""
        if c in self.controls:
            self.controls.remove(c); self.controls.insert(0, c)

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

    # ---- Event-Handler (fuer den integrierten Code-Editor) ----
    def primary_event(self, c: Control) -> str | None:
        """Das Haupt-Event eines Controls (erstes in der Palette-Spec) oder None."""
        sp = palette_spec(c.kind)
        return sp.events[0] if sp and sp.events else None

    def ensure_handler(self, c: Control) -> str | None:
        """Stellt sicher, dass das Haupt-Event von `c` einen Handler-Namen hat
        (legt ihn + einen leeren Code-Eintrag an, falls noetig). Liefert den
        Namen oder None, wenn das Control kein Event unterstuetzt."""
        ev = self.primary_event(c)
        if ev is None:
            return None
        name = getattr(c, ev)
        if not name:
            suffix = {"on_click": "Click", "on_change": "Changed"}.get(ev, "Action")
            name = self._unique_handler_name((c.name or c.kind) + suffix)
            setattr(c, ev, name)
        self.code.setdefault(name, "")
        return name

    def _unique_handler_name(self, base: str) -> str:
        used = set(self.code.keys())
        for c in self.controls:
            for ev in ("on_click", "on_change"):
                if getattr(c, ev):
                    used.add(getattr(c, ev))
        if base not in used:
            return base
        i = 2
        while f"{base}{i}" in used:
            i += 1
        return f"{base}{i}"

    # ---- .gbform IO (Runtime-Format) ----
    def to_dict(self) -> dict:
        d: dict = {
            "title": self.title, "x": self.x, "y": self.y, "w": self.w, "h": self.h,
            "movable": self.movable, "closable": self.closable, "visible": self.visible,
            "resizable": self.resizable,
            "widgets": [c.to_dict() for c in self.controls],
        }
        for k in ("min_w", "min_h", "max_w", "max_h"):   # nur wenn gesetzt
            if getattr(self, k):
                d[k] = getattr(self, k)
        if self.code:
            # Designer-Metadaten (Handler-Koerper); die Runtime ignoriert `code`.
            d["code"] = {str(k): str(v) for k, v in self.code.items()}
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "FormDoc":
        doc = cls(
            title=str(d.get("title", "Form1")),
            x=int(d.get("x", 200)), y=int(d.get("y", 120)),
            w=int(d.get("w", 360)), h=int(d.get("h", 260)),
            movable=bool(d.get("movable", True)),
            closable=bool(d.get("closable", True)),
            visible=bool(d.get("visible", True)),
            resizable=bool(d.get("resizable", False)),
            min_w=int(d.get("min_w", 0)), min_h=int(d.get("min_h", 0)),
            max_w=int(d.get("max_w", 0)), max_h=int(d.get("max_h", 0)),
            code={str(k): str(v) for k, v in dict(d.get("code", {})).items()},
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

    def generate_runner(self, form_filename: str, screen_w: int | None = None,
                        screen_h: int | None = None, screen_title: str | None = None,
                        handler_bodies: dict | None = None) -> str:
        """Lauffaehiges GameBasic-Programm: das Fenster wird auf Formulargroesse
        gesetzt, das `.gbform` geladen und **randlos** (chromeless) auf das echte
        OS-Fenster gelegt -- die Form IST das Fenster (Xojo-Lauf). Ist sie
        `resizable`, wird das OS-Fenster nativ groessenveraenderbar und die Form
        fuellt es jeden Frame (Anchoring reflowt). `handler_bodies`: optional
        {name: code-zeilen}; sonst die gespeicherten `code`-Koerper."""
        bodies = handler_bodies if handler_bodies is not None else self.code
        title = screen_title or self.title
        sw = screen_w if screen_w is not None else self.w
        sh = screen_h if screen_h is not None else self.h
        lines = [
            f"' Auto-generiert vom Form-Designer -- Layout: {form_filename}",
            'IMPORT "gui"',
            f'SCREEN({sw}, {sh}, {_gb_str(title)}, 1)',
        ]
        if self.resizable:
            lines.append("WINDOW_RESIZABLE(TRUE)")
            if self.min_w or self.min_h:
                lines.append(f"WINDOW_MIN_SIZE({self.min_w or 1}, {self.min_h or 1})")
            if self.max_w or self.max_h:
                lines.append(f"WINDOW_MAX_SIZE({self.max_w}, {self.max_h})")
        lines += [
            "",
            "DIM frm AS GUI_WINDOW",
            f'frm = GUI_LOAD({_gb_str(form_filename)})',
            "GUI_WINDOW_CHROME(frm, FALSE)        ' randlos -- das OS-Fenster liefert den Rahmen",
            "GUI_WINDOW_RESIZABLE(frm, FALSE)     ' kein innerer Griff; das OS-Fenster resized",
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
            "    GUI_WINDOW_SET_BOUNDS(frm, 0, 0, SCREENWIDTH(), SCREENHEIGHT())   ' Form fuellt das Fenster",
            "    GUI_UPDATE()",
            "    CLS(&H0E1014)",
            "    GUI_DRAW()",
            "    FLIP()",
            "WEND",
            "",
        ]
        return "\n".join(lines)

    # ---- GB-Code-Export (explizite GUI_*-Konstruktion statt GUI_LOAD) ----
    def generate_gb_code(self, screen_w: int = 800, screen_h: int = 480,
                         screen_title: str | None = None,
                         handler_bodies: dict | None = None,
                         with_screen: bool = True, with_loop: bool = True) -> str:
        """Eigenstaendiges GameBasic-Programm, das das Formular **explizit** mit
        den `GUI_*`-Konstruktoren aufbaut (kein `GUI_LOAD`/`.gbform` zur Laufzeit).
        Lesbar + frei editierbar. `with_screen`/`with_loop` schalten SCREEN bzw.
        die GUI-Schleife ab (fuer Tests / Einbettung in eigenen Code)."""
        bodies = handler_bodies if handler_bodies is not None else self.code
        title = screen_title or self.title
        L: list[str] = [
            f"' Auto-generiert vom Form-Designer -- explizite GUI-Konstruktion (ohne GUI_LOAD)",
            'IMPORT "gui"',
        ]
        if with_screen:
            L.append(f'SCREEN({screen_w}, {screen_h}, {_gb_str(title)}, 1)')
        L.append("")
        # Event-Handler
        for name in self.handler_names():
            L.append(f"SUB {name}()")
            body = bodies.get(name)
            if body:
                L.extend("    " + ln for ln in body.splitlines())
            else:
                L.append(f"    ' TODO: {name}")
            L.append("END SUB")
            L.append("")
        # Fenster
        L.append("DIM frm AS GUI_WINDOW")
        L.append(f"frm = GUI_WINDOW({_gb_str(self.title)}, {self.x}, {self.y}, {self.w}, {self.h})")
        if not self.movable:
            L.append("GUI_WINDOW_MOVABLE(frm, FALSE)")
        if not self.closable:
            L.append("GUI_WINDOW_CLOSABLE(frm, FALSE)")
        if not self.visible:
            L.append("GUI_WINDOW_VISIBLE(frm, FALSE)")
        if self.resizable:
            L.append("GUI_WINDOW_RESIZABLE(frm, TRUE)")
        if self.min_w or self.min_h:
            L.append(f"GUI_WINDOW_SET_MIN_SIZE(frm, {self.min_w}, {self.min_h})")
        if self.max_w or self.max_h:
            L.append(f"GUI_WINDOW_SET_MAX_SIZE(frm, {self.max_w}, {self.max_h})")
        L.append("")
        # Controls
        used = {"frm"} | set(self.handler_names())
        for idx, c in enumerate(self.controls):
            var = self._gb_var(c, idx, used)
            block = self._gb_control(c, var)
            if block:
                L.extend(block)
                L.append("")
        # Hauptschleife
        if with_loop:
            L += [
                "WHILE NOT QUITREQUESTED()",
                "    GUI_UPDATE()",
                "    CLS(&H0E1014)",
                "    GUI_DRAW()",
                "    FLIP()",
                "WEND",
                "",
            ]
        return "\n".join(L)

    def _gb_var(self, c: Control, idx: int, used: set) -> str:
        base = _gb_ident(c.name) if c.name else f"{c.kind}{idx + 1}"
        name = base
        i = 2
        while name in used:
            name = f"{base}{i}"; i += 1
        used.add(name)
        return name

    def _gb_control(self, c: Control, var: str) -> list:
        """GB-Zeilen, die ein Control aufbauen (DIM + Konstruktor + Setter)."""
        k = c.kind
        if k == "image":
            return [f"' image '{var}' uebersprungen -- GUI_IMAGE braucht eine "
                    f"Bildquelle (LOADIMAGE), die das .gbform nicht speichert"]
        out = [f"DIM {var} AS GUI_WIDGET"]
        if k == "button":
            out.append(f"{var} = GUI_BUTTON(frm, {_gb_str(c.text)}, {c.x}, {c.y}, {c.w}, {c.h})")
        elif k == "label":
            if c.color != 0xFFFFFF:
                out.append(f"{var} = GUI_LABEL(frm, {_gb_str(c.text)}, {c.x}, {c.y}, {_gb_hex(c.color)})")
            else:
                out.append(f"{var} = GUI_LABEL(frm, {_gb_str(c.text)}, {c.x}, {c.y})")
        elif k == "checkbox":
            out.append(f"{var} = GUI_CHECKBOX(frm, {_gb_str(c.text)}, {c.x}, {c.y}, {_gb_bool(c.checked)})")
        elif k == "radio":
            out.append(f"{var} = GUI_RADIO(frm, {_gb_str(c.group)}, {_gb_str(c.text)}, {c.x}, {c.y})")
        elif k == "slider":
            out.append(f"{var} = GUI_SLIDER(frm, {c.x}, {c.y}, {c.w}, "
                       f"{_gb_num(c.min)}, {_gb_num(c.max)}, {_gb_num(c.value)})")
        elif k == "textinput":
            out.append(f"{var} = GUI_TEXTINPUT(frm, {c.x}, {c.y}, {c.w}, {c.h}, {_gb_str(c.placeholder)})")
        elif k == "panel":
            out.append(f"{var} = GUI_PANEL(frm, {c.x}, {c.y}, {c.w}, {c.h}, {_gb_str(c.text)})")
        elif k == "groupbox":
            out.append(f"{var} = GUI_GROUPBOX(frm, {c.x}, {c.y}, {c.w}, {c.h}, {_gb_str(c.text)})")
        elif k == "separator":
            out.append(f"{var} = GUI_SEPARATOR(frm, {c.x}, {c.y}, {c.w})")
        elif k == "progress":
            out.append(f"{var} = GUI_PROGRESS(frm, {c.x}, {c.y}, {c.w}, {c.h})")
        elif k == "canvas":
            out.append(f"{var} = GUI_CANVAS(frm, {c.x}, {c.y}, {c.w}, {c.h})")
        elif k in ("dropdown", "listbox"):
            iv = var + "_items"
            out.append(f"DIM {iv}[{len(c.items)}] AS STRING")   # 1D ARRAY OF STRING
            for j, it in enumerate(c.items):
                out.append(f"{iv}[{j}] = {_gb_str(it)}")
            ctor = "GUI_DROPDOWN" if k == "dropdown" else "GUI_LISTBOX"
            out.append(f"{var} = {ctor}(frm, {c.x}, {c.y}, {c.w}, {c.h}, {iv})")
        else:
            return [f"' Control-Typ '{k}' uebersprungen (kein Konstruktor)"]
        # Nachbearbeitung (nur abweichende Werte)
        if k == "dropdown" and c.sel not in (-1, 0):
            out.append(f"GUI_DROPDOWN_SET_SELECTED({var}, {c.sel})")
        if k == "listbox" and c.sel >= 0:
            out.append(f"GUI_LISTBOX_SET_SELECTED({var}, {c.sel})")
        if k == "progress" and c.value != 0.0:
            out.append(f"GUI_SET_VALUE({var}, {_gb_num(c.value)})")
        if not c.enabled:
            out.append(f"GUI_SET_ENABLED({var}, FALSE)")
        if not c.visible:
            out.append(f"GUI_SET_VISIBLE({var}, FALSE)")
        if c.font_size:
            out.append(f"GUI_SET_FONT_SIZE({var}, {c.font_size})")
        if c.anchor and c.anchor != "lt":
            out.append(f"GUI_SET_ANCHOR({var}, {_gb_str(c.anchor)})")
        for role, col in c.ov.items():
            out.append(f"GUI_SET_COLOR({var}, {_gb_str(role)}, {_gb_hex(col)})")
        if c.on_click:
            out.append(f"GUI_ON_CLICK({var}, {c.on_click})")
        if c.on_change:
            out.append(f"GUI_ON_CHANGE({var}, {c.on_change})")
        return out
