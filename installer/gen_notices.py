#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Erzeugt installer/THIRD-PARTY-NOTICES.txt -- die gesammelten Lizenztexte aller
mit GameBasic ausgelieferten Drittkomponenten (Python + Rust). Tool-frei:
Python ueber importlib.metadata + die .dist-info-Lizenzdateien, Rust ueber das
in cargo eingebaute `cargo metadata` + die LICENSE-Dateien der Crate-Quellen.

MIT/BSD/Apache verlangen, dass ihr Copyright-/Lizenztext der Distribution
beiliegt -- genau das leistet diese Datei. Qt/PySide6 (LGPLv3) bekommt einen
expliziten Compliance-Hinweis.

Aufruf:  <venv>\\python.exe installer\\gen_notices.py
"""
from __future__ import annotations

import json
import subprocess
import sys
from importlib import metadata as im
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = Path(__file__).resolve().parent / "THIRD-PARTY-NOTICES.txt"
LIC_DIR = Path(__file__).resolve().parent / "licenses"


def _canonical(name: str) -> str:
    """Liest einen mitgelieferten kanonischen Lizenztext (installer/licenses/)."""
    p = LIC_DIR / name
    return p.read_text(encoding="utf-8") if p.exists() else f"({name} nicht gefunden)"

# Mit der IDE gebuendelte Python-Drittpakete (PyInstaller -> dist/GameBasic).
PY_PACKAGES = ["PySide6", "PySide6_Essentials", "PySide6_Addons",
               "shiboken6", "numpy", "pillow"]
# Im ausgelieferten dhrt aktive Cargo-Features (rust/build_runtime.py Standard).
RUST_FEATURES = "graphics db net http"

SEP = "=" * 78
LICENSE_NAMES = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE")


def _read_license_files(folder: Path) -> str:
    """Sammelt LICENSE/COPYING-Texte aus einem Ordner (nicht rekursiv + eine
    Ebene tief, z.B. .dist-info/licenses/)."""
    texts = []
    cands = list(folder.glob("*")) + list(folder.glob("licenses/*")) \
        + list(folder.glob("license_files/*"))
    for p in sorted(set(cands)):
        if not p.is_file():
            continue
        if p.name.upper().startswith(LICENSE_NAMES) or "LICENSE" in p.name.upper():
            try:
                t = p.read_text(encoding="utf-8", errors="replace").strip()
                if t:
                    texts.append(f"--- {p.name} ---\n{t}")
            except OSError:
                pass
    return "\n\n".join(texts)


def python_section() -> str:
    out = [SEP, "PYTHON-KOMPONENTEN", SEP, ""]
    out.append("Diese Anwendung enthaelt CPython (Python Software Foundation "
               "License) sowie die folgenden Python-Pakete:\n")
    seen = set()
    for name in PY_PACKAGES:
        try:
            dist = im.distribution(name)
        except im.PackageNotFoundError:
            continue
        meta = dist.metadata
        real = meta["Name"] or name
        if real.lower() in seen:
            continue
        seen.add(real.lower())
        lic = meta.get("License-Expression") or meta.get("License") or "(siehe Klassifizierer)"
        if lic and len(lic) > 200:
            lic = "(Lizenztext unten)"
        classifiers = [c for c in meta.get_all("Classifier", []) if "License" in c]
        out.append(f"\n{SEP}\n{real} {meta['Version']}\nLizenz: {lic}")
        for c in classifiers:
            out.append(f"  {c}")
        # Lizenztext aus dem .dist-info-Ordner (PathDistribution._path) bzw.
        # dessen licenses/-Unterordner.
        txt = ""
        distinfo = getattr(dist, "_path", None)
        if distinfo:
            try:
                txt = _read_license_files(Path(str(distinfo)))
            except Exception as e:  # noqa
                print(f"  WARN {real}: {e}", file=sys.stderr)
        if txt:
            out.append("\n" + txt)
    out.append("\n")
    return "\n".join(out)


def qt_section() -> str:
    return "\n".join([
        SEP, "Qt (via PySide6) -- LGPL v3", SEP, "",
        "GameBasic nutzt das Qt-Framework ueber PySide6 unter der GNU Lesser",
        "General Public License, Version 3 (LGPLv3).",
        "",
        "Gemaess LGPLv3 gilt:",
        " - Die Qt-Bibliotheken (Qt6*.dll, PySide6, shiboken6) liegen als",
        "   austauschbare Dateien im Ordner _internal der Installation; sie",
        "   koennen durch kompatible Versionen ersetzt werden.",
        " - Der vollstaendige Quellcode von Qt ist erhaeltlich unter",
        "   https://www.qt.io/ bzw. https://download.qt.io/ .",
        " - Der vollstaendige Lizenztext (LGPLv3 + der referenzierte GPLv3) folgt",
        "   direkt anschliessend.",
        "",
        SEP, "GNU LESSER GENERAL PUBLIC LICENSE Version 3", SEP, "",
        _canonical("LGPL-3.0.txt"),
        "",
        SEP, "GNU GENERAL PUBLIC LICENSE Version 3 (von der LGPLv3 referenziert)", SEP, "",
        _canonical("GPL-3.0.txt"),
        "",
    ])


def mpl_section() -> str:
    return "\n".join([
        SEP, "Mozilla Public License 2.0 (Volltext)", SEP, "",
        "Einige Rust-Crates (u.a. symphonia*, triple_buffer) stehen unter der",
        "MPL-2.0. Der vollstaendige Lizenztext:",
        "",
        _canonical("MPL-2.0.txt"),
        "",
    ])


def rust_section() -> str:
    out = [SEP, "RUST-KOMPONENTEN (native Runtime dhrt)", SEP, ""]
    crate = ROOT / "rust" / "drachenhauch_runtime"
    try:
        raw = subprocess.run(
            ["cargo", "metadata", "--format-version", "1",
             "--no-default-features", "--features", RUST_FEATURES],
            cwd=crate, capture_output=True, encoding="utf-8", errors="replace",
            timeout=180)
        data = json.loads(raw.stdout)
    except Exception as e:
        out.append(f"(cargo metadata fehlgeschlagen: {e})")
        return "\n".join(out)
    root_id = (data.get("resolve") or {}).get("root")
    pkgs = [p for p in data["packages"] if p.get("id") != root_id
            and p.get("name") != "drachenhauch_runtime"]
    pkgs.sort(key=lambda p: p["name"].lower())
    out.append(f"Die native Runtime dhrt enthaelt {len(pkgs)} Rust-Crates "
               f"(Features: {RUST_FEATURES}):\n")
    for p in pkgs:
        lic = p.get("license") or p.get("license_file") or "(unbekannt)"
        out.append(f"\n{SEP}\n{p['name']} {p['version']}\nLizenz: {lic}")
        src = Path(p["manifest_path"]).parent
        txt = _read_license_files(src)
        if txt:
            out.append("\n" + txt)
    out.append("\n")
    return "\n".join(out)


def main():
    parts = [
        "GameBasic -- Drittanbieter-Lizenzen (THIRD-PARTY NOTICES)",
        "",
        "GameBasic wird mit den unten aufgefuehrten Drittkomponenten ausgeliefert.",
        "Deren Lizenz- und Copyright-Hinweise sind nachfolgend wiedergegeben, wie",
        "es die jeweiligen Lizenzen (u.a. MIT, BSD, Apache-2.0, Zlib, LGPLv3)",
        "verlangen. Diese Datei beruehrt NICHT deine Rechte an eigenem GameBasic-",
        "Code; sie erfuellt die Weitergabe-Auflagen der Fremdkomponenten.",
        "",
        f"Automatisch erzeugt von installer/gen_notices.py.",
        "",
        python_section(),
        qt_section(),
        rust_section(),
        mpl_section(),
    ]
    OUT.write_text("\n".join(parts), encoding="utf-8")
    print("geschrieben:", OUT, f"({OUT.stat().st_size // 1024} KB)")


if __name__ == "__main__":
    main()
