"""Qt-freies Datenmodell des Form-Designers.

`.dhform` = JSON im Runtime-GUI-Format (Window + widgets[]). Zusatzfelder, die
nur der Designer braucht (`name`), ignoriert die Runtime beim Laden.
"""
from __future__ import annotations

import copy
import json
import math
import re
from dataclasses import dataclass, field, replace
from pathlib import Path

from ..tokens import KEYWORDS


# --- Tolerante JSON-Coercion ------------------------------------------------
# Ein `.dhform` kann von Hand geschrieben, von einem Fremdwerkzeug erzeugt oder
# leicht beschaedigt sein. Die Laufzeit (`gui.rs`, `widget_from_json`) faellt in
# so einem Fall durchgaengig auf den Default zurueck (`as_i64().unwrap_or(d)`)
# statt abzubrechen -- der Designer macht es genauso, sonst quittiert er eine
# Datei, die dhrt anstandslos laedt, mit einem rohen Python-Traceback.
def _as_int(d: dict, key: str, default: int) -> int:
    v = d.get(key, default)
    if isinstance(v, bool) or v is None:
        return default
    try:
        return int(v)
    except (TypeError, ValueError):
        return default


def _as_float(d: dict, key: str, default: float) -> float:
    v = d.get(key, default)
    if isinstance(v, bool) or v is None:
        return default
    try:
        f = float(v)
    except (TypeError, ValueError):
        return default
    return f if math.isfinite(f) else default


def _as_bool(d: dict, key: str, default: bool) -> bool:
    v = d.get(key, default)
    return v if isinstance(v, bool) else default


def _as_str(d: dict, key: str, default: str = "") -> str:
    v = d.get(key, default)
    return v if isinstance(v, str) else default


# --- Control-Palette --------------------------------------------------------
# Pro Widget-Art: Anzeigename, Default-Groesse, unterstuetzte Events.
# Alle Ereignisse, die die Laufzeit ausloesen kann, mit dem Namenszusatz fuer
# den erzeugten Handler. Bis hierher stand diese Liste an rund zehn Stellen
# einzeln -- ein weiteres Ereignis nachzutragen hiess, keine davon zu
# vergessen. Wer die Laufzeit erweitert, ergaenzt jetzt hier eine Zeile.
EVENTS: dict = {
    "on_click":  "Click",
    "on_change": "Changed",
    "on_hover":  "Hover",
    "on_leave":  "Leave",
    "on_focus":  "Focus",
    "on_blur":   "Blur",
    "on_enter":  "Enter",
}

# Zeigen/Verlassen kann jedes anklickbare Control; Fokus bekommt seit
# 2026-08-30 JEDES bedienbare (die Laufzeit setzt `focus_widget` nicht mehr
# nur fuer Textfeld/Textbereich/Zahlenfeld, sondern fuer alles, was man auch
# anklicken kann -- siehe `Kind::fokussierbar` in gui.rs). Reine Deko und
# Anzeigen bleiben aussen vor: ein Ereignis anzubieten, das nie feuert, waere
# schlimmer als keines -- man sucht den Fehler dann im eigenen Programm.
_ZEIGEN = ("on_hover", "on_leave")
_FOKUS = ("on_focus", "on_blur")


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
    PaletteSpec("button",    "Button",       100, 28, ("on_click",) + _ZEIGEN + _FOKUS, has_text=True),
    PaletteSpec("label",     "Label",         80, 16, (), has_text=True),
    PaletteSpec("checkbox",  "Checkbox",      16, 16, ("on_click", "on_change") + _ZEIGEN + _FOKUS, has_text=True),
    PaletteSpec("radio",     "RadioButton",   16, 16, ("on_click", "on_change") + _ZEIGEN + _FOKUS, has_text=True),
    PaletteSpec("slider",    "Slider",       160, 14, ("on_change",) + _ZEIGEN + _FOKUS),
    PaletteSpec("textinput", "TextInput",    180, 26, ("on_change", "on_enter") + _ZEIGEN + _FOKUS),
    PaletteSpec("dropdown",  "Dropdown",     160, 24, ("on_change",) + _ZEIGEN + _FOKUS, has_items=True),
    PaletteSpec("listbox",   "ListBox",      160, 96, ("on_change",) + _ZEIGEN + _FOKUS, has_items=True),
    PaletteSpec("progress",  "ProgressBar",  180, 18, _ZEIGEN),
    PaletteSpec("image",     "Image",         96, 96, _ZEIGEN),
    PaletteSpec("table",     "Tabelle",      320, 140, ("on_change",) + _ZEIGEN + _FOKUS),
    PaletteSpec("canvas",    "Canvas",       200, 150, ()),
    PaletteSpec("panel",     "Panel",        160, 100, (), has_text=True),
    PaletteSpec("groupbox",  "GroupBox",     160, 100, (), has_text=True),
    PaletteSpec("separator", "Separator",    160,  8, ()),
    PaletteSpec("textarea",  "TextArea",     220, 90, ("on_change",) + _ZEIGEN + _FOKUS),
    PaletteSpec("spinner",   "Zahlenfeld",   120, 24, ("on_change",) + _ZEIGEN + _FOKUS),
    PaletteSpec("knob",      "Drehknopf",     48, 48, ("on_change",) + _ZEIGEN + _FOKUS),
    PaletteSpec("toggle",    "Umschalter",    46, 22, ("on_click", "on_change") + _ZEIGEN + _FOKUS, has_text=True),
    PaletteSpec("tree",      "Baum",         180, 140, ("on_change",) + _ZEIGEN + _FOKUS, has_items=True),
    PaletteSpec("layout",    "Layout",       200, 120, ()),
    PaletteSpec("toolbar",   "Werkzeugleiste", 240, 32, ()),
    PaletteSpec("splitter",  "Trenner",      160,  6, _ZEIGEN),
    PaletteSpec("colorpicker", "Farbwaehler", 200, 150, ("on_change",) + _ZEIGEN + _FOKUS),
    PaletteSpec("datepicker", "Datumswaehler", 200, 180, ("on_change",) + _ZEIGEN + _FOKUS),
]

# Arten, deren Konstruktor die Groesse SELBST bestimmt -- danach muss
# `GUI_SET_BOUNDS` die im Designer eingestellte wiederherstellen, sonst geht
# sie verloren und das Anchoring rechnet mit einer falschen Basis.
_EIGENE_GROESSE = ("label", "checkbox", "radio", "slider", "separator",
                   "toggle", "splitter", "spinner", "knob")

_SPEC_BY_KIND = {p.kind: p for p in PALETTE}


def palette_spec(kind: str) -> PaletteSpec | None:
    return _SPEC_BY_KIND.get(kind)


# --- GB-Code-Emit-Helfer (fuer generate_gb_code) ----------------------------
def _knoten_liste(tj) -> list:
    """Die `nodes`-Liste aus einem `tree`-Feld -- oder eine leere."""
    if isinstance(tj, dict) and isinstance(tj.get("nodes"), list):
        return [n for n in tj["nodes"] if isinstance(n, dict)]
    return []


def _baum_knoten(c) -> list | None:
    """Knoten fuer die Datei -- aus `items` gebaut, ODER die vorhandenen.

    **Ein tieferer Baum wird NICHT ueberschrieben.** Wer eine Datei oeffnet,
    die ein Programm mit verschachtelten Knoten gespeichert hat, soll sie
    nicht dadurch verlieren, dass der Designer nur die oberste Ebene anzeigt.
    Neu gebaut wird nur, wenn alle vorhandenen Knoten auf der obersten Ebene
    liegen -- dann ist `items` die vollstaendige Wahrheit.
    """
    vorhanden = _knoten_liste(c.extra.get("tree"))
    tief = any(int(n.get("parent", -1)) >= 0 for n in vorhanden)
    if tief:
        return None                      # unveraendert durchreichen (via extra)
    if not c.items:
        return vorhanden or None
    return [{"label": str(t), "parent": -1, "level": 0,
             "expanded": False, "has_children": False} for t in c.items]


def _gb_str(s: str) -> str:
    """Drachenhauch-String-Literal: `"` wird zu `""` escaped, Zeilenumbrueche raus."""
    t = str(s).replace("\r", " ").replace("\n", " ").replace('"', '""')
    return f'"{t}"'


def _gb_hex(i: int) -> str:
    return f"&H{int(i) & 0xFFFFFF:06X}"


def hex_zu_int(t) -> int | None:
    """`"#RRGGBB"` (so schreibt die Laufzeit den Farbwert) -> Zahl."""
    if not isinstance(t, str):
        return None
    try:
        return int(t.lstrip("#"), 16) & 0xFFFFFF
    except ValueError:
        return None


def _gb_bool(b: bool) -> str:
    return "TRUE" if b else "FALSE"


def _gb_num(x) -> str:
    """FLOAT-Literal mit Dezimalpunkt (GB-FLOAT erwartet z.B. `0.0`, nicht `0`).

    Bewusst Fixpunkt statt `repr()`: `repr` schaltet ab |x| >= 1e16 bzw. unter
    1e-4 auf Exponentialschreibweise (`1e-05`), die Drachenhauchs Lexer NICHT kennt
    -- ein Slider mit feiner Skala erzeugte damit nicht parsbaren Export-Code.
    `inf`/`nan` (aus einem beschaedigten `.dhform`) werden zu 0.0."""
    try:
        f = float(x)
    except (TypeError, ValueError):
        return "0.0"
    if not math.isfinite(f):
        return "0.0"
    t = f"{f:.10f}".rstrip("0")
    return t + "0" if t.endswith(".") else t


def _gb_ident(s: str) -> str:
    """Bezeichner aus einem Control-Namen: nur [A-Za-z0-9_], nicht mit Ziffer,
    kein reserviertes Wort (sonst `_`-Suffix -- `DIM Print AS ...` waere sonst
    ein Parse-Fehler im exportierten Programm). `KEYWORDS` deckt exakt die
    Woerter ab, die dhrt als Bezeichner ablehnt (empirisch abgeglichen)."""
    t = re.sub(r"[^A-Za-z0-9_]", "_", str(s))
    if not t or t[0].isdigit():
        t = "_" + t
    if t.lower() in KEYWORDS:
        t += "_"
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
    """Manifest eines Multi-Form-Projekts (`.dhproj`, JSON). Verweist auf die
    einzelnen `.dhform`-Dateien (relativ zum Projekt-Verzeichnis) -- jede Form
    bleibt ihre eigene Datei. `main` = Startformular (einer der `forms`-Pfade).
    Qt-frei + headless testbar."""
    forms: list = field(default_factory=list)   # list[str] relative .dhform-Pfade
    main: str = ""

    @staticmethod
    def norm(rel: str) -> str:
        """Manifest-Pfade auf eine Schreibweise bringen (`/` statt `\\`, kein
        `./`). Ohne das galten `forms/a.dhform`, `forms\\a.dhform` und
        `./forms/a.dhform` als drei verschiedene Formulare -- auf Windows
        dieselbe Datei, die dann mehrfach in der Projektliste stand."""
        return Path(str(rel)).as_posix()

    def add(self, rel: str) -> None:
        rel = self.norm(rel)
        if rel not in self.forms:
            self.forms.append(rel)
        if not self.main:
            self.main = rel

    def remove(self, rel: str) -> None:
        rel = self.norm(rel)
        if rel in self.forms:
            self.forms.remove(rel)
        if self.main == rel:
            self.main = self.forms[0] if self.forms else ""

    def to_dict(self) -> dict:
        return {"forms": list(self.forms), "main": self.main}

    @classmethod
    def from_dict(cls, d: dict) -> "FormProject":
        if not isinstance(d, dict):
            d = {}
        raw = d.get("forms")
        forms: list = []
        for x in (raw if isinstance(raw, list) else []):
            n = cls.norm(x)
            if n not in forms:
                forms.append(n)
        main = cls.norm(d.get("main", "")) if d.get("main") else ""
        if main not in forms:                       # defensiv: main muss Mitglied sein
            main = forms[0] if forms else ""
        return cls(forms=forms, main=main)

    @staticmethod
    def looks_like_manifest(d) -> bool:
        """Ist dieses JSON ein `.dhproj`-Manifest (und KEIN Formular)? `FormDoc.
        from_dict` ist so permissiv, dass ein Manifest klaglos als leeres
        Formular durchginge -- ein anschliessendes Speichern hat dann die
        Projektdatei ueberschrieben."""
        return isinstance(d, dict) and "widgets" not in d and (
            "forms" in d or "main" in d)

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
    on_hover: str = ""
    on_leave: str = ""
    on_focus: str = ""
    on_blur: str = ""
    on_enter: str = ""
    # Text und Formular (gui-Ausbau Punkt 1): Ausrichtung ("" = Vorgabe der
    # Art), Umbruch (Label), Passwort/Nur-Lesen/Hoechstlaenge/Zahlenfilter
    # (TextInput) -- dieselben Schluessel wie `gui.rs::widget_json`.
    align: str = ""
    wrap: bool = False
    passwort: bool = False
    nur_lesen: bool = False
    maxlaenge: int = 0
    zahlen: int = 0
    tooltip: str = ""
    ov: dict = field(default_factory=dict)   # Farb-Overrides: bg/fg/border/accent -> int
    # Laufzeit-Felder, die der Designer (noch) nicht darstellt -- z.B. `table`,
    # `tree`, `tab_page`, `font`. Sie werden unveraendert durchgereicht, damit
    # ein mit GUI_SAVE erzeugtes Formular beim Oeffnen+Speichern im Designer
    # nichts verliert. Siehe `gui.rs::widget_json`.
    extra: dict = field(default_factory=dict)

    # Felder, die der Designer selbst kennt -- alles andere landet in `extra`.
    _KNOWN = frozenset((
        "kind", "name", "x", "y", "w", "h", "text", "color", "value", "min",
        "max", "checked", "placeholder", "group", "items", "sel", "enabled",
        "visible", "font_size", "anchor", "ov",
        "align", "wrap", "passwort", "nur_lesen", "maxlaenge", "zahlen", "tooltip",
        *EVENTS,
    ))

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
        for _ev in EVENTS:
            if getattr(self, _ev):
                d[_ev] = getattr(self, _ev)
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
        if self.align:
            d["align"] = self.align
        if self.wrap:
            d["wrap"] = True
        if self.passwort:
            d["passwort"] = True
        if self.nur_lesen:
            d["nur_lesen"] = True
        if self.maxlaenge > 0:
            d["maxlaenge"] = self.maxlaenge
        if self.zahlen:
            d["zahlen"] = self.zahlen
        if self.tooltip:
            d["tooltip"] = self.tooltip
        if self.kind == "tree":
            knoten = _baum_knoten(self)
            if knoten is not None:
                d["tree"] = {"nodes": knoten,
                             "selected": (self.extra.get("tree") or {}).get("selected", -1)}
        for k, v in self.extra.items():       # unbekannte Laufzeit-Felder zurueckgeben
            # tief kopiert: `to_dict()` liefert auch die Undo-Snapshots -- ein
            # geteiltes `table`/`menus`-Dict wuerde einen alten Snapshot
            # nachtraeglich veraendern.
            d.setdefault(k, copy.deepcopy(v))
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Control":
        if not isinstance(d, dict):
            d = {}
        ov_src = d.get("ov")
        ov = {}
        if isinstance(ov_src, dict):
            for k, v in ov_src.items():
                if isinstance(v, int) and not isinstance(v, bool):
                    ov[str(k)] = v
        items_src = d.get("items")
        items = [x for x in items_src if isinstance(x, str)] \
            if isinstance(items_src, list) else []
        # Ein von der LAUFZEIT gespeicherter Baum hat kein `items`, sondern
        # `tree.nodes`. Die oberste Ebene daraus fuellt die Liste, damit der
        # Designer zeigt, was in der Datei steht.
        if d.get("kind") == "tree" and not items:
            items = [str(n.get("label", "")) for n in _knoten_liste(d.get("tree"))
                     if int(n.get("parent", -1)) < 0]
        extra = {k: v for k, v in d.items() if k not in cls._KNOWN}
        return cls(
            kind=_as_str(d, "kind", "label") or "label",
            name=_as_str(d, "name"),
            x=_as_int(d, "x", 0), y=_as_int(d, "y", 0),
            w=_as_int(d, "w", 100), h=_as_int(d, "h", 24),
            text=_as_str(d, "text"),
            color=_as_int(d, "color", 0xFFFFFF),
            value=_as_float(d, "value", 0.0),
            min=_as_float(d, "min", 0.0), max=_as_float(d, "max", 1.0),
            checked=_as_bool(d, "checked", False),
            placeholder=_as_str(d, "placeholder"),
            group=_as_str(d, "group"),
            items=items,
            sel=_as_int(d, "sel", -1),
            enabled=_as_bool(d, "enabled", True),
            visible=_as_bool(d, "visible", True),
            font_size=_as_int(d, "font_size", 0),
            anchor=_as_str(d, "anchor", "lt") or "lt",
            align=_as_str(d, "align") if _as_str(d, "align") in ("links", "mitte", "rechts") else "",
            wrap=_as_bool(d, "wrap", False),
            passwort=_as_bool(d, "passwort", False),
            nur_lesen=_as_bool(d, "nur_lesen", False),
            maxlaenge=max(0, _as_int(d, "maxlaenge", 0)),
            zahlen=min(2, max(0, _as_int(d, "zahlen", 0))),
            tooltip=_as_str(d, "tooltip"),
            **{_ev: _as_str(d, _ev) for _ev in EVENTS},
            ov=ov,
            extra=extra,
        )

    def clone(self) -> "Control":
        return replace(self, items=list(self.items), ov=dict(self.ov),
                       extra=copy.deepcopy(self.extra))


# --- FormDoc ----------------------------------------------------------------
# Auswaehlbare gui-Themen. "" = die Vorgabe der Laufzeit (Cyan), damit ein
# altes Formular ohne Eintrag genauso aussieht wie bisher.
#
# ACHTUNG: die Farben unten sind ein NACHBAU der Presets aus
# `rust/drachenhauch_runtime/src/gui.rs` -- der Designer zeichnet mit Qt, kann also
# nicht die Laufzeit fragen. `tests/test_formdesigner_theme.py` vergleicht
# beide gegeneinander, damit sie nicht auseinanderlaufen.
FORM_THEMES = ("", "glas_dunkel", "glas_hell", "modern_dark", "modern_light",
               "dark", "light", "retro", "contrast")

# Farben + Plastik je Thema -- Nachbau von `preset()`/`preset_metrics()` aus
# gui.rs. Nur die Rollen, die der Designer zum Zeichnen braucht.
# Schluessel: win_bg, win_border, title_bg, title_fg, widget_bg,
# widget_border, text_fg, muted_fg, accent; dazu radius/gradient/gloss.
FORM_THEME_COLORS = {
    "": dict(win_bg=0x18222E, win_border=0x2E3C50, title_bg=0x123C50,
             title_fg=0xE6F7FF, widget_bg=0x26323F, widget_border=0x46586E,
             text_fg=0xFFFFFF, muted_fg=0x7A8AA0, accent=0x2BC4E8,
             radius=0, gradient=0, gloss=0),
    "dark": dict(win_bg=0x18222E, win_border=0x2E3C50, title_bg=0x123C50,
                 title_fg=0xE6F7FF, widget_bg=0x26323F, widget_border=0x46586E,
                 text_fg=0xFFFFFF, muted_fg=0x7A8AA0, accent=0x2BC4E8,
                 radius=0, gradient=0, gloss=0),
    "light": dict(win_bg=0xF4F6F9, win_border=0xA8AEB6, title_bg=0xD0D5DC,
                  title_fg=0x202428, widget_bg=0xD8DCE2, widget_border=0x9AA0A8,
                  text_fg=0x202428, muted_fg=0x90969C, accent=0x2A7DE1,
                  radius=0, gradient=0, gloss=0),
    "retro": dict(win_bg=0x020802, win_border=0x1F8C3C, title_bg=0x0A2A0A,
                  title_fg=0x33FF66, widget_bg=0x0A1A0A, widget_border=0x1F8C3C,
                  text_fg=0x33FF66, muted_fg=0x1F8C3C, accent=0x33FF66,
                  radius=0, gradient=0, gloss=0),
    "contrast": dict(win_bg=0x000000, win_border=0xFFD400, title_bg=0x202000,
                     title_fg=0xFFFFFF, widget_bg=0x000000, widget_border=0xFFD400,
                     text_fg=0xFFFFFF, muted_fg=0xAAAAAA, accent=0xFFD400,
                     radius=0, gradient=0, gloss=0),
    "modern_dark": dict(win_bg=0x1E2630, win_border=0x33414F, title_bg=0x161D26,
                        title_fg=0xEAF2F8, widget_bg=0x2A3542, widget_border=0x3C4A5A,
                        text_fg=0xEAF2F8, muted_fg=0x8493A4, accent=0x2BC4E8,
                        radius=7, gradient=0, gloss=0),
    "modern_light": dict(win_bg=0xFAFBFC, win_border=0xD8DEE6, title_bg=0xEFF2F5,
                         title_fg=0x1F2733, widget_bg=0xFFFFFF, widget_border=0xCBD3DD,
                         text_fg=0x1F2733, muted_fg=0x8A93A0, accent=0x2A7DE1,
                         radius=7, gradient=0, gloss=0),
    "glas_dunkel": dict(win_bg=0x232A33, win_border=0x151A21, title_bg=0x2C343F,
                        title_fg=0xEAF2F8, widget_bg=0x39424F, widget_border=0x161B22,
                        text_fg=0xEAF2F8, muted_fg=0x8B97A6, accent=0x2FA8D8,
                        radius=5, gradient=16, gloss=26),
    "glas_hell": dict(win_bg=0xE2E8EF, win_border=0x6E7C8C, title_bg=0xD2DAE3,
                      title_fg=0x1E2530, widget_bg=0xFBFCFE, widget_border=0x63707F,
                      text_fg=0x1E2530, muted_fg=0x66707C, accent=0x2A8FD0,
                      radius=5, gradient=16, gloss=26),
}


def theme_colors(name: str) -> dict:
    """Farb-/Plastik-Tabelle eines Themas (unbekannt -> Vorgabe)."""
    return FORM_THEME_COLORS.get(name or "", FORM_THEME_COLORS[""])


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
    theme: str = ""                                # gui-Preset ("" = Vorgabe der Laufzeit)
    controls: list = field(default_factory=list)   # list[Control]
    code: dict = field(default_factory=dict)       # Event-Handler-Koerper: name -> GB-Code
    # Fenster-Felder der Laufzeit, die der Designer nicht darstellt (`chrome`,
    # `menus`, `tabs`, `active_tab`) -- unveraendert durchgereicht, siehe
    # `Control.extra`.
    extra: dict = field(default_factory=dict)

    # Standard- und Abbrechen-Knopf als Widget-INDEX (wie in der Laufzeit);
    # -1 = keiner. Der Index zeigt in `controls`.
    default_button: int = -1
    cancel_button: int = -1

    _KNOWN = frozenset((
        "title", "x", "y", "w", "h", "movable", "closable", "visible",
        "resizable", "min_w", "min_h", "max_w", "max_h", "theme", "widgets", "code",
        "default_button", "cancel_button",
    ))

    # ---- Bearbeiten ----
    def add(self, kind: str, x: int, y: int) -> Control:
        sp = palette_spec(kind) or PaletteSpec(kind, kind, 100, 24)
        c = Control(kind=kind, x=x, y=y, w=sp.w, h=sp.h, name=self._unique_name(kind))
        if sp.has_text:
            c.text = sp.label
        if sp.has_items:
            c.items = ["Eintrag 1", "Eintrag 2", "Eintrag 3"]
            c.sel = 0 if kind == "dropdown" else -1
        if kind == "table":
            # Ohne Spalten waere die Tabelle auf der Design-Flaeche ein leeres
            # Rechteck -- man saehe nicht, was man da hingesetzt hat. Die
            # ZEILEN bleiben leer: sie kommen im Normalfall zur Laufzeit aus
            # dem Programm (Datei, Datenbank), nicht aus dem Designer.
            c.extra["table"] = {
                "headers": ["Spalte 1", "Spalte 2", "Spalte 3"],
                "rows": [],
            }
        if kind == "slider":
            c.min, c.max, c.value = 0.0, 100.0, 50.0
        self.controls.append(c)
        return c

    # Identitaets- statt Wertvergleich: `Control` ist eine dataclass mit
    # generiertem `__eq__`, `list.remove()`/`in` treffen also das ERSTE
    # feldgleiche Control. Zwei feldgleiche Controls sind leicht herstellbar --
    # eine per GUI_SAVE erzeugte `.dhform` hat gar keine `name`-Felder. Vorher
    # loeschte "das obere markieren + Entf" das untere.
    def _index_of(self, c: Control) -> int:
        for i, x in enumerate(self.controls):
            if x is c:
                return i
        return -1

    def remove(self, c: Control) -> None:
        i = self._index_of(c)
        if i >= 0:
            del self.controls[i]
            self.prune_code()

    def prune_code(self) -> None:
        """Handler-Koerper verwerfen, auf die kein Control mehr zeigt -- sonst
        wachsen `.dhform`-Dateien mit Leichen, und `_unique_handler_name` haengt
        einem neu angelegten Handler wegen des toten Namens ein `2` an."""
        live = set(self.handler_names())
        self.code = {k: v for k, v in self.code.items() if k in live}

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
        i = self._index_of(c)
        if i >= 0:
            del self.controls[i]; self.controls.append(c)

    def to_back(self, c: Control) -> None:
        """Z-Reihenfolge: an den Anfang (zuerst gezeichnet = unten)."""
        i = self._index_of(c)
        if i >= 0:
            del self.controls[i]; self.controls.insert(0, c)

    def _unique_name(self, kind: str) -> str:
        base = {"textinput": "txt", "button": "btn", "checkbox": "chk", "radio": "rad",
                "dropdown": "dd", "listbox": "lst", "slider": "sld", "label": "lbl",
                "table": "tbl",
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
            suffix = EVENTS.get(ev, "Action")
            # Durch `_gb_ident`: der Name wird als `SUB <name>()` und als
            # FUNCREF emittiert. Ein Control "OK Knopf" erzeugte sonst
            # `SUB OK KnopfClick()` -- Parse-Fehler, und weil F5 die
            # dhrt-Ausgabe verwarf, passierte einfach gar nichts.
            name = self._unique_handler_name(_gb_ident((c.name or c.kind) + suffix))
            setattr(c, ev, name)
        self.code.setdefault(name, "")
        return name

    def rename_control_handlers(self, c: Control, old_name: str) -> list[tuple[str, str]]:
        """Handler mitziehen, wenn ein Control umbenannt wird.

        Betroffen sind NUR Handler, die noch ihren aus dem alten Control-Namen
        abgeleiteten Namen tragen (`btn1` -> `btn1Click`). Ein selbst
        vergebener Name bleibt unangetastet -- wer seinen Handler bewusst
        `SpielStarten` genannt hat, will ihn nicht beim Umbenennen des Knopfes
        verlieren.

        Der Code-Rumpf wandert mit, sonst laege er unter dem alten Schluessel
        als unerreichbare Leiche herum und der Export erzeugte fuer den neuen
        Namen nur ein `' TODO`.

        Liefert die (alt, neu)-Paare.
        """
        umbenannt: list[tuple[str, str]] = []
        if not old_name or not c.name or old_name == c.name:
            return umbenannt
        for ev, suffix in EVENTS.items():
            jetzt = getattr(c, ev)
            if not jetzt or jetzt != _gb_ident(old_name + suffix):
                continue
            neu = self._unique_handler_name(_gb_ident(c.name + suffix))
            if neu == jetzt:
                continue
            setattr(c, ev, neu)
            if jetzt in self.code and neu not in self.code:
                self.code[neu] = self.code.pop(jetzt)
            umbenannt.append((jetzt, neu))
        return umbenannt

    def _unique_handler_name(self, base: str) -> str:
        used = set(self.code.keys())
        for c in self.controls:
            for ev in EVENTS:
                if getattr(c, ev):
                    used.add(getattr(c, ev))
        if base not in used:
            return base
        i = 2
        while f"{base}{i}" in used:
            i += 1
        return f"{base}{i}"

    # ---- .dhform IO (Runtime-Format) ----
    # ---- Layout-Behaelter ----
    # Im Designer haengen die Kinder eines Behaelters an NAMEN (in
    # `extra["layout"]["kinder"]` als [name | None, gewicht]); die Laufzeit
    # will Indizes. Umgerechnet wird nur an der Dateigrenze -- so ueberleben
    # Zuordnungen das Loeschen und Umsortieren von Controls, ohne dass jede
    # dieser Operationen Indizes nachfuehren muesste. None ist ein Leerraum.
    def layout_von(self, c: "Control"):
        """Der Behaelter, in dem `c` steckt, und sein Gewicht -- oder (None, 0)."""
        for l in self.controls:
            if l.kind != "layout":
                continue
            for k in (l.extra.get("layout") or {}).get("kinder") or []:
                if isinstance(k, (list, tuple)) and k and k[0] == c.name:
                    return l, int(k[1]) if len(k) > 1 else 0
        return None, 0

    def layout_zuordnen(self, c: "Control", layout: "Control | None", gewicht: int = 0) -> None:
        """`c` in `layout` legen (oder mit None herausnehmen)."""
        for l in self.controls:
            if l.kind != "layout":
                continue
            lj = l.extra.get("layout")
            if isinstance(lj, dict) and isinstance(lj.get("kinder"), list):
                lj["kinder"] = [k for k in lj["kinder"] if not (isinstance(k, (list, tuple)) and k and k[0] == c.name)]
        if layout is not None and layout is not c:
            lj = layout.extra.setdefault("layout", {"art": "spalte"})
            lj.setdefault("kinder", []).append([c.name, int(gewicht)])

    def _kinder_zu_indizes(self, widgets: list) -> None:
        namen = {c.name: i for i, c in enumerate(self.controls)}
        for c, d in zip(self.controls, widgets):
            if c.kind != "layout":
                continue
            lj = dict(c.extra.get("layout") or {"art": "spalte"})
            kinder = []
            for k in lj.get("kinder") or []:
                if not isinstance(k, (list, tuple)) or not k:
                    continue
                g = int(k[1]) if len(k) > 1 and isinstance(k[1], (int, float)) else 0
                if k[0] is None:
                    kinder.append([-1, g])
                elif k[0] in namen:
                    kinder.append([namen[k[0]], g])
            lj["kinder"] = kinder
            d["layout"] = lj

    @staticmethod
    def _kinder_zu_namen(controls: list) -> None:
        for c in controls:
            if c.kind != "layout":
                continue
            lj = c.extra.get("layout")
            if not isinstance(lj, dict):
                continue
            kinder = []
            for k in lj.get("kinder") or []:
                if not isinstance(k, (list, tuple)) or not k:
                    continue
                g = int(k[1]) if len(k) > 1 and isinstance(k[1], (int, float)) else 0
                idx = k[0]
                if isinstance(idx, str):            # schon Name (Designer-Datei)
                    kinder.append([idx, g])
                elif isinstance(idx, (int, float)) and int(idx) < 0:
                    kinder.append([None, g])
                elif isinstance(idx, (int, float)) and 0 <= int(idx) < len(controls):
                    kinder.append([controls[int(idx)].name, g])
            # Kopie statt in place: `lj` ist womoeglich noch das Dict des
            # Aufrufers (from_dict kopiert extra nicht tief).
            c.extra["layout"] = dict(lj, kinder=kinder)

    def to_dict(self) -> dict:
        d: dict = {
            "title": self.title, "x": self.x, "y": self.y, "w": self.w, "h": self.h,
            "movable": self.movable, "closable": self.closable, "visible": self.visible,
            "resizable": self.resizable,
            "widgets": [c.to_dict() for c in self.controls],
        }
        self._kinder_zu_indizes(d["widgets"])
        if self.theme:
            # Nur schreiben, wenn gesetzt -- sonst bekaeme jede alte Datei beim
            # blossen Oeffnen+Speichern ein neues Feld.
            d["theme"] = self.theme
        for k in ("min_w", "min_h", "max_w", "max_h"):   # nur wenn gesetzt
            if getattr(self, k):
                d[k] = getattr(self, k)
        if self.code:
            # Designer-Metadaten (Handler-Koerper); die Runtime ignoriert `code`.
            d["code"] = {str(k): str(v) for k, v in self.code.items()}
        if 0 <= self.default_button < len(self.controls):
            d["default_button"] = self.default_button
        if 0 <= self.cancel_button < len(self.controls):
            d["cancel_button"] = self.cancel_button
        for k, v in self.extra.items():       # unbekannte Laufzeit-Felder zurueckgeben
            d.setdefault(k, copy.deepcopy(v))
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "FormDoc":
        if not isinstance(d, dict):
            d = {}
        code_src = d.get("code")
        code = {str(k): v for k, v in code_src.items() if isinstance(v, str)} \
            if isinstance(code_src, dict) else {}
        doc = cls(
            title=_as_str(d, "title", "Form1"),
            x=_as_int(d, "x", 200), y=_as_int(d, "y", 120),
            w=_as_int(d, "w", 360), h=_as_int(d, "h", 260),
            movable=_as_bool(d, "movable", True),
            # Default `False` wie `Gui::from_json` (gui.rs) -- der Designer zeigte
            # sonst bei einer Datei ohne `closable` ein Schliessen-Kreuz an, das
            # die laufende Form nicht hat. Neue Formulare bleiben schliessbar
            # (Dataclass-Default), denn `to_dict` schreibt das Feld immer.
            closable=_as_bool(d, "closable", False),
            visible=_as_bool(d, "visible", True),
            resizable=_as_bool(d, "resizable", False),
            min_w=_as_int(d, "min_w", 0), min_h=_as_int(d, "min_h", 0),
            max_w=_as_int(d, "max_w", 0), max_h=_as_int(d, "max_h", 0),
            default_button=_as_int(d, "default_button", -1),
            cancel_button=_as_int(d, "cancel_button", -1),
            theme=_as_str(d, "theme", ""),
            code=code,
            extra={k: v for k, v in d.items() if k not in cls._KNOWN},
        )
        widgets = d.get("widgets")
        doc.controls = [Control.from_dict(w) for w in widgets] \
            if isinstance(widgets, list) else []
        FormDoc._kinder_zu_namen(doc.controls)
        return doc

    def save(self, path: str) -> None:
        Path(path).write_text(json.dumps(self.to_dict(), indent=2, ensure_ascii=False),
                              encoding="utf-8")

    @classmethod
    def load(cls, path: str) -> "FormDoc":
        d = json.loads(Path(path).read_text(encoding="utf-8"))
        if FormProject.looks_like_manifest(d):
            raise ValueError(f"{Path(path).name} ist ein Projekt-Manifest "
                             "(.dhproj), kein Formular")
        return cls.from_dict(d)

    # ---- Code-Generierung ----
    def handler_names(self) -> list[str]:
        """Alle eindeutigen Event-Handler-Namen (Reihenfolge der Controls)."""
        seen: list[str] = []
        for c in self.controls:
            for h in (getattr(c, ev) for ev in EVENTS):
                if h and h not in seen:
                    seen.append(h)
        return seen

    def generate_runner(self, form_filename: str, screen_w: int | None = None,
                        screen_h: int | None = None, screen_title: str | None = None,
                        handler_bodies: dict | None = None) -> str:
        """Lauffaehiges Drachenhauch-Programm: das Fenster wird auf Formulargroesse
        gesetzt, das `.dhform` geladen und **randlos** (chromeless) auf das echte
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
        if self.theme:
            # VOR dem Laden der Form: GUI_THEME_PRESET setzt auch die Metriken
            # (Eckenradius, Verlauf), und die gehen in die Groessen der
            # Widgets ein.
            lines.append(f"GUI_THEME_PRESET({_gb_str(self.theme)})")
        if self.resizable:
            lines.append("WINDOW_RESIZABLE(TRUE)")
            if self.min_w or self.min_h:
                lines.append(f"WINDOW_MIN_SIZE({self.min_w or 1}, {self.min_h or 1})")
            if self.max_w or self.max_h:
                # `or 32000` wie beim MIN-Pendant: `Graphics::window_max_size`
                # reicht eine 0 direkt an GLFW durch (anders als die
                # GUI-Variante, die >0 als "gesetzt" prueft) -- bei nur einer
                # gesetzten Grenze waere das Fenster in der anderen Achse
                # auf 0 geklemmt.
                lines.append(f"WINDOW_MAX_SIZE({self.max_w or 32000}, "
                             f"{self.max_h or 32000})")
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
    def _gb_menus(self) -> list[str]:
        """GUI_MENU/GUI_CONTEXT/GUI_SUBMENU/GUI_MENU_ITEM-Zeilen aus `extra["menus"]`
        (dem Format von gui.rs::to_json, Untermenues verschachtelt)."""
        menus = self.extra.get("menus")
        if not isinstance(menus, list) or not menus:
            return []
        out: list[str] = []
        zaehler = [0]

        def eintraege(var: str, items) -> None:
            for it in items or []:
                if not isinstance(it, dict):
                    continue
                if it.get("separator"):
                    out.append(f"GUI_MENU_SEPARATOR({var})")
                    continue
                label = str(it.get("label", ""))
                if isinstance(it.get("items"), list):
                    zaehler[0] += 1
                    sv = f"{var}_u{zaehler[0]}"
                    out.append(f"DIM {sv} AS INTEGER : {sv} = GUI_SUBMENU({var}, {_gb_str(label)})")
                    eintraege(sv, it["items"])
                    continue
                zaehler[0] += 1
                iv = f"{var}_e{zaehler[0]}"
                kuerzel = str(it.get("shortcut") or "")
                args = f"{var}, {_gb_str(label)}" + (f", {_gb_str(kuerzel)}" if kuerzel else "")
                out.append(f"DIM {iv} AS INTEGER : {iv} = GUI_MENU_ITEM({args})")
                if it.get("checkable"):
                    out.append(f"GUI_MENU_CHECK({iv}, {_gb_bool(bool(it.get('checked')))})")
                if it.get("enabled") is False:
                    out.append(f"GUI_MENU_ENABLE({iv}, FALSE)")

        for k, m in enumerate(menus):
            if not isinstance(m, dict):
                continue
            mv = f"menue{k + 1}"
            if m.get("in_bar", True):
                out.append(f"DIM {mv} AS INTEGER : {mv} = GUI_MENU(frm, {_gb_str(str(m.get('label', '')))})")
            else:
                out.append(f"DIM {mv} AS INTEGER : {mv} = GUI_CONTEXT(frm)")
            eintraege(mv, m.get("items"))
        if out:
            out.append("")
        return out

    @staticmethod
    def _gb_text_extras(c: "Control", var: str) -> list[str]:
        """Ausrichtung, Umbruch, Textfeld-Grenzen, Tooltip -- nur, wo gesetzt."""
        out: list[str] = []
        if c.align and c.kind in ("label", "button", "textinput"):
            out.append(f"GUI_SET_ALIGN({var}, {_gb_str(c.align)})")
        if c.wrap and c.kind == "label":
            out.append(f"GUI_SET_WRAP({var}, {c.w})")
        if c.kind == "textinput":
            for schluessel, wert in (("passwort", int(c.passwort)), ("nur_lesen", int(c.nur_lesen)),
                                     ("maxlaenge", c.maxlaenge), ("zahlen", c.zahlen)):
                if wert:
                    out.append(f"GUI_TEXTINPUT_SET({var}, {_gb_str(schluessel)}, {wert})")
        if c.tooltip:
            out.append(f"GUI_TOOLTIP({var}, {_gb_str(c.tooltip)})")
        if c.kind == "listbox":
            lj_roh = c.extra.get("list")
            lj: dict = lj_roh if isinstance(lj_roh, dict) else {}
            if lj.get("multi"):
                out.append(f'GUI_LISTBOX_SET({var}, "mehrfachauswahl", 1)')
            if lj.get("kaestchen"):
                out.append(f'GUI_LISTBOX_SET({var}, "kaestchen", 1)')
            for i, an in enumerate(lj.get("checked") or []):
                if an:
                    out.append(f"GUI_LISTBOX_SET_CHECKED({var}, {i}, TRUE)")
        return out

    def generate_gb_code(self, screen_w: int = 800, screen_h: int = 480,
                         screen_title: str | None = None,
                         handler_bodies: dict | None = None,
                         with_screen: bool = True, with_loop: bool = True) -> str:
        """Eigenstaendiges Drachenhauch-Programm, das das Formular **explizit** mit
        den `GUI_*`-Konstruktoren aufbaut (kein `GUI_LOAD`/`.dhform` zur Laufzeit).
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
        # Beide Richtungen emittieren: `GUI_WINDOW` legt `closable: false` an
        # (gui.rs::new_window), `GUI_LOAD` liest den `.dhform`-Wert. Ohne den
        # Positivfall fehlte dem exportierten Fenster das Schliessen-Kreuz,
        # obwohl im Designer "schliessbar" angehakt war.
        L.append(f"GUI_WINDOW_CLOSABLE(frm, {_gb_bool(self.closable)})")
        if not self.visible:
            L.append("GUI_WINDOW_VISIBLE(frm, FALSE)")
        if self.resizable:
            L.append("GUI_WINDOW_RESIZABLE(frm, TRUE)")
        if self.min_w or self.min_h:
            L.append(f"GUI_WINDOW_SET_MIN_SIZE(frm, {self.min_w}, {self.min_h})")
        if self.max_w or self.max_h:
            L.append(f"GUI_WINDOW_SET_MAX_SIZE(frm, {self.max_w}, {self.max_h})")
        L.append("")
        # Menues (aus `extra`, der Designer bearbeitet sie nicht -- aber ein
        # geladenes Formular soll sie im erzeugten Programm nicht verlieren).
        L.extend(self._gb_menus())
        # Controls
        used = {"frm"} | set(self.handler_names())
        vars_: dict[int, str] = {}
        for idx, c in enumerate(self.controls):
            var = self._gb_var(c, idx, used)
            vars_[idx] = var
            block = self._gb_control(c, var)
            if block:
                L.extend(block)
                L.extend(self._gb_text_extras(c, var))
                L.append("")
        # Layout-Kinder -- nach den Controls, sie brauchen die Handles.
        namen = {c.name: i for i, c in enumerate(self.controls)}
        for idx, c in enumerate(self.controls):
            if c.kind != "layout" or idx not in vars_:
                continue
            for k in (c.extra.get("layout") or {}).get("kinder") or []:
                if not isinstance(k, (list, tuple)) or not k:
                    continue
                g = int(k[1]) if len(k) > 1 and isinstance(k[1], (int, float)) else 0
                if k[0] is None:
                    L.append(f"GUI_LAYOUT_SPACER({vars_[idx]}, {max(1, g)})")
                elif k[0] in namen and namen[k[0]] in vars_:
                    L.append(f"GUI_LAYOUT_ADD({vars_[idx]}, {vars_[namen[k[0]]]}, {g})")
        # Standard- und Abbrechen-Knopf -- nach den Controls, sie brauchen die Handles.
        for feld, befehl in (("default_button", "GUI_WINDOW_DEFAULT"),
                             ("cancel_button", "GUI_WINDOW_CANCEL")):
            i = getattr(self, feld)
            if 0 <= i < len(self.controls) and self.controls[i].kind == "button" and i in vars_:
                L.append(f"{befehl}(frm, {vars_[i]})")
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
                    f"Bildquelle (LOADIMAGE), die das .dhform nicht speichert"]
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
        elif k == "table":
            out.append(f"{var} = GUI_TABLE(frm, {c.x}, {c.y}, {c.w}, {c.h})")
            tj = c.extra.get("table") or {}
            kopf = [str(s) for s in (tj.get("headers") or [])]
            if kopf:
                hv = var + "_kopf"
                out.append(f"DIM {hv}[{len(kopf)}] AS STRING")
                for j, s in enumerate(kopf):
                    out.append(f"{hv}[{j}] = {_gb_str(s)}")
                out.append(f"GUI_TABLE_HEADERS({var}, {hv})")
            br = [int(x) for x in (tj.get("col_widths") or []) if isinstance(x, (int, float))]
            if br:
                bv = var + "_breiten"
                out.append(f"DIM {bv}[{len(br)}] AS INTEGER")
                for j, x in enumerate(br):
                    out.append(f"{bv}[{j}] = {x}")
                out.append(f"GUI_TABLE_COL_WIDTHS({var}, {bv})")
            # Nur ausgeben, was vom Standard abweicht -- sonst stuenden zehn
            # GUI_TABLE_SET-Zeilen in jedem erzeugten Programm.
            for schluessel, feld, standard in (
                    ("zeilenhoehe", "row_h", 20), ("kopfhoehe", "header_h", 22),
                    ("zebra", "zebra", False), ("gitter", "grid", True),
                    ("filterzeile", "filter_row", False), ("sortierbar", "sortable", True),
                    ("spalten_ziehbar", "resizable_cols", True),
                    ("spalten_verschiebbar", "reorderable", False),
                    ("mehrfachauswahl", "multi", False),
                    ("feste_spalten", "frozen", 0)):
                wert = tj.get(feld, standard)
                if wert != standard:
                    out.append(f"GUI_TABLE_SET({var}, {_gb_str(schluessel)}, {int(wert)})")
            for j, an in enumerate(tj.get("col_edit") or []):
                if an:
                    out.append(f"GUI_TABLE_COL_EDIT({var}, {j}, TRUE)")
            # Zeilen bleiben dem Programm ueberlassen: eine Tabelle wird im
            # Normalfall zur Laufzeit gefuellt (Datei, Datenbank), nicht im
            # Designer abgetippt.
            out.append(f"' {var}: Zeilen zur Laufzeit fuellen -- GUI_TABLE_ADD_ROW / GUI_TABLE_ROWS")
        elif k == "textarea":
            out.append(f"{var} = GUI_TEXTAREA(frm, {c.x}, {c.y}, {c.w}, {c.h}, "
                       f"{_gb_str(c.placeholder)})")
        elif k == "spinner":
            # GUI_SPINNER kennt keine Hoehe -- die Zeile darunter holt sie zurueck.
            out.append(f"{var} = GUI_SPINNER(frm, {c.x}, {c.y}, {c.w}, "
                       f"{_gb_num(c.min)}, {_gb_num(c.max)}, {_gb_num(c.value)})")
        elif k == "knob":
            # Ein Drehknopf ist rund: EIN Mass, keine Breite/Hoehe.
            out.append(f"{var} = GUI_KNOB(frm, {c.x}, {c.y}, {min(c.w, c.h)}, "
                       f"{_gb_num(c.min)}, {_gb_num(c.max)}, {_gb_num(c.value)})")
        elif k == "toggle":
            out.append(f"{var} = GUI_TOGGLE(frm, {_gb_str(c.text)}, {c.x}, {c.y}, "
                       f"{_gb_bool(c.checked)})")
        elif k == "toolbar":
            out.append(f"{var} = GUI_TOOLBAR(frm, {c.x}, {c.y}, {c.w}, {c.h})")
        elif k == "splitter":
            # `text` traegt die Richtung ("h"/"v"), min/max die Grenzen, in die
            # der Nutzer den Trenner schieben darf.
            richtung = (c.text or "h").lower()
            if richtung not in ("h", "v"):
                richtung = "h"
            laenge = c.h if richtung == "v" else c.w
            mn, mx = int(c.min), int(c.max)
            if mx <= mn:
                mn, mx = 40, max(80, laenge)
            out.append(f"{var} = GUI_SPLITTER(frm, {c.x}, {c.y}, {laenge}, "
                       f"{_gb_str(richtung)}, {mn}, {mx})")
        elif k == "tree":
            out.append(f"{var} = GUI_TREE(frm, {c.x}, {c.y}, {c.w}, {c.h})")
            # `items` sind die Knoten der obersten Ebene. Tiefere Baeume baut
            # das Programm zur Laufzeit -- ein Baum ist Daten, kein Layout.
            for it in c.items:
                out.append(f"GUI_TREE_ADD({var}, -1, {_gb_str(it)})")
            if c.items:
                out.append(f"' {var}: tiefere Ebenen mit GUI_TREE_ADD({var}, elternIndex, text$)")
        elif k == "colorpicker":
            out.append(f"{var} = GUI_COLORPICKER(frm, {c.x}, {c.y}, {c.w}, {c.h})")
            f = hex_zu_int(c.extra.get("color_value"))
            if f is not None:
                out.append(f"GUI_SET_PICKED_COLOR({var}, {_gb_hex(f)})")
        elif k == "layout":
            yj0 = c.extra.get("layout")
            lj: dict = yj0 if isinstance(yj0, dict) else {}
            art = str(lj.get("art") or "spalte")
            if art == "raster":
                art = f"raster:{int(lj.get('spalten') or 2)}"
            out.append(f"{var} = GUI_LAYOUT(frm, {_gb_str(art)}, {c.x}, {c.y}, {c.w}, {c.h})")
            for schluessel, standard in (("abstand", 6), ("rand", 0), ("ausrichtung", 0)):
                wert = lj.get(schluessel, standard)
                if isinstance(wert, (int, float)) and wert != standard:
                    out.append(f"GUI_LAYOUT_SET({var}, {_gb_str(schluessel)}, {int(wert)})")
            if lj.get("dehnen") is False:
                out.append(f'GUI_LAYOUT_SET({var}, "dehnen", 0)')
        elif k == "datepicker":
            out.append(f"{var} = GUI_DATEPICKER(frm, {c.x}, {c.y}, {c.w}, {c.h})")
            d = c.extra.get("date")
            # Ohne Datum zeigt ein frischer Waehler HEUTE -- das ist fast immer
            # gewollt und darf nicht durch ein eingefrorenes Datum ersetzt werden.
            if isinstance(d, str) and d:
                out.append(f"GUI_SET_DATE({var}, {_gb_str(d)})")
        elif k in ("dropdown", "listbox"):
            iv = var + "_items"
            out.append(f"DIM {iv}[{len(c.items)}] AS STRING")   # 1D ARRAY OF STRING
            for j, it in enumerate(c.items):
                out.append(f"{iv}[{j}] = {_gb_str(it)}")
            ctor = "GUI_DROPDOWN" if k == "dropdown" else "GUI_LISTBOX"
            out.append(f"{var} = {ctor}(frm, {c.x}, {c.y}, {c.w}, {c.h}, {iv})")
        else:
            return [f"' Control-Typ '{k}' uebersprungen (kein Konstruktor)"]
        # Diese Konstruktoren berechnen ihre Groesse selbst (Label aus der
        # Textlaenge, Checkbox/Radio aus `check_size`, Slider/Separator aus den
        # Metriken) -- die im Designer eingestellte Groesse ginge sonst verloren
        # und das Anchoring rechnete mit einer falschen Basis.
        if k in _EIGENE_GROESSE:
            out.append(f"GUI_SET_BOUNDS({var}, {c.x}, {c.y}, {c.w}, {c.h})")
        # Nachbearbeitung (nur abweichende Werte)
        if k == "dropdown" and c.sel not in (-1, 0):
            out.append(f"GUI_DROPDOWN_SET_SELECTED({var}, {c.sel})")
        if k == "listbox" and c.sel >= 0:
            out.append(f"GUI_LISTBOX_SET_SELECTED({var}, {c.sel})")
        if k == "progress":
            # `GUI_PROGRESS` legt den Balken fest auf min=0/max=1 an und kennt
            # keinen Range-Setter; `GUI_SET_VALUE` clampt auf [min,max]. Ein
            # roher Wert 25 (bei max=100) wurde dadurch zu 1.0 = randvoll.
            # Deshalb auf den Anteil normieren -- optisch identisch zum
            # `.dhform`-Weg, den die Laufzeit als (value-min)/(max-min) zeichnet.
            span = c.max - c.min
            frac = 0.0 if span == 0 else min(1.0, max(0.0, (c.value - c.min) / span))
            if (c.min, c.max) != (0.0, 1.0):
                out.append(f"' {var}: GUI_PROGRESS rechnet in 0..1 -- "
                           f"{_gb_num(c.value)} von {_gb_num(c.min)}..{_gb_num(c.max)} "
                           f"entspricht {_gb_num(frac)}")
            if frac != 0.0:
                out.append(f"GUI_SET_VALUE({var}, {_gb_num(frac)})")
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
        # Alle Ereignisse ueber EVENTS -- so faellt beim Hinzufuegen eines
        # weiteren keines aus dem erzeugten Programm heraus.
        for ev in EVENTS:
            name = getattr(c, ev)
            if name:
                out.append(f"{ev.upper().replace('ON_', 'GUI_ON_', 1)}({var}, {name})")
        return out
