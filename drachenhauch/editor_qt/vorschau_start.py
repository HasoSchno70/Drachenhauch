"""Ein Vorschau-Programm aus einem Editor starten.

Die Begleit-Editoren (Animation, Sprite, Partikel, Form-Designer) starten mit
F5 ein kleines Programm zur Ansicht. Bisher lief das ueber `subprocess.Popen`
-- und weil eine GUI-Anwendung keine eigene Konsole hat, legte Windows dem
Kind eine an: ein leeres schwarzes Fenster neben der Vorschau.

Das Flag `CREATE_NO_WINDOW` waere die schnelle Antwort gewesen, aber eine
schlechte: die Konsole war die EINZIGE Stelle, an der eine Absturzmeldung des
Vorschau-Programms ankam. Hier laeuft der Start deshalb ueber `QProcess`:

* Qt setzt `CREATE_NO_WINDOW` selbst -- kein Fenster (nachgemessen).
* Die Ausgabe wird eingesammelt und bei einem Fehlschlag IM EDITOR gezeigt,
  statt in einem Fenster zu verschwinden, das sich sofort wieder schliesst.
* Qt leert die Ausgabe-Kanaele nebenher. Mit `subprocess.PIPE` und ohne
  Mitlesen bliebe ein Programm stehen, sobald es den Puffer vollgeschrieben
  hat (typisch ~64 KB) -- eine Vorschau mit PRINT in der Schleife waere
  eingefroren.

Ein absichtliches Beenden (Stop-Knopf) gilt NICHT als Fehler.
"""
from __future__ import annotations

from pathlib import Path

from PySide6.QtCore import QProcess
from PySide6.QtWidgets import QMessageBox


class Vorschau:
    """Laufender Vorschau-Prozess. `stoppen()` beendet ihn ohne Fehlermeldung."""

    def __init__(self, parent, befehl, arbeitsverzeichnis, titel):
        self._titel = titel
        self._parent = parent
        self._gewollt_beendet = False
        self.proc = QProcess(parent)
        self.proc.setProcessChannelMode(QProcess.ProcessChannelMode.MergedChannels)
        if arbeitsverzeichnis:
            self.proc.setWorkingDirectory(str(arbeitsverzeichnis))
        self.proc.finished.connect(self._fertig)
        self.proc.errorOccurred.connect(self._panne)
        self.proc.start(str(befehl[0]), [str(a) for a in befehl[1:]])

    # -- Zustand ------------------------------------------------------------
    def laeuft(self) -> bool:
        return self.proc.state() != QProcess.ProcessState.NotRunning

    def stoppen(self) -> None:
        """Beendet den Lauf hart -- `kill()`, NICHT `terminate()`.

        `QProcess::terminate()` schickt auf Windows nur ein WM_CLOSE an die
        Fenster des Prozesses. Ein `dhrt`-Lauf hat in dem Moment oft noch gar
        keins (oder reagiert nicht darauf), und der Prozess lief einfach
        weiter: der Testlauf im Vorschau-Test hing 15 Sekunden im Timeout, wo
        er nach Millisekunden fertig sein sollte. Das vorherige
        `subprocess.terminate()` war ein hartes TerminateProcess -- `kill()`
        ist dessen Entsprechung und haelt das Verhalten des Stop-Knopfes.
        """
        self._gewollt_beendet = True
        if self.laeuft():
            self.proc.kill()

    # -- Rueckmeldung -------------------------------------------------------
    def _ausgabe(self) -> str:
        roh = bytes(self.proc.readAllStandardOutput()).decode("utf-8", "replace")
        return roh.strip()[-4000:]      # lange Ausgaben hinten abschneiden

    def _fertig(self, code, status) -> None:
        if self._gewollt_beendet:
            return
        schlecht = (code != 0
                    or status == QProcess.ExitStatus.CrashExit)
        if schlecht:
            text = self._ausgabe() or f"Das Programm endete mit Code {code}."
            QMessageBox.warning(self._parent, f"{self._titel} fehlgeschlagen", text)

    def _panne(self, fehler) -> None:
        if self._gewollt_beendet:
            return
        if fehler == QProcess.ProcessError.FailedToStart:
            QMessageBox.critical(self._parent, f"{self._titel} fehlgeschlagen",
                                 "Das Programm liess sich nicht starten:\n"
                                 f"{self.proc.program()}")


def starte_vorschau(parent, befehl, arbeitsverzeichnis=None, titel="Vorschau"):
    """Startet `befehl` (Liste) ohne Konsolenfenster. Liefert eine `Vorschau`."""
    return Vorschau(parent, [Path(befehl[0])] + list(befehl[1:]),
                    arbeitsverzeichnis, titel)


def starte_werkzeug(befehl, arbeitsverzeichnis=None) -> bool:
    """Ein eigenstaendiges WERKZEUG starten (kein Vorschau-Lauf).

    Unterschied zu `starte_vorschau`: der gestartete Editor soll seinen
    Starter UEBERLEBEN. Ein `QProcess` mit Eltern-Objekt taete das nicht --
    sein Destruktor beendet den Prozess, das Schliessen des Score-Editors
    haette also den gerade geoeffneten Tracker mitgerissen.

    `startDetached` loest beides: der Prozess laeuft unabhaengig weiter und
    bekommt trotzdem kein Konsolenfenster (beides nachgemessen -- das Kind
    schrieb seine Datei noch 6 s nach dem Ende des Elternprozesses zu Ende,
    und die Fensterwache sah 0 Konsolen).

    Eine Rueckmeldung bei einem Absturz gibt es hier bewusst nicht: ein
    eigenstaendiger Editor meldet seine Fehler selbst, und mitzulesen hiesse,
    ihn wieder an den Starter zu binden.
    """
    ok, _pid = QProcess.startDetached(str(befehl[0]), [str(a) for a in befehl[1:]],
                                      str(arbeitsverzeichnis or ""))
    return bool(ok)
