# Kühlschrank-App im Drachenhauch Studio

> Dogfooding-Runde für das Studio: die PySide6-App „Kühlschrank“ des Nutzers
> (`C:\Programmieren\Python\Kühlschrank`, rund 29 000 Zeilen, SQLite
> `fridge_database.db`) wird als Studio-Projekt `examples/studio/kuehlschrank/`
> nachgebaut. Die Python-App selbst bleibt unverändert. Gültig ist, was im Code
> steht und was CLAUDE.md sagt.

## Bestandsaufnahme (2026-11-08)

Oberfläche rund 17 900 Zeilen (Reiter Bestand, Lagerorte, Einkauf, Statistik,
Rezepte; Dialoge Stapelscan, Übernahme in den Bestand, Kochen, Einstellungen
mit sechs Reitern), Logik rund 11 400 (Datenbankschicht, Open Food Facts,
Spoonacular, Lernfunktion, SmartDefaults, Meldungen, Rückgängig). Die Datei hat
17 eigene Tabellen und `sqlite_sequence`. Fehler der Python-App, die der Port
nicht übernimmt: Rückgängig auf Spalten, die es nicht gibt
(`inventory.opened`, `products.image_path`), der Nachfüll-Dienst schreibt
`shopping_list.name`, Stapelscan aus dem Einkaufsreiter mit vertauschten
Argumenten, doppelte Tastenkürzel, `store_layouts` in der Erstanlage mit
anderem Aufbau als im Code, drei Einstellungen ohne Wirkung. Der Stufenplan
(0 Gerüst + Datenbank, 1 Vorrat, 2 Lagerorte, 3 Einkauf, 4 Rezepte,
5 Statistik, 6 Scannen/HTTP, 7 Einstellungen/Meldungen, 8 Abschluss) steht in
den Notizen der Sitzung.

Das Projekt entsteht über das Modell des Studios (`projektNeu`,
`codeZerlegen`, `projektSchreiben`, Layouts aus echten gui-Aufrufen über
`GUI_TO_JSON`) -- den Zugriff aufs Studio-Fenster per Maus gab es nicht. Jede
Datei steht so da, wie Strg+S im Studio sie schreibt (Rundweg-Fall), und jede
Stufe läuft mit F5 aus dem Studio.

## Stufe 0: Gerüst und Datenbank (2026-11-08)

Elemente: App (Thema `modern_dark`, Akzent im Teal der Python-App), Datenbank
`Daten` (Aufbau aus `fridge_database.db` übernommen, Datei gleichen Namens --
die Drachenhauch-Fassung liest dieselbe Datei), Module `Datenbank` (`db`,
`dbZahl`, `heute$` mit `KUEHL_HEUTE` für Prüfungen, `einstellung$` liest das
JSON in `app_settings`) und `Startdaten` (neun Lagerorte mit Zone aus der
Temperatur, 18 Kategorien, 20 Einheiten, 13 Einstellungen -- nur in eine
leere Datei, `INSERT OR IGNORE`), Klasse `Kennzahlen` (wie `get_statistics`:
bald = heute bis heute + Warnschwelle), Hauptfenster (Werkzeugleiste,
Kennzahl-Kacheln, fünf Reiter, Statusleiste mit Uhr, Timer `auffrischen`
30 s, Menüs mit Strg+N/Z/E/Q, F5, Strg+,). Was erst später kommt, ist
gesperrt statt stumm.

**Funde im Studio und in der Laufzeit** (je ein Commit mit Fällen und
Gegenproben):

1. **Keine vorhandene SQLite-Datei als Datenbank-Element.** Neu Projekt →
   Datenbank aus Datei übernehmen (`dbSchemaAusDatei$`): Arten, Pflicht,
   Vorgaben, Eindeutig, Verweise (FOREIGN KEY). Was keine Art trifft
   (`DATE`, `TIMESTAMP`, `DEFAULT CURRENT_TIMESTAMP`, Pflicht ohne Vorgabe),
   steht wörtlich als `sql`/`sqlvorgabe` im Schema -- sonst hätte der Abgleich
   jede Tabelle neu aufgebaut und dabei AUTOINCREMENT und FOREIGN KEY
   verloren. Dabei gefunden: der Neuaufbau schrieb bei einer Pflichtspalte
   ohne Vorgabe `COALESCE(x, )`; `ADD COLUMN` nimmt kein `CURRENT_TIMESTAMP`
   (dann baut der Abgleich neu auf, das gebaute Programm ergänzt ohne);
   Eindeutig legte einen zweiten Index neben ein UNIQUE der Datei; der
   Daten-Editor las über `id` und scheiterte an `app_settings` (jetzt
   `rowid`). Tests `werkzeug_studio_uebernehmen.dhtest` (4, der Aufbau der
   Kühlschrank-App), ein Fall in `werkzeug_studio_datenbank.dhtest`.
2. **Werkzeug- und Statusleiste ohne Inhalt und ohne Ereignis.** Inspektor
   Knöpfe (`Text|Sinnbild|Tipp; -; ~`), Mit Text, Felder
   (`Text|Breite|Ausrichtung|klick`); Ereignisse `Knopf(nr)` und
   `Feld(nr)`. Tests `werkzeug_studio_leisten.dhtest` (3).
3. **Die Menüleiste nahm dem Fenster 26 Punkte.** Sie schob den Inhalt nach
   unten, ohne das Fenster größer zu machen: die Statusleiste am unteren Rand
   fiel im Programm heraus, und im Entwurf lagen Rahmen und gemeldete Lagen
   26 Punkte über den Controls. Jetzt ist `h` im Layout die Höhe des Inhalts,
   die Leiste kommt obendrauf (`menueLeisteH`, im Entwurf `kopfH`, im
   Programm Fenster und Schirm `h + 26`).
4. **Ein großes Fenster lag über Inspektor und Ausgabe.** Neu in der Laufzeit
   `GUI_WINDOW_CLIP` (nur Entwurfsmodus, über `sicht_rect` gilt der Zuschnitt
   für Zeichnen, `GUI_HIT_TEST` und `GUI_WINDOW_AT`; mit Zoom erst beim
   Übertragen auf den Schirm, `zoom_beenden` mit `clip`). Das Studio schneidet
   den Entwurf auf die Mitte zu, nimmt Klicks nur dort an und öffnet ein
   Fenster, das bei 100 % nicht passt, bei 75 oder 50 % (`zoomEinpassen`).
   Tests drei Fälle in `gui_fenster_zoom.dhtest`, drei in
   `werkzeug_studio_menueleiste.dhtest` (mit Fund 3).
5. **Reiterköpfe zu schmal.** Die Breite wurde an der Zeichenzahl geschätzt
   (8 je Zeichen); bei „Lagerorte“ lief der Text in den nächsten Kopf. Jetzt
   misst `leisten_pass` die Beschriftungen je Bild wie bei der
   Werkzeugleiste, `tc_koepfe` bleibt die eine Quelle; neu
   `GUI_TABCONTROL_ITEM_RECT`, über den das Studio seine Reiter protokolliert
   (vorher rechnete es die Schätzung nach -- eine zweite Quelle). Gemessen ist
   ein neuer Kopf erst einige Bilder später; das Studio schreibt die Lagen
   darum je Bild, sobald sie sich ändern (vom Kreuz-Fall in
   `werkzeug_studio2.dhtest` gefunden). Ein Fall in `gui_reiterwerk.dhtest`.

Gegenproben: 14 (Übernahme), 7 (Leisten), 5 (Menüleiste, Einpassen,
Reiterprotokoll) Verfälschungen im Studio fallen je in ihrem Fall; in der
Laufzeit fallen die geschätzten Köpfe, das Zeichnen ohne Zuschnitt und der
Treffertest ohne Zuschnitt. Nur indirekt geprüft: dass das Studio Klicks
außerhalb der Mitte nicht mehr dem Entwurf gibt (`imFenster`), und der Aufruf
von `GUI_WINDOW_CLIP` im Studio selbst -- bei eingepasstem Zoom liegt das
Fenster ohnehin in der Mitte.

Nicht vom Port: der volle Lauf `dhrt test tests/pruef` meldet in diesem
Worktree `beispiel_bastelzentrale` (Bau ohne `--hardware`) und
`werkzeug_paket` (sucht dhrt im üblichen `target/`, gebaut wurde nach
`CARGO_TARGET_DIR`); im Hauptordner ist `werkzeug_paket` grün.

**Fallen:** `GUI_STATUSBAR` bringt sein erstes Feld selbst mit -- wer danach
fünf Felder anhängt, hat sechs. `dhrt --check` meldet „keine Befunde“ als
`[]`. Die Einheiten der echten Datei tragen das Sinnbild der Klappliste
(`📦 Stück`) -- der Vorrat muss es beim Anzeigen abnehmen (Stufe 1).

Tests `tests/pruef/kuehlschrank/stufe0_geruest.dhtest` (3: Rundweg, F5 auf
neuer Datei mit Startdaten und Kennzahlen, F5 auf einer Datei im Aufbau der
Python-App -- sie bleibt Zeichen für Zeichen gleich); acht Verfälschungen des
Beispiels fallen je in ihrem Fall. Hilfen in `tests/pruef/_hilfen/kuehlschrank.dh`
(`kKopieren`, `kF5`, `kProbe`, `kCheck$`, `kAufbau$`).

## Stufe 1: der Vorrat (2026-11-08)

Der Reiter Bestand ist das eigene Control `BestandSeite` (Barcode-Feld,
Hinzufügen, Export, Suche, fünf Filter, Tabelle mit −1/+1 je Zeile, Farben
für Status, Mindestmenge und Markt wie in der Python-App, Kontextmenü
Bearbeiten/Mindestmenge/Verbraucht/Abgelaufen/Löschen); es meldet
`Geaendert`, `Bearbeiten`, `Export`, `Mindestmenge` ans Hauptfenster. Dazu
die Fenster `ProduktFenster` (anlegen und bearbeiten in einem, Prüfung wie
InputValidator, +3/+7/+14/+30 Tage) und `MindestmengeFenster`, das Modul
`Bestand` (lesen mit den Tagen bis zum Ablauf aus SQLite, entnehmen mit
einer Zeile in `consumption_history`, Einheiten ohne das Sinnbild, das die
Python-App manchmal mitschrieb, CSV mit Semikolon) und die Klassen `Eintrag`,
`Verlauf`, `Schritt`, `Aenderung`: Rückgängig und Wiederholen über den
Stand jeder betroffenen Zeile vorher und nachher -- mit derselben id und
per UPDATE (ein INSERT OR REPLACE auf `products` nähme über ON DELETE
CASCADE den Bestand mit; SQLite kaskadiert auch beim REPLACE).

**Funde:**

6. **Ein eigenes Control auf einer Reiterseite zeigte sich auf jeder.** Die
   Teile einer Instanz standen in keinem `tabctl.kinder`; `seitenDerTeile`
   trägt sie auf der Seite der Instanz ein (Entwurf und Programm).
7. **Kein Klick in eine Zelle, kein Haken je Bild.** Die Tabelle meldete nur
   Geaendert und Doppelklick -- −1 in der schon gewählten Zeile kam nirgends
   an --, und ein Kontextmenü ließ sich im Code bauen, aber nirgends
   abfragen. Neu `Zelle(zeile, spalte)` und `Takt` für Fenster und Controls.
   Einen Kontextmenü-Editor im Studio gibt es noch nicht (siehe offen).
8. **Ein gemeldetes Ereignis mit modalem Fenster lief ohne Ende.** Die
   Meldung eines Controls blieb bis zum Ende des Fenstertakts stehen; die
   Schleife des modalen Produktfensters rief die Methode wieder auf, bis die
   Aufruftiefe überlief. `studioAbholen` nimmt sie vor dem Aufruf weg.
9. **Zweimal schnell „+1“ öffnete den Bearbeiten-Dialog.** Ein Doppelklick
   auf eine Zelle der Art `knopf` oder `haken` zählte als Doppelklick der
   Zeile; jetzt sind es zwei Klicks auf den Knopf (gui).

Dabei: das Studio schreibt Befunde der Prüfung mit Text ins Protokoll -- ein
Name `frage` neben der Laufzeit-Funktion `Frage` hielt F5 an, und das
Protokoll sagte nur „geprueft 1 0“.

**Offen:** ein kompaktes Datumsfeld zum Aufklappen (wie QDateEdit) hat die
gui nicht -- Kauf- und Ablaufdatum sind Textfelder mit `GUI_RULE "datum"`
und Schnellknöpfen; der Stapelscan (Stufe 6) braucht Datum je Tabellenzeile.
Ein Kontextmenü entsteht im Code (`GUI_CONTEXT` in `Geoeffnet`, Abfrage in
`Takt`), nicht im Menü-Editor des Studios.

Tests `tests/pruef/kuehlschrank/stufe1_vorrat.dhtest` (4: Tabelle mit
Status, Filtern und Suche; ändern, entnehmen, anlegen über den Dialog,
bearbeiten, je mit Rückgängig und Wiederholen, CSV; echte Klicks auf −1/+1
mit Strg+Z/Strg+Y; Rückgängig in einer Datei mit Fremdschlüssel), je ein
Fall in `werkzeug_studio_controls.dhtest` (Reiterseite, modales Fenster),
`werkzeug_studio_takt.dhtest` und `gui_tabelle_bedienung.dhtest`.
Gegenproben: zehn im Beispiel, zwei für die Reiterseite, vier für
Takt/Zelle, eine für das Abholen, eine für den Doppelklick in der Laufzeit
-- jede fällt in ihrem Fall.

## Stufe 2: die Lagerorte (2026-11-08)

Der Reiter Lagerorte ist das eigene Control `LagerorteSeite`: je Ort eine
Kachel auf einer Zeichenfläche (Glas in der Farbe des Orts, Sinnbild, Name,
Temperaturzone, Anzahl, was abgelaufen ist und was bald abläuft), so viele
Spalten, wie in die Breite passen, das Mausrad rollt. Farbe und Sinnbild
sucht das Modul `Lagerorte` wie LocationSettingsManager: `location_settings`
mit gleichem Namen, dann ein Stichwort im Namen (oder der Name im
Stichwort), dann die eingebauten Stichwörter in ihrer Reihenfolge
(„Kühlschrank“ ist `kühl`, nicht `schrank`), sonst Stahlblau mit Paket --
`storage_locations.icon` nimmt die Python-App dafür nicht. Anders als dort
zählt die Kachel Abgelaufenes getrennt (die Python-App zählte es zu „laufen
bald ab“). Ein Klick auf eine Kachel öffnet `LagerortFenster` (wie
LocationDetailsDialog): je Eintrag eine Zeile mit „Entnehmen“, das ein Stück
herausnimmt, beim letzten als verbraucht -- die Python-App zeigte jedes Stück
als eigene Zeile, bei 500 g wären das fünfhundert. Rechts neben dem Bestand
steht die Liste der Orte und ihrer Bereiche (wie locations_sidebar.py); eine
Zeile der Tabelle dorthin gezogen verlegt den Eintrag, ein Schritt für
Strg+Z, und wie dort setzt das Ziel auch den Bereich.

**Funde:**

10. **Eine Tabellenzeile ließ sich ziehen, trug aber nichts.** Aus einer
    Tabelle kamen `GUI_DRAG_INDEX` -1 und kein `GUI_DROP_TEXT`; jetzt die
    Datenzeile und der Text ihrer ersten Zelle, wie bei Liste und Baum (gui).
    Im Studio heißen Quelle und Ablage im Inspektor „Ziehbar“ und „Ablage“,
    das Ereignis `Abgelegt(zeile, ziel)`.
11. **Eine Zeichenfläche auf einer verdeckten Reiterseite malte über die
    vordere.** `GUI_VISIBLE` bleibt dort wahr; neu `GUI_SHOWN(wdg)`, das
    Fenster, Fensterreiter und jedes Reiterwerk darüber mitzählt (gui), und
    das erzeugte `Zeichnen` fragt es.
12. **Text auf einer Zeichenfläche stand in der Pixelschrift**, die gui
    daneben in Sans. Das Studio setzt um `Zeichnen` die Schrift der
    Oberfläche (`GUI_FONT` oder die eingebaute Sans) und danach die des
    Programms zurück.
13. **Ein Klick auf die Kachel öffnete das Detailfenster ohne Ende.**
    `ZeigenModal` drehte sein erstes Bild noch im Bild des Klicks; dort ist
    `MOUSE_HIT` noch wahr, das Maus-Ereignis der Zeichenfläche feuerte wieder
    -- bis die Aufruftiefe überlief (Verwandter von Fund 8). Eine innere
    Schleife (`ZeigenModal`, `Frage`) zeichnet jetzt erst das laufende Bild
    fertig (`studioMalen`) und verteilt dann neue Eingaben.
14. **`GUI_HIT_TEST` sieht durch den Schleier.** Ein Klick auf die
    Zeichenfläche neben dem modalen Fenster hätte ein zweites geöffnet;
    Maus, Rad und Ziehen einer Fläche fragen `studioFrei(fenster)`.
15. **`Taste` erreichte ein modales Fenster nie** -- die Weitergabe galt nur
    ohne `GUI_MODAL()`, ESC im Mindestmengen-Dialog tat seit Stufe 1 nichts.
    Jetzt bekommt das oberste Fenster die Taste, wenn es nicht hinter einem
    Dialog oder Schleier steht.
16. **Standard- und Abbrechen-Knopf ließen sich nicht einstellen.** Die gui
    kennt sie (`GUI_WINDOW_DEFAULT`/`CANCEL`, auch in der `.dhform`), der
    Inspektor nicht. Neu „Standard-Knopf (Enter)“ und „Abbrechen-Knopf
    (Esc)“ beim Fenster, gemerkt beim Namen (Umbenennen zieht mit; ein
    Index wanderte beim Löschen eines Widgets davor).
17. **Ein Dialog über einem modalen Fenster nahm ihm die Modalität.** Die
    gui hatte einen einzigen Platz dafür; nach der Antwort auf „Ein Stück
    entnehmen?“ war das Detailfenster nicht mehr modal und sein
    Abbrechen-Knopf taub. Jetzt ein Stapel: geht der obere, ist der darunter
    wieder modal (gui).

**Eigene Fehler, gefunden in Stufe 2:** Im Produkt- und im
Mindestmengen-Dialog fehlten seit Stufe 1 unten die Knöpfe -- das `h` einer
`.dhform` zählt die Titelleiste mit, das Bauskript hatte nur den Inhalt
gerechnet (die Proben riefen die Methoden direkt, keine sah das Fenster).
Neu prüft `stufe0_geruest`, dass jedes Fenster seine Widgets fasst; die
Regel in CLAUDE.md und im Handbuch sagt jetzt „mit Titelleiste“ statt „Höhe
des Inhalts“. Der Klick-Test aus Stufe 1 hatte die Spaltenbreiten fest
eingetragen und klickte daneben, sobald die Seitenleiste die Tabelle
schmaler machte; `kSpalteMitte` liest sie aus der `.dhform`.

**Fallen:** `KEY_ANY_HIT` sieht eingespielte Tasten nicht -- ESC aus einer
Aufnahme erreicht `Taste` nie, wohl aber den Abbrechen-Knopf (die gui liest
die Tasten selbst); `Taste` prüft nur eine echte Fensternachricht. Eine
Probe mit `--bilder 30` endet, bevor ein drittes Bild-Paket dran ist --
Bildschirmfotos danach zeigen alle dasselbe. Mess- und Abspiellauf teilen
die Datei im Projektordner: Probedaten vorher löschen, sonst stehen sie
doppelt da.

Tests `tests/pruef/kuehlschrank/stufe2_lagerorte.dhtest` (4: Kacheln mit
Farbe, Sinnbild, Zählern und Rollen; Verlegen mit Bereich und Rückgängig;
Detailfenster mit Entnehmen; echte Eingaben -- Zeile ziehen, Strg+Z, Reiter,
Kachel, Entnehmen, Enter, Esc, zurück zum Bestand), in `stufe0_geruest`
„jedes Fenster fasst seine Widgets“, `werkzeug_studio_modal.dhtest` (4),
je ein Fall in `werkzeug_studio_takt`, `gui_reiterwerk`,
`gui_dialoge_ausbau` und `gui_tabelle_bedienung`. Gegenproben: elf im
Beispiel, neun im Studio, fünf in der Laufzeit -- jede fällt in ihrem Fall.
Die fürs Fertigzeichnen vor der inneren Schleife fiel zuerst nirgends:
`studioFrei` verdeckte sie, solange der öffnende Klick hinter dem Schleier
lag; erst ein Fenster, das genau unter dem Klick aufgeht und selbst eine
Zeichenfläche hat, zeigt sie.

## Stufe 3: der Einkauf (2026-11-08)

Der Reiter Einkauf ist das eigene Control `EinkaufSeite` (wie
tabs/shopping_tab.py): Liste mit Haken und Papierkorb je Zeile, Eingabe mit
Vorschlägen aus den Produkten, fünf Schnellknöpfe, Auto-Ergänzen aus den
Mindestmengen, „Alle umschalten“, „Erledigte löschen“, der Markt mit
Gangplan (nach Gang sortieren, Gänge als Überschrift, „Nach Markt“), Drucken
und „Einkauf abschließen“. Die sieben eingebauten Gangpläne der Python-App
stehen im Modul `Einkauf`, eigene aus `store_layouts` gehen vor; der gewählte
Markt steht wie dort in `store_settings`. `AbschlussFenster` (wie
TransferToInventoryDialog) zeigt je abgehaktem Posten Menge, Einheit,
Ablaufdatum, Lagerort und Preis zum Bearbeiten in der Tabelle, dazu
Sammelaktionen; Übernehmen legt fehlende Produkte an, trägt den Bestand ein,
merkt sich Markt und Preis (`product_stores`) und nimmt die Posten von der
Liste -- ein Schritt für Strg+Z, wie jede Änderung der Liste außer dem
Haken. Gedruckt wird der Studio-Bericht `Einkaufszettel`: seine Quelle ist
die Abfrage `Zettelzeilen` im Datenbank-Element; Gang und Stelle je Posten
rechnet das Programm und gibt sie als Parameter `:gaenge` (JSON) mit, die
Abfrage liest sie mit `json_each`.

**Abweichungen von der Python-App** (Fehler dort, die der Port nicht
übernimmt): `product_stores.purchase_count` gibt es in ihrer Datei nicht --
jedes „gelernte“ Speichern scheiterte still; hier gilt der zuletzt genutzte
Markt. Der Gangplan suchte über den Namen unter einem Schlüssel `name`, den
ein Posten nicht hat (er heißt `product_name`) -- die Suche fand nie etwas;
neu findet er auch eine Abteilung, die in der Kategorie steckt („Fleisch &
Wurst“). Die Schnellknöpfe zählten die offenen Posten der Liste selbst; hier
das am häufigsten Verbrauchte, das nicht schon offen dasteht. Ein Posten von
Hand bekommt die Kategorie des Produkts gleichen Namens. Die Vorgaben im
Abschluss kommen aus der Kategorie und dem letzten Lagerort (SmartDefaults
folgen mit Stufe 6).

**Funde:**

18. **Ein `TEXT_SIZE` in `Zeichnen` ließ die ganze Oberfläche schrumpfen**
    (eigener Fehler aus Stufe 2): `studioSchriftAus` setzte die Schrift
    zurück, die Größe nicht -- und ohne eigene Schrift zeichnet die gui in
    der Textgröße des Programms. Jetzt `GFX_PUSH`/`GFX_POP` um `Zeichnen`.
    Auf dem Bild von Stufe 2 war es schon zu sehen (kleinere Schrift als in
    Stufe 1), aufgefallen ist es erst im Vergleich.
19. **„Nur Zeilen mit `markt <> ""`“ ging nicht** -- die Zerlegung lehnte
    jeden leeren Wert ab, auch einen in Anführungszeichen; „Markt:“ im Kopf
    des Zettels erschien nie (Studio, Berichte).
20. **Die Einträge einer Klappliste ließen sich nicht lesen** -- unter ihrem
    Namen; `GUI_LISTBOX_COUNT/ITEM` konnten es, sagten es aber nicht. Neu
    `GUI_DROPDOWN_COUNT(dd)` und `GUI_DROPDOWN_ITEM(dd, i)` (gui).

**Fallen:** Ein Control meldet mit `Ausloesen`; das Fenster holt es im
nächsten Bild ab -- eine Probe, die den Knopf direkt ruft, muss danach Bilder
drehen. Eine Marktliste, die beim Öffnen gefüllt wird, kennt einen später
angelegten Gangplan nicht (`maerkteFuellen`). Die Zeilen der Liste stehen
nach Priorität (ein Schnellknopf legt mit 1 an, die Python-App mit 0 als
Vorgabe der Tabelle) -- eine Probe sucht Zeilen beim Namen, nicht bei der
Stelle.

Tests `tests/pruef/kuehlschrank/stufe3_einkauf.dhtest` (5: Liste, Gangplan,
Überschriften, eigener Plan, nach Markt; anlegen, Schnellknöpfe, Haken,
Auto-Ergänzen, löschen mit Rückgängig; Abschluss mit Vorgaben, Prüfung,
Abweichung, neuem Produkt, Rückgängig und Wiederholen; der Zettel als PDF,
sortiert und unsortiert; echte Klicks), je ein Fall in
`werkzeug_studio_berichte`, `werkzeug_studio_takt` und `gui_form_widgets`.
Gegenproben: achtzehn im Beispiel, zwei im Studio, zwei in der Laufzeit --
jede fällt in ihrem Fall.

## Stufe 4: die Rezepte (2026-11-08)

Der Reiter Rezepte ist das eigene Control `RezepteSeite` (wie
tabs/recipes_tab.py): Suche (in Name, Beschreibung und Zutaten), acht
Sortierungen, die Schnellfilter „sofort kochbar“, „schnell“ (bis 30 min) und
„wenig Zutaten“ (bis 5), eine Tabelle mit Marke (✅ kochbar, 🟡 ab 75 %),
Zeitfarbe und den Knöpfen 🍳 und 🗑 je Zeile. Ein Doppelklick öffnet
`RezeptFenster` (Details), 🍳 oder „Jetzt kochen“ dort `KochenFenster` (wie
CookRecipeDialog): Portionen, Faktor, je Zutat benötigt/vorhanden/Status,
„Fehlende einkaufen“ und „Jetzt kochen!“ -- das zieht vom Bestand ab, was am
frühesten abläuft zuerst; ein ganzer Eintrag geht als `cooked` in den
Verbrauchsverlauf, ein Rest bleibt mit der kleineren Menge, alles ein
Schritt für Strg+Z. `RezeptNeuFenster` legt von Hand an (eine Zutat mit dem
Namen eines Produkts wird verknüpft), `KatalogFenster` sucht in den 29
deutschen Rezepten der Python-App (`Rezeptkatalog`, aus GERMAN_RECIPES
übernommen) mit denselben Punkten wie LocalRecipeSearch -- die Ähnlichkeit
rechnet `aehnlichkeit` wie difflib (Ratcliff/Obershelp), nachgerechnet auf
allen Suchwörtern des Tests. `VorschlagFenster` zeigt „Was kann ich
kochen?“ und „Ablaufende verwerten“; Löschen merkt erst die Zutaten, dann das
Rezept, damit Rückgängig in einer Datei mit Fremdschlüssel das Rezept zuerst
wieder einsetzt. `entnehmen` im Modul `Bestand` teilt sich dafür in
`entnehmen` (Schritt und Transaktion) und `verbrauchen` (ein Eintrag in den
Verlauf, ohne beides).

**Abweichungen von der Python-App:** Sie verglich Zutat und Produkt als
Teilzeichenketten -- „Ei“ fand „Reis“, im Kochen-Dialog standen 260 Stück Ei
(10 Eier und 250 g Reis). Hier Wörter: jedes Wort des kürzeren Namens steht
im längeren, gleich oder mit Plural-Endung (`namePasst`: „Zwiebel“ und
„Zwiebeln“, aber nicht „Mehl“ und „Kartoffeln mehlig“). Der Import aus dem
Katalog ließ die Beschreibung fallen; die Suche versprach im Platzhalter
Zutaten und suchte sie nicht. Die drei Dialoge für Kochbares, Ablaufendes und
Fehlendes sind hier ein Fenster mit zwei Arten. Die Suche im Internet
(Spoonacular) kommt mit Stufe 6; bis dahin nennt der Knopf die Suchwörter.
Eine Datei, die das Studio anlegt, kennt den Verweis der Zutat aufs Rezept
nur als Spalte -- ohne `ON DELETE CASCADE` wie die Python-Datei; darum löscht
`rezeptLoeschen` die Zutaten selbst (offen fürs Studio: Fremdschlüssel aus
dem Datenbank-Element).

**Funde:**

21. **Eine Methode ersetzte still die geerbte** -- `SUB zeigen()` im
    Kochen-Fenster war `Zeigen()` der Studio-Laufzeit (Namen ohne Groß/klein),
    jeder Aufruf `kf.Zeigen()` landete dort, und eine Zeile tiefer brach
    `GUI_VALUE` ab. Gewarnt wurde nur bei anderer Parameterzahl; jetzt auch
    bei anderer Schreibweise (Compiler meldet einen Kandidaten mit der Zeile
    der Elternklasse, `lsp::verdeckung_filtern` vergleicht die beiden Köpfe).
22. **Die Meldung von `FORMAT$` ohne Platzhalter hatte Lücken** -- eine über
    drei Zeilen fortgesetzte Zeichenkette in Rust trug die Einrückung mit
    („kaeme                             nicht vor“). Aufgefallen mit der
    VB-Maske `"0.0"`, die hier `"%.1f"` heißt.
23. **⏱ ⏰ ⏳ ⭐ standen als Fragezeichen da** -- die Emoji im Block
    „Verschiedene technische Zeichen“ (U+231A..U+23FF) und in „Pfeile und
    Symbole“ (U+2B00..U+2BFF) gingen an Segoe UI statt an die Emoji-Schrift.
24. **Enter in einem modalen Fenster traf die Tabelle dahinter.** Der Fokus
    blieb auf ihr, Enter war dort ein Doppelklick und öffnete das nächste
    Fenster, statt den Standard-Knopf zu drücken (der echte Klicklauf
    öffnete Details über Details). Jetzt nimmt `GUI_WINDOW_MODAL` den Fokus
    aus anderen Fenstern und gibt ihn beim Gehen zurück (gui).

**Fallen:** Ein Messlauf und der Abspiellauf teilen die Datei -- Testdaten
erst löschen. Eine Probe, die die Datei des Projektordners braucht, setzt
`DH_STUDIO_DATEN`; ohne liest sie `_studio/fridge_database.db`.

Tests `tests/pruef/kuehlschrank/stufe4_rezepte.dhtest` (7: Katalogsuche mit
den Punkten der Python-App, Wörter statt Teilzeichenketten und Import mit
Beschreibung; Liste, Suche, Sortierungen, Schnellfilter; Kochen mit
Portionen, ältestes zuerst, Rückgängig und Wiederholen, Fehlendes einkaufen;
anlegen von Hand, verknüpfen, löschen mit Rückgängig; Vorschläge für
Kochbares und Ablaufendes; Löschen in einer Datei mit Fremdschlüssel; echte
Klicks mit Doppelklick, Enter in zwei Dialogen, Papierkorb und Strg+Z), je ein
Fall in `compiler_warnungen`, `konsole_werkzeug`, `schriften_vorrat` und
`gui_dialoge_ausbau`. Gegenproben: einunddreißig im Beispiel, sechs in der
Laufzeit -- jede fällt in ihrem Fall. Sieben im Beispiel fielen erst nach
neuen Proben (kurzes Wort mit Endung „Ei“/„Eis“, das größere von zwei
passenden Produkten, Kochen über das verknüpfte Produkt, der Rang der
Ablaufenden, der früheste Block bei difflib, Suche nur in den Zutaten, der
Filter „schnell“ allein): grün waren sie vorher auch ohne die Stelle.

## Stufe 5: die Statistik (2026-11-09)

Der Reiter Statistik ist das eigene Control `StatistikSeite` (wie
tabs/stats_tab.py): oben fünf Kacheln im Verlauf (Gesamt, Produkte,
Ablaufend, Abgelaufen, Cache), darunter ein Reiterwerk mit vier Seiten --
Übersicht (Bestand frisch/bald/abgelaufen, die fünf häufigsten Kategorien,
die Lagerorte mit dem, was dort Aufmerksamkeit braucht), Charts (zwei Kuchen
und der Ablauf-Status als Balken, Studio-Diagramme), Insights (Hinweise aus
den Zahlen) und Cache (Einträge, gefunden, nicht gefunden; leeren mit
Frage). Oben rechts Thema hell/dunkel für die ganze Oberfläche, eine
Testbenachrichtigung (`NOTIFY`) und Aktualisieren; das Hauptfenster frischt
die Seite mit allem anderen auf. Die Zahlen rechnet die Klasse
`Statistikdaten` (wie get_statistics und get_storage_summary), den
Barcode-Cache liest das Modul `Barcodecache` (barcode_cache.json neben der
Datenbank; das Scannen in Stufe 6 schreibt ihn).

**Abweichungen von der Python-App:** Je Lagerort zählte sie, was bald
abläuft, als Summe der Mengen -- 500 g Hack waren 500; hier Einträge, die
abgelaufen sind oder bald ablaufen. Orte gruppierte sie nach dem vollen
Namen und kürzte erst beim Anzeigen auf den Teil vor " - " -- „Kühlschrank“
stand so zweimal da; hier zählt „Kühlschrank - Tür“ zu „Kühlschrank“. Eine
Kategorie NULL und eine leere hießen beide „Sonstige“ und standen getrennt;
hier einmal. Gleich häufige Kategorien und Orte stehen nach dem Namen statt
zufällig. Die Balken der Top-Kategorien rechnete sie aus und zeigte sie nie;
hier stehen sie da. Ohne matplotlib gab es dort keine Charts. Die
Übersicht steht nebeneinander statt untereinander.

**Funde:**

25. **Das Studio hatte kein Diagramm**, obwohl die Laufzeit Kuchen, Balken,
    Linie und Tacho kann (Modul `chart`). Neu der Baustein „Diagramm“:
    eine Zeichenfläche mit Art und Titel im Inspektor, einem Muster im
    Entwurf und im Code ein `StudioDiagramm` (Leeren, Hinzufuegen, Reihe,
    Wert, Setzen ...), das sich selbst zeichnet, im Thema der Oberfläche.
26. **Ein Reiterwerk in einem eigenen Control schaltete nichts um** -- alle
    vier Seiten lagen übereinander. Beim Ausklappen ins Fenster verschob das
    Studio die Nummern der Kinder eines Layouts, die Seiten eines
    Reiterwerks (`tabctl.kinder`) nicht; sie zeigten auf fremde Widgets.
    Dazu gehören die Teile eines Controls im Control jetzt zu jeder Seite
    auf ihrer Kette, nicht nur zu einer im Fenster.
27. **Die Farbe aus `CHART_ADD` galt im Balken nicht** -- der Kuchen färbte
    das Segment, der Balken nahm still die Farbe der Reihe; der Ablauf-Status
    stand ganz in Hellblau (chart). Bei einer Reihe gilt sie jetzt.

**Fallen:** `LOADIMAGE` liefert für denselben Pfad das zuerst geladene
Bild -- eine Probe, die zweimal fotografiert, braucht zwei Dateinamen.
`SAVESCREENSHOT` nimmt, was zuletzt gezeigt wurde: nach dem `FLIP` rufen.
Das Thema der gui heißt in `GUI_THEME_GET` `win_bg`, nicht `bg`.

Tests `tests/pruef/kuehlschrank/stufe5_statistik.dhtest` (4: Zahlen, Orte,
Kategorien, Hinweise und die Werte der Diagramme; der Barcode-Cache mit der
Grenze von 30 Tagen und Leeren mit Esc und Enter; Thema hell und dunkel samt
Diagrammen; echte Klicks auf Statistik, Charts und Aktualisieren),
`werkzeug_studio_diagramm` (2) und je ein Fall in `werkzeug_studio_takt`
und `modules_chart`. Gegenproben: achtzehn im Beispiel, elf im Studio, eine
in der Laufzeit -- jede fällt in ihrem Fall. Die Kette über zwei Controls
fiel erst, als ein inneres Control dazukam, das in seinem äußeren auf keiner
Seite steht: eins auf einer Seite ist schon durch sie verdeckt.
