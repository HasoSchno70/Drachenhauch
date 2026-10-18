# Funkenflug

Ein **Schaltplan-Editor in Drachenhauch** mit GTK 3 als Oberfläche – die
Neufassung der Idee hinter [SparkNode](https://github.com/HasoSchno70/elektronik)
(ursprünglich Python + PySide6). GTK/Cairo-Ansatz wie bei *Glutfaden* (der
GTK-Portierung von PySticky).

![Funkenflug](screenshot.png)

## Starten

```
dhrt run funkenflug/main.dh                       # Demo-Schaltung
dhrt run funkenflug/main.dh -- schaltplan.fkn      # eine Datei öffnen
dhrt run funkenflug/main.dh -- --foto bild.png     # Selbsttest (headless, PNG)
```

GTK 3 muss vorhanden sein: unter Linux fast immer (`libgtk-3-0`), unter macOS
über Homebrew (`brew install gtk+3`), unter Windows aus MSYS2 im PATH – oder aus
einem Programm, das es mitbringt (**Inkscape**, GIMP). Die Bibliotheks-Namen in
`gtk/gtk.dh` nennen diese Fundorte durch `|` getrennt.

## Bedienung

| Werkzeug | |
|---|---|
| **Auswahl** (Taste `A`) | Bauteil oder Label anklicken, ziehen zum Verschieben; **Doppelklick**, `E` oder `Enter` bearbeitet (Bauteil: Referenz/Wert, Label: Text) |
| **Draht** (Taste `W`) | klicken setzt Eckpunkte, Doppelklick / `Enter` / rechte Maustaste schließt ab, `Esc` bricht ab. Enden fangen an Pins und Drahtenden |
| **Label** (Taste `L`) | klicken, Netznamen eingeben. Der Anker ist der elektrische Punkt; **gleichnamige Labels sind ein Netz** |
| **Bauteil** | in der Leiste aus der Liste wählen, dann auf die Fläche klicken; rechte Maustaste oder `R` dreht die Vorschau |

Weiteres: `Strg+Z` / `Strg+Y` (oder `Strg+Umschalt+Z`) bzw. die Pfeile in der
Kopfleiste machen rückgängig und wiederholen. `R` dreht die Auswahl, `Entf`
löscht sie, Mausrad zoomt um den Zeiger, mittlere Maustaste schiebt,
`+`/`-`/`0` zoomen, `Strg+S` speichert, `Strg+O` öffnet. Eine schon vergebene
Referenz lehnt der Eigenschaften-Dialog ab.

## Aufbau

```
kern/       Modell, Geometrie, Bibliothek, Netze, Datei – ohne Oberfläche,
            Namensraum "kern"
  geometrie.dh   Punkt, Lage (Transform), Raster/Snap, Konstanten
  modell.dh      Pin, Primitiv, Symbol, Bauteil, Draht, Knoten, Label, Blatt, Dokument
  bibliothek.dh  10 eingebaute Symbole (R, C, Elko, L, Diode, LED, NPN, Schalter, GND, VCC)
  netze.dh       Netz-Erkennung (eine reine Funktion) + Auto-Knoten
  datei.dh       Speichern/Laden als .fkn (JSON, Version 1)
gtk/        Deklarationen für GTK 3, GDK, Cairo, Pango, GLib (aus Glutfaden)
ansicht/    Zeichenfläche und Werkzeuge – globaler Namensraum
  leinwand.dh    Welt↔Schirm, Grid, Zeichnen von Bauteilen/Drähten/Knoten/Labels
  werkzeuge.dh   Maus, Tastatur, Werkzeug-Logik
  verlauf.dh     Rückgängig/Wiederholen
main.dh     Fenster, Kopfleiste, Werkzeugleiste, Statuszeile, Dialoge
```

Die Tests laufen ohne Mensch: `dhrt run funkenflug/pruef_kern.dh` prüft Modell,
Transform, Raster, Netze (auch T-Verbindung und Labels) und Save/Load.
`main.dh -- --foto bild.png` spielt die Werkzeuge durch – **samt der echten
Dialoge**, die ein Timer ausfüllt und bestätigt –, prüft Undo/Redo, gibt je
Prüfung `ok`/`FEHLER` aus und zeichnet das Fenster in eine PNG.

## Was gegenüber SparkNode bewusst anders ist

SparkNode hatte mehrere Konzeptfehler, die hier von Anfang an vermieden sind:

1. **Eine Rotations-/Transform-Konvention für Modell UND Zeichnen.** In
   SparkNode drehte das Modell gegen, der Maler (`QPainter`) mit dem
   Uhrzeigersinn – gedrehte Pins lagen gespiegelt zu den elektrischen. Hier geht
   **alles** durch `kern.transformPunkt` / `kern.transformRichtung` (Y nach
   unten, Drehung im Uhrzeigersinn), und der Canvas baut seine Cairo-Matrix so
   auf, dass sie dieselbe Rechnung ist. Gezeichnet == elektrisch, per
   Konstruktion.
2. **Pin-Punkt eindeutig.** `pin.pos` IST der elektrische Andockpunkt;
   `pin.laenge` ist nur der gezeichnete Stummel. Kein Nebeneinander von
   „position" und „connection point".
3. **Eine Netz-Erkennung.** `kern.netzeBerechnen` ist die einzige Quelle für
   Konnektivität (später auch ERC/Netzliste). Alle Punkte aufs Raster
   quantisiert; T-Verbindung zählt, eine reine Kreuzung ohne Knoten nicht.
4. **Label-Anker getrennt von Darstellung.** Der Anker ist der elektrische
   Punkt; Text und Unterstrich sind nur Anzeige und verschieben ihn nie.
5. **Undo über vollständige Schnappschüsse** statt einzelner Befehle. SparkNodes
   Undo-Fehler kamen von Befehlen, die nur einen Teil des Zustands kannten
   (welches Blatt, Annotation). Ein Schnappschuss kennt immer alles; Ziehen ist
   ein Schritt, ein bloßer Klick keiner.
6. **Bewusst weggelassen** (in SparkNode überproportional viele Bugquellen):
   Bus/Bus-Entry, Hierarchie-Sheets, Pin-Swap, Multi-Unit. Kommt erst, wenn
   wirklich gebraucht.

## Hinweis für Drachenhauch-Code

`/` liefert in Drachenhauch **immer** eine Kommazahl (seit 2026-09-24), auch
bei zwei ganzen Zahlen; ganzzahlig teilt `\`. `rastern()` hatte das falsch und
verschob jeden Punkt um ein halbes Raster – unbemerkt, weil `RETURN` eines
ganzzahligen Kommawerts aus einer `AS INTEGER`-Funktion still durchgeht.

## Noch offen (nächste Runden)

ERC, Netzlisten-Export, Netznamen in der Statuszeile, Drähte auswählen und
löschen, mehrere Blätter in der Oberfläche, eigene Symbole, Platinen-Layout.
