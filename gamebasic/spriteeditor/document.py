"""Datenmodell + Persistenz fuer den Sprite-Editor.

Eingangs-Konstrukte: `Frame` (ein Pixel-Bild + Undo-/Redo-History +
Frame-Dauer), `SpriteDoc` (mehrere Frames + Save/Load fuer .gbsprite,
.png, .gif, Sheets).

Hier kein Qt-Import -- alles arbeitet auf PIL.Image. Die Konversion
nach Qt liegt in `pil_to_qpixmap`, die zwar Qt-Klassen nutzt, aber
hier statt im UI-Layer wohnt, weil sie direkt mit PIL-Bildern arbeitet
und sowohl vom Canvas als auch vom Frames-Panel verwendet wird.
"""
from __future__ import annotations

import base64
import io
import itertools
import json
from collections import deque
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from PIL import Image
from PySide6.QtGui import QImage, QPixmap


DEFAULT_FRAME_DURATION_MS = 125   # entspricht 8 fps -- typischer Sprite-Animations-Default


# Globale, monoton steigende Sequenz fuer ALLE Undo-Eintraege (Pixel wie
# Struktur). Damit entscheidet Ctrl+Z einfach: der Eintrag mit der hoechsten
# Sequenz ist die juengste Aktion.
_UNDO_SEQ = itertools.count(1)


@dataclass
class Layer:
    """Eine Ebene eines Frames. Frames haben mindestens eine Ebene;
    gezeichnet wird immer auf der aktiven, angezeigt/exportiert wird
    das Composite aller sichtbaren Ebenen (unten -> oben)."""
    pixels: Image.Image
    name: str = "Ebene 1"
    visible: bool = True
    opacity: float = 1.0     # 0..1, skaliert den Alpha-Kanal beim Composite


def _with_opacity(img: Image.Image, opacity: float) -> Image.Image:
    if opacity >= 1.0:
        return img
    opacity = max(0.0, opacity)
    r, g, b, a = img.split()
    a = a.point(lambda v: int(v * opacity))
    return Image.merge("RGBA", (r, g, b, a))


class Frame:
    """Ein Animations-Frame: Ebenen-Stapel + Dauer + optionaler Name.

    `frame.pixels` ist die **aktive Ebene** (Lesen UND Schreiben) -- so
    arbeiten alle Pixel-Tools unveraendert auf der aktiven Ebene. Fuer
    Anzeige/Export liefert `composite()` alle sichtbaren Ebenen
    uebereinander. Bei einem einlagigen Frame ist beides dasselbe Bild.
    """

    def __init__(self, pixels: Optional[Image.Image] = None,
                 duration_ms: int = DEFAULT_FRAME_DURATION_MS,
                 name: str = "",
                 layers: Optional[list[Layer]] = None,
                 active_layer: int = 0):
        if layers is None:
            if pixels is None:
                raise ValueError("Frame braucht pixels oder layers")
            layers = [Layer(pixels=pixels)]
        self.layers: list[Layer] = layers
        self.active_layer = max(0, min(active_layer, len(layers) - 1))
        self.duration_ms = duration_ms
        self.name = name     # optionaler Name -> Sprite-ID im Atlas-Export
        self.history: deque = deque(maxlen=80)   # (seq, layer_idx, Image)
        self.redo_stack: list = []               # (seq, layer_idx, Image)

    # --- aktive Ebene als pixels-Property (Tool-/Bestands-API) ---------

    @property
    def pixels(self) -> Image.Image:
        return self.layers[self.active_layer].pixels

    @pixels.setter
    def pixels(self, img: Image.Image):
        self.layers[self.active_layer].pixels = img

    def composite(self, active_override: Optional[Image.Image] = None) -> Image.Image:
        """Alle sichtbaren Ebenen uebereinander (unten -> oben), Opacity
        angewendet. `active_override` ersetzt die aktive Ebene (fuer die
        Live-Preview waehrend eines Tool-Drags). Das Ergebnis NICHT
        mutieren -- im Ein-Ebenen-Fall ist es das Live-Bild."""
        # Fast-Path: eine sichtbare Ebene ohne Opacity -> Live-Bild.
        if len(self.layers) == 1:
            ly = self.layers[0]
            img = active_override if active_override is not None else ly.pixels
            if ly.visible and ly.opacity >= 1.0:
                return img
            base = Image.new("RGBA", img.size, (0, 0, 0, 0))
            if ly.visible:
                base.alpha_composite(_with_opacity(img, ly.opacity))
            return base
        size = self.layers[0].pixels.size
        base = Image.new("RGBA", size, (0, 0, 0, 0))
        for i, ly in enumerate(self.layers):
            if not ly.visible:
                continue
            img = (active_override
                   if (active_override is not None and i == self.active_layer)
                   else ly.pixels)
            base.alpha_composite(_with_opacity(img, ly.opacity))
        return base

    def clone(self) -> "Frame":
        """Tiefe Kopie (Ebenen-Bilder kopiert), ohne Undo-Historie."""
        return Frame(
            layers=[Layer(pixels=ly.pixels.copy(), name=ly.name,
                          visible=ly.visible, opacity=ly.opacity)
                    for ly in self.layers],
            active_layer=self.active_layer,
            duration_ms=self.duration_ms,
            name=self.name,
        )

    # --- Ebenen-Operationen ---------------------------------------------
    # Struktur-Undo dafuer macht der Aufrufer via doc.push_struct().

    def add_layer(self, name: Optional[str] = None) -> int:
        """Neue leere Ebene UEBER der aktiven; wird aktiv."""
        size = self.layers[0].pixels.size
        if name is None:
            name = f"Ebene {len(self.layers) + 1}"
        self.layers.insert(self.active_layer + 1,
                           Layer(pixels=Image.new("RGBA", size, (0, 0, 0, 0)),
                                 name=name))
        self.active_layer += 1
        return self.active_layer

    def duplicate_layer(self) -> int:
        src = self.layers[self.active_layer]
        self.layers.insert(self.active_layer + 1,
                           Layer(pixels=src.pixels.copy(),
                                 name=src.name + " (Kopie)",
                                 visible=src.visible, opacity=src.opacity))
        self.active_layer += 1
        return self.active_layer

    def delete_layer(self) -> bool:
        if len(self.layers) <= 1:
            return False
        del self.layers[self.active_layer]
        self.active_layer = max(0, min(self.active_layer, len(self.layers) - 1))
        # Undo-Eintraege zeigen auf Layer-Indizes -- nach Strukturwechsel
        # waeren sie irrefuehrend (Struktur-Undo stellt den Stand wieder her).
        self.history.clear()
        self.redo_stack.clear()
        return True

    def move_layer(self, delta: int) -> bool:
        new_idx = self.active_layer + delta
        if not (0 <= new_idx < len(self.layers)):
            return False
        ly = self.layers.pop(self.active_layer)
        self.layers.insert(new_idx, ly)
        self.active_layer = new_idx
        self.history.clear()
        self.redo_stack.clear()
        return True

    def merge_down(self) -> bool:
        """Aktive Ebene (mit ihrer Opacity) auf die darunterliegende
        stempeln und entfernen. Die untere Ebene wird aktiv."""
        if self.active_layer == 0:
            return False
        top = self.layers[self.active_layer]
        below = self.layers[self.active_layer - 1]
        if top.visible:
            below.pixels.alpha_composite(_with_opacity(top.pixels, top.opacity))
        del self.layers[self.active_layer]
        self.active_layer -= 1
        self.history.clear()
        self.redo_stack.clear()
        return True

    # --- Pixel-Undo (pro Ebene) -------------------------------------------

    def snapshot(self):
        self.history.append((next(_UNDO_SEQ), self.active_layer,
                             self.pixels.copy()))
        self.redo_stack.clear()

    def undo(self) -> bool:
        if not self.history:
            return False
        _, li, img = self.history.pop()
        li = min(li, len(self.layers) - 1)
        self.redo_stack.append((next(_UNDO_SEQ), li,
                                self.layers[li].pixels.copy()))
        self.layers[li].pixels = img
        return True

    def redo(self) -> bool:
        if not self.redo_stack:
            return False
        _, li, img = self.redo_stack.pop()
        li = min(li, len(self.layers) - 1)
        self.history.append((next(_UNDO_SEQ), li,
                             self.layers[li].pixels.copy()))
        self.layers[li].pixels = img
        return True

    def last_undo_seq(self) -> int:
        return self.history[-1][0] if self.history else 0

    def last_redo_seq(self) -> int:
        return self.redo_stack[-1][0] if self.redo_stack else 0


@dataclass
class Anim:
    """Benannter Animations-Bereich (Frames first..last inklusiv) --
    entspricht 1:1 SPRITE_ADD_ANIM(name, first, last, fps) der Engine."""
    name: str
    first: int
    last: int
    fps: int = 8


class SpriteDoc:
    def __init__(self, width: int, height: int):
        self.width = width
        self.height = height
        self.frames: list[Frame] = [self._blank_frame()]
        self.anims: list[Anim] = []
        self.current_index = 0
        self.filepath: Optional[Path] = None
        self.dirty = False
        # Struktur-Undo: Voll-Snapshots des Dokuments VOR jeder Struktur-
        # Mutation (Frame add/delete/move, Resize, Dauern/Namen). Pixel-
        # Striche laufen weiter ueber das leichte per-Frame-Undo.
        self._struct_undo: deque = deque(maxlen=30)   # (seq, state)
        self._struct_redo: list = []                  # (seq, state)

    def _blank_frame(self) -> Frame:
        return Frame(pixels=Image.new("RGBA", (self.width, self.height),
                                      (0, 0, 0, 0)))

    @property
    def current(self) -> Frame:
        return self.frames[self.current_index]

    # --- Struktur-Undo ----------------------------------------------------

    def _capture_state(self) -> dict:
        return {
            "width": self.width,
            "height": self.height,
            "current_index": self.current_index,
            "frames": [f.clone() for f in self.frames],
            "anims": [Anim(a.name, a.first, a.last, a.fps) for a in self.anims],
        }

    def _restore_state(self, st: dict):
        self.width = st["width"]
        self.height = st["height"]
        # Wiederhergestellte Frames starten mit leerer Pixel-History --
        # die Struktur-Snapshots sichern den Zustand, nicht die Historie.
        # Erneut klonen: derselbe Snapshot kann mehrfach restauriert
        # werden (undo -> redo -> undo), darf also nicht aliasen.
        self.frames = [f.clone() for f in st["frames"]]
        self.anims = list(st["anims"])
        self.current_index = max(0, min(st["current_index"], len(self.frames) - 1))
        self.dirty = True

    def push_struct(self):
        """VOR einer Struktur-Mutation aufrufen (Frame-Ops machen das
        selbst; UI-Ops wie Dauer/Name rufen es explizit)."""
        self._struct_undo.append((next(_UNDO_SEQ), self._capture_state()))
        self._struct_redo.clear()

    def discard_last_struct_undo(self):
        """Nimmt den zuletzt via `push_struct()` angelegten Snapshot wieder
        zurueck -- fuer Aufrufer, die VOR einer moeglicherweise fehlschlagenden
        Mutation pushen (z.B. `move_layer`, das bei einer Grenz-Position
        nichts tut) und bei Fehlschlag keinen No-op-Undo-Schritt behalten
        wollen. Kein-Op, wenn nichts zum Zuruecknehmen da ist."""
        if self._struct_undo:
            self._struct_undo.pop()

    def last_struct_undo_seq(self) -> int:
        return self._struct_undo[-1][0] if self._struct_undo else 0

    def last_struct_redo_seq(self) -> int:
        return self._struct_redo[-1][0] if self._struct_redo else 0

    def frame_with_latest_undo(self) -> tuple[int, int]:
        """`(frame_index, seq)` des Frames mit dem juengsten Pixel-Undo-Eintrag
        ueber ALLE Frames -- Pixel-History liegt pro Frame, ein globales Ctrl+Z
        muss aber das juengste Pixel-Undo finden, egal in welchem Frame es
        passierte (sonst ist es nach einem Frame-Wechsel unerreichbar).
        `seq=0`/`index=-1`, wenn nirgends ein Pixel-Undo vorliegt."""
        best_idx, best_seq = -1, 0
        for i, f in enumerate(self.frames):
            s = f.last_undo_seq()
            if s > best_seq:
                best_idx, best_seq = i, s
        return best_idx, best_seq

    def frame_with_latest_redo(self) -> tuple[int, int]:
        """Pendant zu `frame_with_latest_undo` fuer Redo."""
        best_idx, best_seq = -1, 0
        for i, f in enumerate(self.frames):
            s = f.last_redo_seq()
            if s > best_seq:
                best_idx, best_seq = i, s
        return best_idx, best_seq

    def undo_struct(self) -> bool:
        if not self._struct_undo:
            return False
        self._struct_redo.append((next(_UNDO_SEQ), self._capture_state()))
        _, st = self._struct_undo.pop()
        self._restore_state(st)
        return True

    def redo_struct(self) -> bool:
        if not self._struct_redo:
            return False
        self._struct_undo.append((next(_UNDO_SEQ), self._capture_state()))
        _, st = self._struct_redo.pop()
        self._restore_state(st)
        return True

    def add_frame(self, copy_current: bool = False) -> int:
        self.push_struct()
        new_img = (self.current.pixels.copy() if copy_current
                   else Image.new("RGBA", (self.width, self.height), (0, 0, 0, 0)))
        self.frames.insert(self.current_index + 1, Frame(pixels=new_img))
        self._anims_after_insert(self.current_index + 1)
        self.current_index += 1
        self.dirty = True
        return self.current_index

    def paste_as_frame(self, img: Image.Image) -> int:
        """Fuegt nach dem aktuellen Frame ein neues ein, das `img` enthaelt
        (auf Dokumentgroesse oben-links eingepasst -- groesseres wird
        beschnitten, kleineres transparent aufgefuellt). Liefert den Index
        des neuen Frames."""
        self.push_struct()
        canvas = Image.new("RGBA", (self.width, self.height), (0, 0, 0, 0))
        src = img.convert("RGBA")
        if src.width > self.width or src.height > self.height:
            src = src.crop((0, 0, min(self.width, src.width),
                            min(self.height, src.height)))
        canvas.alpha_composite(src)
        self.frames.insert(self.current_index + 1, Frame(pixels=canvas))
        self._anims_after_insert(self.current_index + 1)
        self.current_index += 1
        self.dirty = True
        return self.current_index

    def delete_frame(self) -> bool:
        if len(self.frames) <= 1:
            return False
        self.push_struct()
        del self.frames[self.current_index]
        self._anims_after_delete(self.current_index)
        self.current_index = max(0, min(self.current_index, len(self.frames) - 1))
        self.dirty = True
        return True

    # --- Anim-Bereiche pflegen -------------------------------------------
    # Anims referenzieren Frame-INDIZES (wie SPRITE_ADD_ANIM). Beim
    # Einfuegen/Loeschen von Frames werden die Bereiche mitgezogen;
    # leerlaufende Bereiche fliegen raus. Frame-UMORDNEN (move_frame)
    # laesst die Indizes bewusst stehen -- welcher Bereich "mitwandern"
    # soll, ist dort nicht entscheidbar.

    def _anims_after_insert(self, at: int):
        for a in self.anims:
            if a.first >= at:
                a.first += 1
            if a.last >= at:
                a.last += 1

    def _anims_after_delete(self, at: int):
        """Bereiche nach dem Loeschen von Frame `at` anpassen -- der Frame
        ist zu diesem Zeitpunkt bereits aus `self.frames` entfernt.
        `first` wird NUR verschoben, wenn der geloeschte Frame VOR dem
        Bereich lag (`first > at`); liegt er GENAU auf `first` (Bereich
        beginnt beim geloeschten Frame), bleibt `first` unveraendert -- der
        bisherige zweite Frame des Bereichs ruecht an dessen Stelle nach.
        Ein Bereich, der genau aus dem geloeschten Frame bestand, faellt
        beim abschliessenden `last >= first`-Check automatisch raus.
        (Frueher stand hier zusaetzlich ein `elif ... : pass`-Zweig fuer
        genau diesen Fall -- der war ein reines No-Op und aenderte nichts
        am Ergebnis, machte den Code aber irrefuehrend.)"""
        kept: list[Anim] = []
        for a in self.anims:
            if a.first > at:
                a.first -= 1
            if a.last >= at:
                a.last -= 1
            if a.last >= a.first and a.first >= 0:
                a.last = min(a.last, len(self.frames) - 1)
                kept.append(a)
        self.anims = kept

    def anim_fps_suggestion(self, first: int, last: int) -> int:
        """FPS-Vorschlag aus den tatsaechlichen Frame-Dauern des Bereichs
        (Mittelwert) -- statt eines hardcodierten Defaults."""
        frames = self.frames[max(0, first):min(len(self.frames), last + 1)]
        if not frames:
            return 8
        avg_ms = sum(max(1, f.duration_ms) for f in frames) / len(frames)
        return max(1, min(60, round(1000.0 / avg_ms)))

    # --- Engine-Export: GB-Code + .gbanim ---------------------------------

    def _effective_anims(self) -> list[Anim]:
        """Definierte Bereiche -- oder ein Default-Bereich "idle" ueber
        alle Frames mit FPS aus den echten Frame-Dauern."""
        if self.anims:
            return self.anims
        n = len(self.frames)
        return [Anim("idle", 0, n - 1, self.anim_fps_suggestion(0, n - 1))]

    def generate_gb_snippet(self, sheet_filename: str) -> str:
        """GB-Code-Schnipsel zum Einbinden des Sprites: SPRITE_NEW +
        eine SPRITE_ADD_ANIM-Zeile PRO definiertem Anim-Bereich (FPS aus
        den Bereichen; ohne Bereiche: "idle" ueber alles)."""
        anims = self._effective_anims()
        lines = [
            'IMPORT "sprite"',
            "",
            "DIM sheet AS IMAGE",
            f'sheet = LoadImage("{sheet_filename}")',
            "",
            "DIM sp AS SPRITE",
            f"sp = SPRITE_NEW(sheet, {self.width}, {self.height})",
        ]
        for a in anims:
            lines.append(f'SPRITE_ADD_ANIM(sp, "{a.name}", {a.first}, {a.last}, {a.fps})')
        lines += [
            f'SPRITE_PLAY(sp, "{anims[0].name}")',
            "SPRITE_SET_POS(sp, 100, 100)",
            "",
            "' Im Game-Loop:",
            "' SPRITE_UPDATE(sp, 16)",
            "' SPRITE_DRAW(sp)",
        ]
        return "\n".join(lines) + "\n"

    def generate_gbanim(self) -> dict:
        """Animations-FSM-Vorlage (.gbanim) aus den Anim-Bereichen:
        ein State pro Bereich (name=anim, first/last/fps), erster Bereich
        als default. Transitions/Parameter ergaenzt der User in gbanim --
        die Datei ist direkt von ANIM_FSM_LOAD ladbar."""
        anims = self._effective_anims()
        return {
            "default": anims[0].name,
            "params": [],
            "states": [
                {"name": a.name, "first": a.first, "last": a.last,
                 "fps": a.fps, "loop": True}
                for a in anims
            ],
            "transitions": [],
        }

    def move_frame(self, delta: int) -> bool:
        new_idx = self.current_index + delta
        if not (0 <= new_idx < len(self.frames)):
            return False
        self.push_struct()
        f = self.frames.pop(self.current_index)
        self.frames.insert(new_idx, f)
        self.current_index = new_idx
        self.dirty = True
        return True

    def select(self, idx: int):
        if 0 <= idx < len(self.frames):
            self.current_index = idx

    def resize(self, new_w: int, new_h: int):
        self.push_struct()
        for f in self.frames:
            for ly in f.layers:
                new_img = Image.new("RGBA", (new_w, new_h), (0, 0, 0, 0))
                new_img.paste(ly.pixels, (0, 0))
                ly.pixels = new_img
            f.history.clear()
            f.redo_stack.clear()
        self.width = new_w
        self.height = new_h
        self.dirty = True

    # --- Persistenz ---

    @staticmethod
    def _b64_png(img: Image.Image) -> str:
        buf = io.BytesIO()
        img.save(buf, format="PNG")
        return base64.b64encode(buf.getvalue()).decode("ascii")

    def save_native(self, path: Path):
        data = {
            # V3: Frame-name; V4: benannte Anim-Bereiche; V5: Ebenen
            "version": 5,
            "width": self.width,
            "height": self.height,
            "frames": [],
        }
        if self.anims:
            data["anims"] = [
                {"name": a.name, "first": a.first, "last": a.last, "fps": a.fps}
                for a in self.anims
            ]
        for f in self.frames:
            # "data" ist IMMER das geflattete Composite -- aeltere Leser
            # (und externe Tools) sehen damit weiterhin ein korrektes Bild.
            fd = {
                "data": self._b64_png(f.composite()),
                "duration_ms": int(f.duration_ms),
            }
            if f.name:
                fd["name"] = f.name
            # Ebenen nur schreiben, wenn sie Information tragen (mehr als
            # eine, oder Nicht-Default-Eigenschaften).
            if (len(f.layers) > 1
                    or not f.layers[0].visible
                    or f.layers[0].opacity < 1.0
                    or f.layers[0].name != "Ebene 1"):
                fd["layers"] = [
                    {"name": ly.name, "visible": ly.visible,
                     "opacity": ly.opacity, "data": self._b64_png(ly.pixels)}
                    for ly in f.layers
                ]
                fd["active_layer"] = f.active_layer
            data["frames"].append(fd)
        path.write_text(json.dumps(data, indent=2), encoding="utf-8")
        self.filepath = path
        self.dirty = False

    @classmethod
    def load_native(cls, path: Path) -> "SpriteDoc":
        data = json.loads(path.read_text(encoding="utf-8"))
        doc = cls(int(data["width"]), int(data["height"]))
        doc.frames = []
        for fd in data["frames"]:
            # Backward-compat: Version-1-Dateien hatten kein duration_ms,
            # Version-1/2 hatten kein name-Feld, vor V5 keine Ebenen.
            duration_ms = int(fd.get("duration_ms", DEFAULT_FRAME_DURATION_MS))
            name = str(fd.get("name", ""))
            if fd.get("layers"):
                layers = []
                for ld in fd["layers"]:
                    raw = base64.b64decode(ld["data"])
                    img = Image.open(io.BytesIO(raw)).convert("RGBA")
                    # Auf 0..1 klemmen -- eine korrupte/handbearbeitete Datei
                    # koennte sonst einen unsinnigen Wert (z.B. 5.0 = "500%")
                    # unveraendert ins Datenmodell tragen (_with_opacity selbst
                    # ist zwar bereits sicher dagegen, aber der Wert wuerde
                    # z.B. in einem Opacity-Slider/Prozent-Text falsch anzeigen).
                    opacity = max(0.0, min(1.0, float(ld.get("opacity", 1.0))))
                    layers.append(Layer(
                        pixels=img,
                        name=str(ld.get("name", f"Ebene {len(layers) + 1}")),
                        visible=bool(ld.get("visible", True)),
                        opacity=opacity,
                    ))
                doc.frames.append(Frame(
                    layers=layers,
                    active_layer=int(fd.get("active_layer", 0)),
                    duration_ms=duration_ms, name=name))
            else:
                raw = base64.b64decode(fd["data"])
                img = Image.open(io.BytesIO(raw)).convert("RGBA")
                doc.frames.append(Frame(pixels=img, duration_ms=duration_ms,
                                        name=name))
        if not doc.frames:
            doc.frames = [doc._blank_frame()]
        # V4: benannte Anim-Bereiche (aeltere Dateien: leer)
        doc.anims = [
            Anim(name=str(a.get("name", "")), first=int(a.get("first", 0)),
                 last=int(a.get("last", 0)), fps=int(a.get("fps", 8)))
            for a in data.get("anims", [])
            if str(a.get("name", "")).strip()
        ]
        doc.current_index = 0
        doc.filepath = path
        doc.dirty = False
        return doc

    @staticmethod
    def _scaled(img: Image.Image, scale: int) -> Image.Image:
        """Integer-Hochskalierung (Nearest-Neighbor, pixelart-treu).
        scale <= 1 liefert das Original unveraendert."""
        scale = int(scale)
        if scale <= 1:
            return img
        return img.resize((img.width * scale, img.height * scale),
                          Image.NEAREST)

    def save_png_single(self, path: Path, scale: int = 1):
        self._scaled(self.current.composite(), scale).save(path, format="PNG")
        self.filepath = path
        self.dirty = False

    def save_animated_gif(self, path: Path, fps: Optional[int] = None,
                          loop: int = 0, scale: int = 1):
        """Schreibt alle Frames als animiertes GIF.

        Args:
            path: Ziel-Datei (.gif)
            fps: wenn gesetzt, ueberschreibt alle Frame-Dauern mit
                 1000/fps. Wenn None, wird pro Frame die individuelle
                 duration_ms verwendet.
            loop: 0 = unendlich, sonst Anzahl Wiederholungen
            scale: Integer-Hochskalierung (Nearest-Neighbor), 1 = Original
        """
        if not self.frames:
            raise ValueError("Keine Frames zum Exportieren")

        if fps is not None:
            uniform_ms = max(20, int(1000 / max(1, fps)))
            durations = [uniform_ms] * len(self.frames)
        else:
            durations = [max(20, int(f.duration_ms)) for f in self.frames]

        gif_frames = []
        for f in self.frames:
            rgba = self._scaled(f.composite(), scale)
            p = rgba.convert("RGB").convert(
                "P", palette=Image.Palette.ADAPTIVE, colors=255)
            alpha = rgba.split()[3]
            mask = alpha.point(lambda a: 255 if a < 128 else 0)
            p.paste(255, mask=mask)
            gif_frames.append(p)

        gif_frames[0].save(
            path,
            save_all=True,
            append_images=gif_frames[1:],
            duration=durations,
            loop=loop,
            transparency=255,
            disposal=2,
            optimize=False,
        )

    def save_sheet_png(self, path: Path, layout: str = "horizontal",
                       scale: int = 1):
        n = len(self.frames)
        if layout == "vertical":
            sheet = Image.new("RGBA", (self.width, self.height * n), (0, 0, 0, 0))
            for i, f in enumerate(self.frames):
                sheet.paste(f.composite(), (0, i * self.height))
        else:
            sheet = Image.new("RGBA", (self.width * n, self.height), (0, 0, 0, 0))
            for i, f in enumerate(self.frames):
                sheet.paste(f.composite(), (i * self.width, 0))
        self._scaled(sheet, scale).save(path, format="PNG")

    def save_sheet_atlas(self, png_path: Path, json_path: Path,
                         name_prefix: Optional[str] = None,
                         layout: str = "horizontal",
                         scale: int = 1) -> dict:
        """Schreibt einen Sprite-Atlas: PNG-Sheet + JSON-Manifest mit
        benannten Frame-Rects. Direkt von ATLAS_LOAD lesbar.

        Manifest-Format (gleich der `SPRITE_ATLAS`-Spec in GameBasic):

            {
              "image": "<png_path.name>",
              "sprites": {
                "<prefix>_0": [x, y, w, h],
                "<prefix>_1": [...],
                ...
              }
            }

        - ``png_path``: Ziel-Pfad des Sheet-PNG.
        - ``json_path``: Ziel-Pfad des JSON-Manifests. ``image``-Feld wird
          relativ zum JSON-Verzeichnis gesetzt -- ATLAS_LOAD wertet das so
          aus, also liegen PNG und JSON idealerweise im selben Verzeichnis.
        - ``name_prefix``: Praefix fuer Sprite-Namen. Wenn None, wird der
          PNG-Basename (ohne Endung) verwendet. Frames mit eigenem ``name``
          nutzen diesen statt ``<prefix>_<idx>`` als Sprite-ID.
        - ``layout``: ``"horizontal"`` (Default) oder ``"vertical"``.
        - ``scale``: Integer-Hochskalierung (Nearest-Neighbor); die
          Manifest-Rects werden mitskaliert, ATLAS_LOAD passt weiterhin.

        Liefert das geschriebene Manifest-Dict (fuer Tests/Logging).
        """
        if name_prefix is None:
            name_prefix = png_path.stem
        scale = max(1, int(scale))
        # PNG schreiben (Re-Use der bestehenden Sheet-Logik)
        self.save_sheet_png(png_path, layout=layout, scale=scale)
        # Rects fuer das Manifest berechnen (gleiche Layout-Logik)
        w = self.width * scale
        h = self.height * scale
        sprites: dict = {}
        for i, f in enumerate(self.frames):
            if layout == "vertical":
                rect = [0, i * h, w, h]
            else:
                rect = [i * w, 0, w, h]
            # Benanntes Frame -> eigener Name; sonst <prefix>_<idx>.
            key = (f.name or "").strip()
            if not key:
                key = f"{name_prefix}_{i}"
            # Kollisionen (doppelte Namen / Ueberlapp mit <prefix>_<idx>)
            # eindeutig machen, sonst ueberschreibt der spaetere Eintrag.
            if key in sprites:
                key = f"{key}_{i}"
            sprites[key] = rect
        # image-Pfad ist relativ zum JSON -- nimm den png-Dateinamen wenn
        # beide im selben Verzeichnis liegen, sonst relativen Pfad.
        try:
            rel_img = str(png_path.relative_to(json_path.parent))
        except ValueError:
            # Verschiedene Verzeichnisse oder verschiedene Drives: nimm
            # einfach den vollen Pfad als String -- JSON-Spec verlangt str.
            rel_img = str(png_path)
        # Auf Windows back-slashes durch forward-slashes ersetzen, damit
        # das Manifest plattform-unabhaengig ist.
        rel_img = rel_img.replace("\\", "/")
        manifest = {
            "image": rel_img,
            "sprites": sprites,
        }
        json_path.write_text(
            json.dumps(manifest, indent=2),
            encoding="utf-8",
        )
        return manifest

    @classmethod
    def load_image(cls, path: Path,
                   frame_w: Optional[int] = None,
                   frame_h: Optional[int] = None) -> "SpriteDoc":
        img = Image.open(path).convert("RGBA")
        iw, ih = img.size
        if frame_w and frame_h and (frame_w != iw or frame_h != ih):
            cols = max(1, iw // frame_w)
            rows = max(1, ih // frame_h)
            doc = cls(frame_w, frame_h)
            doc.frames = []
            for r in range(rows):
                for c in range(cols):
                    box = (c * frame_w, r * frame_h,
                           c * frame_w + frame_w, r * frame_h + frame_h)
                    sub = img.crop(box).convert("RGBA")
                    doc.frames.append(Frame(pixels=sub))
            if not doc.frames:
                doc.frames = [doc._blank_frame()]
        else:
            doc = cls(iw, ih)
            doc.frames = [Frame(pixels=img)]
        doc.current_index = 0
        doc.filepath = path
        doc.dirty = False
        return doc


# ============================================================
# Onion-Skin-Helfer (pure, Qt-frei -- vom Canvas genutzt)
# ============================================================

def onion_indices(current: int, total: int, depth: int) -> list[tuple[int, str, float]]:
    """Welche Frames als Onion-Skin gerendert werden.

    Liefert (frame_index, mode, falloff)-Tupel: mode ist 'blue' (vorher)
    bzw. 'red' (nachher), falloff ein Faktor <= 1.0, der mit der Distanz
    abnimmt (weiter entfernte Frames werden blasser). Frames werden
    dedupliziert (bei 2 Frames ist vorher == nachher -> nur einmal) und
    das aktuelle Frame selbst ist nie enthalten.
    """
    if total <= 1 or depth <= 0:
        return []
    out: list[tuple[int, str, float]] = []
    seen = {current}
    for d in range(1, depth + 1):
        falloff = 0.55 ** (d - 1)
        for mode, idx in (("blue", (current - d) % total),
                          ("red", (current + d) % total)):
            if idx in seen:
                continue
            seen.add(idx)
            out.append((idx, mode, falloff))
    return out


def onion_tinted(img: Image.Image, mode: str, alpha: float) -> Image.Image:
    """Farbiges, halbtransparentes Onion-Bild (Original bleibt unberuehrt).

    mode 'blue' = kuehler Stich (vorheriges Frame), 'red' = warmer Stich
    (naechstes Frame). alpha 0..1 skaliert den Alpha-Kanal.
    """
    alpha = max(0.0, min(1.0, float(alpha)))
    r, g, b, a = img.convert("RGBA").split()
    a = a.point(lambda v: int(v * alpha))
    if mode == "blue":
        # Rot/Gruen zurueck, Blau bleibt -> kuehler Stich
        r = r.point(lambda v: int(v * 0.5))
        g = g.point(lambda v: int(v * 0.7))
    else:  # red
        b = b.point(lambda v: int(v * 0.5))
        g = g.point(lambda v: int(v * 0.7))
    return Image.merge("RGBA", (r, g, b, a))


# ============================================================
# PIL <-> Qt-Konversion
# ============================================================

def pil_to_qpixmap(img: Image.Image) -> QPixmap:
    """Wandelt PIL.Image (RGBA) zu QPixmap. Erhalt von Alpha."""
    if img.mode != "RGBA":
        img = img.convert("RGBA")
    data = img.tobytes("raw", "RGBA")
    qimg = QImage(data, img.width, img.height, img.width * 4,
                  QImage.Format_RGBA8888)
    # copy() loest die Daten vom temporaeren bytes-Object
    return QPixmap.fromImage(qimg.copy())
