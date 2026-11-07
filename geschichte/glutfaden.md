# Glutfaden (PySticky-Port)

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Alle Stufen 1 bis 26.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Glutfaden: PySticky in Drachenhauch + GTK 3 (Stufe 1, 2026-10-14)

Wunsch des Nutzers: PySticky (Kreuzstich-Editor in Python + PySide6, eigenes
Repo `C:/Programmieren/Python/PySticky`, ~62 000 Zeilen, MIT) nach
Drachenhauch mit GTK portieren -- als Programm aus vielen Dateien.
Name **Glutfaden** (Wunsch des Nutzers 2026-10-15; bis dahin "Sticky"), das
Dateiformat bleibt `.pxs` mit der Kennung `pysticky`, damit PySticky die Dateien
weiter liest.
`examples/glutfaden/`: `kern/` (Muster, Ebenenstapel, Garne, Paletten,
Stichtypen, `.pxs` lesen und schreiben; fuenf Dateien, ueber die Sammeldatei
`kern/kern.dh` alle mit `AS kern` -- EIN Namensraum), `gtk/gtk.dh`
(Deklarationen GTK/GDK/Cairo/Pango/GLib), `ansicht/` (Zeichenflaeche: ein
Punkt je Zelle in einem ARGB-BUFFER, skaliert mit Filter "nearest", darueber
Symbole mit Pango, Teilstiche, Zehnerlinien, Rueckstiche; Seitenleiste mit
Farbfeldern und Ebenen), `daten/` (19 Paletten, Symbole, Demo-Muster,
`LIZENZ-PySticky.txt`). Stufe 1 = oeffnen und ansehen; `--probe` fuehrt vor,
`--foto ziel` sichert Bilder (`gtk_widget_draw` in eine Cairo-Flaeche --
`gdk_pixbuf_get_from_window` ist unter Windows weiss). **Gegengeprueft mit dem
fremden Leser:** PyStickys `load_pattern` liest eine hier geschriebene Datei
(200 x 200, 40 544 Stiche) ohne abweichende Zelle; die Paletten- und
Stichtabellen sind mit core/stitch.py von PySticky verglichen.

**Funde in der Laufzeit** (alle mit Faellen, alle fallen gegen 2026.31):
(1) **BYREF bei Methoden lief still als BYVAL** -- `ebenen.zusammensetzen(farbe,
typ, fertig)` liess die Felder leer. Neu `CALL_METHOD_BYREF` (124): kennt der
Compiler die Klasse (`statischer_typ` + `find_method_sig`, auch der Aufruf auf
Self), erfasst er die Argumente wie bei CALL_USER (`methode_rufen`); die VM
prueft, dass die aufgeloeste Methode dieselben BYREF-Stellen hat (eine
Ueberschreibung mit anderen ist ein Fehler). Ohne bekannte Klasse meldet
CALL_METHOD einen Fehler (`byref_ohne_typ`) statt still zu verlieren. Bereiche
mit dem Befehl bleiben in der VM. (2) **CASE-Konstanten im Namensraum** liefen
am Umbenennen vorbei. (3) **Ein Namensraum ohne eigene Namen** (Sammeldatei)
galt nicht als Namensraum -- `kern.X` hiess "Variable 'kern' nicht
deklariert". (4) Was eine Datei des Namensraums OHNE `AS` einbindet, bleibt
draussen (Absicht) -- die Meldung sagt jetzt, in welcher Datei der Name steht
und dass dort `IMPORT ... AS kern` fehlt (`sammeln` liefert `draussen`).
(5) **TYPEOF** zeigte den internen Namen `KERN@KNOTEN`. (6) **Laufzeitfehler
nannten die Zeile der zusammengefuegten Quelle** mit dem Namen der Hauptdatei,
ein Fehler im Rueckruf "(Zeile 1804)" -- jetzt `datei:zeile` ueber
`preprocess::lauf_merken`/`lauf_stelle` (Thread-Speicher, beim Uebersetzen
gesetzt). Tests `byref.dhtest` (+4), `namensraeume.dhtest` (+3),
`import_meldungen.dhtest` (+2), `glutfaden_kern.dhtest` (5), `glutfaden_ansicht.dhtest`
(Bildproben: Herz-Rot im nahen Bild, keins nach dem Ausblenden der Ebene).
**Stolpersteine beim Schreiben:** `BAND` bindet schwaecher als `-`
(`(f SHR 16) BAND 255 - &HA9` rechnet `BAND 86`), und `UBOUND` gibt es nicht.

**Stufe 2: bearbeiten** (2026-10-15). `kern/formen.dh` (Linie, Rechteck,
Ellipse, Fuellen, Umschalt-Beschraenkungen -- Punkt fuer Punkt gegen die
Ausgabe der PySticky-Algorithmen, `glutfaden_formen.dhtest`; die Ellipse
merkt sich gefuellt je Zeilenabstand die halbe Breite statt einer Menge),
`kern/verlauf.dh` (Rueckgaengig mit Aenderungen statt Kopien, ein Zug = ein
Schritt ueber `beginnen`/`beenden`, Grenze 100, `geaendert()` ueber die
Nummer des obersten Schritts; ein Stiche-Teil haelt seine Ebene als OBJEKT,
nicht als Nummer -- sonst traefe ein Rueckgaengig nach dem Verschieben die
falsche), `kern/bearbeiten.dh` (alles, was das Muster aendert). **Anders als
PySticky** sind Ebenen-Umbauten und "unbenutzte Farben entfernen" Schritte
(dort leeren sie den Verlauf); Sichtbarkeit, Sperre und Deckkraft sind wie
dort keine, zaehlen aber als ungesichert (`extraGeaendert`). Ein voller
Stich in einer Perlen-/Diamantfarbe wird Perle bzw. Drill, ein gewaehlter
Teilstich bleibt (wie `PlaceStitchCommand`). Oberflaeche: `ansicht/
werkzeuge.dh` (Leiste mit selbst gezeichneten Symbolen, Kuerzel P E F I L R
O M, Alt+1..0 Stichart, rechte Taste radiert in jedem Werkzeug, mittlere
verschiebt), Zeichenflaeche schreibt nach einem Strich nur die geaenderten
Punkte (`leinwandZellen`) und mischt Ebenen mit Deckkraft ueber den Stoff
wie das Original, Seitenleiste mit Farbliste, Katalog der 19 Hersteller
(Suche, hoechstens 400 Zeilen, Haken an benutzten), Ebenenliste mit Auge,
Schloss, Doppelklick = umbenennen, Knoepfen und Deckkraft-Regler;
Sichern/Sichern unter, Punkt im Titel, Rueckfrage bei Kreuz und Oeffnen.
**Logo** vom Nutzer (ein JPEG mit gemaltem Schachbrett): ausgeschnitten,
Ecken zeilenweise aus dem Bild gemessen (Squircle, kein Kreisbogen),
Randpunkt mit Teil-Deckkraft in der Farbe von innen, `daten/bilder/
glutfaden_16..256.png`; Fenstersymbol und Ueber-Fenster. `--probe-bearbeiten`
(`ansicht/vorfuehrung.dh`) spielt alles durch; `glutfaden_ansicht.dhtest`
prueft Ausgabe und Bild (Blau da, nach Zurueck weg, bei halber Deckkraft
gemischt), `glutfaden_bearbeiten.dhtest` (5) den Kern. Die gesicherte Datei
liest PyStickys `load_pattern` richtig (Ebenen, DMC 996 mit Symbol,
Halbstiche, radierte Zelle). **Tempo an 1000 x 1000:** Fuellen 2892 -> 14 ms,
Setzen 3455 -> 23 ms, Zurueck 665 -> 8 ms, Zusammenfuehren 400 -> 9 ms --
feste Zahlenfelder in der Schleife statt Aufrufen je Zelle; eine Hilfs-SUB
mit BYREF-Feldern hielt das Fuellen in der VM (ein Feld wird ohnehin als
Verweis uebergeben, BYREF braucht es nur zum Neu-Zuweisen). **Funde in der
Laufzeit:** (1) der Maschinencode kannte die **Bit-Operatoren** nicht (siehe
docs/entwurf-maschinencode.md, Schritt 24); (2) `a IS b` mit einer Variablen
rechts meldete "unbekannter Typ 'b'" -- jetzt der Hinweis, dass dasselbe
Objekt mit `=` verglichen wird (`typtest.dhtest`). Nicht getestet: die
modalen Dialoge (Umbenennen, Rueckfrage, Dateiwahl) -- `gtk_dialog_run`
blockiert die Vorfuehrung.

**Stufe 3: Auswahl, Vieleck, Symmetrie, Rueckstich, Fortschritt, Text,
Farben** (2026-10-15). Kern: `kern/auswahl.dh` -- eine Auswahl ist eine
Maske ueber dem Muster (Rechteck; Lasso = Zellmitten gerade/ungerade plus
die Kanten als Bresenham OHNE feste Richtung, wie das Original), ein
`Ausschnitt` (dabei/Farbe/Art je Zelle) ist Zwischenablage und Wanderinhalt;
Einfuegen, Verschieben, Drehen, Spiegeln sind EINE Operation
(`ausschnittSetzen`: alte Zellen leeren, Zellen mit Stich schreiben -- ein
leerer Platz im Ausschnitt laesst stehen, was dort ist). Auswahl und
Kopieren nehmen wie im Original nur die AKTIVE Ebene. `kern/formen.dh`:
Vieleck (Fuellspannen auf Vieleck UND Muster geschnitten -- das stand nicht
im Erkundungsbericht, der Abgleich fand es), `spiegelZellen` (die Formeln
des Originals sind genau `b-1-x`/`h-1-y`, achtfach nur bei b = h),
`spiegelRueckstich`. `kern/garn.dh`: CIEDE2000 wie core/color_math (auf
sechs Stellen gleich). `kern/bearbeiten.dh`: `zellenSetzenJe` (Farbe und Typ
je Zelle, Perle/Drill einmal je Farbe statt je Zelle), Rueckstiche als
TEIL_LISTEN-Schritte, Fortschritt auf der obersten sichtbaren Ebene,
`farbeErledigt` ueber alle Ebenen, `naechsterOffen` (Lesereihenfolge,
Umlauf), `farbeErsetzen` (nur Sichtbares, verdeckt bleibt; Perle/Drill
angepasst; Toleranz ueber `ersetzenQuellen`), `aehnlicheFarben` und
`selteneErsetzungen` (stabil wie Pythons sort), `symbolSetzen` (ersetzt den
Eintrag statt ihn zu aendern, sonst naehme der Vorher-Stand das neue Symbol
mit; doppelt = Fehlertext). Anders als PySticky ist das Symbol rueckgaengig
zu machen. Oberflaeche: `ansicht/mehr.dh` (die sechs Werkzeuge, Vorschau,
Auswahl-Umriss nur im sichtbaren Ausschnitt -- Rechteck als ein Rahmen),
`ansicht/text.dh` (Pango in ein A8-Bild, ein Punkt je Kaestchen, ab halber
Deckung ein Stich), `ansicht/dialoge.dh` (Farbe ersetzen mit Rangfolge,
Toleranz und Vorschau der Wirkung; seltene Farben; Symbol), Symmetrie-
Auswahl in der Leiste, erledigte Stiche abgeblendet mit Haken, Knoepfe
unter der Farbliste. Pruefungen: `glutfaden_stufe3.dhtest` (Vieleck, Lasso,
Spiegeln Punkt fuer Punkt gegen PySticky, Farbabstand, Rangfolge, seltene
Farben, Auswahl-Operationen, Rueckstiche mit Rundweg, Fortschritt, Farbe
ersetzen, Symbol) und ein dritter Fall in `glutfaden_ansicht.dhtest`
(`--probe-stufe3`, Bildproben: Text-Blau, Zuwachs gruener Haken -- Gruen
gibt es vorher schon an den Kanten blauer Rueckstiche ueber Gelb).
**Funde in der Laufzeit:** (1) `dhrt --check` meldete Befunde in
importierten Dateien mit der Zeile des ZUSAMMENGEFUEGTEN Quelltexts und
ohne Datei -- jetzt die eigene Zeile und `file` (`auf_dateien_umrechnen` in
main.rs; Sprachserver und CODE_CHECK$ rechneten schon selbst zurueck);
(2) `JOIN$` nahm nur ARRAY OF STRING -- jetzt auch Zahlen und
Wahrheitswerte (wie STR$). **Stolpersteine:** `IS` vergleicht Typen, fuer
"dasselbe Objekt" `=` (Stufe 2 hat die Meldung verbessert); die Shell frisst
in Heredocs Rueckstriche (Patches als Dateien).

**Stufe 4: Bildimport** (2026-10-16). Kern `kern/bildimport.dh`: Bild
vorbereiten (weiss unterlegen ueber `IMAGE_DRAW_IMAGE`, Ausschnitt wie das
Original geklemmt, `IMAGE_ENHANCE` fuer Helligkeit/Kontrast/Saettigung,
`IMAGE_SCALE(..., "lanczos")`), dann nur noch Zahlenfelder (`Pixelbild`):
die haeufigsten N Garne (stabil nach Haeufigkeit) oder Median-Cut, naechstes
Garn nach CIEDE2000 mit einer MAP je Farbe, Floyd-Steinberg (Gewichte in
derselben Reihenfolge wie numpy: `fehler * 7 / 16`, dann geklemmt), Bayer
4x4 (Versatz `2M - 16`, ganzzahlig), Confetti (Gebiete per Stapel, Runden
lesen das schon geaenderte Raster), Konturen per Sobel (Rand wie `np.pad`
edge, neun Summanden in der Reihenfolge des Originals). `Bildimport.schritt`
rechnet zeilenweise, die Oberflaeche pumpt dazwischen GTK (`gtkAbarbeiten`).
**Gegen PySticky Punkt fuer Punkt gleich** -- Pixel nach der Vorbereitung,
Garne, jedes Rasterfeld, jeder Rueckstich (`glutfaden_stufe4.dhtest`, die
Aufzeichnungen schrieb ein Skript in PyStickys venv) -- und etwa zehnmal
schneller (200x130, 30 Farben: 215/443/260 ms gegen 2,5/4,9/0,55 s). Bewusst
anders: Farben nach Haeufigkeit (PySticky: Reihenfolge einer Python-Menge),
darum bricht das Confetti einen Gleichstand anders (mit PyStickys Nummern
gefuettert: gleich); Konturen folgen dem Ausschnitt. Oberflaeche
`ansicht/importdialog.dh` (Strg+I, Strg+Umschalt+I = erneut aus dem
Quellbild ueber die Metadaten). **Funde in der Laufzeit:** (1) PILs
Skalierung und `ImageEnhance` gab es nicht -- `resample.rs`, `IMAGE_SCALE`
mit Filter und `IMAGE_ENHANCE`, gegen Pillow-Bilder ohne einen abweichenden
Punkt (`bild_wie_pillow.dhtest`); (2) **ein Feld in einer Methode einer mit
AS importierten Klasse** galt der Namensraum-Pruefung als Name von aussen:
hiess eine Globale des Hauptprogramms genauso, Abbruch; hiess ein Name des
Moduls genauso (hier das Feld `helligkeit` neben der Funktion `helligkeit`),
wurde das Feld STILL umbenannt und schrieb in die Modul-Globale
(`klassen_felder` in namensraum.rs). **Stolpersteine:** Namen ohne
Gross/klein -- ein Feld `voll` oder eine lokale `perle` verdeckt `VOLL`/
`PERLE` (`--check` sagt es als Typwarnung); ein Objekt-Parameter, dem die
Funktion etwas Neues zuweist, braucht BYREF; die helle Schrift fuer
Klapplisten im Stylesheet galt auch in hellen Dialogen.

**Stufe 5: Export** (2026-10-16). Kern `kern/export.dh` (Bedarf, Seitenplan
A4..A0/Letter, Diamond 1:1 in Punkten wie reportlab, Entsprechungen ueber
CIEDE2000), `kern/oxs.dh` (Schreiber byte-gleich mit PySticky bis auf LF,
Leser samt dessen Warnungen; Stiche einer Perlen-/Drillfarbe werden PERLE/
DIAMANT), `kern/bildexport.dh` (PNG/JPG/BMP; die Leinwand zaehlt Koordinaten
an Punktmitten wie PIL, Qt an Kanten -- alles ausser Rechtecken um -0,5),
`kern/pdfexport.dh` (reportlabs Fluss nachgebaut: Absaetze mit Umbruch und
Auszeichnung, Tabellen mit wiederholtem Kopf, Zeichnungen; Tabellen-Leading
bleibt 12 pt, Abstand davor minus Abstand nach dem vorigen Absatz),
`kern/htmlexport.dh` (Kopf mit CSS aus dem Python-Quelltext umgesetzt),
`kern/paket.dh` (ZIP mit pxs/html/png/pdf/garnliste.csv/original/LIESMICH).
**Geprueft gegen PySticky:** HTML Zeile fuer Zeile gleich (auch 2,5 MB),
Garnliste byte-gleich, PDF ueber ein Ablaufprotokoll in PyStickys
Schreibweise gegen Aufzeichnungen aus dessen Story (`ref_pdf.py` faengt
`SimpleDocTemplate.build` ab) samt Seitenzahl; Absichten: kein "None", ◆
statt 💎 (DejaVu hat kein Emoji), Diamond-Groesse = Drills mal Kante, Fuss
"Erstellt mit Glutfaden". Oberflaeche `ansicht/exportdialog.dh` (Strg+E,
ein Dialog fuer alle Formate, nicht Geltendes ausgegraut), `.oxs` oeffnet
im Oeffnen-Dialog, `--probe-stufe5`. **Funde in der Laufzeit:** PDF_CIRCLE/
_FILL, PDF_POLYGON/_FILL, Winkel bei PDF_TEXT, PDF_OPACITY (krilla
ExtGState; GDI und PDF_PREVIEW mischen gegen Weiss), PDF_NEW mit A2/A1/A0,
**ZIP_WRITE wandelte Puffer still in "<BUFFER n Bytes>"**, Namensraum:
Methoden mit dem Namen einer Modulfunktion, `--check` warnt bei einem Text,
den JSON_TYPE nie liefert ("bool" statt "boolean" -- daran luden
Wahrheitswerte einer .pxs nie). Ohne Passwortschutz (krilla verschluesselt
nicht); die Stickpfad-Vorschau kam mit Stufe 7.

**Stufe 6: fremde Formate, Diamond, Blends** (2026-10-16). **PyStickys
Leser fuer .pat und .xsd lesen keine echte Datei** (sie erwarten Kennungen
"PAT" bzw. "PM", die es nicht gibt). `kern/pat.dh` ist an einer echten
PCStitch-8-Datei ("Cement Mixer" von CyberStitchers, nur zum Testen
geladen, nicht im Repo) entschluesselt und gegen das PDF geprueft, das
PCStitch selbst davon druckt (Textlagen und Linien ueber `pdftext.dh`):
alle 1073 Stiche samt Viertel-/Dreiviertelstichen mit Symbol, alle 210
Rueckstiche. Gitter spaltenweise in Laeufen, "Lage" = Ziffernblock-Raster
(1 2 3 / 4 5 6 / 7 8 9), Art 1 ganz, 31..34 Viertel bzw. Dreiviertel (Lage
5); zwei Teilstiche je Zelle -> zweite Ebene. `kern/xsd.dh` folgt der
Formatbeschreibung von Embroiderly (GPL, eigener Code; Herstellertafel
`kern/xsdhersteller.dh`), geprueft an dessen Beispieldatei (250 x 331, acht
Blends): Stichblock XOR-verschluesselt mit wanderndem Schluessel, Texte
UTF-8 oder cp1251, Petite -> Viertel, Spannstich -> Rueckstich, Knoten/
Perlen auf die naechste freie Ebene. **Teilstiche und Rueckstiche in XSD
sind nur an selbst gebauten Dateien geprueft** (das Beispiel hat keine).
Tests bauen beide Formate mit `_hilfen/glutfaden_pat.dh`/`_xsd.dh`.
`kern/diamant.dh` = PySticky `convert_to_mode` (mode_backups je farbId,
Stoff 14 -> 10 -> 14, Umstempeln auch der Teilstiche wie dort), gegen
PyStickys Ausgabe fuer zwei Muster hin/zurueck/hin geprueft; Strg+D,
Knopf in der Kopfleiste, die Leinwand zeichnet ganze Stiche als Drills.
`garnBlend` = Thread.blend (Lab-Mittel, Name "DMC 310 / DMC 321 (1+2)"),
Dialog Strg+B bzw. [Blend...] neben der Garnsuche. **Fund:**
`farbeHinzufuegen` legte einen Blend mit anderem Strangverhaeltnis still auf
den vorhandenen zusammen. `--probe-stufe6`, Sammlung
`tests/pruef/glutfaden/glutfaden_stufe6.dhtest`.

**Stufe 7: der Stickpfad** (2026-10-17). `kern/stickpfad.dh` = PyStickys
`stitch_path_optimizer`: zeilenweise (Schlange je Zeile), naechster Nachbar
(Faecher-Raster mit Ringsuche wie dort, verkettete Listen statt Mengen),
daenisch, spaltenweise, diagonal; Farben in der Reihenfolge ihres ersten
Vorkommens, Sprung = Schritt ueber 4 Stiche, Garn 2,5 cm je Stich plus
0,5 cm je Stich Rueckseite bei 14 ct; `stickplanText$` (PyStickys
"Als Text exportieren"), `spVergleich` (stabil nach Distanz), `spBewertung$`,
`spAusschnitt` (eine Seite: Stiche darin, Abstaende und Spruenge neu).
**Gleichstand beim naechsten Nachbarn:** PySticky nimmt die Reihenfolge
einer Python-Menge, Glutfaden den oberen, dann den linken Stich. Mit dieser
Regel (fuer die Aufzeichnung in PySticky eingesetzt) ist der Pfad Stich fuer
Stich derselbe -- auch am Leuchtturm (1600 x 9 Stiche) --, die vier anderen
Strategien sind es ohnehin, der Stickplan byte-gleich. Das PDF zeichnet wie
PySticky (`include_path_preview`, Vorgabe an) auf jeder Musterseite die Pfade
unter die Symbole (Garnfarbe x 0,7, Deckkraft 0,6; Spruenge rot gestrichelt
0,5) und den Absatz "Stickpfad: n Stiche | c Farben | j Spruenge", nicht im
Diamond-Modus; `PdfOptionen.stickpfad`, Kaestchen im Export-Dialog, die
Stufe-5-Aufzeichnungen laufen mit `stickpfad = FALSE` (`gePdfOptionen`).
Oberflaeche `ansicht/stickpfaddialog.dh` (Strg+Umschalt+O, Knopf in der
Kopfleiste) = StitchPathDialog + PathPreviewWidget: Vorschau mit Pfeilen,
S/E, Nummern ab 16 Punkten, Kontext blass, Grund nach Helligkeit des Garns,
Rad-Zoom um die Maus, Ziehen, Tooltip je Stich; Tabelle je Farbe mit
Garnbalken, Summen, Vergleich (waehlt die beste); Export als Text, PNG der
Farbe (15 Punkte je Stich, Legende) und alle als `01_Name.png`. Anders: der
Pfad steht beim Oeffnen schon da, kein Faden, Meldungen unten statt Kasten.
**Fund in der Laufzeit:** das pdf-Modul konnte nicht stricheln -- neu
`PDF_DASH(p, strich_mm [, luecke_mm])` fuer Linien und Umrisse, im PDF
(krilla `StrokeDash`), im Windows-Druck (geometrischer GDI-Stift) und in
`PDF_PREVIEW` (die Folge laeuft ueber die Ecken weiter). `--probe-stufe7`
(Punktproben an den PNGs), Sammlung `tests/pruef/glutfaden/glutfaden_stufe7.dhtest`
(9; neun Verfaelschungen des Kerns und acht des Dialogs fallen je in ihrem
Fall). **Stolperstein:** ein eben sichtbar gemachtes Widget hat im selben
Schritt noch keine Groesse -- `gtkFoto` gleich danach meldet
`alloc_needed`; das Foto gehoert in den naechsten Schritt.

**Stufe 8: auswerten** (2026-10-17). `kern/analyse.dh` rechnet, was
PyStickys Statistik-Dialog, Heatmap, Farbenblindheits-Ansicht und
"Aehnliche Farben" rechnen: `statistikVon` (get_statistics: Stiche je Ebene
auch unsichtbar, bedeckt ueber das Komposit, Perlen nur sichtbar),
`fortschrittVon` (ohne ausgelassene Farben, je Farbe mit; round(x, 1) als
`VAL(FORMAT$(x, "%.1f"))`), `schwierigkeitVon` (vier Faktoren je 0..3,
Sonderstiche ueber ALLE Ebenen, im Diamond-Modus ist der Drill die
Grundform), `garnRechnen` (je echtem Garn -- ein Blend mit jedem seiner
Garne und der vollen Stichzahl -- CEIL mit Zuschlag), `zeitRechnen`,
`statistikCsv$` (BOM, CR LF, Komma wie Pythons csv-Modul), `heatmapRechnen`
(Werte als single ueber BUFFER_SET/GET_F32 wie numpy float32, Randbloecke
nach ihrer echten Flaeche), `heatmapFarben` (die Farbe je Tausendstel nimmt
der erste Wert darin -- PyStickys Zwischenspeicher, am echten QImage
nachgesehen), `farbenblind` (Vienot mit Pythons round), `aehnlichePaare`
(CIEDE2000, stabil sortiert) und `farbenZusammenfuehren$` (Ketten werden
aufgeloest, eine Farbe in zwei Paaren ist ein Fehler). **Gegen PySticky
gleich** (ref_analyse.py) an einem von PySticky gebauten Probemuster (drei
Ebenen, ausgelassene Farbe, Blend, Perlen, Teilstiche, Erledigtes), im
Diamond-Modus, am Beispielmuster und am Leuchtturm; die CSV byte-gleich.
**Bewusst anders beim Zusammenfuehren:** PySticky loescht mit der Farbe
ihre Rueckstiche (im Probemuster alle zwoelf) und es ist nicht
rueckgaengig zu machen -- hier gehen sie an die bleibende Farbe, und es ist
ein Schritt im Verlauf (`farbenBehalten` mit Umleitung, aus
`unbenutzteEntfernen` herausgeloest; `alleUmfaerben` ueber alle Ebenen --
`farbeErsetzen` nimmt nur Sichtbares, der erste Versuch verlor so 30
verdeckte Stiche). Oberflaeche `ansicht/analysedialoge.dh`: Knopf
"Auswerten" in der Kopfleiste mit Aufklapper (Statistik mit fuenf Reitern
und CSV, auch Strg+Umschalt+G; Heatmap; Aehnliche Farben; Farbenblindheit
als Auswahl -- nur `lwRgb` der Zeichenflaeche, die Seitenleiste bleibt wie
in PySticky). Der Reiter Einkaufsliste kam mit Stufe 9.
`--probe-stufe8`, Sammlung `tests/pruef/glutfaden/glutfaden_stufe8.dhtest` (8; 13
Verfaelschungen des Kerns und 8 der Oberflaeche fallen je in ihrem Fall).

**Stufe 9: planen und Vorrat** (2026-10-17). `kern/planung.dh` nach
PyStickys core/inventory.py, project_list.py, hoop_planner.py und den
Rechnungen der Dialoge fuer Palette und Tauschen. **Garn-Vorrat** als JSON
im Einstellungsordner (`einstellungsDatei$`: APPDATA bzw. ~/.config,
Unterordner Glutfaden) im selben Aufbau wie PyStickys inventory.json
(`{"version":1,"stock":{sortiert}}`, Einzug 2) -- Schluessel
`Hersteller::Nummer`, ohne beides `unknown::unknown::Name`; gelesen wie
Pythons `int()` (Kommazahl Richtung null, Text mit Ziffern und `_`,
TRUE = 1, alles andere uebergangen, auch eine flache Datei ohne `stock`),
0 oder weniger beim Setzen nimmt heraus. **Der Fund dabei steckt in der
Laufzeit:** ein Schluessel wie `unknown::unknown::Farbe 1.5` war im
json-Modul nicht zu erreichen, der Punkt teilt den Pfad -- neu `\.` und
`\\` im Pfad (`json_segmente` in builtins.rs) und **`JSON_KEY$(name)`**,
das einen Namen aus `JSON_KEYS` wieder adressierbar macht. **Einkaufsliste**
(`einkaufslisteMehrere`): je echtes Garn, CEIL je Farbeintrag, ueber
Schluessel summiert und sortiert, der Vorrat EINMAL abgezogen; DP-Muster
mit Drills statt Straengen. **Palette konvertieren**: naechstes Garn nach
CIEDE2000, bei Gleichstand das erste (die Paletten haben echte
Gleichstaende -- die Gegenprobe mit `<=` faellt), gut/akzeptabel/schlecht
bis 10/25, Mehrfachzuordnung gewarnt; ein Schritt im Verlauf (PySticky
leert ihn). **Farben tauschen**: `farbeErsetzen` geht jetzt ueber
`farbenUmsetzen` (Abbildung Farbe -> Ziel, nur Sichtbares auf der
Ursprungsebene, Perle/Drill angepasst), das Tauschen ist dieselbe
Abbildung in beide Richtungen. **Rahmenaufteilung** (`rahmenRaster`/
`rahmenPlanen`, der letzte Sektor rueckt an den Rand, gezaehlt ueber
Zeilensummen statt je Zelle). **Gegen PySticky gleich** (ref_planung.py,
570 Zeilen) an einem von PySticky gebauten Probemuster, einem DP-Muster und
einem grossen fuer die Strangrechnung; Vorrat und Projektliste byte-gleich
bis auf CR LF, das Python unter Windows im Textmodus schreibt. Oberflaeche
`ansicht/planungsdialoge.dh`: Garn-Vorrat mit drei Reitern (Im Muster je
ECHTES Garn statt je Farbeintrag -- PySticky legte den Bestand eines Blends
unter einen Schluessel, den die Einkaufsliste nie fragt; Alle Eintraege mit
Suche und Eintrag von Hand; Mehrere Projekte, die gemeinsame Liste folgt
dem Bestand sofort), Einkaufsliste als sechster Reiter der Statistik (folgt
dem Zuschlag des Garn-Reiters, PySticky liest ihn einmal), Palette
(Strg+Umschalt+K, Ziel von Hand mit Suche), Tauschen (Strg+Umschalt+T),
Rahmen mit Vorschau. Strg+Umschalt+I bleibt "Bild neu importieren", der
Vorrat hat kein Kuerzel. **Stolperstein im Layout:** zwei dehnbare Spalten
in einer Zeile teilen den Platz nach ihrer Textlaenge, die Spalten danach
stehen in jeder Zeile woanders -- `pdReihe` setzt `max_width_chars` 1.
`--probe-stufe9` (Einstellungsordner ueber APPDATA/HOME umgesetzt), Sammlung
`tests/pruef/glutfaden/glutfaden_stufe9.dhtest` (7; elf Verfaelschungen von Kern und
Oberflaeche fallen je in ihrem Fall).

**Stufe 10: Muster anlegen und verwalten** (2026-10-17). `kern/verwalten.dh`
nach PyStickys Pattern.resize, smart_resize, pattern_diff, session_timer,
snapshots und den Dialogen fuer neues Projekt, Eigenschaften und eigene
Vorlagen. **Groesse aendern** zuschneiden/auffuellen oben links (Rueckstiche
mit einem Ende ausserhalb fallen weg) oder Smart-Resize (naechster Nachbar
mit `INT(i * sx)`, Rueckstiche gerundet wie Pythons `round`) -- ein Schritt
im Verlauf (`TEIL_GROESSE`, PySticky leert ihn). **Vergleich** je Zelle
(neu/weg/anders/gleich ueber das zusammengesetzte Bild, Farben ueber ihren
Schluessel, damit verschobene Indizes nicht als Aenderung zaehlen) plus
Rueckstiche. **Stickzeit** in den Metadaten (`total_stitch_seconds`,
`last_session_start`, Sitzungen ueber 12 h zaehlen nicht), laeuft solange
das Werkzeug Fortschritt (K) gewaehlt ist -- dafuer neu in der Laufzeit
**`ZEIT_UNIX()`** (echte UTC-Sekunden als Kommazahl; `ZEIT_JETZT` ist
Ortszeit, PySticky schreibt `time.time()`), `ZEIT_AUS_UNIX`/
`ZEIT_NACH_UNIX` (ueber die Sommerzeitgrenze geprueft). **Versionen**
`snapshots/<Schluessel>/v_JJJJ-MM-TT_hh-mm-ss.pxs` im Einstellungsordner,
hoechstens 20, beim Sichern und alle fuenf Minuten, wenn etwas ungesichert
ist und die letzte aelter als 30 Minuten (PySticky haengt sie an sein
Autosave); ein ungesichertes Muster bekommt eine Kennung, sonst teilten sich
zwei "Neues Muster" einen Ordner. **Vorlagen**: 31 eingebaute in sieben
Kategorien, zwoelf DP-Groessen, eigene in `user_templates.json` (Breite/
Hoehe wie `int()` -- PySticky nimmt 12.7 als 12.7). **Gegen PySticky
gleich** (ref_verwalten.py, 98 Zeilen) bis auf diese eine Zeile.
Oberflaeche `ansicht/verwaltungsdialoge.dh`, Knopf "Muster" in der
Kopfleiste: Neues Projekt (Strg+N; Kategorien, Karten, DP-Preset, Stoff,
Vorschau; Strg+Umschalt+N bleibt "Neue Ebene"), Eigenschaften (Strg+Alt+P;
Stickdatum als TT.MM.JJJJ, der 31.02. ist ein Fehler), Groesse aendern
(Rueckfrage nur beim Verkleinern ohne Smart), Versionen (Strg+Alt+V;
wiederherstellen laedt ungesichert ohne Pfad, loeschen, mit dem aktuellen
vergleichen: Alt/Neu/Diff), Vorlagen sichern und verwalten. **Der Fund
dabei steckt in der Laufzeit** (siehe Dogfooding 12, Punkt 5): ein Fehler im
Zeichnen der Vorschau machte jedes spaetere Zeichnen kaputt. Foto-Falle: ein
eben umgelegtes Kaestchen erscheint im Foto noch leer (GTK blendet es ein),
der Zustand stimmt. `--probe-stufe10`, Sammlung
`tests/pruef/glutfaden/glutfaden_stufe10.dhtest` (6; zwoelf Verfaelschungen von Kern
und Oberflaeche fallen je in ihrem Fall).

**Stufe 11: Farb-Harmonien und Muster-Bibliothek** (2026-10-17).
`kern/harmonie.dh` nach color_harmony_dialog.py: der Farbton dreht im
HSL-Raum (Pythons colorsys samt `% 1.0` und `int()` beim Zurueckrechnen),
Saettigung und Helligkeit bleiben, je Versatz EIN Vorschlag mit dem
naechsten Garn der Palette (CIEDE2000, Gleichstand = das erste); gegen
PySticky gleich (640 Zeilen, 40 Farben, drei Paletten). `kern/bibliothek.dh`
nach pattern_library_data.py/-dialog.py/thumbnail_widget.py: library.json
im selben Aufbau (byte-gleich geschrieben; ein Eintrag mit unbekanntem
Schluessel faellt weg wie bei `LibraryEntry(**e)`), Einsortieren nach
Teilwoertern aus Datei- und Ordnername (wie im Original findet "ei" auch
"Meise"), Stiche ueber ALLE Ebenen, Vorschaubild Punkt fuer Punkt wie Qt
(hoechstens 100 Punkte und doppelt, `INT(massstab)` je Stich -- bei 1,67
bleiben Luecken, unter 1 ueberschreibt), Cache-Name `MD5$(pfad)` 16
Zeichen. Mit Absicht anders: Bibliothek im Einstellungsordner, Vorgabe
"Bordüren" (PySticky "Bordueren" -- so einsortierte Muster standen unter
keiner Kategorie), Sortieren nur in der Anzeige, Vorschaubilder auch fuer
.oxs/.pat/.xsd. Oberflaeche `ansicht/sammlungdialoge.dh`: Harmonien
(Strg+Umschalt+H, [Harmonien…] unter der Farbliste; Karten mit Cairo
gezeichnet, Hinzufuegen als Verlaufsschritte, die Meldung zaehlt nur
Neues), Bibliothek (Strg+L, "Muster" -> Bibliothek; Kategorien, Suche,
Sortierung, Karten im Raster zu fuenf, fehlende Vorschaubilder je Takt,
Kontextmenue per GtkMenu, Schlagwoerter/Notizen beim Kartenwechsel und
Schliessen gesichert). **Zwei Funde in der Laufzeit:** (1) `CATCH e` und
spaeter `DIM e` -- im Hauptprogramm brach der CATCH mit "globale Variable
noch nicht gesetzt" ab (das DIM gibt dem Namen einen Platz, der CATCH
schrieb hinein, bevor er eingehaengt war; jetzt `DECLARE_GLOBAL_SLOT` wie
FOR), und ein DIM mit anderem Typ warnte nur "ist als STRING angesagt" --
jetzt dieselbe Warnung wie bei zwei DIM, mit der Zeile des CATCH. (2)
`--check`, der Sprachserver und CODE_CHECK$ zeigten interne Namen
(`KERN@BIBLIOTHEKSEINTRAG`) -- jetzt `lesbar_text` in `check_source`.
**Stolpersteine:** `IMAGE_FILL_RECT` nimmt zwei Ecken (beide eingeschlossen),
keine Breite; `adLeeren` ist nur fuer Listen (an einer GtkBox lauter
Gtk-CRITICAL, und nichts wird entfernt); die Dialoge sind hell, gezeichnete
Karten brauchen helle Farben. `--probe-stufe11` (Ordner "sammlung" neben dem
Arbeitsordner ueber STARTDIR$), Sammlung `tests/pruef/glutfaden/glutfaden_stufe11.dhtest`
(5; vierzehn Verfaelschungen fallen je in ihrem Fall -- der Gleichstand erst
mit einer eigenen Palette aus zwei gleichen Farben).

**Stufe 12: Muster-Vorschau** (2026-10-17). Vorher: das Hauptfenster nimmt
92 % der Arbeitsflaeche des Bildschirms (`fensterGroesseSetzen` ueber
`gdk_monitor_get_workarea`, Wunsch des Nutzers), die Leiste rechts bleibt
400 breit. `kern/qtfarbe.dh` baut Qts `QColor.darker/lighter` nach -- 16 Bit
je Kanal, HSV in float32 (ueber `BUFFER_SET_F32`), `qRound` und
`qt_div_257` wie Qt, Ketten bleiben auf 16 Bit; gegen PySide gleich (321
Farben, alle Faktoren der Vorschau). `ansicht/vorschaubild.dh` zeichnet nach
preview_render_engine.py/diamond_drill_render.py mit Cairo (ohne Fenster
nutzbar, `vbPng`): Stoff (Aida-Loecher, Kreuze aus Faeden mit Schatten und
Glanz, Teilstiche, Knoten, Perlen), Pixel, Symbol (Raster, jede zehnte Linie
kraeftig ab dem Rand des Ausschnitts), Drills (vier Facetten
heller145/110, dunkler130/105), Rueckstiche (Schatten, im Symbolplan
gestrichelt), Erledigtes. **Wie im Original** bleibt die Kantenglaettung
nach einem Drill aus, bis ein Teilstich/Knoten/Perle sie wieder anschaltet.
Gegen PyStickys `render_full` sind die Bilder nicht gleich (Cairo und Qt
glaetten Kanten verschieden), aber nah: Pixel und Drills unter 1,5 %
deutlich abweichender Punkte, Stoff bei 20 Punkten 0 %, bei 12 rund 4 %,
darunter mehr, weil fast alles Kante ist. Oberflaeche
`ansicht/vorschaudialog.dh` (F5, Knopf in der Leiste): Darstellung, Stoff
(Aida 11..32), Stoff-Farbe, Rueckstiche, Fortschritt, Zoom 20..300 (Regler,
Einpassen, 1:1, Mausrad um den Punkt unter der Maus, mit Strg in feineren
Schritten), Ziehen verschiebt, gezeichnet wird nur der sichtbare
Ausschnitt; "Als Bild speichern" fragt Punkte je Stich und Ziel (ueber 64
Mio. Punkte mit Rueckfrage) und schreibt mit derselben Einstellung wie die
Ansicht. Diamond Painting: nur Stoff/Pixel, ohne Stoff-Auswahl. **Pruefen
ohne Qt-Glyphen:** Qts Offscreen-Plattform zeichnet die meisten Symbole als
leere Kaestchen -- der Symbolplan wird darum an einem Muster ohne Symbole
verglichen. Sammlung `tests/pruef/glutfaden/glutfaden_stufe12.dhtest` (3: Qt-Farben,
zehn Bilder gegen PySticky mit Grenze je Bild, die Probe samt "Export =
vbPng" und "der Punkt unter der Maus bleibt"); acht Verfaelschungen fallen
je in ihrem Fall.

**Stufe 13: Einstellungen** (2026-10-17). `kern/einstellungen.dh` haelt
`einstellungen.json` im Einstellungsordner mit PyStickys QSettings-Namen
(`major_grid_interval`, `fill_tolerance`, `shortcuts` ...); gelesen wird
nachsichtig (Text "true", Zahl als Text, Kurz-Hex; Krummes bleibt bei der
Vorgabe, Zahlen werden geklemmt wie ein Drehfeld; Neben-Raster echt kleiner
als Haupt-Raster). **Uebernommen ist nur, was in Glutfaden wirkt** --
PySticky hatte lange Reiter, die nur schrieben; weggelassen: Sprache,
Datei-Protokoll, Stoff-Textur, magnetisches Raster, laufende Ameisen,
Tablet/Touch, Farbleiste, Querverweis-Paletten, Bibliotheks-/Vorlagenpfad.
Mit Absicht anders: Standard-Palette DMC (PySticky Anchor), "Farbe +
Symbol" als Vorgabe (PySticky "Nur Farbe" -- Glutfaden zeigte Symbole
schon immer), kleinste Zelle 1 (grosse Muster passen sonst nicht ins
Fenster), Startaktion 0 = Beispielmuster. Wirkt auf: Zeichenflaeche (Raster
in drei Stufen mit eigenen Farben, gegen den Stoff geprueft wie
`ensure_contrast` -- BT.601, nicht WCAG; Hintergrund, Stoff, Zellgroessen,
Zoomfaktor, Symbolschrift/-groesse, Anzeige Farbe/Symbol), Werkzeuge
(Startwerkzeug oder das zuletzt benutzte, Pipette danach Stift/bleiben/
Auswahl, Fuellen mit Toleranz und diagonal -- `fuellungWie` mit einer
Tafel passender Farben, gegen PyStickys FillTool gleich; Rueckstich-Breite
und Einrasten), Farben (Katalognummern, Standard-Palette, "?" ohne
automatisches Symbol), Export-/Import-Vorgaben, Autor/Copyright als
Rueckfall in PDF/HTML/Paket (das Muster geht vor). **Tastenkuerzel** aus
EINER Tafel (`kuerzelAktionen`, 49 Befehle; `befehlAusfuehren` in main.dh
statt der festen SELECT-Liste), Text wie "Strg+Umschalt+S", mehrere durch
Komma; ein eigenes Kuerzel, das zusammenstoesst, gilt nicht
(apply_saved_overrides). Umschalt zaehlt bei Zeichen nicht ("=" statt
"Umschalt+="), der Buchstabe kommt aus dem Keyval in Grossschreibung --
Feststelltaste aendert nichts. Tooltips nennen das Kuerzel ueber
Platzhalter (`{sichern}`), `tipp()` merkt sie und frischt nach jeder
Aenderung auf (zerstoerte Widgets fallen per "destroy" heraus). Dialog
`ansicht/einstellungsdialog.dh` (Muster -> Einstellungen, Strg+Komma):
sechs Reiter, erst Uebernehmen/OK schreibt, Kuerzel per Tastendruck mit
Zusammenstoss-Meldung; Raster-Optionen (Strg+Umschalt+R) mit den vier
Voreinstellungen schreiben in dieselben Einstellungen. Dazu Zuletzt
geoeffnet im Muster-Knopf (Vergleich ueber `SAMEFILE` -- ein 8.3-Kurzname
und der lange Name waren sonst zwei Eintraege), Startaktion, gemerkte
Fenstergroesse, `.bak` vor dem Ueberschreiben, Startordner der
Datei-Dialoge. **Automatisch sichern** (`ansicht/autosichern.dh`):
`<datei>.pxs.autosave` bzw. `autosicherung/<sitzung>.pxs`, ueber Nebendatei
+ RENAME; ohne Prozessnummer eine zufaellige Sitzung mit Lebenszeichen
(`.lebt`, alle 30 s) -- angeboten wird beim Start nur, was seit drei Takten
stumm ist, beim Oeffnen ein Stand neuer als die Datei. Sammlung
`tests/pruef/glutfaden/glutfaden_stufe13.dhtest` (6: PySticky-Referenz 712 Zeilen,
JSON, Kuerzel, Export, Probe samt Rasterfarben im Bild, drei Startlaeufe);
21 Verfaelschungen fallen je in ihrem Fall.

**Stufe 14: Farben verwalten** (2026-10-17, PyStickys
`color_management_dialog.py`). Kern `kern/farbverwaltung.dh`: ein
`Farbplan` (die bleibenden alten Nummern in der neuen Reihenfolge, je alter
Farbe Ziel = bleibt/weg/geht auf in, Faeden), den der Dialog bearbeitet;
`farbplanAnwenden` macht daraus EINEN Schritt -- erst die Stiche
weggenommener Farben auf allen Ebenen, auch gesperrten (kurz entsperrt,
sonst blieben dort Nummern stehen, die es nicht mehr gibt), dann Liste,
Faeden (Kopie des Eintrags, sonst aenderte der Verlauf den Vorher-Stand mit),
Rueckstiche und das Umnummerieren ueber einen `TEIL_FARBEN`-Teil. Beim
Zusammenfuehren wird der Typ nur umgestempelt, wenn Quelle oder Ziel Perle
oder Drill ist (wie `merge_colors_stitches`), "erledigt" bleibt; eine Kette
(A in B, dann B in C) landet ganz in C. Acht Sortierungen, stabil auch
absteigend wie Pythons `reverse=True`. **Mit Absicht anders als PySticky,
alles dort Fehler:** Umordnen und Sortieren nummerieren die Stiche mit
(PySticky stellt nur die Liste um und faerbt das Muster damit um; sein
`_swap_colors` setzt jeden Stich als vollen Stich und uebergeht gesperrte
Ebenen); Abbrechen laesst das Muster unberuehrt und der Verlauf bleibt (PySticky
behaelt Teile und leert den Verlauf); "Ungenutzte entfernen" laesst Farben
mit Rueckstichen stehen. Dialog `ansicht/farbdialog.dh` (Strg+Umschalt+F,
Knopf "Verwalten..." in der Seitenleiste; die Kuerzel-Tafel hat damit 50
Befehle): Liste mit Mehrfachauswahl, Sortierung, Suche, Faeden je Auswahl
und fuer alle, Verschieben (nur bei einer Auswahl), Loeschen, Zusammenfuehren
(die oberste gewaehlte bleibt), Ungenutzte, Zuruecksetzen; erst Uebernehmen
aendert das Muster, die aktuelle Farbe wandert mit. **Nebenfund:**
Zuruecknehmen und Wiederholen stellten die Farbliste um, nicht aber die
Nummer der aktuellen Farbe -- nach Sortieren und Strg+Z war aus Gelb Blau
geworden; `werkzeugZurueck`/`Vor` suchen sie jetzt ueber Symbol, Garnname
und Farbe wieder (`wzFarbeWiederfinden`). Sammlung
`tests/pruef/glutfaden/glutfaden_stufe14.dhtest` (4: PySticky-Referenz 64 Zeilen,
wobei Ablaeufe mit Verschieben nur Farben und Rueckstiche vergleichen; das
Bild bleibt bei neun Umordnungen samt gesperrter Ebene und Strg+Z/Strg+Y;
Loeschen/Zusammenfuehren als ein Schritt; die Probe); elf Verfaelschungen
fallen je in ihrem Fall.

**Stufe 15: Zeichenflaeche** (2026-10-17). Kern `kern/flaeche.dh`:
`farbverlauf` (PyStickys GradientTool: Bresenham IN Zugrichtung --
`linie()` tauscht die Enden, hier bekommt der erste Punkt die Startfarbe --,
je Punkt der Anteil i/(n-1), je Kanal abgeschnitten wie Pythons `int()`, davon
die naechste Palettenfarbe nach CIEDE2000; ein einzelner Punkt nimmt die
Startfarbe, eine Farbnummer ausserhalb ergibt nichts), `minikarteMass/
-Punkte/-Ausschnitt` (MinimapWidget: 8 Punkte Rand, eingepasst, Feld
mindestens 120 x 100, naechster Nachbar ueber `(i * b) \ iw`, das
Ausschnitt-Rechteck mit Qts `right()` = links + Breite - 1) und `kachelZelle`
(TilePreviewWidget: 20/30 Punkte Rand, hoechstens 12). Ansicht
`ansicht/flaechenteile.dh`: **Lineale** oben und links (Striche am
Haupt-Raster statt fest alle 10 wie PySticky, beschriftet nur, wo die Zahlen
nicht ineinanderlaufen; Zeile/Spalte unter dem Zeiger gruen; ein Klick holt
sie in die Mitte, die Ecke zentriert ohne Zoom), die **Uebersicht** oben in
der Seitenleiste (Klicken/Ziehen springt), die **Kachel-Vorschau**
(Strg+Umschalt+P, Muster-Knopf; modal statt Dock) und das Werkzeug
**Farbverlauf** (D) mit eigener Leiste (Start, Ende, tauschen), Vorschau je
Punkt in seiner Farbe, Ende ins Muster gezogen, Symmetrie (jedes Spiegelbild
in der Farbe seines Punkts) und Stichart. Lineale und Uebersicht nachgezogen
werden an EINER Stelle (`flaecheNachziehen` am Ende des Zeichnens, wenn sich
Lage, Zoom, Maus oder Groesse geaendert haben) statt an jeder, die
lwX/lwY/lwZelle setzt; abschaltbar (Strg+Umschalt+L/U, Einstellungen),
gemerkt als `show_rulers`/`show_minimap` (PySticky merkt Docks im
Fensterzustand). Start- und Endfarbe wandern bei Farben verwalten und
Rueckgaengig mit wie die aktuelle Farbe (`wzKennungen`). Die Kuerzel-Tafel
hat damit 54 Befehle. Sammlung `tests/pruef/glutfaden/glutfaden_stufe15.dhtest` (3:
PySticky-Referenz 132 Zeilen, die Minikarte als MD5; Randfaelle des
Verlaufs; die Probe samt Lineal-Marke im Foto -- gezaehlt nur im Streifen
des linken Lineals, die Marke ist halbdurchsichtig und die Statuszeile
traegt gruene Punkte); 17 Verfaelschungen
fallen je in ihrem Fall (Runden statt Abschneiden faellt nur an einer
eigens gebauten Palette -- in den Referenzdaten aendert es nie die naechste
Farbe).

**Stufe 16: Farbleiste und Willkommensseite** (2026-10-17). **Farbleiste**
`ansicht/farbleiste.dh` (PyStickys ColorBar) unter der Zeichenflaeche: je
Farbe ein Feld (Verlauf lighter(115)/darker(110) ueber `kern.farbeHeller/
-Dunkler`, Glanz, Symbol, Leuchten und Haken an der gewaehlten); EIN
Zeichenfeld statt eines Widgets je Farbe, Lage aus `fbFeldX`, die Zeichnen,
Treffer und Tooltip teilen. Klick waehlt, Doppelklick aendert das Symbol,
rechte Taste = Menue (Symbol, Harmonien, hervorheben, nicht sticken,
ersetzen, tauschen), ein Feld auf ein anderes gezogen (ab 25 Punkten
Manhattan) tauscht die Farben ueber `tauschenAusfuehren`; am Rand rollt
sie von selbst, das Rad seitwaerts. Groesse `color_bar_size` 20..60,
ein/aus `show_color_bar` (Strg+Umschalt+B). Nicht: Garn aus dem Katalog
hineinziehen (DnD zwischen Widgets). **Hervorheben** (Strg+H, PySticky
"Nur diese Farbe hervorheben"): die anderen Farben gedimmt ueber dem Stoff
(`kern.gedimmt`, Deckkraft max(20, a \ 5)) und ohne Symbol; Uebersicht und
Kacheln zeigen die echten Farben (`lwRgbEcht`). **Nicht sticken**
(`kern.farbeAuslassen`, skip_stitching) ist ein Rueckgaengig-Schritt --
PySticky setzt das Feld nur. **Willkommensseite** `ansicht/willkommen.dh`
(WelcomeWidget) in einem GtkStack an der Stelle der Zeichenflaeche: vier
Kacheln (Neu, Oeffnen, Aus Bild, Demo -- das Demo wie `_on_open_demo` als
neues Muster ohne Datei) und die zuletzt geoeffneten Dateien, die es noch
gibt. Sie kommt bei der neuen Startaktion "Willkommensseite", wenn die
letzte Datei fehlt (wie PySticky; vorher kam das Beispielmuster) oder eine
Datei im Aufruf nicht aufgeht, und ueber den Muster-Knopf (dann mit
"Zurueck zum Muster"). Ohne Muster sind Werkzeuge, Sichern, Export,
Ansicht gesperrt, und `befehlAusfuehren` meldet sich statt abzubrechen
(`ohneMusterErlaubt`). **Drei Fallen:** (1) die Seite liegt in einer
Rolle -- ein GtkStack nimmt die Mindestgroesse aller Kinder, vier Kacheln
nebeneinander hoben sonst eine gemerkte Fenstergroesse auf; (2)
`statusNachziehen` laeuft schon beim Aufbau der Werkzeugleiste, bevor es
die Statuszeile gibt (Gtk-CRITICAL); (3) wieder "alloc_needed": die
Farbleiste setzt ihre Groesse nur bei Aenderung. Die Fotoproben vor Stufe
16 verbergen die Farbleiste (`fbProbeAus`) -- sie zaehlen Farben im
Bereich der Zeichenflaeche, und die Leiste zeigt dieselben. Kuerzel-Tafel
56 Befehle. Sammlung `tests/pruef/glutfaden/glutfaden_stufe16.dhtest` (3: Kern, Probe
samt Rot im Foto, zwei Startlaeufe ohne Muster); 14 Verfaelschungen
fallen je in ihrem Fall. **Dazu:** `gtkFoto` ruft vor dem Zeichnen
`gtk_container_check_resize` auf das Fenster -- GTK vermisst im Takt der
Bildwiederholung, nicht beim Abholen der Ereignisse; ohne das fielen die
Proben von Stufe 10 und 13 unter voller Last in 1 von 3 bis 1 von 5
Laeufen mit Gtk-CRITICAL "alloc_needed", danach 0 von 8.

**Stufe 17: Info- und Fortschritts-Leiste, Symbol-Editor** (2026-10-17).
Die Seitenleiste hat unter der Uebersicht drei Reiter (GtkStackSwitcher,
`slStapel`, nicht gleich hoch): "Muster" (der bisherige Inhalt), "Info"
und "Fortschritt" (`ansicht/infoleiste.dh`, PyStickys InfoPanel und
ProgressPanel). Nachgezogen wird nur der sichtbare Reiter (`ilNachziehen`
aus `seitenleisteZahlen`/`-Muster` und beim Umschalten); Widgets entstehen
nur bei geaendertem Bestand, sonst werden Texte gesetzt (die Lehre aus
"alloc_needed"). **Info:** Stoffzaehlung als Klappliste (eine Zaehlung
ausserhalb 11..32, etwa 10 im Diamond-Modus, wird einsortiert -- PySticky
zeigte dann 11), neun Karten, Quellbild mit Tooltip, Farbliste mit Garn je
Farbe (anklicken waehlt). Die Texte rechnet der Kern (`kern.infoKarten`,
`stickzeitText$`, `garnbedarfText$`, `garnJeFarbeText$`,
`farbeInfoText$`) und sind **gegen PySticky aufgezeichnet**: drei Muster
je wie geladen, Diamond, sechs Stoffe, eine Farbe ausgelassen, eine
erledigt -- 90 Zeilen gleich. Falle dabei: PySticky rundet die Prozente je
Farbe erst auf eine Stelle und formatiert dann ohne Stelle (`round(p, 1)`
-> `:.0f`) -- `farbeProzentText$` nimmt darum das gerundete `farbeProzent`,
nicht die Zahlen. **Stoff aendern ist ein Rueckgaengig-Schritt**
(`kern.stoffSetzen`, neuer Verlaufsteil `TEIL_STOFF`); PySticky setzt das
Feld nur. **Fortschritt:** Sticken-Modus = das Fortschritts-Werkzeug
(Strg+M, Knopf; beenden kehrt zum Werkzeug davor zurueck), Gesamtbalken,
je Farbe mit Stichen Prozent, Balken und "Alle erledigt", und "Fortschritt
zuruecksetzen" (`kern.fortschrittZuruecksetzen`, alle Ebenen auch
unsichtbare und ausgelassene Farben) -- **ein Schritt**, PySticky leert
dafuer den Verlauf. **Symbol-Editor** (`symbolEditor` in
`ansicht/dialoge.dh` statt der Textfrage): Vorschau 90x90, eigenes Zeichen
(Leerraum zaehlt nicht), Auswahl aus `daten/symbole.txt` in einer FlowBox
mit zwoelf Spalten (ausgeblendete Kinder statt Filter-Rueckruf), Suche
nach Zeichen, Unicode-Name oder Codepunkt (`kern.symbolPasst`; die Namen
liegen in `daten/symbolnamen.txt`, einmal aus Pythons `unicodedata`
erzeugt -- die Laufzeit hat keine Unicode-Namen), gegen PySticky
aufgezeichnet (32 Anfragen gleich); ein Symbol einer anderen Farbe wird
abgewiesen und der Dialog bleibt offen. Kuerzel-Tafel 57 Befehle (Stufe 13
neu erzeugt). Sammlung `tests/pruef/glutfaden/glutfaden_stufe17.dhtest` (4: Info und
Suche gegen PySticky, Grenzen der Texte samt Schritten, Probe); 17
Verfaelschungen fallen je in ihrem Fall -- "fremde Zaehlung hinten
angehaengt" blieb im ersten Anlauf gruen, weil die Probe nur 25 ct
(einsortiert) pruefte; jetzt auch 40.

**Stufe 18: Muster drehen, spiegeln, zuschneiden; Ebenen vereinen und
ansehen** (2026-10-18). Im Muster-Knopf 90 Grad rechts/links, 180 Grad,
waagerecht/senkrecht spiegeln und Auto-Zuschneiden (Strg+Umschalt+C), in
der Ebenen-Leiste "Alle vereinen" und die Kaestchen "Nur aktive"/"Andere
abdunkeln". Kern `musterUmlegen`, `musterZuschneiden`, `autoZuschneiden`
(kern/verwalten.dh, ueber den vorhandenen Groessen-Teil des Verlaufs) und
`ebenenVereinen` (kern/bearbeiten.dh, Ebenen-Teil) -- **jedes ein
Rueckgaengig-Schritt**, PySticky leert dafuer den Verlauf. Gegen PySticky
aufgezeichnet (Groesse, je Ebene die belegten Zellen samt Typ und
erledigt, Rueckstiche, Stichzahlen; drei Muster, 36 Abzuege): gleich.
**Drei bewusste Abweichungen**, je mit Fall: (1) PyStickys `rotate_180`
laesst die Teilstich-Typen stehen -- ein Viertelstich oben links bliebe
"oben links", obwohl sein Kaestchen jetzt unten rechts liegt; hier ist 180
Grad zweimal rechts. (2) Auto-Zuschneiden misst ueber ALLE Ebenen, PySticky
nur ueber die sichtbaren -- ein Stich auf einer ausgeblendeten Ebene
ausserhalb ging dort still verloren. (3) Ein leeres Muster bleibt; PySticky
schnitt es auf 1 x 1 (`get_bounds` liefert fuer leer (0,0,0,0)). Vereinen
nimmt wie das Original nur die sichtbaren Ebenen; die Rueckfrage nennt, wie
viele Stiche dabei verloren gehen (`ausgeblendeteStiche`). Die Ansicht der
Ebenen ist nur Anzeige (`lwZeigt`/`lwDeckkraft`/`lwZusammensetzen` in
ansicht/leinwand.dh): "Nur aktive" zeigt die aktive auch ausgeblendet,
"Abdunkeln" halbiert die Deckkraft der anderen wie PySticky; ein Wechsel
der aktiven Ebene zieht die Leinwand nach. Kuerzel-Tafel 66, acht Befehle
ohne Vorgabe (wie in PySticky) -- die Stufe-13-Pruefung zaehlt sie jetzt
nicht mehr als "krumm". **Fund beim vollen Lauf:** die Farbverlauf-Probe
der Stufe 15 fiel unter voller Last (2 von 2, allein nie) -- ein Fenster,
das unter dem Zeiger auf- oder zugeht, schickt Bewegungen, und
`lwBewegung` setzte damit den laufenden Zug um. Waehrend einer Probe
uebergeht die Leinwand jetzt die echte Maus (`lwOhneMaus`), danach zwei
volle Laeufe gruen. Sammlung `tests/pruef/glutfaden/glutfaden_stufe18.dhtest` (3:
gegen PySticky, Grenzen samt Abweichungen und Schritten, Probe); 13
Verfaelschungen fallen je in ihrem Fall.

**Stufe 19: Optionsleisten, Bildschirm-Pipette, Paletten-Dateien, Drucken**
(2026-10-18). Unter der Werkzeugleiste erscheint je nach Werkzeug eine
Leiste (`ansicht/optionsleisten.dh`): Rueckstich mit Dicke, Linienart
(durchgezogen/gestrichelt/gepunktet), Enden (rund/eckig/flach), Einrasten
und Vorschau -- gemerkt als `backstitch_line_style`/`_cap_style` (geklemmt)
und von Leinwand, Vorschau und Druck gleich gezeichnet (`vbStrich`); Text
mit Eingabefeld, Schrift, Groesse, fett/kursiv -- der Text haengt beim
Tippen an der Maus und wird bei jeder Aenderung neu gesetzt, ohne Text
haengt nichts (vorher drei Fragekaesten). **Bildschirm-Pipette** (💧,
`ansicht/farbquellen.dh`): ein Foto des Monitors
(`gdk_pixbuf_get_from_window` auf das Wurzelfenster), randlos darueber, ein
Klick nimmt den Punkt, ESC bricht ab; die Farbe wird das naechste Garn der
gewaehlten Palette (`naechstesGarnIn`, mit Delta E in der Meldung), ohne
Palette ein Garn "#RRGGBB". Die Pipette dreht ein eigenes `gtk_main` --
**ein `destroy`-Handler, der `gtk_main_quit` ruft, beendete dabei die
AEUSSERE Schleife und damit das Programm**; jetzt mit Laufmerker
(`bpLaeuft`). **Paletten-Dateien** (Exportieren…/Importieren… unter der
Farbliste): `paletteDateiText$` schreibt Byte fuer Byte PyStickys
`pysticky_palette`-JSON (gegen dessen Ausgabe geprueft), der Import ist EIN
Rueckgaengig-Schritt, nimmt nur Farben, die noch fehlen, setzt ein Symbol
nur, wenn es frei ist, und lehnt fremde Dateien mit PyStickys Meldung ab.
**Drucken** (Strg+P, `ansicht/drucken.dh`): GTKs Druckdialog
(`GtkPrintOperation` + `draw-page`, neu `signalSeite`), eine Seite, quer
bei breitem Muster, 5 % Rand (`kern.druckLage`), Teilstiche/Knoten/Perlen
wie auf der Leinwand, Symbole ab 8 Punkten je Kaestchen, graues Raster --
**und anders als PySticky die Rueckstiche** (ohne sie fehlen die Konturen
auf dem Blatt). `druckenNach` schreibt dieselbe Seite ohne Dialog als PDF
(GTKs Export), so laesst sich der Druck ohne Drucker pruefen. **GTK-Falle:**
`gtk_widget_set_no_show_all` erst NACH `show_all` + `hide` der Leiste --
vorher blieben ihre Kinder fuer immer unsichtbar. Kuerzel-Tafel 70 (elf ohne
Vorgabe). Die Repo-Pruefung in `dhrt_check.dhtest` laesst
`examples/glutfaden/ansicht/` aus: das sind Teile von main.dh, kein eigenes
Programm. Sammlung `tests/pruef/glutfaden/glutfaden_stufe19.dhtest` (4: Export gegen
PySticky, Lesen samt Rundweg und Grenzen, Drucklage und Einstellungen,
Probe); 16 Verfaelschungen fallen je in ihrem Fall -- zwei erst, nachdem
die Probe die Lage des DRUCKAUFTRAGS (nicht die Rechnung des Kerns) und die
Kanalfolge der Pipette an einem gefuellten Pixbuf prueft; vorher verglich
sie `bpPunkt` mit sich selbst.

**Stufe 20: Ebene leeren und Notiz, Ansichtsschalter, Pfeile und
Rollbalken, Ablegen, F1** (2026-10-18). `kern.ebeneLeeren` (PySticky
`ClearLayerCommand`, ueber `zellenAufEbene`, ein Schritt; Rueckstiche gehoeren
dem Muster und bleiben) -- gegen PySticky aufgezeichnet (drei Muster, je
Ebene leeren und zurueck, die GEFUEHRTEN Stichzahlen ohne Neuzaehlen): gleich
bis auf eine bewusste Abweichung, eine schon leere Ebene ist hier kein
Schritt. Die Notiz der aktiven Ebene steht in einem Feld unter der
Deckkraft und wird ein Schritt (`TEIL_NOTIZ`), wenn man das Feld verlaesst,
die Ebene wechselt, einen Befehl ausloest oder die Leiste neu entsteht --
**aber nur, wenn im Feld etwas anderes steht als geladen**, sonst schriebe
ein Neuaufbau nach Strg+Z die alte Notiz gleich wieder hinein. **Fund der
Probe:** der Neuaufbau zerstoerte das Feld, bevor es gelesen wurde -- der
leere Text galt als geaendert und loeschte die Notiz (samt GTK-CRITICAL);
jetzt wird ganz am Anfang von `seitenleisteMuster` uebernommen, und ein
`destroy`-Rueckruf setzt den Zeiger auf 0. Werkzeugleiste: drei
Ansichtsschalter (Rueckstiche, Fortschritt, Zentrierhilfe; gezeichnete
Sinnbilder -- die Zeichen ↙ ✓ ✞ waren winzig) fuer die Sitzung wie in
PySticky; Rueckstiche hier per Vorgabe AN (PySticky: aus, bis man das
Werkzeug waehlt), das Rueckstich- bzw. Fortschritt-Werkzeug schaltet die
Anzeige wieder ein. Spiegelachsen (blau, orange, bei Achtfach die
Diagonalen, der Modus in der Mitte) erscheinen, solange gespiegelt wird;
Achtfach gilt nur bei quadratischen Mustern (`kern.spiegelWirksam`).
Pfeiltasten schieben das Muster um 20 Punkte, aber nur mit dem Fokus auf
der Flaeche (oder nirgends) und nach den umlegbaren Kuerzeln. Rollbalken
rechts und unten (`kern.rollBereich` nach PySticky `_update_scrollbars`,
nur waechst der Bereich mit, wenn das Muster weiter verschoben ist -- Qt
klemmte still, und der erste Griff liess es springen). Dateien aufs
Fenster ziehen (`ansicht/ablage.dh`, text/uri-list, `g_filename_from_uri`):
Muster oeffnen nach der Rueckfrage, Bilder in den Bild-Import; abgearbeitet
einen Augenblick SPAETER, nicht im Rueckruf -- eine Rueckfrage mitten in
der Ziehen-Schleife des Systems. F1 zeigt alle 75 Kuerzel mit dem
WIRKSAMEN Kuerzel (`kern.kuerzelListe`, dieselbe Tafel wie der
Einstellungsreiter). Die Probe vergleicht die Leinwand als Bild mit sich
selbst (`v20Bild` + Punktvergleich) statt fester Pixel. **Formatgrenze:**
eine `--- datei`-Beilage darf kein Leerzeichen im Namen tragen -- die Datei
"mit leer.pxs" legt der Fall selbst an. Sammlung
`tests/pruef/glutfaden/glutfaden_stufe20.dhtest` (4); Stufe 13 mit 75 Kuerzeln. Von 19
Verfaelschungen fallen 18; die neunzehnte (Sperre in `ebeneLeeren` weg)
faengt `zellenAufEbene` mit derselben Pruefung ab. Eine fiel erst nach
einem Nachtrag: ohne das Uebernehmen vor dem Abbau ging getippter Text bei
einem Neuaufbau verloren, der nicht ueber `befehlAusfuehren` laeuft
(Rueckgaengig in der Kopfleiste ruft `werkzeugZurueck` direkt).

**Stufe 21: Muster importieren, weitere Bildformate, Zoom-Regler, helles
Thema** (2026-10-18). **Muster importieren (XSD/PAT/OXS)** (Strg+Alt+I,
Muster-Knopf; `ansicht/fremdimport.dh`, PySticky `PatternImportDialog`):
Datei, Vorschau (das zusammengesetzte Bild als Pixel, skaliert ohne
Glaetten), Name, Groesse, Farben, Stiche ueber alle Ebenen, Format und die
Warnungen des Lesers, BEVOR das Muster uebernommen wird; ein Fehler sperrt
"Importieren". Gelesen ueber `kern.fremdLaden` -- dieselbe Stelle nimmt
jetzt auch `musterOeffnen`. Gegen PySticky nur an OXS verglichen (drei
Muster aus Glutfadens Schreiber, zwei kaputte): gleich; **PyStickys PAT- und
XSD-Leser lesen keine echte Datei** (Stufe 6, sie scheitern auch an den
Stufe-6-Dateien), dort gelten die Stufe-6-Pruefungen. **Bildformate:** was
`LOADIMAGE` nicht liest, geht ueber GTKs Bildlader (`gdk_pixbuf_new_from_file`
-> PNG in eine `TEMPFILE$` -> `LOADIMAGE`); angeboten werden PyStickys WebP,
TIFF, AVIF (dazu HEIC/HEIF, JPEG XL), **aber nur, was der Lader auf dieser
Installation kennt** -- gefragt ueber `gdk_pixbuf_get_formats`. Gemessen: die
GTK aus Inkscape liest TIFF, AVIF, HEIC, JXL, **aber kein WebP**. Der Umweg
ist an deckenden Punkten exakt; halbdurchsichtige weichen ab (bei Deckkraft
2 traegt die Farbe kaum Stellen), fuer den Import ohne Belang. **Zoom-Regler**
in der Statusleiste (PySticky `ZoomSlider`): Bereich aus den Zellgroessen
der Einstellungen (`kern.zoomBereich` -- das, was PySticky mit
`set_zoom_range` nachtragen musste), ein Klick auf die Prozent = 100 %.
**Helles Erscheinungsbild** (Einstellung `theme` dark/light wie PySticky,
Allgemein -> Erscheinungsbild, sofort): das Stylesheet benutzt Namen
(`@gf_...`, GTKs `@define-color`), die Werte stehen EINMAL in
`kern.themaTafel` (27 Rollen, hell nach PyStickys LIGHT_THEME); die um die
Flaeche gezeichneten Teile fragen `uiFarbe`/`uiSymbol`. `gtkStil$` laedt
beim Wechsel denselben Anbieter neu, statt einen weiteren obenauf zu legen.
Die Leinwand behaelt ihre eigene Farbe (`canvas_bg`, auch in PySticky
unabhaengig vom Thema). **Proben, die an der Installation haengen, sind
vermieden:** die Ablage-Meldung nennt die gefundenen Formate, die Stufe-20-
Probe prueft nur ihren Anfang; die Stufe-21-Probe benutzt TIFF, einen
Standardlader von GdkPixbuf. Kuerzel-Tafel 76. Sammlung
`tests/pruef/glutfaden/glutfaden_stufe21.dhtest` (3); Stufen 13 und 20 nachgezogen; 18
Verfaelschungen fallen je in ihrem Fall.

**Stufe 22: Kleinigkeiten aus PySticky** (2026-10-18). **Laufende
Auswahlkante** (`marching_ants`, Vorgabe an): die Kante wird zweimal
gestrichelt (8 an, 8 aus) in Glut und Dunkel, gegeneinander versetzt, und
rueckt alle 120 ms um zwei Punkte weiter (`mrAmeisen*` in `ansicht/mehr.dh`)
-- der Takt laeuft nur, solange eine Auswahl da ist und ein Auswahlwerkzeug
gewaehlt; er beendet sich selbst. **Aida-Textur** (`fabric_texture`): ein
Punkt (180,170,150, Deckkraft 70) in jeder leeren Zelle ab 8 Pixeln, nicht
beim Diamond Painting, EIN Pfad fuer den ganzen Ausschnitt
(`lwStoffTextur`). **Ziehen** (`ansicht/ziehen.dh`): ein Garn aus dem Katalog
auf die Farbleiste nimmt es ins Muster; eine Ebene in der Liste wird
verschoben oder in der mittleren Haelfte einer anderen Zeile mit ihr
zusammengefuehrt (`kern.ebenenZug` = PyStickys dragMoveEvent/dropEvent,
`kern.ebenenZusammenfuehren` = `merge_layers`; gesperrte Quelle ODER Ziel
abgelehnt wie PyStickys Liste, ohne PyStickys Rueckfrage -- Strg+Z nimmt es
zurueck). **Zwei GTK-Fallen:** die Zeilen einer GtkListBox haben in GTK 3 kein
eigenes Fenster und bekaemen den Druck fuer ein Ziehen nie -- jede Zeile hat
darum einen unsichtbaren `GtkEventBox` als Quelle (`ziehQuelle`); und
ausgefuehrt wird erst nach dem Ziehen (`zhVormerken`, 30 ms), weil
`allesNeu` die Liste mitten im Ziehen des Systems zerstoeren wuerde.
Getragen wird ein Text (`kern.ziehText$`/`ziehLesen`); fremder Text auf der
Farbleiste geht als Datei an die Ablage weiter. **Einstellungen**
`library_path`/`templates_path` (ein Ordner, der nicht geht, faellt auf die
Vorgabe zurueck, `kern.eigenerOrdner$`), `highlight_selected` (**Vorgabe
AUS**, PySticky: an -- dort daempft dann jede Farbwahl alle anderen Farben,
was ohne die Einstellung zu kennen wie ein Fehler aussieht) und
`stitch_timer_enabled`. **"Neu in Glutfaden"** am Logo-Knopf (der jetzt ein
Menue zeigt: Ueber, Neu, Tastenkuerzel), Inhalt in `daten/neuigkeiten.txt`
(`kern.neuigkeitenLesen`). Sammlung `tests/pruef/glutfaden/glutfaden_stufe22.dhtest` (3,
der erste gegen PySticky: 23 Zuege auf zwei Mustern, Entscheidung und
Ausfuehrung gleich); Stufe 13 nachgezogen. 24 von 25 Verfaelschungen fallen;
die 25. (`eigenerOrdner$` nimmt eine Datei als Ordner) ist gleichwertig, weil
`MKDIR` auf eine vorhandene Datei schon vorher wirft. Eine fiel erst nach
einem Nachtrag: die Kante mit nur EINER Farbe -- die Probe "glut und dunkel
vorhanden" traf dunkle Stiche unter den Luecken; jetzt muss jeder Punkt der
Kante vom Bild ohne Auswahl abweichen.

**Stufe 23: magnetisches Raster, Pinsel nach Stiftdruck, Pinch-Zoom**
(2026-10-18). **Einrasten** (`snap_enabled`/`snap_interval`, Vorgabe aus/5,
Knopf mit gezeichnetem Magneten neben der Symmetrie, Befehl `einrasten` ohne
Vorgabe-Kuerzel; Kuerzel-Tafel 77): Linie, Rechteck, Ellipse und Vieleck
setzen Anfang, Zwischenstand und Ende auf das naechste Vielfache
(`kern.einrasten`, ins Muster geklemmt); beim Vieleck auch die Linie zur Maus
und das Schliessen rechts. **Gerundet wird wie Pythons round** (bei .5 zur
geraden Zahl, `kern.rundenGerade`) -- bei geradem Intervall ergaebe ROUND
andere Punkte (1/2 -> 0, nicht 1). **Stiftdruck** (`tablet/pressure_enabled`,
`tablet/max_brush_size`): `lwDruckLesen` fragt `gdk_event_get_axis(...,
GDK_AXIS_PRESSURE)` -- eine Maus hat die Achse nicht und bleibt bei einem
Stich; mit Stift wird jeder Punkt des Strichs zum Kreis mit Radius
Druck * (Groesse - 1) (`kern.pinselZellen` = PyStickys `_brush_cells`), vor
der Symmetrie, nur beim Stift-Werkzeug. **Falle:** ein kuenstliches
GdkEvent ohne Geraet laesst `gdk_event_get_axis` in ein Gdk-CRITICAL laufen
-- die Probe setzt den Zeiger des Sitzes (`gdk_seat_get_pointer`).
**Pinch** (`touch/gestures_enabled`, Vorgabe AUS wie PySticky):
`GtkGestureZoom` auf der Flaeche, aus = Phase NONE; ab Faktor 1,15 gegen den
letzten Schritt ein Zoomschritt (`kern.pinchSchritt`). Ohne Geraet nicht
auszuloesen: die Probe setzt den Zustand, den Stift bzw. Geste hinterlassen.
**Der Fund dieser Stufe:** der Magnet-Knopf machte die Werkzeugleiste
breiter als eine gemerkte Fenstergroesse von 1150 -- GTK machte das Fenster
1166 breit (Stufe 13 sah es). Die Leiste liegt jetzt in einem waagerecht
rollbaren Bereich mit natuerlicher Breite; sie waechst mit jeder Stufe und
darf die Fenstergroesse nicht mehr bestimmen. Und: `daten/neuigkeiten.txt`
waechst mit jeder Stufe -- die Stufe-22-Proben suchen ihren eigenen
Abschnitt, statt Zahl und Reihenfolge festzuhalten. Sammlung
`tests/pruef/glutfaden/glutfaden_stufe23.dhtest` (3, der erste ruft PyStickys
`snap_position`, `_brush_cells` und `_handle_gesture` mit Stellvertretern auf:
gleich); Stufen 13 und 20 nachgezogen; 20 von 20 Verfaelschungen fallen.

**Stufe 24: Deutsch und Englisch, Teil 1** (2026-10-18, Wunsch des
Nutzers; Teil 2 = alle Dialoge, Teil 3 = Texte aus dem Kern: Verlauf,
PDF/HTML/Druck, Meldungen, "Ebene N"). `kern/sprache.dh`: jeder Anzeigetext
geht durch `tr$("deutsch")` (auf Deutsch unveraendert, auf Englisch aus
`daten/sprache/en.txt`, je Zeile Deutsch TAB Englisch -- KEIN JSON, das
json-Modul liest einen Punkt im Schluessel als Pfad), Texte mit Werten durch
`trf$("... {0} ...", [w])`: erst uebersetzen, dann einsetzen (von hinten,
ein Wert wird nicht noch einmal gelesen). PySticky schrieb `t(f"...")` und
hat solche Texte darum nie uebersetzt. Einstellung `ui_language`
(auto/de/en, `einst.sprache`, im Dialog unter Allgemein), gilt ab dem
naechsten Start; "auto" ueber `gtk_get_default_language` (beginnt mit "de"
-> Deutsch, sonst Englisch). **Eine Probe (`--probe-...`) bleibt deutsch**,
ihre Erwartungen stehen deutsch da; `DH_GLUT_SPRACHE` ueberstimmt beides.
Kuerzel bleiben deutsch GESPEICHERT und werden englisch ANGEZEIGT
(`kuerzelAnzeige$`: Strg -> Ctrl, Umschalt -> Shift, Entf -> Del ...),
Zahlen ueber `zahlText$`/`zahlAnzeige$` (ZAHL$ schreibt immer deutsch, auf
Englisch werden Punkt und Komma getauscht). Eingepackt hat ein Werkzeug
(Ketten mit `+` zu `trf$`, ausgenommen Vergleiche, PRINT, CASE-Marken,
Schrift- und Symbolnamen), danach von Hand nachgesehen. **Die Fallen:**
(1) ein Anzeigetext, der als SCHLUESSEL dient (`mrEinfuegenText` verglich
mit "Einfuegen"), muss auf beiden Seiten eingepackt sein; (2)
zusammengesetztes Deutsch ("Vor dem {0} sichern?", "wird {1}gestickt")
laesst sich nicht uebersetzen -- ganze Saetze je Fall
(`ungesichertFrage$`); (3) Schriftnamen wurden mit eingepackt.
`daten/neuigkeiten.en.txt` fuer Englisch. Sammlung
`tests/pruef/glutfaden/glutfaden_sprache.dhtest` (3): der erste liest den QUELLTEXT
(jedes Literal in tr$/trf$ und jeder Name der Kuerzel-Tafel steht in der
Tafel, gleiche Platzhalter, keine verwaisten oder doppelten Eintraege, kein
tr$ mit {0}, kein trf$ ohne) -- ein fehlender Eintrag ist sonst unsichtbar,
der Text bleibt einfach deutsch; die Probe sammelt alle Texte des
Hauptfensters und der F1-Uebersicht und listet deutsch aussehende (0 von
631 bzw. 143). Stufe 13 nachgezogen; 10 von 10 Verfaelschungen fallen --
eine davon (Wiederholen-Tooltip) nur im Quelltext-Fall, weil das Programm
den Tooltip der Verlaufsknoepfe ohnehin ueberschreibt.

**Stufe 25: Englisch in jedem Dialog, Teile 2 und 3** (2026-10-18): alle
28 Dialoge, dazu die Texte aus dem Kern -- Verlauf (`v.beginnen(tr$(...))`),
PDF/HTML/CSV/Stickplan/Paket, Meldungen der PAT/XSD/OXS/PXS-Leser,
Ebenen- und Mustervorgaben ("Ebene {0}", "Hintergrund"). Tafel 1637
Eintraege. **Die Regel: Schluessel bleiben deutsch, uebersetzt wird beim
ANZEIGEN.** Listen aus dem Kern (Bibliothek-Kategorien samt "Alle"/
"Favoriten", Sortierungen, Harmonie-Arten, Vorlagen-Kategorien und -Namen,
Startaktionen, Pipetten-Arten, Rastervoreinstellungen ...) sind dort
Schluessel -- gespeichert, verglichen (`CASE "Groesse"`), in `library.json`
geschrieben; die Ansicht zeigt sie ueber `tr$(t)` bzw. `trListe(liste)`
(einstellungsstand.dh). Ebenso nie uebersetzt: PDF-Stilnamen (`pdfAbsatz(s,
"Title1", ...)`), Zaehlerarten, OXS-Elemente (`oxsText$` ganz ausgenommen),
Palettennamen ("DMC Diamond Painting", "Anchor"), Dateikennungen
(`" Floss Palette"` im PAT-Leser), `ZEIT_FORMAT$`-Muster, CSS. `<html
lang>` folgt der Sprache. **Was das Werkzeug falsch machte** (und jetzt
nicht mehr): (1) ein Text in einem IIF mitten in einer Kette ergab zwei
ueberlappende Ersetzungen (jetzt verschachtelt: die innere steht im Wert der
aeusseren); (2) `"<span>" + CHR$(&H2003) * 3 + "</span>"` -- die Kette brach
vor dem `*` ab, `trf$(...) * 3` haette den GANZEN Text verdreifacht (staerker
bindende Operatoren an der Kette = nicht anfassen); (3) reines Markup/CSS
wurde eingepackt (`markup_zurueck.py` macht es rueckgaengig). **Ein Fund im
Bild:** "Max. stitches: 2.500" -- auch Dialoge und Kern gehen jetzt ueber
`zahlText$`/`zahlAnzeige$`. Und `INSTR(bew, "Beste")` im Stickpfad-Dialog
verglich mit einem uebersetzten Text (jetzt `"★★★"`). **Die Probe**
(`--probe-sprache`) oeffnet jeden Dialog ueber `g_timeout_add` +
`gtk_grab_get_current` (sammelt, fotografiert, schliesst mit -4), liest auch
Reiterbeschriftungen, waehlt beim Neuen Projekt eine Vorlagen-Kategorie und
erkennt Deutsches an Umlauten, Woertern, Tausenderpunkten UND an Saetzen der
Tafel, die woertlich stehen geblieben sind (Ebenen-, Muster- und Dateinamen
sind Daten und fallen heraus). Weil ein uebersetzter SCHLUESSEL auf Deutsch
nie auffaellt, exportiert sie auf Englisch: PDF-Stile, `lang='en'`, CSV-Kopf,
und die OXS-Datei muss in beiden Sprachen Byte fuer Byte gleich sein. Der
Quelltext-Fall prueft zusaetzlich jede Kern-Liste gegen die Tafel. Auf
Deutsch blieben alle 25 Glutfaden-Sammlungen unveraendert gruen; 20 von 20
Verfaelschungen fallen.
