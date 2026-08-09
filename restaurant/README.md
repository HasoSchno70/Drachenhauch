# Restaurant — vom Wagen zum eigenen Laden

Ein Aufbau- und Hektik-Spiel im Stil von *Rising Chef*: tagsüber kochen und
servieren, abends abrechnen und investieren.

Dieser Stand ist bewusst **eine senkrechte Scheibe** — ein Gericht, eine
Schlange, ein Tag. Das beantwortet die einzige Frage, die am Anfang zählt:
*Macht diese eine Minute Spaß?* Erst danach lohnt sich der Ausbau nach oben.

## Starten

```bash
dhrun.py restaurant\restaurant.dh
```

## Steuerung

| Taste | Wirkung |
|---|---|
| `1` / `2` | Kochstelle anheizen (die zweite gibt es erst nach dem Kauf) |
| `LEER` | Fertiges Gericht an den vordersten Gast servieren |
| `K` | Am Feierabend: zweite Kochstelle kaufen |
| `ESC` | Beenden |

Ein verkohltes Gericht räumst du mit derselben Zifferntaste aus.

## Was das Spiel schon kann

* **Gäste** kommen, stellen sich an, haben Namen und Geduld. Läuft die Geduld
  ab, gehen sie — und der Ruf sinkt.
* **Kochen mit Zeitfenster:** zu früh servieren geht nicht, zu spät verkohlt.
  Der Balken zeigt erst den Fortschritt und danach die Restzeit bis zum
  Anbrennen.
* **Trinkgeld** hängt daran, wie lange jemand warten musste. Das ist die
  Spannung zwischen schnell und sorgfältig.
* **Ruf** steuert den Andrang: guter Ruf = mehr Gäste = mehr Hektik. Er steigt
  langsam und fällt spürbar.
* **Feierabend** mit Abrechnung, **Speicherstand** und dem ersten Ausbau
  (zweite Kochstelle = zwei Gerichte parallel).
* **Alle Klänge sind gerechnet**, nicht geladen (`AUDIO_SFX`) — das Spiel
  braucht keine einzige Datei und läuft damit auch im Browser.

## Wie es weitergehen soll

Jede Stufe bringt eine neue **Mechanik**, nicht nur ein größeres Bild:

| Stufe | Neu ist |
|---|---|
| Wagen *(hier)* | ein Gericht, eine Kochstelle |
| Kiosk | zweite Station → Parallelität |
| Imbiss | Sitzplätze: Gäste warten, statt sofort zu gehen |
| Restaurant | Personal: du kochst nicht mehr, du verteilst |
| Filiale | läuft ohne dich weiter |

Weitere Ideen, die noch nicht drin sind: Stammgäste mit Vorlieben (wer sich
Frau Bergers „ohne Zwiebeln" merkt, bekommt doppeltes Trinkgeld), eine
Vorbereitungs-Phase vor dem Öffnen, der Kritiker als Boss-Gast, und der alte
Wagen als Nebeneinnahme auf Volksfesten.

## Zahlen zum Drehen

Alles Wichtige steht als `CONST` ganz oben in `restaurant.dh`:
Tagesdauer, Kochzeit, Brennzeit, Geduld, Preis, Ausbaupreis. Nach einer
Balance-Änderung lohnt der Selbsttest — eine Kopie, die sich selbst spielt,
zeigt in einem Lauf, ob ein Tag noch zu schaffen ist (bei perfektem Spiel:
19 Gäste, 0 verloren, 342 EUR, Ruf 76).
