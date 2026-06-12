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
import json
from collections import deque
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from PIL import Image
from PySide6.QtGui import QImage, QPixmap


DEFAULT_FRAME_DURATION_MS = 125   # entspricht 8 fps -- typischer Sprite-Animations-Default


@dataclass
class Frame:
    pixels: Image.Image
    duration_ms: int = DEFAULT_FRAME_DURATION_MS
    name: str = ""           # optionaler Name -> Sprite-ID im Atlas-Export
    history: deque = field(default_factory=lambda: deque(maxlen=80))
    redo_stack: list = field(default_factory=list)

    def snapshot(self):
        self.history.append(self.pixels.copy())
        self.redo_stack.clear()

    def undo(self) -> bool:
        if not self.history:
            return False
        self.redo_stack.append(self.pixels.copy())
        self.pixels = self.history.pop()
        return True

    def redo(self) -> bool:
        if not self.redo_stack:
            return False
        self.history.append(self.pixels.copy())
        self.pixels = self.redo_stack.pop()
        return True


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

    def _blank_frame(self) -> Frame:
        return Frame(pixels=Image.new("RGBA", (self.width, self.height),
                                      (0, 0, 0, 0)))

    @property
    def current(self) -> Frame:
        return self.frames[self.current_index]

    def add_frame(self, copy_current: bool = False) -> int:
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
        kept: list[Anim] = []
        for a in self.anims:
            if a.first > at:
                a.first -= 1
            elif a.first == at and a.first > a.last - 1:
                pass   # Bereich war genau dieses Frame -> unten geprueft
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
        f = self.frames.pop(self.current_index)
        self.frames.insert(new_idx, f)
        self.current_index = new_idx
        self.dirty = True
        return True

    def select(self, idx: int):
        if 0 <= idx < len(self.frames):
            self.current_index = idx

    def resize(self, new_w: int, new_h: int):
        for f in self.frames:
            new_img = Image.new("RGBA", (new_w, new_h), (0, 0, 0, 0))
            new_img.paste(f.pixels, (0, 0))
            f.pixels = new_img
            f.history.clear()
            f.redo_stack.clear()
        self.width = new_w
        self.height = new_h
        self.dirty = True

    # --- Persistenz ---

    def save_native(self, path: Path):
        data = {
            "version": 4,    # V3: Frame-name; V4: benannte Anim-Bereiche
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
            buf = io.BytesIO()
            f.pixels.save(buf, format="PNG")
            fd = {
                "data": base64.b64encode(buf.getvalue()).decode("ascii"),
                "duration_ms": int(f.duration_ms),
            }
            if f.name:
                fd["name"] = f.name
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
            raw = base64.b64decode(fd["data"])
            img = Image.open(io.BytesIO(raw)).convert("RGBA")
            # Backward-compat: Version-1-Dateien hatten kein duration_ms,
            # Version-1/2 hatten kein name-Feld
            duration_ms = int(fd.get("duration_ms", DEFAULT_FRAME_DURATION_MS))
            name = str(fd.get("name", ""))
            doc.frames.append(Frame(pixels=img, duration_ms=duration_ms, name=name))
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

    def save_png_single(self, path: Path):
        self.current.pixels.save(path, format="PNG")
        self.filepath = path
        self.dirty = False

    def save_animated_gif(self, path: Path, fps: Optional[int] = None,
                          loop: int = 0):
        """Schreibt alle Frames als animiertes GIF.

        Args:
            path: Ziel-Datei (.gif)
            fps: wenn gesetzt, ueberschreibt alle Frame-Dauern mit
                 1000/fps. Wenn None, wird pro Frame die individuelle
                 duration_ms verwendet.
            loop: 0 = unendlich, sonst Anzahl Wiederholungen
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
            rgba = f.pixels
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

    def save_sheet_png(self, path: Path, layout: str = "horizontal"):
        n = len(self.frames)
        if layout == "vertical":
            sheet = Image.new("RGBA", (self.width, self.height * n), (0, 0, 0, 0))
            for i, f in enumerate(self.frames):
                sheet.paste(f.pixels, (0, i * self.height))
        else:
            sheet = Image.new("RGBA", (self.width * n, self.height), (0, 0, 0, 0))
            for i, f in enumerate(self.frames):
                sheet.paste(f.pixels, (i * self.width, 0))
        sheet.save(path, format="PNG")

    def save_sheet_atlas(self, png_path: Path, json_path: Path,
                         name_prefix: Optional[str] = None,
                         layout: str = "horizontal") -> dict:
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

        Liefert das geschriebene Manifest-Dict (fuer Tests/Logging).
        """
        if name_prefix is None:
            name_prefix = png_path.stem
        # PNG schreiben (Re-Use der bestehenden Sheet-Logik)
        self.save_sheet_png(png_path, layout=layout)
        # Rects fuer das Manifest berechnen (gleiche Layout-Logik)
        sprites: dict = {}
        for i, f in enumerate(self.frames):
            if layout == "vertical":
                rect = [0, i * self.height, self.width, self.height]
            else:
                rect = [i * self.width, 0, self.width, self.height]
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
