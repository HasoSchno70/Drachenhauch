#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Baut die GameBasic-Distribution fuer das aktuelle Betriebssystem.

Schritte (alle Plattformen):
  1. dhrt-Runtime sicherstellen (rust/build_runtime.py, falls fehlend).
  2. App-Icon aus gamebasic/assets/logo.png erzeugen (.ico Windows, .icns macOS).
  3. PyInstaller: gbrun.py + gamebasic-Paket -> dist/GameBasic[.app]/ (onedir,
     ohne Python).
  4. Plattformspezifische Paketierung:
     - Windows: Inno Setup (ISCC) -> installer/output/GameBasic-Setup-<version>.exe
     - macOS:   dhrt neben die App-Binary legen, .app in .dmg packen (hdiutil)
     - Linux:   dhrt neben die App-Binary legen, .tar.gz + install.sh (XDG
                Desktop-Integration ohne sudo/root)

Aufruf (im .venv):
  .venv\\Scripts\\python.exe installer\\build_installer.py     # Windows
  .venv/bin/python installer/build_installer.py                # Linux/macOS
Optionen:
  --no-installer   nur PyInstaller (kein Paketier-Schritt)
  --rebuild-dhrt   dhrt vorher neu bauen

**Cross-Platform-Status:** Der Windows-Pfad ist etabliert und lokal
verifiziert. macOS/Linux sind neu (Cross-Platform-Migration Phase 4) und
NICHT auf echter Hardware getestet (diese Entwicklungsumgebung ist
Windows-only) -- nur PyInstaller selbst lokal gegen den Windows-Zweig
regressionsgetestet. Rueckmeldungen von echten macOS-/Linux-Nutzern
willkommen.
"""
import os
import platform
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INST = ROOT / "installer"
PY = Path(sys.executable)
SYSTEM = platform.system()  # "Windows" | "Darwin" | "Linux"
EXE_SUFFIX = ".exe" if SYSTEM == "Windows" else ""
DHRT = ROOT / "rust" / "drachenhauch_runtime" / "target" / "release" / f"dhrt{EXE_SUFFIX}"


def log(msg):
    print(f"\n=== {msg} ===", flush=True)


# --------------------------------------------------------------- Code-Signing
# Inert, solange KEIN Zertifikat konfiguriert ist. Aktivieren ueber Umgebungs-
# variablen (dann werden GameBasic.exe, dhrt.exe UND der Installer signiert):
#   GB_SIGN_CERT  = Pfad zur .pfx-Datei  ODER  SHA1-Thumbprint im Windows-Zertspeicher
#   GB_SIGN_PASS  = Passwort der .pfx    (nur bei .pfx noetig)
#   GB_SIGN_TS    = RFC3161-Timestamp-URL (Default: DigiCert)
# Beispiel:
#   set GB_SIGN_CERT=C:\keys\meincert.pfx
#   set GB_SIGN_PASS=geheim
#   .venv\Scripts\python.exe installer\build_installer.py
# Nur Windows -- macOS-Codesigning/Notarization (Apple-Entwicklerkonto noetig)
# und Linux-Paket-Signierung (GPG) sind separate, hier NICHT implementierte
# Themen.
def _find_signtool():
    if os.environ.get("SIGNTOOL") and Path(os.environ["SIGNTOOL"]).exists():
        return os.environ["SIGNTOOL"]
    found = shutil.which("signtool")
    if found:
        return found
    # Windows-SDK: neueste signtool.exe (x64) suchen.
    base = Path(r"C:\Program Files (x86)\Windows Kits\10\bin")
    cands = sorted(base.glob("*/x64/signtool.exe"), reverse=True) if base.exists() else []
    return str(cands[0]) if cands else None


def sign(path: Path):
    """Signiert eine Datei -- NUR wenn GB_SIGN_CERT gesetzt ist (sonst No-Op).
    Nur unter Windows relevant."""
    if SYSTEM != "Windows":
        return
    cert = os.environ.get("GB_SIGN_CERT")
    if not cert:
        return  # Signierung nicht konfiguriert -> still ueberspringen
    st = _find_signtool()
    if not st:
        print("WARN: signtool.exe nicht gefunden (Windows SDK) -> nicht signiert:", path)
        return
    ts = os.environ.get("GB_SIGN_TS", "http://timestamp.digicert.com")
    args = [st, "sign", "/fd", "SHA256", "/tr", ts, "/td", "SHA256"]
    if Path(cert).exists():                 # .pfx-Datei
        args += ["/f", cert]
        if os.environ.get("GB_SIGN_PASS"):
            args += ["/p", os.environ["GB_SIGN_PASS"]]
    else:                                   # Thumbprint im Zertifikatsspeicher
        args += ["/sha1", cert]
    args.append(str(path))
    log(f"Signiere {Path(path).name}")
    subprocess.run(args, check=True)


def version():
    sys.path.insert(0, str(ROOT))
    from gamebasic import __version__
    return __version__


def ensure_dhrt(rebuild):
    if rebuild or not DHRT.exists():
        log("dhrt-Runtime bauen")
        subprocess.run([str(PY), str(ROOT / "rust" / "build_runtime.py")],
                       cwd=ROOT, check=True)
    if not DHRT.exists():
        # Nicht hart abbrechen -- ein Paketier-Testlauf (z.B. in CI, ohne
        # volle Grafik-Toolchain) soll trotzdem eine Bundle-Struktur pruefen
        # koennen. Die fertige App findet dann beim Start kein dhrt und
        # meldet das ihrerseits klar (siehe dhrt_locate.py).
        print(f"WARNUNG: {DHRT} wurde nicht gebaut -- Paket enthaelt keine "
              f"Runtime (nur zum Struktur-Testen brauchbar, nicht zum Verteilen).")
    else:
        print("dhrt:", DHRT)


def _schreibe_ico(ziel, eintraege):
    """Ein .ico mit je Groesse EIGENER Vorlage schreiben.

    Pillows `save(format="ICO")` skaliert immer dasselbe Bild -- fuer
    unterschiedliche Zeichnungen je Groesse muss der Behaelter selbst
    geschrieben werden. Das Format ist schlicht: Kopf, ein 16-Byte-Eintrag je
    Aufloesung, dann die Bilddaten. Seit Windows Vista duerfen die Eintraege
    PNG-komprimiert sein (traegt Alpha sauber und spart Platz).

    `eintraege`: Liste von (Kantenlaenge, Bild), absteigend sortiert.
    """
    import io
    import struct

    from PIL import Image      # wie in make_icon() erst hier -- Pillow ist
                               # nur fuer das Paketieren noetig, nicht zum Bauen

    roh = []
    for kante, bild in eintraege:
        puffer = io.BytesIO()
        bild.resize((kante, kante), Image.LANCZOS).save(
            puffer, format="PNG", optimize=True)
        roh.append((kante, puffer.getvalue()))

    kopf = struct.pack("<HHH", 0, 1, len(roh))       # reserviert, Typ 1, Anzahl
    verzeichnis, daten = b"", b""
    versatz = len(kopf) + 16 * len(roh)
    for kante, bytes_ in roh:
        b = 0 if kante >= 256 else kante             # 256 wird als 0 kodiert
        verzeichnis += struct.pack("<BBBBHHII", b, b, 0, 0, 1, 32,
                                   len(bytes_), versatz)
        daten += bytes_
        versatz += len(bytes_)
    ziel.write_bytes(kopf + verzeichnis + daten)


def make_icon():
    log("App-Icon erzeugen")
    try:
        from PIL import Image
    except ImportError:
        print("Pillow fehlt -> ohne Icon."); return
    src = ROOT / "gamebasic" / "assets" / "logo.png"
    if not src.exists():
        print("logo.png fehlt -> ohne Icon."); return
    im = Image.open(src).convert("RGBA")
    # Auf Quadrat bringen (transparent gepolstert).
    s = max(im.size)
    canvas = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    canvas.paste(im, ((s - im.width) // 2, (s - im.height) // 2))

    # Kleine Groessen duerfen eine EIGENE Zeichnung bekommen: das volle Logo
    # traegt einen langen Schweif, der bei 16 px nur noch Rauschen ist und dem
    # Motiv Platz wegnimmt. Liegt daneben ein `logo-kachel.png` (nur das
    # quadratische Motiv), wird das ab 64 px abwaerts benutzt.
    klein_src = ROOT / "gamebasic" / "assets" / "logo-kachel.png"
    klein = None
    if klein_src.exists():
        k = Image.open(klein_src).convert("RGBA")
        s2 = max(k.size)
        klein = Image.new("RGBA", (s2, s2), (0, 0, 0, 0))
        klein.paste(k, ((s2 - k.width) // 2, (s2 - k.height) // 2))

    def fuer(kante):
        """Welche Vorlage gehoert in diese Kantenlaenge?"""
        return klein if (klein is not None and kante <= 64) else canvas

    if SYSTEM == "Windows":
        ico = INST / "GameBasic.ico"
        kanten = [256, 128, 64, 48, 32, 16]
        if klein is None:
            canvas.save(ico, sizes=[(k, k) for k in reversed(kanten)])
        else:
            _schreibe_ico(ico, [(k, fuer(k)) for k in kanten])
        print("Icon:", ico, "(zwei Vorlagen)" if klein is not None else "")
    elif SYSTEM == "Darwin":
        icns = INST / "GameBasic.icns"
        canvas.save(icns, format="ICNS")
        print("Icon:", icns)
    else:
        # Linux: kein Container-Format noetig, .desktop referenziert direkt
        # ein PNG. 256x256 ist die von der XDG-Icon-Spec bevorzugte Groesse
        # fuer hicolor/256x256/apps/.
        png = INST / "GameBasic.png"
        canvas.resize((256, 256)).save(png)
        print("Icon:", png)


def gen_notices():
    log("Drittanbieter-Lizenzen sammeln (THIRD-PARTY-NOTICES.txt)")
    try:
        subprocess.run([str(PY), str(INST / "gen_notices.py")], cwd=ROOT, check=True)
    except subprocess.CalledProcessError as e:
        print("WARN: Notices-Generierung fehlgeschlagen:", e)


def run_pyinstaller():
    log("PyInstaller (eingefrorene IDE)")
    dist = ROOT / "dist"
    for stale in (dist / "GameBasic", dist / "GameBasic.app"):
        if stale.exists():
            shutil.rmtree(stale)
    subprocess.run(
        [str(PY), "-m", "PyInstaller", str(INST / "GameBasic.spec"),
         "--noconfirm", "--distpath", str(dist),
         "--workpath", str(ROOT / "build" / "pyi")],
        cwd=ROOT, check=True)
    if SYSTEM == "Darwin":
        app = dist / "GameBasic.app"
        if not app.exists():
            sys.exit("FEHLER: PyInstaller-Ausgabe (.app) fehlt.")
        print("App:", app)
        return app
    exe = dist / "GameBasic" / f"GameBasic{EXE_SUFFIX}"
    if not exe.exists():
        sys.exit("FEHLER: PyInstaller-Ausgabe fehlt.")
    print("IDE:", exe)
    sign(exe)            # die ausgelieferte App-Exe signieren (falls konfiguriert, nur Windows)
    return exe


# ----------------------------------------------------------------- Windows
def find_iscc():
    for p in (os.environ.get("ISCC"),
              r"C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
              r"C:\Program Files\Inno Setup 6\ISCC.exe"):
        if p and Path(p).exists():
            return p
    found = shutil.which("ISCC")
    return found


def run_inno(ver):
    iscc = find_iscc()
    if not iscc:
        print("\nInno Setup (ISCC.exe) nicht gefunden. dist/GameBasic ist fertig.")
        print("Installer manuell bauen: ISCC.exe /DAppVersion=%s installer\\GameBasic.iss" % ver)
        return
    sign(DHRT)           # dhrt.exe signieren, BEVOR Inno sie einpackt
    log("Inno Setup (Installer verpacken)")
    subprocess.run([iscc, f"/DAppVersion={ver}", str(INST / "GameBasic.iss")],
                   cwd=INST, check=True)
    out = INST / "output" / f"GameBasic-Setup-{ver}.exe"
    if out.exists():
        sign(out)        # zuletzt den fertigen Installer signieren
    print("\nFERTIG ->", out if out.exists() else (INST / "output"))


# ------------------------------------------------------------------- macOS
def run_macos(app: Path, ver: str):
    # dhrt gehoert NEBEN die eingefrorene Binary (Contents/MacOS/), damit
    # dhrt_locate.py sie ueber sys.executable findet (siehe .spec-Kommentar).
    macos_dir = app / "Contents" / "MacOS"
    if DHRT.exists():
        shutil.copy2(DHRT, macos_dir / "dhrt")
        os.chmod(macos_dir / "dhrt", 0o755)
        print("dhrt kopiert ->", macos_dir / "dhrt")
    # EULA/Notices als Referenz mit ins Bundle (Resources) -- Pendant zu den
    # Startmenue-Eintraegen unter Windows.
    resources = app / "Contents" / "Resources"
    for f in ("EULA.txt", "THIRD-PARTY-NOTICES.txt"):
        src = INST / f
        if src.exists():
            shutil.copy2(src, resources / f)

    log("DMG erzeugen (hdiutil)")
    out_dir = INST / "output"
    out_dir.mkdir(exist_ok=True)
    dmg = out_dir / f"GameBasic-{ver}-macOS.dmg"
    if dmg.exists():
        dmg.unlink()
    hdiutil = shutil.which("hdiutil")
    if not hdiutil:
        print("WARNUNG: hdiutil nicht gefunden (kein macOS?) -- .app liegt "
              f"unverpackt unter {app}, kein .dmg erzeugt.")
        return
    subprocess.run([hdiutil, "create", "-volname", "GameBasic",
                     "-srcfolder", str(app), "-ov", "-format", "UDZO", str(dmg)],
                    check=True)
    print("\nFERTIG ->", dmg)


# ------------------------------------------------------------------- Linux
_INSTALL_SH = """#!/bin/sh
# Installiert GameBasic fuer den aktuellen Nutzer (kein root/sudo noetig),
# nach der XDG-Basisverzeichnis-Konvention. Zum Deinstallieren die unten
# genannten Pfade einfach loeschen.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
DEST="${HOME}/.local/share/GameBasic"
BIN="${HOME}/.local/bin"
APPS="${HOME}/.local/share/applications"
ICONS="${HOME}/.local/share/icons/hicolor/256x256/apps"

echo "Installiere nach $DEST ..."
mkdir -p "$DEST" "$BIN" "$APPS" "$ICONS"
cp -r "$HERE/GameBasic/." "$DEST/"
chmod +x "$DEST/GameBasic" 2>/dev/null || true
[ -f "$DEST/dhrt" ] && chmod +x "$DEST/dhrt"

cat > "$BIN/gamebasic" <<EOF
#!/bin/sh
exec "$DEST/GameBasic" "\\$@"
EOF
chmod +x "$BIN/gamebasic"

[ -f "$HERE/GameBasic.png" ] && cp "$HERE/GameBasic.png" "$ICONS/gamebasic.png"

cat > "$APPS/gamebasic.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=GameBasic
Comment=BASIC-Dialekt mit Pascal-strikter Typisierung und OOP fuer Spiele
Exec=$DEST/GameBasic %f
Icon=gamebasic
Categories=Development;IDE;
MimeType=text/x-gamebasic;
EOF

update-desktop-database "$APPS" 2>/dev/null || true
gtk-update-icon-cache 2>/dev/null || true

echo "Fertig. Falls '$BIN' nicht in deinem PATH ist, fuege es in deiner Shell-rc hinzu:"
echo "  export PATH=\\"\\$PATH:$BIN\\""
echo "GameBasic sollte jetzt auch im Anwendungsmenue auftauchen."
"""


def run_linux(exe: Path, ver: str):
    app_dir = exe.parent  # dist/GameBasic/
    if DHRT.exists():
        shutil.copy2(DHRT, app_dir / "dhrt")
        os.chmod(app_dir / "dhrt", 0o755)
        print("dhrt kopiert ->", app_dir / "dhrt")

    log("Tarball + install.sh erzeugen")
    out_dir = INST / "output"
    out_dir.mkdir(exist_ok=True)
    stage = ROOT / "build" / "linux-package"
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)
    shutil.copytree(app_dir, stage / "GameBasic")
    icon = INST / "GameBasic.png"
    if icon.exists():
        shutil.copy2(icon, stage / "GameBasic.png")
    for f in ("EULA.txt", "THIRD-PARTY-NOTICES.txt"):
        src = INST / f
        if src.exists():
            shutil.copy2(src, stage / f)
    install_sh = stage / "install.sh"
    install_sh.write_text(_INSTALL_SH, encoding="utf-8", newline="\n")
    os.chmod(install_sh, 0o755)

    tar_path = out_dir / f"GameBasic-{ver}-linux-x86_64.tar.gz"
    if tar_path.exists():
        tar_path.unlink()
    with tarfile.open(tar_path, "w:gz") as tar:
        tar.add(stage, arcname="GameBasic-dist")
    print("\nFERTIG ->", tar_path)
    print("Installation (kein sudo noetig): tar xzf", tar_path.name,
          "&& ./GameBasic-dist/install.sh")


def main():
    rebuild = "--rebuild-dhrt" in sys.argv
    no_inst = "--no-installer" in sys.argv
    ver = version()
    print(f"GameBasic {ver} -- Distributions-Build ({SYSTEM})")
    ensure_dhrt(rebuild)
    make_icon()
    gen_notices()
    built = run_pyinstaller()
    if no_inst:
        return
    if SYSTEM == "Windows":
        run_inno(ver)
    elif SYSTEM == "Darwin":
        run_macos(built, ver)
    elif SYSTEM == "Linux":
        run_linux(built, ver)
    else:
        print(f"WARNUNG: unbekanntes System '{SYSTEM}' -- nur PyInstaller-Bundle erzeugt: {built}")


if __name__ == "__main__":
    main()
