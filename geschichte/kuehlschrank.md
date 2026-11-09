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

## Stufe 6, erster Teil: Scannen (2026-11-09)

„Einkauf erfassen“ im Kopf öffnet den Stapelscan `ScanFenster` (wie
BatchScanDialog): Barcode tippen oder scannen, Enter; Open Food Facts liefert
den Namen -- erst aus dem Cache (`barcode_cache.json` wie
services/api/barcode_cache.py, auch „nicht gefunden“), sonst im Hintergrund
(`HTTP_REQUEST_START`, das Fenster friert nicht ein; Modul `OpenFoodFacts`,
Klasse `Fund`). Jede Zeile bekommt Lagerort und Ablaufdatum aus dem
Gelernten (`user_learning` wie LearningService), sonst aus den Stichwörtern
(Modul `Vorgaben`, die Listen von SmartDefaults). In der Tabelle lassen sich
Name, Menge, MHD und Ort ändern, fünf Knöpfe je Zeile setzen das MHD, das
Kreuz entfernt nach Frage; dazu Alle/Keine/Umkehren, MHD und Lagerort für die
markierten, Markierte löschen, Vorlagen (`shopping_templates` wie
TemplateService) und „Alle speichern“ mit dem Markt aus dem Kopf: Produkte
samt Marke und Kategorie anlegen, Bestand eintragen, Markt merken
(`marktMerken`, jetzt auch im Abschluss), das Gewählte lernen -- ein Schritt
für Strg+Z. Der Produktdialog sucht seinen Barcode ebenso (Enter, „Suchen“,
🗘 ohne Cache; mit Barcode geöffnet gleich zu Anfang) und füllt Name, Marke,
Kategorie, Notizen und das Bild. Der Abschluss des Einkaufs (Stufe 3) nimmt
die Stichwörter jetzt auch, wo Produkt und Kategorie nichts sagen.

**Abweichungen von der Python-App:** SmartDefaults nahm das erste Stichwort
der Liste, das irgendwo im Namen steckt: „Reis“ kam in den Gefrierschrank
(„eis“), „Rindersteak“ hielt 180 Tage im Vorratsraum („tea“),
„Pfefferminztee“ war ein Gewürz („pfeffer“), „Kölsch“ Öl. Hier zählt, wo es
im Wort steht -- am Ende (das Grundwort) vor dem Anfang vor der Mitte, kurze
Stichwörter nicht mitten im Wort, dann das frühere Wort, dann das längere
Stichwort; „Zahnpasta“ bleibt Pasta, wie dort. Scannte man schneller, als
das Netz antwortet, brach sie die vorige Suche ab und die Zeile fehlte; hier
sind mehrere unterwegs. Nach „Menge erhöht“ schrieb sie gleich darauf
„hinzugefügt“ über den Status. Eine Vorlage las sie mit Menge, Lagerort und
Haltbarkeit und nahm doch die Vorgaben; Vorlagenzeilen ohne Barcode legten
das zweite Produkt mit leerem Barcode gegen UNIQUE an. Neue Produkte bekamen
weder Marke noch Kategorie. Die Märkte im Kopf waren eine feste Liste. Ein
Netzfehler hieß im Produktdialog „nicht in der Online-Datenbank gefunden“.
Der Offline-Manager wurde angelegt und nie benutzt -- er fehlt hier.

**Funde:**

28. **Das Studio hatte nur Meldung, Frage und Eingabe** -- keine Auswahl
    aus einer Liste und keine Zahl (QInputDialog.getItem/getInt). Neu in der
    Laufzeit `Auswahl` (Index, -1) und `EingabeZahl` ((TRUE, zahl) oder
    (FALSE, vorgabe)), beide warten wie `Frage`.
29. **`dhrt run datei.dh a b` gab dem Programm still nichts** -- Absicht
    (die Runtime behält ihre Schalter), aber ohne ein Wort: der Testserver
    bekam seinen Ordner nicht und schrieb neben sich, gesucht wurde im
    Programm. Jetzt nennt `dhrt run` solche Wörter und den Aufruf mit `--`.
30. **Enter im Zahlenfeld erreichte den Standard-Knopf nicht**, obwohl das
    Zahlenfeld es dafür weitergibt -- die Regel des Standard-Knopfs ließ nur
    Textfelder durch; „MHD setzen“ ließ sich nicht mit Enter bestätigen (gui).
31. **🗘 stand als Fragezeichen da** -- die Ausweichschrift nahm die erste
    Datei der Liste, die es gibt (Segoe UI Emoji), und gab auf, wenn ihr das
    Zeichen fehlte; Segoe UI Symbol dahinter hat es. Jetzt fragt das nächste
    Bild die nächste Datei.

**Offen:** Kürzel ohne Menü -- Strg+S im Barcode-Feld speichert nicht, denn
`Taste` erreicht ein Studio-Fenster nur, wenn kein Eingabefeld die Taste
nimmt, und ein Fenster ohne Menüleiste hat keine Kürzel.

**Fallen:** `LOADIMAGE` erkennt das Format an der Endung -- ein Bild aus dem
Netz braucht die aus der Adresse. Eine Klappliste mit Fokus nimmt Enter
selbst (sie klappt auf); in `Auswahl` bekommt sie darum keinen Fokus. Proben
fragen nie das echte Open Food Facts: `kOffStarten` startet einen falschen
Server (`_hilfen/off_server.dh`), ohne ihn zeigt `KUEHL_OFF_URL` auf eine tote
Adresse.

Tests `tests/pruef/kuehlschrank/stufe6_scannen.dhtest` (7: Vorgaben und
Lernen; Open Food Facts mit Cache, „nicht gefunden“, 404 und Kennung;
Stapelscan mit drei Barcodes unterwegs, Duplikat, MHD-Knopf, Entfernen,
Speichern mit Markt, Rückgängig und Gelerntem im zweiten Fenster; Vorlagen;
Sammelaktionen; Produktdialog mit Bild; echte Klicks über eine Vorlage), je
ein Fall in `werkzeug_studio_modal`, `os_builtins`, `gui_dialoge_ausbau` und
`schriften_vorrat`. Gegenproben: sechsunddreißig im Beispiel, drei im Studio,
drei in der Laufzeit -- jede fällt in ihrem Fall. Eine fiel erst nach einem
neuen Produkt des falschen Servers (4005, ganz ohne Namen): der Name aus den
Rohdaten war sonst nie gefragt.

## Stufe 6, zweiter Teil: Rezepte aus dem Internet (2026-11-09)

Die Rezeptsuche `KatalogFenster` (wie RecipeSearchDialog) sucht erst im
Katalog, dann im Internet (Spoonacular, Modul `Spoonacular`): im Namen oder
nach Zutaten (mit Komma), höchstens 5 bis 20 Treffer, mit oder ohne
Internet, mit oder ohne Übersetzung. Alles aus dem Netz läuft im Takt des
Fensters: die Treffer des Katalogs stehen sofort da, die aus dem Internet
kommen dazu; erst die Auswahl lädt die Einzelheiten eines Rezepts (ein Punkt
vom Tageskontingent), importieren geht, sobald sie und ihre Übersetzung da
sind -- mit Beschreibung und Quelle. Suchwörter und Zutaten übersetzt ein
Wörterbuch (Modul `Uebersetzung`, ohne Netz, damit eine Zutat „Zwiebel“
heißt und zum Vorrat passt; zusammengesetzte Wörter über ihr Grundwort),
Titel, Beschreibung und Anleitung ein Dienst im Netz (MyMemory, frei und ohne
Schlüssel; die Python-App nahm Google über deep_translator). Er bekommt Stücke
bis 400 Zeichen, vier zugleich; die englischen Texte stehen sofort da und
werden deutsch, sobald er antwortet; was einmal übersetzt ist, steht in
`uebersetzungen.json`. Einheiten wie UnitConverter, metrisch, wo Spoonacular
es mitliefert. Der Schlüssel kommt aus `SPOONACULAR_API_KEY` (wie die `.env`
dort) oder aus `app_settings` („🔑 API-Schlüssel …“), er geht als Kopfzeile
hinaus. „Ablaufende verwerten“ → „Im Internet suchen“ öffnet die Suche nach
den Zutaten. `KUEHL_SPOON_URL` und `KUEHL_UE_URL` ersetzen die Adressen.

**Abweichungen von der Python-App** (am Python-Code selbst nachgeprüft, mit
den Daten des falschen Servers): Die Suchwörter übersetzte sie nur, wenn ein
Umlaut oder eins von vier Wörtern darin stand -- „Nudeln mit Tomaten“ ging
deutsch hinaus und fand nichts. Jeden Titel, jede Beschreibung, jede
Anleitung und jede Zutat übersetzte sie einzeln, bevor sie etwas zeigte. Ein
Fehler (kein oder falscher Schlüssel, Kontingent, kein Netz) hieß „Keine
Rezepte gefunden“, der Grund stand im Log; der Schlüssel stand in jeder
Adresse. Kam die Antwort zu einem früher gewählten Rezept zuletzt, zeigte sie
sie unter dem jetzt gewählten -- und importierte sie so; hier gehören
Einzelheiten zu ihrem Rezept. `clean_unit_name` rief sie nie: „oz.“ und
„Tbsps“ blieben englisch, „large“ und keine Einheit wurden nicht „Stück“,
0,25 TL wurden 0,2. Unbekannte Zeiten (-1) standen als „-1 Min“ da, aus 45
Minuten wurden 13 + 31. „&amp;“ blieb stehen, mehrere Abschnitte der
Anleitung fingen je bei 1 an (ohne Namen), eine Anleitung als HTML-Liste
klebte zu „Boil the water.Add the chicken.“ zusammen, und ohne Anleitung
speicherte sie „Keine Anleitung verfügbar.“ als Anleitung. Für Ablaufendes
gab sie die ersten drei Namen als Suchwort für Titel hinaus, zweimal
„Milch“, wenn zwei Packungen ablaufen; hier jedes einmal, als Zutaten.

**Funde:**

32. **Das Studio stapelte Leerzeilen** -- wer eine Datei von außen öffnet, in
    der DIMs zwischen Methoden stehen, bekam beim Speichern so viele
    Leerzeilen in Folge, wie Methoden dazwischen waren (jede herausgelöste
    ließ ihre Trennzeile bei den Eigenschaften). Im Modul `Uebersetzung`
    vierzehn, in `Lagerorte` und `Rezepte` standen sie seit Stufe 2 und 4
    unbemerkt. Jetzt genügt eine (`codeZerlegen`).
33. **`HTTP_STATUS()` behielt nach einem Netzfehler den vorigen Status** --
    kam gar keine Antwort, blieben Status und Kopfzeilen der letzten stehen,
    und die Suche meldete „Spoonacular antwortet mit Fehler 200“. Jetzt 0 und
    keine Kopfzeilen, blockierend, im Hintergrund und bei `HTTP_DOWNLOAD`
    (wie `HTTP_BYTES()` schon leer war).

**Fallen:** Abrufnummern zählen ab 0 wie bei `timer` -- „0 = keiner
unterwegs“ hielt die erste Suche für keine, und ihre Antwort wurde nie
abgeholt. Das Handbuch sagt es jetzt; „keiner“ ist -1. Eine Probe mit
Netzverkehr braucht mehr als fünf Bilder (`kProbe(..., 1500)` und `END` am
Ende), sonst endet `studioBild()` mitten im Warten. Vier Übersetzungen
gleichzeitig kommen in wechselnder Reihenfolge an -- die Probe sortiert sie.
Keine Probe fragt das echte Spoonacular oder den echten Übersetzer
(`_hilfen/spoon_server.dh`, `kNetzStarten`), und keine sieht einen
Schlüssel aus der Umgebung (`kSchluessel`).

Tests `tests/pruef/kuehlschrank/stufe6_rezeptsuche.dhtest` (6: Wörterbuch,
Einheiten, Stücke und Katalog nach Zutaten; Suche mit Übersetzung im
Hintergrund, Auswahl, zu frühes und richtiges Importieren, Schalter,
Rückgängig; sieben Fehlerfälle samt Kontingent des Übersetzers; späte
Antwort; Ablaufende nach Zutaten, Schlüssel und Cache der Übersetzung; echte
Klicks durch zwei modale Fenster), je ein Fall in `werkzeug_studio2` und
`http_request`. Gegenproben: zweiundsechzig im Beispiel, eine im Studio,
zwei in der Laufzeit -- jede fällt in ihrem Fall. Drei im Beispiel fielen erst
nach schärferen Proben: das Satzende lag im Testtext zufällig auf dem
letzten Leerzeichen vor der Grenze, das zweite Fenster fand die Übersetzung
im Speicher statt in der Datei, und eine englische Zutat sperrte das
Importieren schon ohne die Prüfung der Texte.

## Stufe 7, erster Teil: Einstellungen und Benachrichtigungen (2026-11-09)

„Einstellungen“ (Werkzeugleiste, Extras, Strg+,) öffnet `EinstellungenFenster`
(wie SettingsDialog) mit den Reitern Allgemein (dunkles Design,
Auffrischen in Sekunden, Barcode-Cache, Standard-Einheit), Benachrichtigungen
(an/aus, fünf Arten, Prüfintervall, tägliche Zusammenfassung mit Uhrzeit,
Test-Benachrichtigung) und Ablaufdaten (Warnschwelle mit Beispieldatum,
Standard-Haltbarkeit, Farbschema); Lagerorte, Supermärkte und Datenbank sind
der zweite Teil. Die Klasse `Einstellungen` liest und schreibt `app_settings`
mit denselben Schlüsseln und demselben JSON wie AppConfig (json.dumps), legt
Unlesbares auf die Vorgabe und zu Großes auf die Grenze des Fensters.
Gespeichert gilt alles sofort: Takt des Auffrischens und Prüfens, Prüfen an
oder aus, Thema, Standard-Einheit (auch für Posten der Einkaufsliste ohne
Einheit, wie dort), Warnschwelle. Das Modul `Benachrichtigungen` (wie
ExpiryNotifier, Klasse `Nachricht`) meldet Abgelaufenes, was heute, morgen
und bald abläuft, und was unter der Mindestmenge liegt -- jedes Produkt
einmal je Art, „bald“ einmal am Tag --, dazu die tägliche Übersicht. Es
sendet mit `NOTIFY`; nimmt das System sie nicht, zeigt das Hauptfenster sie
als Toast, die letzte steht in der Statusleiste. Der Hauptfenster-Timer
`erstpruefung` prüft fünf Sekunden nach dem Start, `pruefen` im Intervall,
`zusammenfassung` sieht jede Minute nach der Uhrzeit. `KUEHL_MELDUNGEN`
schreibt die Benachrichtigungen in eine Datei, `KUEHL_JETZT` setzt die
Uhrzeit (für Prüfungen).

**Abweichungen von der Python-App** (der Prüfer am Python-Code selbst
nachgefahren, mit einer Attrappe der Datenbank): Der ExpiryNotifier las die
Häkchen je Art nie -- „Bei abgelaufenen Produkten“ aus hieß trotzdem
Meldungen über Abgelaufenes. Er prüfte nur einmal am Tag; das Prüfintervall
tat danach nichts, und was am Nachmittag dazukam und heute abläuft, meldete
niemand. Die Zusammenfassung nahm von der Uhrzeit nur die Stunde, ihr
24-Stunden-Timer lief ab dem Start (also zur Startzeit), und nach 9 Uhr
gestartet gab es heute keine; hier einmal am Tag ab der Uhrzeit, gemerkt in
`daily_summary_last`. Benachrichtigungen ein- oder ausschalten wirkte erst
nach einem Neustart. Dunkles Design, Sprache und Barcode-Cache standen im
Fenster und wurden nie gelesen; hier wirken Design und Cache, die Sprache
fehlt (es gab nur Deutsch). „Zurücksetzen“ meldete „Einstellungen
zurückgesetzt!“, obwohl erst „Speichern“ sie schrieb -- „Abbrechen“ danach
behielt die alten; hier sagt es das. Das Thema der Statistik vergaß sich beim
nächsten Start; hier ist es dieselbe Einstellung. Namen stehen in der
Reihenfolge des Ablaufs, jeder einmal (dort aus einer Menge, jedes Mal
anders, und zwei Packungen Milch zählten bei „bald“ doppelt).

**Funde:**

34. **`GUI_SET_TEXT` wählte in einer Klappliste nichts** -- bei einer, in
    die man nicht tippen kann, stand der Text nur im Widget: die Box zeigte
    weiter die alte Auswahl (oder nichts), und `GUI_TEXT` lieferte den
    gesetzten Text auch noch, nachdem der Nutzer etwas anderes gewählt hatte.
    Die Standard-Einheit stand leer da. Jetzt wählt `GUI_SET_TEXT` den
    Eintrag gleichen Namens (Groß/klein egal, sonst keinen) und `GUI_TEXT`
    liefert die Auswahl (gui). Die Klapplisten im Produktdialog sind
    bearbeitbar und waren nicht betroffen.
35. **Enter schaltete ein Kästchen um, statt den Dialog abzuschicken.** Haken
    anklicken, Enter -- der Haken war wieder da, gespeichert wurde nichts.
    Die Regel stand so im Handbuch („ein Knopf oder Kästchen mit Fokus nimmt
    Enter selbst“); unter Windows, macOS, in Qt (die Python-App) und im Web
    schaltet die Leertaste, Enter drückt den Standard-Knopf. Jetzt auch hier,
    für Kästchen, Schalter und Optionsfelder; ein Knopf mit Fokus nimmt Enter
    weiter selbst (gui).

**Eigene Fehler:** `einstellungZahl` machte aus einem unlesbaren Wert 0 --
eine Warnschwelle „kaputt“ hätte nichts mehr als bald gezählt; jetzt gilt die
Vorgabe. Eine Klasse `Meldung` hätte die Studio-Funktion `Meldung` (das
Meldungsfenster) verdeckt -- Namen ohne Groß/klein --, darum heißt sie
`Nachricht`.

**Fallen:** `GUI_TIME$` liefert Sekunden mit („09:00:00“). Eine Werkzeugleiste
verrät die Lage ihrer Knöpfe nicht -- die Klick-Probe öffnet die
Einstellungen mit Strg+,. Der Open-Food-Facts-Fall aus Stufe 6 hatte dieselbe
Falle wie die Rezeptsuche (fünf Bilder, die Antwort kam unter Last später)
und fiel im Sammellauf; jetzt 1500 Bilder und `END`.

Tests `tests/pruef/kuehlschrank/stufe7_einstellungen.dhtest` (6: Laden,
Grenzen und Speichern wie json.dumps; die Arten mit Häkchen, einmal je
Produkt, Neues später, Mindestmenge; Zusammenfassung mit Minuten, einmal am
Tag; das Hauptfenster nach dem Speichern samt Pruef-Timer; echte Tasten und
Klicks: Strg+, , Reiter, Haken, Enter, Zurücksetzen und ESC; Thema beim
Start), je ein Fall in `gui_form_widgets` und `gui_dialoge_ausbau`.
Gegenproben: vierundvierzig im Beispiel, vier in der gui -- jede fällt in
ihrem Fall. Eine im Beispiel fiel erst mit einer zweiten Packung Käse:
doppelte Namen fing sonst schon das Merken der Gemeldeten ab.
