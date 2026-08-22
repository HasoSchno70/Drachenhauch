#!/usr/bin/env python
# -*- coding: utf-8 -*-
r"""Bilder fuers Einstiegsbuch aufnehmen.

Rendert jede figures/*.dh ueber dhrt nach images/<name>.png.

Warum figures/ neben code/ steht: der Screenshot faellt beim LETZTEN Frame.
Die Kapitelprogramme der ersten Teile zeichnen aber nur EINMAL und warten
dann mit SLEEP -- ein einzelner FLIP kommt zu frueh, das Bild wird schwarz.
Die Figurenquelle enthaelt deshalb dieselben Zeichenbefehle in einer kurzen
Schleife. Sie ist keine zweite Wahrheit: pruef_codebloecke.js prueft den
Abdruck im Buch, und beides laeuft durch `dhrt --check`.

SCALE=3 bei 640x400 -> 1920x1200. Gemessen: mehr geht nicht. Mit SCALE=4
waere das Fenster 2560x1600 und damit hoeher als der Bildschirm; Windows
schiebt es dann nach unten weg, und die Aufnahme bekam oben 179 schwarze
Zeilen. 1920 Breite ist zugleich das Mass des Referenzbuchs und reicht fuer
den 300-dpi-Druck.

Aufruf:  <venv>\python.exe shoot.py [nur_diese_basename ...]
"""
import os
import re
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
FIG = os.path.join(HIER, "figures")
IMG = os.path.join(HIER, "images")
DHRT = os.path.join(HIER, "..", "..", "rust", "drachenhauch_runtime",
                    "target", "release", "dhrt.exe")
FONT = os.environ.get("BUCH_FONT", r"C:\Windows\Fonts\segoeui.ttf")
SCALE = "3"
FRAMES_VORGABE = 12

# Mehr Frames, wo sich etwas entwickeln muss (Animation, Physik, Partikel).
FRAMES = {
    "kap05_1_wanderer": 60, "kap05_2_fallender_ball": 46,
    "kap05_3_spur": 38, "kap05_4_regen": 40, "kap05_5_monde": 70,
    # Ampel: Bild 75 faellt in Phase 1 -- rot UND gelb, man sieht,
    # dass gleich etwas passiert. Ein Standbild mit nur einer Lampe
    # saehe aus wie die feste Ampel aus Kapitel 1.
    "kap08_3_ball": 44, "kap09_snake": 90, "kap10_3_sirene": 720, "kap11_1_treffer": 40, "kap11_2_laser": 20, "kap11_3_pong_mit_ton": 300, "kap12_instrument": 20, "kap13_1_baum": 12, "kap13_2_wald": 12, "kap13_3_naehe": 12, "kap13_4_pong_kuerzer": 300, "kap14_1_funken": 120, "kap14_2_leben": 180, "kap14_3_bestenliste": 12, "kap15_1_farbregister": 12, "kap15_2_tastenzaehler": 12, "kap16_1_funken_klasse": 120, "kap16_2_planeten": 200, "kap17_1_schreibmaschine": 130, "kap17_2_laufschrift": 60, "kap17_3_buchstaben": 12, "kap18_1_bestwert": 12, "kap19_1_bild_zeigen": 12, "kap19_2_schiff_steuern": 12, "kap19_3_flotte": 30, "kap21_1_zwei_frames": 20, "kap21_2_muenze": 5, "kap21_3_flotte_lebt": 30, "kap22_1_rechtecke": 12, "kap22_2_einsammeln": 12, "kap23_arcade": 90, "kap24_1_erster_knopf": 12, "kap24_2_schieber": 12, "kap25_1_liste_fuellen": 12, "kap25_2_auswahl": 12, "kap26_1_reiter": 12, "kap27_1_vokabeln_db": 12, "kap18_2_vokabeln": 12, "kap08_pong": 300,
    "kap06_1_abpraller": 50, "kap06_3_kasten": 55, "kap06_5_ampel": 75,
    # Kapitel 29 holt die Liste im Hintergrund: gemessen waren es rund
    # 200 ms bis zur Antwort, bei 60 Bildern je Sekunde also gut ein
    # Dutzend Bilder. 90 lassen Luft, wenn die Leitung langsamer ist.
    "kap29_2_liste_laden": 90, "kap32_verwalten": 30,
    "kap33_trainer": 30,
}


def quelle_finden(name):
    """Erst figures/, dann das Kapitelprogramm selbst.

    Ab Kapitel 5 haben die Programme eine eigene Schleife und laufen so
    lange, bis man sie beendet -- die lassen sich unveraendert aufnehmen,
    und eine zweite Quelle waere nur eine zweite Wahrheit. Nur die
    linearen Programme der ersten Kapitel brauchen die Schleifenfassung
    unter figures/, weil ein einzelner FLIP fuer die Aufnahme zu frueh
    kommt. Aus `kap05_1_wanderer` wird `../code/kap05/1_wanderer.dh`.
    """
    eigen = os.path.join(FIG, name + ".dh")
    if os.path.exists(eigen):
        return eigen
    # `anhang` steht neben den kapNN-Ordnern: die Anhaenge haben keine
    # Kapitelnummer, ihre Programme aber dieselbe Behandlung verdient.
    m = re.match(r"(kap\d+|anhang)_(.+)$", name)
    if m:
        p = os.path.join(HIER, "..", "code", m.group(1), m.group(2) + ".dh")
        if os.path.exists(p):
            return p
    return None


def aufnehmen(name):
    quelle = quelle_finden(name)
    if quelle is None:
        print("  (keine Quelle)", name)
        return False
    ziel = os.path.join(IMG, name + ".png")
    frames = FRAMES.get(name, FRAMES_VORGABE)
    umgebung = dict(os.environ)
    umgebung.update({"DHRT_FRAMES": str(frames), "DHRT_SCALE": SCALE,
                     "DHRT_SCREENSHOT": ziel})
    if os.path.exists(FONT):
        umgebung["DHRT_FONT"] = FONT
    r = subprocess.run([DHRT, "run", quelle], env=umgebung,
                       capture_output=True, text=True, encoding="utf-8")
    if r.returncode != 0 or not os.path.exists(ziel):
        print("  FEHLER", name, (r.stderr or "").strip()[:200])
        return False
    print(f"  {name}.png  ({frames} Frames)")
    return True


def main(argv):
    os.makedirs(IMG, exist_ok=True)
    if argv:
        namen = argv
    else:
        namen = {f[:-3] for f in os.listdir(FIG) if f.endswith(".dh")}
        code = os.path.join(HIER, "..", "code")
        for kap in sorted(os.listdir(code)):
            for f in sorted(os.listdir(os.path.join(code, kap))):
                if f.endswith(".dh") and f"{kap}_{f[:-3]}" not in namen:
                    # Nur Programme mit eigener Schleife: die linearen haben
                    # ihre Schleifenfassung schon unter figures/.
                    text = open(os.path.join(code, kap, f), encoding="utf-8").read()
                    if "WHILE" in text.upper():
                        namen.add(f"{kap}_{f[:-3]}")
        namen = sorted(namen)
    gut = sum(1 for n in namen if aufnehmen(n))
    print(f"{gut}/{len(namen)} Bilder")
    return 0 if gut == len(namen) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
