# Drachenhauch 2026.21

*Die Notizen zu dieser Fassung. Zwei Tage und sieben Pull Requests nach
2026.20: kleinere Programme zum Weitergeben, Lambdas, Meldungen auf
Englisch und ein Weg, Bibliotheken zu teilen.*

## Neu in dieser Fassung

### Pakete: `dhrt paket`

Eine Bibliothek ist ein Ordner mit `.dh`-Dateien. `dhrt paket` holt sie in
das Projekt, merkt sich die Fassung und holt auf einem anderen Rechner genau
dieselbe wieder:

```bash
dhrt paket hole github:hans/spielkiste@1.2
```

```basic
IMPORT "spielkiste/vektor.dh"
```

* **Kein eigener Server:** ein Paket kommt von GitHub
  (`github:nutzer/repo@stand`), von einer https-Adresse (ZIP oder einzelne
  `.dh`) oder aus einem Ordner. Der Stand ist Pflicht -- "der neueste" wäre
  morgen ein anderer.
* **Pakete gehören zum Projekt:** sie liegen in `pakete/<name>/` neben der
  `paket.json`. `paket.lock.json` hält je Paket eine SHA-256-Prüfsumme über
  den Inhalt fest; kommt beim nächsten Holen etwas anderes an, ist das ein
  Fehler, und das alte Paket bleibt stehen.
* **`IMPORT` sucht in `pakete/`**, von der importierenden Datei aus nach
  oben. Braucht ein Paket selbst ein Paket, findet es zuerst sein eigenes.
* Beim Holen wird nichts ausgeführt, und kein ZIP-Eintrag kommt aus seinem
  Ordner heraus. `hole`, `entferne`, `liste`, `neu`; mit `-C <ordner>` in
  einem anderen Ordner.
* **In der IDE:** Datei → Pakete des Projekts (Strg+Alt+P) holt, erneuert
  und entfernt Pakete; danach ist ein `IMPORT` in ein eben geholtes Paket
  sofort kein Fehler mehr. Umbenennen und Ersetzen im ganzen Projekt, die
  Projektsuche und "Projekt prüfen" lassen `pakete/` aus -- dort liegt
  fremder Code, den das nächste Holen überschreibt.

Mehr in [Pakete](pakete.md).

### Lambdas

```basic
DIM zahlen AS ARRAY OF INTEGER
zahlen = [1, 2, 3, 4]
DIM doppelt AS ARRAY OF INTEGER
doppelt = ARRAY_MAP(zahlen, FUNCTION(x) x * 2)          ' 2, 4, 6, 8
DIM gerade AS ARRAY OF INTEGER
gerade = ARRAY_FILTER(zahlen, FUNCTION(x) x MOD 2 = 0)  ' 2, 4
```

* `FUNCTION(x) ausdruck` und `SUB() anweisung`, beide in einer Zeile, in
  der Schreibweise von VB.NET -- etwa als Rückruf:
  `GUI_ON_CLICK(knopf, SUB() zaehler += 1)`. Lokale Variablen werden beim Anlegen
  kopiert (ein Lambda kann länger leben als sein Aufruf), globale sieht es
  lebendig.
* Dazu `ARRAY_MAP`, `ARRAY_FILTER`, `ARRAY_REDUCE` und `ARRAY_FIND`, und ein
  Wert lässt sich direkt aufrufen: `a[i]()`, `f(1)(2)`.

### Meldungen auf Englisch

Mit `DHRT_LANG=en` kommen die Meldungen von dhrt englisch: auf der Konsole,
bei `--check`, im Sprachserver und Debugger und in `CODE_CHECK$`. Die IDE
setzt es selbst, wenn ihre Oberfläche englisch ist. Drei Runden decken die
Sprache, die Laufzeit, die Befehle und Module sowie gui, Grafik und Ton ab.
Was ein Programm mit `CATCH` sieht, bleibt deutsch, damit ein Vergleich nicht
an einer Umgebungsvariable hängt; deutsch bleiben auch die Ausgaben der
Werkzeuge auf der Kommandozeile (`dhrt test`, `dhrt paket`, der Export).

### Kleinere Programme zum Weitergeben

`dhrt --export --schlank` nimmt statt der vollen Laufzeit (rund 34 MB) die
kleinste, die alle Befehle des Programms hat: **`dhrt-konsole`** (Daten,
Netz, Mail; 16,5 MB) oder **`dhrt-spiel`** (Grafik, Ton, Dialoge, Physik;
21,6 MB). Gefragt werden die Laufzeiten selbst, keine Liste, die beim
nächsten neuen Befehl still falsch wäre -- und gezählt wird auch ein
Befehl in einem Unterprogramm, das nie läuft. Der Maschinencode ist in allen
dreien.

### Tray-Symbol und Mitteilungen

`TRAY_SHOW`, `TRAY_MENU`, `TRAY_CLICKED`, `TRAY_MENU_CLICKED$` und
`NOTIFY` für Programme, die im Hintergrund weiterlaufen; dazu
`WINDOW_HIDE`/`WINDOW_SHOW`. Das Symbol gibt es bisher nur unter Windows,
Mitteilungen auf allen drei Systemen.

## Behoben

* In der englischen Fassung einer Meldung fiel ein Beispiel mit
  geschweiften Klammern still weg (`f"{value:.2f}"` wurde zu `f""`) --
  Platzhalter sind jetzt nur `{}` und `{Zahl}`.
* Die Meldung von `GUI_SCALE` trug 25 Leerzeichen mitten im Satz.

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 2086 auf **2102** Einträge, es
bleiben 48 Module und **221** Beispiele. **4768 Fälle in 367
Prüfsammlungen** (vorher 4710 in 361) und **493 Rust-Testfunktionen**
(vorher 484).

* Die Übersetzung sitzt an der Ausgabe (`meldung.rs`, Katalog
  `daten/meldungen.en.txt` mit über 1400 Vorlagen); `dhrt pruef meldungen`
  meldet jede Vorlage, deren deutscher Text nicht mehr im Quelltext steht.
* Der Installer wächst von 43 auf **52 MB**: zum ersten Mal liegen die
  beiden kleinen Laufzeiten für den schlanken Export bei.
* Jede neue Funktion ist gegengeprüft: gezielt verfälschte Fassungen der
  Laufzeit bzw. der IDE fallen je in ihrem Prüffall.

## Was offen bleibt

* Das Handbuch (`docs/`) gibt es nur auf Deutsch.
* Ein Tray-Symbol unter macOS und Linux.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt; der Installer ist nicht signiert.
