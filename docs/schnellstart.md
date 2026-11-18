# Schnellstart

In fünf Minuten vom ersten Blick zum eigenen Programm -- erst im Browser,
dann auf dem eigenen Rechner.

## 1. Im Browser ausprobieren

Die [Spielwiese](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=de)
führt Drachenhauch ohne Installation aus. Vorn in der Leiste stehen drei
Startprojekte, mit ★ markiert:

* [Mini-Spiel](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=de&beispiel=spiel)
  -- Sterne sammeln; `TEMPO` oder `STERNE` ändern.
* [Formular](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=de&beispiel=formular)
  -- Fenster mit Eingabe, Knopf und Liste; einen zweiten Knopf dazubauen.
* [Malen](https://hasoschno70.github.io/Drachenhauch/spielwiese/index.html?lang=de&beispiel=malen)
  -- mit der Maus zeichnen; `DICKE` ändern.

Eine Zahl ändern, **Ausführen** drücken -- das ist die erste eigene Änderung.

## 2. Installieren

[Drachenhauch herunterladen](https://github.com/HasoSchno70/Drachenhauch/releases/latest):
unter Windows ein Installer, unter macOS eine `.dmg`, unter Linux ein
`.tar.gz`. Danach **Drachenhauch** starten; der Startbildschirm fragt
„Wie möchtest du arbeiten?“ -- für den Anfang **Die IDE** wählen.

## 3. Das erste eigene Programm

In der IDE **Datei → Neu** (Strg+N), diesen Text hineinkopieren und **F5**
drücken:

```basic
' Sammle die Sterne! Pfeiltasten bewegen den Drachen.
CONST TEMPO = 220.0
SCREEN(480, 320, "Drachenhauch", 2)

DIM x AS FLOAT = 240.0
DIM y AS FLOAT = 160.0
DIM sx AS INTEGER : sx = RANDINT(20, 460)
DIM sy AS INTEGER : sy = RANDINT(40, 300)
DIM score AS INTEGER = 0

WHILE NOT QUITREQUESTED()
    DIM dt AS FLOAT : dt = DELTA()
    IF KEY_DOWN(KEY_LEFT) THEN x = x - TEMPO * dt
    IF KEY_DOWN(KEY_RIGHT) THEN x = x + TEMPO * dt
    IF KEY_DOWN(KEY_UP) THEN y = y - TEMPO * dt
    IF KEY_DOWN(KEY_DOWN) THEN y = y + TEMPO * dt
    ' ein Stern erwischt: Punkt, und er springt woandershin
    IF ABS(x - sx) < 20 AND ABS(y - sy) < 20 THEN
        score = score + 1
        sx = RANDINT(20, 460)
        sy = RANDINT(40, 300)
    END IF
    CLS(RGB(16, 22, 36))
    CIRCLE(sx, sy, 7, RGB(255, 220, 80))
    CIRCLE(INT(x), INT(y), 14, RGB(255, 120, 40))
    TEXT(10, 8, "Score: " + STR$(score))
    FLIP()
WEND
```

Was darin steckt:

* `SCREEN` öffnet das Fenster, `WHILE NOT QUITREQUESTED()` ist die
  Spielschleife, `FLIP` zeigt das gemalte Bild.
* `DELTA()` ist die Zeit seit dem letzten Bild -- mit ihr läuft der Drache
  auf jedem Rechner gleich schnell.
* `KEY_DOWN` fragt eine gehaltene Taste ab, `KEY_HIT` eine eben gedrückte.
* Jede Variable bekommt mit `DIM` einen Typ. Ein Tippfehler im Namen fällt
  auf, bevor das Programm startet -- die IDE unterstreicht ihn schon beim
  Schreiben.

## 4. Eine Anwendung mit Fenster

Statt eines Spiels ein Formular -- dieselbe Sprache, dazu das Modul `gui`:

```basic
IMPORT "gui"
SCREEN(420, 300, "Drachenhauch", 1)

DIM w AS GUI_WINDOW : w = GUI_WINDOW("Hello", 10, 10, 400, 280)
GUI_LABEL(w, "Name:", 20, 20)
DIM nameBox AS GUI_WIDGET : nameBox = GUI_TEXTINPUT(w, 20, 46, 240, 30, "Name")
DIM ok AS GUI_WIDGET : ok = GUI_BUTTON(w, "OK", 270, 46, 110, 30)
DIM list AS GUI_WIDGET : list = GUI_LISTBOX(w, 20, 90, 360, 160, [])

WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    ' Knopf oder Enter im Feld: Gruß in die Liste
    IF GUI_CLICKED(ok) OR GUI_ENTERED(nameBox) THEN
        GUI_LISTBOX_ADD(list, "Hello, " + GUI_GET_TEXT$(nameBox) + "!")
    END IF
    CLS(RGB(30, 34, 44))
    GUI_DRAW()
    FLIP()
WEND
```

Größere Anwendungen baut man im **Drachenhauch Studio**: Fenster
zusammenklicken, Ereignisse schreiben, F5 -- Datenbank, Tabellen und
Berichte inklusive. Es startet über denselben Startbildschirm.

## 5. Weitergeben

```bash
dhrt --export spiel.dh
```

macht aus dem Programm eine eigenständige Datei, die ohne installiertes
Drachenhauch läuft (in der IDE: **Ausführen → Als eigenständiges Programm
exportieren**, Strg+F6).

## Wie es weitergeht

* **[Einstieg](../buecher/einstieg/README.md)** -- das Buch für Anfänger,
  Kapitel für Kapitel zum eigenen Spiel.
* **[Umstieg](umstieg.md)** -- wer QBasic, VB, Blitz oder Python kennt:
  was hier anders heißt.
* **Beispiele** -- in der IDE unter **Werkzeuge → Beispiele nach Themen**,
  über 230 Programme vom Partikelsturm bis zur Rechnung.
* **[Sprachreferenz](sprache.md)** und **[Befehlsnamen](builtins-namen.md)**
  zum Nachschlagen.
