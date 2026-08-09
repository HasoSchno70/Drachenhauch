"""Qt-freies Datenmodell des Animations-FSM-Editors `dhanim`.

Eine Animation-State-Machine im Unity-Mecanim-Stil:

* **States** -- Knoten im Graph, je an eine Sprite-Animation (`anim`) gebunden,
  mit optionaler Frame-Range (`first`/`last`/`fps`) und Editor-Position (`x`/`y`).
* **Parameter** -- benannte Werte (`bool`/`float`/`int`/`trigger`), die das Spiel
  pro Frame setzt.
* **Transitions** -- gerichtete Kanten (`from` -> `to`, `from == "*"` = Any State)
  mit UND-verknuepften **Conditions** und optionalem `wait_finished`.

Serialisiert als `.gbanim`-JSON, das das Runtime-Modul `animfsm` direkt laedt
(`ANIM_FSM_LOAD`). Reine Logik -- headless testbar, kein Qt.
"""
from __future__ import annotations

import json
import math
from dataclasses import dataclass, field
from pathlib import Path

ANY_STATE = "*"
GRID = 8

# Parameter-Typen und Vergleichs-Operatoren (Single-Source-of-Truth fuer den
# Editor: Dropdowns ziehen daraus; muessen mit animfsm.rs uebereinstimmen).
PARAM_TYPES = ("bool", "float", "int", "trigger")
OPS = ("gt", "lt", "ge", "le", "eq", "ne", "is_true", "is_false", "trigger")

# Menschenlesbare Beschriftungen fuer die Operator-Dropdowns.
OP_LABELS = {
    "gt": ">", "lt": "<", "ge": ">=", "le": "<=", "eq": "=", "ne": "<>",
    "is_true": "ist wahr", "is_false": "ist falsch", "trigger": "ausgeloest",
}


def snap(v: int, grid: int = GRID) -> int:
    """Rundet `v` aufs naechste Vielfache von `grid` (round-half-up, symmetrisch)."""
    return int(math.floor(v / grid + 0.5)) * grid


def unique_name(base: str, taken: set[str]) -> str:
    """Liefert `base`, sonst `base2`, `base3`, ... bis frei von `taken`."""
    if base not in taken:
        return base
    i = 2
    while f"{base}{i}" in taken:
        i += 1
    return f"{base}{i}"


# --- Bausteine --------------------------------------------------------------
@dataclass
class Param:
    name: str
    ptype: str = "float"        # bool | float | int | trigger
    default: float = 0.0        # numerisch gespeichert (bool via != 0)

    def to_dict(self) -> dict:
        d: dict = {"name": self.name, "type": self.ptype}
        if self.ptype == "bool":
            d["default"] = bool(self.default)
        elif self.ptype == "int":
            d["default"] = int(self.default)
        elif self.ptype == "float":
            d["default"] = float(self.default)
        # trigger: kein default
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Param":
        ptype = str(d.get("type", "float"))
        dv = d.get("default", 0.0)
        if isinstance(dv, bool):
            num = 1.0 if dv else 0.0
        else:
            try:
                num = float(dv)
            except (TypeError, ValueError):
                num = 0.0
        return cls(name=str(d.get("name", "")), ptype=ptype, default=num)


@dataclass
class Condition:
    param: str
    op: str = "gt"
    value: float = 0.0

    def to_dict(self) -> dict:
        d: dict = {"param": self.param, "op": self.op}
        if self.op in ("gt", "lt", "ge", "le", "eq", "ne"):
            d["value"] = float(self.value)
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Condition":
        return cls(
            param=str(d.get("param", "")),
            op=str(d.get("op", "trigger")),
            value=float(d.get("value", 0.0) or 0.0),
        )


@dataclass
class Transition:
    from_state: str
    to_state: str
    wait_finished: bool = False
    conditions: list = field(default_factory=list)  # list[Condition]

    def to_dict(self) -> dict:
        d: dict = {"from": self.from_state, "to": self.to_state}
        if self.wait_finished:
            d["wait_finished"] = True
        d["conditions"] = [c.to_dict() for c in self.conditions]
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "Transition":
        return cls(
            from_state=str(d.get("from", ANY_STATE)),
            to_state=str(d.get("to", "")),
            wait_finished=bool(d.get("wait_finished", False)),
            conditions=[Condition.from_dict(c) for c in d.get("conditions", [])],
        )


@dataclass
class State:
    name: str
    anim: str = ""
    loop: bool = True
    first: int = 0
    last: int = 0
    fps: float = 8.0
    x: int = 120
    y: int = 120

    @property
    def anim_name(self) -> str:
        return self.anim or self.name

    def to_dict(self) -> dict:
        d: dict = {
            "name": self.name,
            "anim": self.anim_name,
            "loop": self.loop,
            "first": int(self.first),
            "last": int(self.last),
            "fps": float(self.fps),
            "x": int(self.x),
            "y": int(self.y),
        }
        return d

    @classmethod
    def from_dict(cls, d: dict) -> "State":
        name = str(d.get("name", ""))
        return cls(
            name=name,
            anim=str(d.get("anim", name) or name),
            loop=bool(d.get("loop", True)),
            first=int(d.get("first", 0)),
            last=int(d.get("last", 0)),
            fps=float(d.get("fps", 8.0)),
            x=int(d.get("x", 120)),
            y=int(d.get("y", 120)),
        )


# --- Dokument ---------------------------------------------------------------
@dataclass
class AnimDoc:
    sheet: str = ""             # Sprite-Sheet-Bildpfad (Editor/Runner)
    frame_w: int = 16
    frame_h: int = 16
    scale: float = 4.0
    default_state: str = ""
    params: list = field(default_factory=list)       # list[Param]
    states: list = field(default_factory=list)       # list[State]
    transitions: list = field(default_factory=list)  # list[Transition]

    # ---- Abfragen ----
    def state_names(self) -> set[str]:
        return {s.name for s in self.states}

    def state_by_name(self, name: str):
        for s in self.states:
            if s.name == name:
                return s
        return None

    def param_by_name(self, name: str):
        for p in self.params:
            if p.name == name:
                return p
        return None

    def effective_default(self) -> str:
        if self.default_state and self.state_by_name(self.default_state):
            return self.default_state
        return self.states[0].name if self.states else ""

    # ---- Mutationen ----
    def add_state(self, x: int, y: int, name: str | None = None) -> State:
        nm = unique_name(name or "state", self.state_names())
        # Sinnvolle Default-Frames: eine Frame mehr als das letzte `last`
        # (statt immer bei 0 zu starten) -- spart Klicks, wenn mehrere
        # States nacheinander auf ein gemeinsames Sheet gelegt werden.
        first = self.states[-1].last + 1 if self.states else 0
        s = State(name=nm, anim=nm, x=int(x), y=int(y), first=first, last=first)
        self.states.append(s)
        if not self.default_state:
            self.default_state = nm
        return s

    def can_rename_state(self, old: str, new: str) -> bool:
        """Reine Vorab-Pruefung (keine Mutation) -- damit UI-Code vor einem
        Undo-Snapshot wissen kann, ob `rename_state` ueberhaupt etwas tun
        wird (sonst landet ein wirkungsloser Snapshot auf dem Undo-Stack)."""
        new = new.strip()
        if not new or new == ANY_STATE:
            return False
        if new in self.state_names() - {old}:
            return False
        return self.state_by_name(old) is not None

    def rename_state(self, old: str, new: str) -> bool:
        if not self.can_rename_state(old, new):
            return False
        new = new.strip()
        s = self.state_by_name(old)
        if s.anim == old:        # Animation folgte dem Namen -> mitziehen
            s.anim = new
        s.name = new
        for t in self.transitions:
            if t.from_state == old:
                t.from_state = new
            if t.to_state == old:
                t.to_state = new
        if self.default_state == old:
            self.default_state = new
        return True

    def remove_state(self, name: str) -> None:
        self.states = [s for s in self.states if s.name != name]
        self.transitions = [
            t for t in self.transitions
            if t.from_state != name and t.to_state != name
        ]
        if self.default_state == name:
            self.default_state = self.states[0].name if self.states else ""

    def can_add_transition(self, from_state: str, to_state: str) -> bool:
        """Reine Vorab-Pruefung (keine Mutation), analog `can_rename_state`."""
        if to_state == ANY_STATE or not to_state:
            return False
        if from_state != ANY_STATE and self.state_by_name(from_state) is None:
            return False
        if self.state_by_name(to_state) is None:
            return False
        return from_state != to_state

    def add_transition(self, from_state: str, to_state: str) -> Transition | None:
        if not self.can_add_transition(from_state, to_state):
            return None
        t = Transition(from_state=from_state, to_state=to_state)
        self.transitions.append(t)
        return t

    def remove_transition(self, t: Transition) -> None:
        self.transitions = [x for x in self.transitions if x is not t]

    def add_param(self, name: str, ptype: str = "float") -> Param:
        nm = unique_name(name or "param", {p.name for p in self.params})
        p = Param(name=nm, ptype=ptype)
        self.params.append(p)
        return p

    def rename_param(self, old: str, new: str) -> bool:
        new = new.strip()
        if not new or new in {p.name for p in self.params} - {old}:
            return False
        p = self.param_by_name(old)
        if p is None:
            return False
        p.name = new
        for t in self.transitions:
            for c in t.conditions:
                if c.param == old:
                    c.param = new
        return True

    def remove_param(self, name: str) -> None:
        self.params = [p for p in self.params if p.name != name]
        # Conditions, die diesen Parameter nutzen, entfernen.
        for t in self.transitions:
            t.conditions = [c for c in t.conditions if c.param != name]

    # ---- Serialisierung ----
    def to_dict(self) -> dict:
        return {
            "version": 1,
            # Editor-/Runner-Felder (Runtime ignoriert sie):
            "sheet": self.sheet,
            "frame_w": int(self.frame_w),
            "frame_h": int(self.frame_h),
            "scale": float(self.scale),
            # Runtime-Felder:
            "default": self.effective_default(),
            "params": [p.to_dict() for p in self.params],
            "states": [s.to_dict() for s in self.states],
            "transitions": [t.to_dict() for t in self.transitions],
        }

    @classmethod
    def from_dict(cls, d: dict) -> "AnimDoc":
        doc = cls(
            sheet=str(d.get("sheet", "")),
            frame_w=int(d.get("frame_w", 16)),
            frame_h=int(d.get("frame_h", 16)),
            scale=float(d.get("scale", 4.0)),
            default_state=str(d.get("default", "")),
            params=[Param.from_dict(p) for p in d.get("params", [])],
            states=[State.from_dict(s) for s in d.get("states", [])],
            transitions=[Transition.from_dict(t) for t in d.get("transitions", [])],
        )
        return doc

    def save(self, path: str) -> None:
        Path(path).write_text(
            json.dumps(self.to_dict(), indent=2, ensure_ascii=False),
            encoding="utf-8",
        )

    @classmethod
    def load(cls, path: str) -> "AnimDoc":
        return cls.from_dict(json.loads(Path(path).read_text(encoding="utf-8")))

    # ---- Code-Generierung: Vorschau-Runner ----
    def _param_range(self, p: Param) -> tuple[float, float]:
        """Slider-Bereich fuer einen numerischen Parameter -- aus den Condition-
        Schwellen abgeleitet (damit man die Uebergaenge im Vorschaufenster auch
        wirklich ausloesen kann)."""
        hi = 1.0
        for t in self.transitions:
            for c in t.conditions:
                if c.param == p.name and c.op in ("gt", "lt", "ge", "le", "eq", "ne"):
                    hi = max(hi, abs(c.value) * 2.0)
        if p.ptype == "int":
            hi = max(hi, 10.0)
        return (0.0, float(hi))

    def generate_runner(self, gbanim_filename: str, screen_w: int = 640,
                        screen_h: int = 420, title: str | None = None) -> str:
        """Lauffaehiges GameBasic-Vorschauprogramm (wie Unitys Animator-Preview):
        links ein Live-Parameter-Panel (Slider/Checkbox/Button via `ui`-Modul),
        rechts der Sprite, der den aktuellen State spielt -- inkl. State-Anzeige.
        """
        ttl = (title or "dhanim Vorschau").replace('"', "'")
        L: list[str] = []
        ad = L.append
        ad('IMPORT "animfsm"')
        ad('IMPORT "sprite"')
        ad('IMPORT "ui"')
        ad("")
        ad(f'SCREEN({screen_w}, {screen_h}, "{ttl}", 1)')
        ad("")
        has_sheet = bool(self.sheet)
        if has_sheet:
            ad("DIM sheet AS IMAGE")
            ad(f'sheet = LOADIMAGE("{self.sheet}")')
            ad("DIM spr AS SPRITE")
            ad(f"spr = SPRITE_NEW(sheet, {self.frame_w}, {self.frame_h})")
        else:
            ad("DIM spr AS SPRITE")
            ad(f"spr = SPRITE_NEW(0, {self.frame_w}, {self.frame_h})")
        ad(f"SPRITE_SET_SCALE(spr, {self.scale}, {self.scale})")
        spr_x = screen_w - 230
        spr_y = screen_h // 2 - 20
        ad(f"SPRITE_SET_POS(spr, {spr_x}, {spr_y})")
        ad("")
        ad("DIM fsm AS ANIM_FSM")
        ad(f'fsm = ANIM_FSM_LOAD("{gbanim_filename}")')
        ad("ANIM_FSM_SETUP(fsm, spr)")
        ad("")
        ad("DIM last_ms AS INTEGER")
        ad("last_ms = MILLIS()")
        ad("")
        ad("WHILE NOT QUITREQUESTED()")
        ad("    IF KEYPRESSED(27) THEN BREAK")
        ad("    DIM now_ms AS INTEGER")
        ad("    now_ms = MILLIS()")
        ad("    DIM dt AS INTEGER")
        ad("    dt = now_ms - last_ms")
        ad("    last_ms = now_ms")
        ad("")
        ad("    CLS(RGB(26, 28, 40))")
        panel_h = 60 + max(1, len(self.params)) * 52
        ad(f'    UI_PANEL(10, 10, 260, {panel_h}, "Parameter")')
        y = 44
        for i, p in enumerate(self.params):
            pid = p.name.replace('"', "'")
            var = f"pv{i}"
            if p.ptype in ("float", "int"):
                lo, hi = self._param_range(p)
                dflt = max(lo, min(hi, p.default))
                ad(f'    UI_LABEL(22, {y}, "{pid}  ({p.ptype})")')
                ad(f"    DIM {var} AS FLOAT")
                ad(f'    {var} = UI_SLIDER("{pid}", 22, {y + 18}, 230, {lo}, {hi}, {dflt})')
                if p.ptype == "int":
                    ad(f'    ANIM_FSM_SET_INT(fsm, "{pid}", INT({var}))')
                else:
                    ad(f'    ANIM_FSM_SET_FLOAT(fsm, "{pid}", {var})')
            elif p.ptype == "bool":
                checked = "TRUE" if p.default != 0 else "FALSE"
                ad(f"    DIM {var} AS BOOLEAN")
                ad(f'    {var} = UI_CHECKBOX("{pid}", 22, {y}, "{pid}", {checked})')
                ad(f'    ANIM_FSM_SET_BOOL(fsm, "{pid}", {var})')
            else:  # trigger
                ad(f'    IF UI_BUTTON("{pid}", 22, {y}, 230, 26, "{pid} ausloesen") THEN')
                ad(f'        ANIM_FSM_TRIGGER(fsm, "{pid}")')
                ad("    END IF")
            y += 52
        ad("")
        ad("    ANIM_FSM_UPDATE(fsm, spr, dt)")
        if has_sheet:
            ad("    SPRITE_DRAW(spr)")
        else:
            # Ohne Sheet: Platzhalter mit aktuellem Frame statt SPRITE_DRAW.
            ad(f"    BOX({spr_x}, {spr_y}, {spr_x + 64}, {spr_y + 64}, RGB(90, 120, 200))")
            ad(f'    TEXT({spr_x}, {spr_y + 72}, "Frame " + STR$(SPRITE_GET_FRAME(spr)))')
        ad(f'    TEXT({screen_w - 250}, 20, "State:  " + ANIM_FSM_STATE(fsm))')
        ad('    TEXT(12, ' + str(screen_h - 22) + ', "ESC = Ende")')
        ad("    UI_END_FRAME()")
        ad("    FLIP()")
        ad("WEND")
        return "\n".join(L) + "\n"


# --- Undo/Redo --------------------------------------------------------------
class History:
    """Zwei-Stack-Undo/Redo ueber komplette Doc-Snapshots (dict).

    VOR einer Mutation `push(current_snapshot)`. `undo(current)`/`redo(current)`
    bekommen den aktuellen Zustand und liefern den wiederherzustellenden Snapshot.
    """

    def __init__(self, limit: int = 200):
        self._undo: list[dict] = []
        self._redo: list[dict] = []
        self._limit = max(1, limit)

    def clear(self) -> None:
        self._undo.clear()
        self._redo.clear()

    def push(self, snapshot: dict) -> None:
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
