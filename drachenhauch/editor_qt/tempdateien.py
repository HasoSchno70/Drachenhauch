"""Temporaere `.dh`-Dateien der IDE -- anlegen und liegengebliebene aufraeumen.

Fehlerpruefung, Debugger und Profiler schreiben den Puffer in eine temporaere
Datei und lassen `dhrt` darauf los. Diese Datei liegt **absichtlich neben der
Quelle**: `IMPORT "helfer.dh"` loest relativ zum Verzeichnis der Datei auf --
im Systemverzeichnis meldete der Pruefer Fehler, die es gar nicht gibt.

Der Preis dafuer: wird der Lauf abgebrochen (IDE erschlagen, Absturz, ein
Test, der das Fenster nicht sauber schliesst), bleibt sie liegen. In
`examples/` kippt so ein Streuner jede Zaehlung und jeden
`glob("*.dh")`-Test -- das ist schon mehrfach passiert.

**Wie hier gelöscht wird, ohne fremde Dateien zu treffen:**

1. Der Name traegt ein eigenes Praefix UND die Prozessnummer, die sie
   angelegt hat: `_dhtmp_<pid>_xxxxxxxx.dh`. Eine Datei ohne dieses Muster
   wird nie angefasst -- auch nicht `tmpXXXXXXXX.dh` aus der Zeit davor:
   was sich nicht als unser Werk ausweist, koennte jemandem gehoeren.
2. Geloescht wird nur, wenn der Prozess mit dieser Nummer **nicht mehr
   laeuft**. Eine zweite IDE mitten in einer Pruefung -- oder in einer
   stundenlangen Debug-Sitzung -- verliert ihre Datei also nicht. Ein
   Altersvergleich koennte das nicht leisten: eine Sitzung darf beliebig
   lange dauern.
3. Zusaetzlich muss die Datei mindestens eine Minute alt sein. Prozessnummern
   werden vom Betriebssystem wiederverwendet; das faengt den Fall ab, dass
   eine gerade angelegte Datei die Nummer eines laengst beendeten Prozesses
   traegt.
"""
from __future__ import annotations

import os
import re
import tempfile
from pathlib import Path

PRAEFIX = "_dhtmp_"
MUSTER = re.compile(r"^_dhtmp_(\d+)_")
MINDESTALTER_S = 60.0


def neu(verzeichnis) -> tuple[int, str]:
    """Wie `tempfile.mkstemp(suffix=".dh")`, nur mit erkennbarem Namen.

    `verzeichnis` darf None sein (dann das Systemverzeichnis) -- der Aufrufer
    entscheidet, ob die Datei neben der Quelle liegen muss.
    """
    return tempfile.mkstemp(prefix=f"{PRAEFIX}{os.getpid()}_", suffix=".dh",
                            dir=verzeichnis)


def _laeuft(pid: int) -> bool:
    """Gibt es diesen Prozess noch?

    `os.kill(pid, 0)` schickt kein Signal, sondern fragt nur nach -- auch
    unter Windows (dort ueber OpenProcess). `PermissionError` heisst: es gibt
    ihn, er gehoert nur jemand anderem. **Im Zweifel `True`**: lieber eine
    Datei zu viel liegen lassen als eine fremde loeschen.
    """
    if pid <= 0:
        return True
    try:
        os.kill(pid, 0)
    except PermissionError:
        return True
    except OSError:
        return False
    except Exception:
        return True
    return True


def aufraeumen(verzeichnisse, jetzt: float | None = None,
               eigene_auch: bool = False) -> list[Path]:
    """Liegengebliebene Temp-Dateien entfernen. Liefert die geloeschten Pfade.

    `eigene_auch` nimmt die Dateien DIESES Prozesses dazu, ohne Altersgrenze.
    Das ist am ENDE eines Laufs richtig und sonst nie: dort steht fest, dass
    sie niemand mehr braucht. Der Testlauf braucht es, weil er seine Prozesse
    per `os._exit()` beendet -- das `finally`, das sonst aufraeumt, kommt
    dann nicht mehr dran.

    Wirft nie -- ein Aufraeumen darf den Start der IDE nicht verhindern.
    """
    import time
    jetzt = time.time() if jetzt is None else jetzt
    weg: list[Path] = []
    gesehen: set[Path] = set()
    for v in verzeichnisse:
        try:
            # Nicht nur OSError: `Path(None)` wirft TypeError, `Path("\0")`
            # ValueError. Ein Aufraeumen darf an keiner Eingabe scheitern.
            d = Path(v).resolve()
            if d in gesehen or not d.is_dir():
                continue
        except (OSError, TypeError, ValueError):
            continue
        gesehen.add(d)
        try:
            kandidaten = list(d.glob(f"{PRAEFIX}*.dh"))
        except OSError:
            continue
        for p in kandidaten:
            m = MUSTER.match(p.name)
            if not m:
                continue
            eigene = int(m.group(1)) == os.getpid()
            if not (eigene and eigene_auch):
                try:
                    if jetzt - p.stat().st_mtime < MINDESTALTER_S:
                        continue
                except OSError:
                    continue
                if _laeuft(int(m.group(1))):
                    continue
            try:
                p.unlink()
                weg.append(p)
            except OSError:
                pass
    return weg
