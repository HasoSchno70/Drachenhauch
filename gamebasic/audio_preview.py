"""Additiver Software-Mixer fuer Live-Vorhoeren in den Qt-Editoren.

Qt-frei (nur `threading`+`numpy`+`sounddevice`) -- geteilt zwischen
`trackereditor_qt.py` und `scoreeditor_qt.py`, die beide mehrere gleichzeitig
klingende Stimmen (Kanaele/Spuren) ueber ein einziges Audio-Geraet abspielen
muessen.
"""
from __future__ import annotations

import threading
from typing import Any

import numpy as np


class Mixer:
    """Additiver Software-Mixer fuer ueberlappende Stimmen.

    `sounddevice.play()` ersetzt bei jedem Aufruf die vorher gestartete
    Wiedergabe (Doku: "It cannot be used for multiple overlapping
    playbacks.") -- ein Tracker/Notenblatt-Editor braucht aber mehrere
    GLEICHZEITIG klingende Kanaele/Spuren (Akkorde/Drums) und Noten, die ueber
    die naechste Zeile/den naechsten Takt hinaus nachklingen. Ein einziger
    dauerhaft offener `OutputStream` mischt stattdessen alle aktiven Stimmen
    additiv."""

    def __init__(self, sr: int = 44100):
        self._sr = sr
        self._lock = threading.Lock()
        self._voices: list[tuple[np.ndarray, int]] = []
        # sounddevice hat keine Typstubs und wird nur bei Bedarf lokal
        # importiert (optionale Editor-Abhaengigkeit) -- Any statt eines
        # unbekannten Stream-Typs.
        self._stream: Any = None
        # Kein Audio-Geraet (haeufig: Sandboxes/CI/`SDL_AUDIODRIVER=dummy`) ->
        # sd.OutputStream() scheitert dauerhaft. EIN gescheiterter Versuch
        # reicht -- kein Retry bei jedem play()-Aufruf noetig/sinnvoll.
        self._stream_failed = False

    def _callback(self, outdata, frames, _time, _status) -> None:
        outdata.fill(0.0)
        with self._lock:
            alive = []
            for arr, pos in self._voices:
                end = min(pos + frames, arr.size)
                n = end - pos
                if n > 0:
                    outdata[:n, 0] += arr[pos:end]
                if end < arr.size:
                    alive.append((arr, end))
            self._voices = alive
        np.clip(outdata, -1.0, 1.0, out=outdata)

    def play(self, arr: np.ndarray) -> None:
        if arr is None or arr.size == 0:
            return
        if self._stream is None and not self._stream_failed:
            try:
                import sounddevice as sd
                self._stream = sd.OutputStream(
                    samplerate=self._sr, channels=1, dtype="float32",
                    callback=self._callback)
                self._stream.start()
            except Exception:
                self._stream = None
                self._stream_failed = True
        if self._stream is None:
            # Kein Geraet -- Stimme NICHT queuen (der Callback, der _voices
            # drainen wuerde, laeuft ohne Stream nie; sonst waechst _voices
            # bei jedem play()-Aufruf unbegrenzt, siehe Review-Fund).
            return
        voice = np.ascontiguousarray(arr, dtype=np.float32)
        with self._lock:
            self._voices.append((voice, 0))

    def stop(self) -> None:
        """Beendet den Stream (falls vorhanden) und leert die Warteschlange --
        Editoren rufen das aus ihrem closeEvent(), sonst lebt der Stream +
        sein Hintergrund-Thread bis zur naechsten GC weiter."""
        with self._lock:
            self._voices = []
        stream, self._stream = self._stream, None
        if stream is not None:
            try:
                stream.stop()
                stream.close()
            except Exception:
                pass

    def __del__(self):
        try:
            self.stop()
        except Exception:
            pass
