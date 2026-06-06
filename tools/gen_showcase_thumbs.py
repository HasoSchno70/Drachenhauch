"""Erzeugt Showcase-Thumbnails fuer das Welcome-Panel.

Fuer jede Demo aus `gamebasic.editor_qt.showcase.SHOWCASE` wird die native
Runtime `gbrt` headless gestartet (`GBRT_FRAMES` begrenzt die Frames,
`GBRT_SCREENSHOT` zieht beim letzten Frame einen PNG-Screenshot), das Bild
auf Thumbnail-Breite herunterskaliert und unter
`examples/screenshots/<stem>.png` abgelegt.

Aufruf (gbrt muss gebaut sein -- `rust/build_runtime.py`):
    .venv\\Scripts\\python.exe tools\\gen_showcase_thumbs.py [stem ...]

Ohne Argumente werden alle Showcase-Demos erzeugt; mit Argumenten nur die
genannten (z.B. `48_orbital`). raylib oeffnet pro Demo kurz ein Fenster.
"""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from gamebasic.editor_qt.showcase import SHOWCASE, thumb_path  # noqa: E402

THUMB_WIDTH = 480


def _find_gbrt() -> Path | None:
    base = ROOT / "rust" / "gb_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


def _generate(entry: dict, gbrt: Path) -> bool:
    from PIL import Image

    src = ROOT / "examples" / entry["file"]
    if not src.exists():
        print(f"  ! Quelle fehlt: {src.name}")
        return False
    out = thumb_path(ROOT, entry)
    out.parent.mkdir(parents=True, exist_ok=True)

    # raylib speichert den Basename ins Arbeitsverzeichnis (= examples/, noetig
    # fuer LOADIMAGE("assets/...")). Wir holen das Bild danach dort ab.
    stem = Path(entry["file"]).stem
    raw = src.parent / f"{stem}.png"
    env = dict(
        os.environ,
        GBRT_FRAMES=str(entry.get("frames", 120)),
        GBRT_SCREENSHOT=str(raw),
    )
    # `gbrt run`: kompiliert die Quelle selbst (Stufe B -- kein Python-Compiler
    # mehr) und chdirt ins examples/-Verzeichnis. Manche Demos laufen in einem
    # `WHILE TRUE`/ESC-Loop und beenden sich NICHT ueber das Frame-Limit -- der
    # Headless-Screenshot wird trotzdem beim Erreichen von GBRT_FRAMES gezogen.
    try:
        subprocess.run(
            [str(gbrt), "run", str(src)],
            env=env, timeout=30, capture_output=True, text=True,
        )
    except subprocess.TimeoutExpired:
        pass

    if not raw.exists():
        print(f"  ! Kein Screenshot fuer {src.name}")
        return False

    # Auf Thumbnail-Breite skalieren und ablegen, Voll-Bild entfernen.
    img = Image.open(raw).convert("RGB")
    ratio = THUMB_WIDTH / img.width
    img = img.resize((THUMB_WIDTH, max(1, round(img.height * ratio))),
                     Image.LANCZOS)
    img.save(out, "PNG")
    raw.unlink(missing_ok=True)
    print(f"  + {out.relative_to(ROOT)}  ({out.stat().st_size // 1024} KB)")
    return True


def main() -> int:
    gbrt = _find_gbrt()
    if gbrt is None:
        print("gbrt nicht gefunden -- erst bauen: rust/build_runtime.py")
        return 1
    wanted = set(sys.argv[1:])
    entries = [e for e in SHOWCASE
               if not wanted or Path(e["file"]).stem in wanted]
    print(f"Erzeuge {len(entries)} Thumbnail(s) ...")
    ok = 0
    for e in entries:
        print(f"- {e['file']}")
        if _generate(e, gbrt):
            ok += 1
    print(f"Fertig: {ok}/{len(entries)} erzeugt.")
    return 0 if ok == len(entries) else 2


if __name__ == "__main__":
    raise SystemExit(main())
