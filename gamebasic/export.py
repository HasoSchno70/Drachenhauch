"""Standalone-Export: ein GameBasic-Programm zu einer eigenstaendigen `.exe`
buendeln, die OHNE Python laeuft (Schritt 7 der nativen Runtime).

Prinzip (kein Recompile noetig): der kompilierte Bytecode (`.gbc`) wird an eine
Kopie der nativen Runtime `gbrt.exe` **angehaengt**. `gbrt` erkennt beim Start
den angehaengten Payload (Magic-Footer am Datei-Ende), wechselt ins
Exe-Verzeichnis und fuehrt den eingebetteten Bytecode aus. Das Anhaengen von
Daten an eine PE-Exe bricht sie nicht (gleiches Prinzip wie PyInstaller-onefile).

Layout der letzten 16 Bytes der gebundelten Exe:
    [u64 Laenge der .gbc-Bytes, little-endian][8 Byte Magic "GBRTPAY1"]
Die `.gbc`-Bytes liegen direkt vor diesem Footer.

Assets (Bilder/Sounds/Fonts/Modelle/...) werden per Konvention im Unterordner
`assets/` neben der Quelle erwartet und mit ins Ausgabeverzeichnis kopiert --
relative Pfade wie `LOADIMAGE("assets/held.png")` funktionieren dann beim
Doppelklick von ueberall (gbrt wechselt ins Exe-Verzeichnis).
"""
from __future__ import annotations

import os
import shutil
import struct
from pathlib import Path

from .serialize import compile_gb_to_gbc_text

PAYLOAD_MAGIC = b"GBRTPAY1"


def append_payload(exe_bytes: bytes, gbc_bytes: bytes) -> bytes:
    """Haengt den `.gbc`-Payload + Footer an die Runtime-Exe-Bytes an."""
    footer = struct.pack("<Q", len(gbc_bytes)) + PAYLOAD_MAGIC
    return exe_bytes + gbc_bytes + footer


def export_standalone(src_gb: str | Path, gbrt_path: str | Path,
                      out_dir: str | Path | None = None,
                      *, copy_assets: bool = True) -> Path:
    """Buendelt `src_gb` zu einer eigenstaendigen Exe im Ausgabeverzeichnis.

    - `src_gb`     : Pfad zur `.gb`-Quelldatei.
    - `gbrt_path`  : Pfad zur gebauten nativen Runtime (`gbrt.exe` / `gbrt`).
    - `out_dir`    : Zielordner (Default: `<quelle>_dist/` neben der Quelle).
    - `copy_assets`: kopiert `<quelle-ordner>/assets/` ins Ausgabeverzeichnis.

    Liefert den Pfad zur erzeugten Exe. Wirft `FileNotFoundError`, wenn die
    Runtime fehlt, und reicht Compile-Fehler (`GameBasicError`) durch.
    """
    src = Path(src_gb).resolve()
    gbrt = Path(gbrt_path)
    if not gbrt.exists():
        raise FileNotFoundError(f"Native Runtime nicht gefunden: {gbrt}")

    out = Path(out_dir) if out_dir is not None else src.parent / f"{src.stem}_dist"
    out.mkdir(parents=True, exist_ok=True)

    # 1) Bytecode in-memory erzeugen (kompakt).
    gbc_text = compile_gb_to_gbc_text(src)
    gbc_bytes = gbc_text.encode("utf-8")

    # 2) Runtime + Payload zur Spiel-Exe zusammensetzen.
    suffix = ".exe" if gbrt.suffix == ".exe" or os.name == "nt" else ""
    out_exe = out / f"{src.stem}{suffix}"
    out_exe.write_bytes(append_payload(gbrt.read_bytes(), gbc_bytes))
    if os.name != "nt":
        out_exe.chmod(0o755)

    # 3) Assets mitkopieren (Konvention: Unterordner assets/).
    if copy_assets:
        assets = src.parent / "assets"
        if assets.is_dir():
            shutil.copytree(assets, out / "assets", dirs_exist_ok=True)

    return out_exe
