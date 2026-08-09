#!/usr/bin/env python
"""Laedt ein HDR-Environment-Asset fuer das geplante HDR-IBL-Feature
(examples/99_ibl_hdr.dh, naechste Session -- siehe docs/rust-runtime.md).

Die .hdr (~1-2 MB, equirectangular 1k) liegt bewusst NICHT im Git-Repo.
Dieses Skript holt sie einmalig von Poly Haven:

    py examples/assets/download_hdri.py

Asset:  "kloofendal_43d_clear" (Outdoor, klarer Himmel -- gut fuer
        Metall-Reflexionen / Chrom-Kugeln im IBL-Demo)
Quelle: https://polyhaven.com/a/kloofendal_43d_clear
Lizenz: CC0 1.0 (Public Domain) -- keine Namensnennung erforderlich.

Falls die direkte CDN-URL nicht mehr stimmt: auf polyhaven.com das Asset
oeffnen, 1k .hdr herunterladen und als examples/assets/ibl_env.hdr ablegen.
"""
import urllib.request
from pathlib import Path

# Poly-Haven-CDN (1k HDR). Bei Aenderung des CDN-Layouts hier anpassen.
URL = "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/kloofendal_43d_clear_1k.hdr"
DEST = Path(__file__).resolve().parent / "ibl_env.hdr"


def main() -> int:
    if DEST.exists() and DEST.stat().st_size > 100_000:
        print(f"Schon vorhanden: {DEST} ({DEST.stat().st_size // 1024} KB)")
        return 0
    print(f"Lade {URL}\n  -> {DEST} ...")
    try:
        req = urllib.request.Request(URL, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=120) as r, open(DEST, "wb") as f:
            f.write(r.read())
    except Exception as exc:  # pragma: no cover
        print(f"FEHLER beim Download: {exc}")
        print("Manuell von https://polyhaven.com/a/kloofendal_43d_clear laden "
              "(1k .hdr) und als examples/assets/ibl_env.hdr ablegen.")
        return 1
    if DEST.stat().st_size < 100_000:
        print("FEHLER: Datei zu klein -- vermutlich kein gueltiger Download.")
        return 1
    print(f"OK ({DEST.stat().st_size // 1024} KB). HDRI: Poly Haven, CC0.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
