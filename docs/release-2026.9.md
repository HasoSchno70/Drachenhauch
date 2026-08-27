# Drachenhauch 2026.9

*Die Notizen zu dieser Fassung. Was dahintersteckt und warum es so und nicht
anders gebaut wurde, steht im [zweiten Allzweck-Audit](allzweck-audit-2.md)
und den beiden Entwürfen daneben.*

## Neu in dieser Fassung

2026.8 machte aus Drachenhauch eine Sprache, mit der man alles schreiben
**kann**. Diese Fassung stellt die nächste Frage: **würde jemand sie wählen, um
damit Software zu bauen?** Die Antwort saß an drei Stellen — der Übersetzer
prüfte weniger, als die Sprache verspricht; Daten kamen schwerer rein und raus,
als sie sollten; und um die Sprache herum fehlte die Werkzeugkette, an der man
abliest, dass sie zum Arbeiten gedacht ist.

Dazu ein zweites Buch — für Menschen, die noch nie programmiert haben.

**Der Übersetzer prüft jetzt, was die Sprache verspricht.** Drachenhauch war
zur *Laufzeit* Pascal-streng und beim *Übersetzen* fast typenblind. Damit hatte
man die Kosten der Strenge ohne ihren Nutzen: bei einem Spiel fällt so ein
Fehler in Minuten auf, bei Software läuft der betroffene Zweig womöglich erst
beim Kunden. `dhrt --check` meldet jetzt fünf Sorten mehr:

```basic
s = 5                ' Typ, den das Ziel nie annimmt (s ist STRING)
n = 7 / 2            ' Kommazahl an eine ganzzahlige Variable
zeichne(s)           ' falscher Argumenttyp — zeichne erwartet INTEGER
PRINT p.anzhal       ' Mitglied, das die Klasse nicht hat ("Meintest du 'anzahl'?")
```

Dazu der fünfte Fall, der kein Hinweis ist, sondern ein Loch im Typsystem: eine
`FUNCTION f() AS INTEGER` **ohne jedes `RETURN`** lieferte still `NIL` — und
`NIL` ist kein INTEGER. Der Aufrufer bekam einen Wert, den seine eigene Ansage
ausschließt, und merkte es erst weiter unten oder gar nicht.

Die Arbeit steckte nicht im Finden, sondern im **Schweigen**: eine als
Basisklasse angesagte Variable darf zur Laufzeit eine abgeleitete halten,
INTEGER passt in FLOAT, ein Sammel-Parameter nimmt alles. Vor jedem Schritt
wurde der Bestand gemessen — alle 365 `.dh`-Dateien im Repo durch `--check`,
**keine einzige neue Meldung** außer drei echten Blindgängern in einem
Beispiel. Es sind Warnungen, keine Fehler: die Herleitung des Ausdruckstyps ist
neu, und ein Irrtum darin soll kein laufendes Programm unübersetzbar machen.

**Sieben neue Module.** Damit stehen 46 statt 39 zur Auswahl, und die
Befehlszahl wächst von 1401 auf 1527.

| Modul | Wofür |
|---|---|
| `pdf` | druckfertige Seiten — Rechnung, Lieferschein, Bericht. Für kaufmännische Software ist ein PDF fast immer die erste Forderung nach „speichern" |
| `xlsx` | der Schritt hinter CSV: mehrere Blätter, fette Kopfzeile, Zahlen- und Datumsformate — und **Text von Zahl unterschieden**, damit die Postleitzahl 01067 beim Öffnen nicht zu 1067 wird |
| `smtp` | die andere Hälfte der Kette: `xlsx`/`pdf` bauen den Bericht, `smtp` schickt ihn los — mit Anhang und STARTTLS |
| `xml` | Rechnungen, Ausfuhrlisten, GPX-Spuren, SVG, ältere Web-Schnittstellen: alles XML, und ohne Leser war jede dieser Quellen außer Reichweite |
| `ini` | Einstellungsdateien, die ein Mensch mit dem Editor anfassen soll. Ohne eigenen Handle-Typ — eine INI-Datei **ist** eine `MAP OF STRING` |
| `httpd` | ein kleiner Webserver im Takt der Hauptschleife. Die erste Roadmap hatte ihn gestrichen; vor dem Bastler-Leitbild zu Unrecht, denn mit `mqtt`, `firmata`, `serial` und `net` an Bord fehlte für „meine Heizungssteuerung hat eine kleine Weboberfläche" genau dieser eine Baustein |
| `geld` | ein Betrag als **eigener Wert**, der mitdenkt: `preis * 3` geht, `preis + 1.0` ist ein Fehler |

Zu `geld` gehört ein Befund, der die Untersuchung wert war: der übliche
Ratschlag gegen Fließkomma-Geld lautet „rechne in ganzen Cent" — und der
Schritt dorthin ist selbst eine Falle. `INT(19.99 * 100)` ergibt **1998**.
Darum gibt es beides: `CENT`, `EURO$` und `ROUND_HALF_UP` als Rechenweise für
alle, die weiter mit Zahlen rechnen wollen, und den Typ `GELD` für die, die es
der Sprache überlassen.

**Daten kommen jetzt rein und raus.** JSON ließ sich lesen, aber nicht
schreiben — wer eins bauen wollte, klebte Zeichenketten zusammen und brach am
ersten Anführungszeichen in einem Namen. Ausgerechnet die Fähigkeit, die 2026.8
gerade gebaut hatte (jede angemeldete REST-Schnittstelle anrufen), scheiterte
damit an ihrem eigenen Rumpf. Dazu: eine Textdatei, die nicht UTF-8 war, ließ
sich **gar nicht** lesen — und genau die schreibt Excel auf einem deutschen
Windows beim CSV-Export. Jetzt gehen cp1252 und latin1. Und `STDIN()` liefert
die Standardeingabe als `FILE`-Handle, womit `dir | meinwerkzeug | sort`
schreibbar wird.

**Die Werkzeugkette.** `dhrt --version` antwortete bisher mit *„Kann
'--version' nicht lesen"* — es hielt den Namen für eine Datei. Jetzt gibt es
`--version` (samt der eingebauten Bestandteile, denn ein Bau ohne `--hardware`
lässt Module weg, ohne dass man es dem Binary ansieht), `--help`, `dhrt test`
und `dhrt fmt`. `dhrun.py --doku datei.dh` erzeugt die Referenz **aus dem
Quelltext**, statt eine zweite von Hand gepflegte zu verlangen, die ab dem
ersten Tag abdriftet. Und `IMPORT` sucht nicht mehr nur neben der eigenen
Datei: über `DH_PATH` und einen Ordner im Benutzerprofil lässt sich eine
Bibliothek endlich **teilen**, statt sie in jedes Projekt zu kopieren.

**Dateien.** Es gab `MKDIR`, aber kein Gegenstück zum Löschen; keinen
Zeitstempel — die Grundlage jeder Sicherung und jedes „was ist neu"; kein
rekursives Auflisten; kein Namensmuster; keinen Temp-Ordner. Alle fünf sind da.

**Objekte können sich selbst als Rückruf eintragen.** `obj.methode` ohne
Klammern ist jetzt eine `FUNCREF`, die ihre Instanz mitträgt:

```basic
GUI_ON_CLICK(knopf, spieler.klick)
TIMER_EVERY(500, gegner.zucken)
SORT(zahlen, regel.cmp)
```

Neun Stellen der Laufzeit verlangen eine `FUNCREF` — sechs `GUI_ON_*`, beide
`TIMER_*` und der Vergleicher von `SORT` —, und die konnte bisher
nur eine benannte Funktion auf oberster Ebene sein — objektorientierter Code
kam an seinen eigenen Zustand nicht heran und brauchte für jeden Knopf eine
globale Variable daneben.

**Und man kann fragen, was man in der Hand hält.** Polymorphie funktionierte
längst; ansehen konnte man einer Referenz aber nichts. `TYPEOF` sagte pauschal
`"OBJECT"`, und einen Typtest gab es gar nicht:

```basic
PRINT TYPEOF(t)      ' "HUND" statt "OBJECT"
PRINT t IS Hund      ' TRUE
PRINT t IS Tier      ' TRUE  -- jede Elternklasse trifft
PRINT t IS NOT NIL   ' TRUE
```

Rechts von `IS` steht ein Typname, kein Ausdruck. Ein unbekannter Name ist ein
Übersetzungsfehler — ein Tippfehler wäre sonst still für immer `FALSE`, und ein
Test, der nie zuschlägt, fällt niemandem auf. Nebenbei erledigt: `IS NIL` und
`IS NOT NIL` standen in der Dokumentation ausdrücklich als *nicht vorhanden*.

**`RGB` und `RGBA` nehmen Kommazahlen an** und runden sie. Sie waren die
einzigen Ausreißer unter den Zeichen-Befehlen — ausgerechnet die Farbe war
streng, dabei wird sie am häufigsten ausgerechnet, und `x * 255 / 640` für
einen Verlauf liefert immer FLOAT.

## Der Einstieg — ein zweites Buch

Im Regal stand [Das Lehrbuch](../buch-referenz/README.md): vollständig,
systematisch, jeder Befehl mit Beispiel, Konsole zuerst und Grafik ab Teil IV.
Es setzt voraus, dass jemand schon weiß, warum er das lernen will.

**[Der Einstieg](../buch-einstieg/README.md) geht den umgekehrten Weg.** Das
allererste Programm ist fünf Zeilen lang und öffnet ein Fenster mit einer
leuchtenden Sonne; verstanden wird hinterher. Nach jedem Kapitel steht etwas
auf dem Bildschirm, das man jemandem zeigen möchte — ein Sternenhimmel, ein
hüpfender Ball, Pong, Snake, ein Instrument zum Draufspielen, ein Feuerwerk aus
fünfhundert Funken.

231 Seiten, 33 Kapitel in sechs Teilen, drei Anhänge, 104 lauffähige Programme.
Am Ende steht ein **Vokabeltrainer** mit Fenster, Knöpfen und Eingabefeldern:
er holt seine Listen aus dem Internet, kann mehrere Sprachen, nimmt eigene
Listen entgegen und fragt nicht stumpf zufällig ab — ein Leitner-Karteikasten
entscheidet, was drankommt, und das Fach entscheidet, *wie* gefragt wird.
Getippte Antworten dürfen einen Tippfehler haben (gemessen mit dem
Levenshtein-Abstand). `dhrt --export` macht daraus eine einzelne `.exe`, die
man weitergeben kann.

Beide Bände beschreiben dieselbe Sprache und widersprechen sich nicht. Wer hier
durch ist, schlägt dort nach.

## Unter der Haube

**Zitierte Fehlermeldungen werden jetzt geprüft.** Anlass war die
RGB-Änderung: seit `RGB` Kommazahlen annimmt, stand *„RGB erwartet INTEGER,
erhalten FLOAT"* an fünf Stellen im Buch — aufgefallen ist das nur, weil jemand
zufällig hinsah. `pruef_meldungen.js` löst zu jeder zitierten Meldungsfamilie
ein winziges Programm bei der echten Laufzeit aus und vergleicht, was
herauskommt. Eine von Hand gepflegte Erwartungsliste gibt es bewusst **nicht**
— sie wäre die zweite Stelle, die abdriften kann. Das Werkzeug deckt inzwischen
alle vier Bücher und `docs/` ab.

**Die Qt-Editortests laufen je Datei in einem eigenen Prozess.** Vorher
sammelten sich in einem gemeinsamen Lauf tausende QObjects mit hunderten
scharfen Zeitgebern; ein einziges `processEvents()` kam dann nie zurück.

Die Testsuite ist von rund 3350 auf **3792** Prüfungen gewachsen. Die
Befehlsreferenz zählt 1527 Einträge — 126 mehr als in 2026.8.
