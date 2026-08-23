# Modul `ini`

Einstellungsdateien im INI-Format — das Format, das ein Mensch mit dem Editor
anfassen soll.

```basic
IMPORT "ini"
```

## Warum nicht JSON

JSON und CSV gab es schon. Für eine Datei, die jemand von Hand bearbeitet, ist
beides unhandlich: JSON verzeiht kein Komma zu viel, CSV hat keine benannten
Felder. INI ist seit Jahrzehnten genau dafür da — und ein ESP32-Bastler hat es
ohnehin schon auf der Platine.

## Eine INI-Datei ist eine MAP

Es gibt **keinen eigenen Handle-Typ**. Eine INI-Datei ist hier eine
`MAP OF STRING` mit Punkt-Schlüsseln:

```text
[fenster]                    "fenster.breite" -> "1280"
breite=1280        wird zu   "fenster.titel"  -> "Mein Spiel"
titel=Mein Spiel             "ton.laut"       -> "0.8"

[ton]
laut=0.8
```

Damit braucht das Modul vier Befehle statt zwei Dutzend Getter und Setter —
`MAPGETOR`, `MAPPUT`, `MAPKEYS`, `MAPHAS` und `VAL` können das alles schon,
und wer die Sprache kennt, kennt damit auch dieses Modul.

| Funktion | Zweck |
|---|---|
| `INI_PARSE(text$)` → MAP OF STRING | aus einer Zeichenkette |
| `INI_LOAD(pfad$[, kodierung$])` → MAP OF STRING | aus einer Datei |
| `INI_TEXT$(m)` → STRING | zurück in INI-Text |
| `INI_SAVE(pfad$, m[, kodierung$])` | in eine Datei |

## Beispiel

```basic
IMPORT "ini"

DIM cfg AS MAP OF STRING
DIM breite AS INTEGER
DIM titel AS STRING

IF FILEEXISTS("einstellungen.ini") THEN
    cfg = INI_LOAD("einstellungen.ini")
ELSE
    cfg = INI_PARSE("")
END IF

' MAPGETOR liefert die Vorgabe, wenn der Schlüssel fehlt -- der Normalfall
' bei einer Einstellungsdatei, und deshalb kein Fehler.
breite = VAL(MAPGETOR(cfg, "fenster.breite", "1280"))
titel  = MAPGETOR(cfg, "fenster.titel", "Mein Spiel")

' ... und beim Beenden zurückschreiben
MAPPUT(cfg, "fenster.breite", STR$(breite))
INI_SAVE("einstellungen.ini", cfg)
```

## Was gelesen wird

* `[abschnitt]` — alles danach bekommt `abschnitt.` als Vorsatz
* `name = wert` — Leerraum außen fällt weg, im Wert bleibt er
* `;` und `#` **am Zeilenanfang** sind Kommentare. Mittendrin nicht — sonst
  ließe sich ein Pfad wie `C:/a;C:/b` nicht speichern
* ein `=` im Wert bleibt (`formel=a=b+c` ergibt `a=b+c`)
* Schlüssel **vor** dem ersten Abschnitt behalten ihren nackten Namen
* `"wert"` in Anführungszeichen — sie fallen weg, der Leerraum darin bleibt.
  Das ist der einzige Weg, führende Leerzeichen zu behalten

**Kaputte Zeilen halten nicht auf.** Eine Zeile ohne `=` wird übersprungen,
statt den Start abzubrechen. Das ist Absicht: eine Einstellungsdatei
bearbeitet ein Mensch, oft kein Programmierer — anders als bei JSON, wo eine
kaputte Datei fast immer ein Fehler des *Programms* ist.

## Was beim Schreiben passiert

Abschnitte kommen in der Reihenfolge ihres ersten Auftretens, Schlüssel in
ihrer eigenen — eine gelesene und wieder geschriebene Datei sieht aus wie
vorher, statt bei jedem Durchlauf neu gemischt zu werden. Werte, die sonst
anders zurückkämen (Leerraum am Rand, ein führendes `;` oder `#`), bekommen
Anführungszeichen.

**Zwei Grenzen:**

* **Kommentare gehen beim Zurückschreiben verloren.** Sie zu erhalten hieße,
  die ursprüngliche Datei mitzuführen — und dann wäre es keine `MAP` mehr,
  sondern doch wieder ein Handle.
* **Ein Punkt im Schlüsselnamen ist nicht adressierbar** — er trennt Abschnitt
  und Name. Dieselbe Grenze wie bei den JSON-Pfaden.

Die **Kodierung** aus [Textkodierung](builtins-core.md#textkodierung) gilt
auch hier: eine alte Einstellungsdatei ist oft `cp1252`.

Beispiel: [examples/174_einstellungen.dh](../examples/174_einstellungen.dh).
