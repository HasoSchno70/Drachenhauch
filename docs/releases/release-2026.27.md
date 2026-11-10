# Drachenhauch 2026.27

*Die Notizen zu dieser Fassung. Zwei Tage nach 2026.26, fast alles aus
zwei Dogfooding-Runden: ein Notizzettel und ein kleines Spiel, mit echten
Tasten und Klicks bedient. Was dabei hakte, ist jetzt leichter -- die
meisten Änderungen merkt man, ohne etwas Neues zu lernen.*

## Neu in dieser Fassung

### `PRINT` erscheint, wenn es geschrieben wird

Bisher sammelte dhrt die Ausgabe von `PRINT` bis zum Programmende -- auch im
Terminal. Ein „Bitte warten …“ vor einer langen Rechnung oder einem `SLEEP`
erschien erst danach. Jetzt geht im Terminal jede Zeile sofort hinaus. In
eine Datei oder an ein anderes Programm wird gebündelt (dort zählt der
Durchsatz), aber spätestens nach 50 ms, vor `SLEEP` und bei `FLIP`.

```basic
PRINT "Bitte warten ..."
SLEEP(2000)
PRINT "fertig"
```

### Index und Zuweisung auf dem Ergebnis eines Aufrufs

```basic
CLASS Held
    DIM hp AS INTEGER
END CLASS
DIM h AS Held
h = NEW Held()
FUNCTION held() AS Held
    RETURN h
END FUNCTION
FUNCTION teile(s AS STRING) AS ARRAY OF STRING
    RETURN SPLIT$(s, ",")
END FUNCTION
PRINT teile("a,b,c")[1]
held().hp = 5
PRINT h.hp
```

Ein Feld, ein Objekt und ein Struct kommen als Verweis zurück -- lesen und
schreiben geht darum direkt auf dem Ergebnis: `reihe(5)[0].x`, `ort(7).y`,
`held().hp = 0`. Der Aufruf läuft dabei genau einmal. Nur `+=` und `++`
darauf sind eine Meldung, weil der Aufruf zweimal liefe.

### Was Anwendungen leichter macht

* **`INSTRREV(s, teil [, start])`** -- die letzte Fundstelle (ab 0, -1 wenn
  nicht da), etwa um einen Text an der letzten Wortgrenze zu teilen.
* **`GUI_WINDOW_CONTENT_W/H(win)`** -- Breite und Höhe des Inhaltsbereichs
  unter Titel-, Menü- und Reiterleiste. Eine Statusleiste sitzt so genau
  unten, ohne die Höhe der Menüleiste zu raten.
* **`GUI_CLICKED` auf einer Liste** meldet jetzt jeden Klick auf einen
  Eintrag, auch auf den schon gewählten -- vorher war es dort immer FALSE.
* **Der Platzhalter** eines Textfelds und Textbereichs bleibt sichtbar, bis
  man tippt, auch wenn das Feld den Fokus hat.
* **Markieren nur mit Absicht:** bekam ein Textfeld den Fokus, während die
  Maustaste noch unten war (ein Klick in eine Liste öffnet eine Notiz und
  setzt den Fokus in den Text), markierte es bis zur Maus -- das erste
  Tippen hätte den Text ersetzt.
* Der Hinweis einer leeren Liste, eines Baums oder einer Tabelle bricht um,
  statt aus dem Feld zu laufen; ein abgeschnittener Text in der Statusleiste
  endet mit `...`.
* `PDF_PAGE` direkt nach `PDF_NEW` legt keine leere erste Seite mehr an.

### Was Spiele schneller und sicherer macht

* **`^` im Maschinencode:** jede Schleife mit der üblichen Abstandsrechnung
  `dx ^ 2 + dy ^ 2` lief bisher in der VM. Jetzt rechnet der Maschinencode
  sie selbst, mit denselben Ergebnissen -- bis zu 35-mal schneller.
* **`IF obj.feld AND x > 0`** in einer Schleife über Objekte bleibt jetzt im
  Maschinencode, ebenso ein Objektfeld als Argument einer übersetzten
  Funktion.
* **`PARTICLE_UPDATE(p, DELTA())`** ließ die Partikel nie altern: `DELTA()`
  liefert Sekunden, der Befehl will Millisekunden, und 0,016 wurde zu 0
  gerundet -- ohne Meldung. Eine Zahl zwischen 0 und 1 ist jetzt ein Fehler,
  der den richtigen Aufruf nennt (`INT(DELTA() * 1000)`); dasselbe gilt für
  `SPRITE_UPDATE` und `ANIM_FSM_UPDATE`.

### Zwei neue Beispiele

* **`examples/notizen`** -- ein Notizzettel: SQLite, Liste mit Suche,
  formatierter Text als Markdown, PDF, Tray-Symbol, Rückfrage bei
  Ungesichertem.
* **`examples/kometen`** -- ein kleines Arcade-Spiel: Kometen, die
  zerbrechen, Funken, Klänge, Szenen, Bestenliste, Pause bei Fokusverlust.

## Unter der Haube

**Zahlen.** **2112** Befehle in 48 Modulen (vorher 2109), 225 Beispiele auf
der obersten Ebene von `examples/` (die zwei neuen liegen in eigenen
Ordnern). **4939 Fälle in 380 Prüfsammlungen** (vorher 4899 in 373) und
**533 Rust-Testfunktionen**.

* Beide Runden sind mit echten Fensternachrichten gefahren (Tippen,
  Strg-Kürzel, Klicks, Fokusverlust) und am Bild angesehen. Fast jeder
  Stolperstein war zuerst ein Fehler im eigenen Programm -- und führte doch
  an eine Stelle, an der die Laufzeit es unnötig schwer machte.
* Beim Spiel war die Bilanz des Maschinencodes (`DHRT_JIT_BILANZ=1`) das
  wichtigste Werkzeug: sie nennt je Schleife, warum sie in der VM bleibt.

## Was offen bleibt

* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
  `VALIST` und die Bitfelder auf Linux-ARM sind nach den Regeln gebaut, aber
  nicht ausprobiert.
* Ob ein echter Mac und eine echte Linux-Leiste das Tray-Symbol auch zeigen,
  ist nicht ausprobiert.
* Die Formular-Fälle gegen die Datenbank-Server brauchen ein Fenster und
  sind nur lokal belegt.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt; der Installer ist nicht signiert.
