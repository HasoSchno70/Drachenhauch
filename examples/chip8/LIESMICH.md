# CHIP-8

Ein Emulator für CHIP-8, die kleine virtuelle Maschine aus dem Jahr 1977:
4096 Bytes Speicher, 16 Register, zwei Zeitgeber mit 60 Hz und ein Bild aus
64 × 32 Punkten. Dazu ein Assembler und ein eigenes Spiel.

    dhrt run examples/chip8/main.dh                 ; das mitgelieferte Spiel
    dhrt run examples/chip8/main.dh -- spiel.ch8    ; ein fertiges Programm
    dhrt run examples/chip8/main.dh -- quelle.c8s   ; Quelltext, wird assembliert

Eine Datei lässt sich auch auf das Fenster ziehen.

- **Tastenfeld:** 1 2 3 4 / Q W E R / A S D F / Z X C V. Gezählt wird die
  Lage, nicht die Beschriftung: auf einer deutschen Tastatur ist es Y statt Z.
- **F5** startet neu, **P** hält an, **Bild auf/ab** (oder **+**/**−** im
  Ziffernblock) ändert das Tempo, **F2** schaltet die Eigenheiten des
  COSMAC VIP um, **ESC** beendet.

## Die Dateien

- `cpu.dh` -- der Rechenkern ohne Fenster: alle 35 Befehle, Zeitgeber,
  Sprites mit XOR und Kollision. Die bekannten Eigenheiten der Varianten
  (Schieben über VY, `I` beim Laden weiterzählen, `BXNN`, Logik setzt VF,
  Rand umbrechen) stehen als Merker; die Vorgabe ist das Verhalten, das die
  meisten Spiele seit CHIP-48 erwarten.
- `asm.dh` -- ein Assembler in der Schreibweise aus *Cowgod's Chip-8
  Technical Reference*: Marken, Kommentare ab `;`, Zahlen dezimal, hex
  (`#`, `$`, `0x`) und binär (`%`), `DB`/`DW` für Daten. Zwei Durchgänge,
  also dürfen Marken auch weiter unten stehen; Fehler kommen mit Zeile.
- `faenger.c8s` -- ein Spiel: ein Ball springt, unten fängt ihn der
  Schläger (Tasten 4 und 6, auf der Tastatur Q und E). Punkte links oben,
  Leben rechts oben.
- `main.dh` -- Fenster, Tastatur, Ton und Laden. Punkte, die ausgehen,
  leuchten kurz nach wie auf einem alten Bildschirm -- das dämpft das
  Flackern, das CHIP-8-Spiele vom Zeichnen mit XOR haben.

Der Kern schafft gut eine Million Befehle je Sekunde; ein CHIP-8 braucht
etwa tausend. Die Vorgabe sind 15 je Bild (900 je Sekunde).

Entstanden beim Dogfooding am 2026-10-24. Geprüft wird in
`tests/pruef/beispiel_chip8.dhtest`: jeder Befehl einzeln mit Übertrag und
Borgen, beide Eigenheiten, Zeichnen mit Rand, Warten auf eine Taste bis zum
Loslassen, die Fehler, der Assembler Byte für Byte, das Spiel ohne Fenster
und das Fenster über `dhrt bild`.
