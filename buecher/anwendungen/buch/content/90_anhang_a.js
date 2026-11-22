const { neu } = require("../code_lesen");

// Anhang A: die Loesungen. Der Code kommt aus code/loesungen/ -- nur die
// Stellen zwischen ' >>> neu und ' <<< neu. Die Dateien und Projekte
// selbst laufen in der Pruefsammlung.
module.exports = (H) => [
  H.chapter("A · Lösungen der Übungen"),

  H.p("Zu den Drehen-Übungen steht hier die Antwort mit ihrer Begründung, zu Erweitern eine Lösung als Datei oder Projekt in code/loesungen. Abgedruckt sind nur die Stellen, die gegenüber dem Stand des Kapitels neu sind; in den Dateien sind sie mit >>> und <<< markiert."),

  H.h2("Kapitel 1 · Ein Fenster und ein Knopf"),

  H.h3("Drehen 1"),

  H.p("Der helle Rahmen um „Aufnehmen“ verschwindet, und Enter im Feld tut nichts mehr. Der Knopf selbst arbeitet weiter, nur der zweite Weg zu ihm ist weg. Der Rahmen war die Ankündigung dieses Wegs — ohne Standard-Knopf gibt es nichts anzukündigen."),

  H.h3("Drehen 2"),

  H.p("Ein Titel aus lauter Leerzeichen kommt durch: Die Statuszeile meldet einen Titel mehr, und in der Liste steht ein leerer Eintrag zwischen zwei Kommas. Auch „ Momo“ mit einem Leerzeichen vorn wäre ein anderer Titel als „Momo“. TRIM$ macht aus beidem, was gemeint war."),

  H.h3("Drehen 3"),

  H.p("Der Knopf funktioniert weiter, nur ein Bild später. GUI_CLICKED meldet, was das letzte GUI_UPDATE festgestellt hat, und das bleibt bis zum nächsten GUI_UPDATE so. Steht die Abfrage davor, sieht sie im nächsten Durchgang den Klick des vorigen. Bei sechzig Bildern in der Sekunde merkt das niemand — mit WINDOW_WAIT_EVENTS aber kann das nächste Bild bis zu einer Sekunde auf sich warten lassen, wenn danach keine Eingabe mehr kommt. Die Reihenfolge des Kapitels ist darum die richtige: erst GUI_UPDATE, dann fragen."),

  H.h3("Erweitern 1: Leeren"),

  H.p("Ein zweiter Knopf und eine zweite Abfrage in der Schleife (code/loesungen/kap01_leeren.dh):"),

  ...neu("loesungen/kap01_leeren.dh").map((t) => H.code(t)),

  H.p("titel = [] ersetzt das Feld durch ein leeres. Der Leeren-Knopf ist bewusst NICHT der Standard-Knopf: Enter soll aufnehmen, nicht alles löschen."),

  H.h3("Erweitern 2: keine doppelten Titel"),

  H.p("Eine Funktion, die nachsieht, und ein Fall mehr in aufnehmen (code/loesungen/kap01_doppelt.dh):"),

  ...neu("loesungen/kap01_doppelt.dh").map((t) => H.code(t)),

  H.p("LOWER$ auf beiden Seiten macht den Vergleich unabhängig von Groß- und Kleinschreibung. Die neue Prüfung steht gleich hinter dem Lesen des Felds und kehrt mit RETURN zurück, bevor aufgenommen wird; das Feld behält dabei seinen Text, damit man ihn ändern kann. Ein leerer Titel kann hier nicht hängen bleiben — in der Mediathek steht nie ein leerer — und bekommt weiter unten seine eigene Meldung."),

  H.h2("Kapitel 2 · Ordnung im Fenster"),

  H.h3("Drehen 1"),

  H.p("Jede Beschriftung ist nur noch so breit wie ihr Wort, und jedes Feld fängt 12 Punkte dahinter an: das Titelfeld weit links, das Jahr weit rechts. Die rechten Kanten der langen Felder stehen dagegen weiter bündig, denn sie hängen am Rand der Spalte, nicht an der Beschriftung. Ohne Größengruppe weiß keine Zeile von der anderen."),

  H.h3("Drehen 2"),

  H.p("Das Titelfeld ist 160 Punkte breit — so breit legt die gui ein Eingabefeld an, dem man keine Breite gegeben hat — und bleibt es auch, wenn das Fenster wächst. Maximiert auf einem Bildschirm mit 1920 Punkten Breite ist das Feld für den Urheber 1725 Punkte breit, das für den Titel weiter 160. Gewicht 0 heißt: eigene Größe, kein Anteil am Rest."),

  H.h3("Drehen 3"),

  H.p("Die Knopfreihe steht direkt unter dem Feld für das Jahr, im üblichen Abstand von 8 Punkten, und der freie Platz liegt unten im Fenster. Der Leerraum war das Einzige in der Spalte mit Gewicht; ohne ihn bekommt niemand den Rest, und die Spalte füllt sich nur von oben."),

  H.h3("Erweitern 1: zwei Spalten"),

  H.p("Zwischen die äußere Spalte und die Zeilen kommt eine Zeile mit zwei Spalten, jede mit Gewicht 1 (code/loesungen/kap02_zwei_spalten.dh):"),

  ...neu("loesungen/kap02_zwei_spalten.dh").map((t) => H.code(t)),

  H.p("Die SUB zeile bekommt dazu die Spalte, in die die Zeile gehört, und den Namen der Größengruppe. Mit einer gemeinsamen Gruppe für alle vier Beschriftungen wäre auch „Titel“ so breit wie „Erscheinungsjahr“, und links stünde viel leerer Platz — die eine Gruppe je Spalte richtet jede Seite für sich aus."),

  H.h3("Erweitern 2: die Größe merken"),

  H.p("Das Modul ini liest und schreibt Einstellungsdateien, die man auch mit einem Editor öffnen kann. Vor SCREEN wird gelesen, nach der Schleife geschrieben (code/loesungen/kap02_groesse.dh):"),

  ...neu("loesungen/kap02_groesse.dh").map((t) => H.code(t)),

  H.p("MAP_GET_OR liefert die Vorgabe, solange es die Datei oder den Eintrag nicht gibt — beim allerersten Start. Fenster und Spalte entstehen danach mit breite und hoehe statt mit 560 und 300. Wer das Fenster maximiert und dann beendet, findet es beim nächsten Start so groß wie den Bildschirm, aber nicht maximiert; wer das unterscheiden will, fragt WINDOW_MAXIMIZED() und legt es mit ab."),

  H.h2("Kapitel 3 · Listen und Tabellen"),

  H.h3("Drehen 1"),

  H.p("Tetris verschwindet, und nur Tetris. In der sortierten Tabelle steht es weiter unten, aber seine Datenzeile ist 2, wie beim Anhängen: GUI_TABLE_SEL_ROW liefert 2, und mediathek[2] ist Tetris. Sortieren hat nur die Anzeige umgestellt."),

  H.h3("Drehen 2"),

  H.p("Strg+Klick wählt dann um statt dazu: Es bleibt immer genau eine Zeile gewählt, die zuletzt angeklickte. Der Knopf arbeitet trotzdem — GUI_TABLE_SEL_COUNT ist 1 und GUI_TABLE_SEL_ROW(tabelle, 0) die gewählte Zeile —, er entfernt eben nur diesen einen Eintrag. Der Code muss sich nicht ändern, wenn man die Mehrfachauswahl ein- oder ausschaltet."),

  H.h3("Drehen 3"),

  H.p("Es bleiben Momo und Die unendliche Geschichte, beide von Michael Ende — der Filter sucht Teiltext und kennt keine Groß- und Kleinschreibung. Ein Doppelklick auf die zweite zeigt „Die unendliche Geschichte von Michael Ende (Buch, 1979)“: Sichtbar ist sie Zeile 1, ihre Datenzeile ist aber 8, und GUI_TABLE_SELECTED liefert die 8."),

  H.h3("Erweitern 1: eine Farbe je Art"),

  H.p("Zwei Funktionen und ein Aufruf an beiden Stellen, an denen Zeilen entstehen (code/loesungen/kap03_farben.dh):"),

  ...neu("loesungen/kap03_farben.dh").map((t) => H.code(t)),

  H.p("ARRAY_INDEXOF sucht die Art in derselben Liste, aus der das Formular seine Auswahl baut, und die Farben stehen in derselben Reihenfolge. GUI_TABLE_CELL_COLOR setzt Schrift- und Hintergrundfarbe einer Zelle; -1 heißt „nicht gesetzt“, und der Hintergrund bleibt der der Zeile, mit Zebrastreifen und Auswahl. Weil das Anhängen und das Färben jetzt in einer Funktion stehen, kann man das eine nicht ohne das andere tun."),

  H.h3("Erweitern 2: Suche über alle Spalten"),

  H.p("Die Filterzeile der Tabelle filtert je Spalte; wer dort sucht, muss wissen, in welcher Spalte das Gesuchte steht, und mehrere Filter gelten zugleich. Für eine Suche über alle Spalten baut die Lösung die Tabelle selbst neu, nur mit den Treffern (code/loesungen/kap03_suche.dh). Damit ist Zeile z nicht mehr Eintrag z, und ein zweites Feld merkt sich, welcher es ist:"),

  ...neu("loesungen/kap03_suche.dh").map((t) => H.code(t)),

  H.p("Jede Stelle, die von einer Zeile zu einem Eintrag geht, übersetzt jetzt über zu_eintrag — zeigen und das Entfernen. Entfernt wird aus dem Feld, dann wird die Tabelle neu gebaut; das ist einfacher, als beide Felder und die Tabelle gleichzeitig nachzuführen. GUI_CHANGED(suche) ist in jedem Bild wahr, in dem jemand im Suchfeld etwas geändert hat, die Tabelle folgt also jedem Buchstaben."),

  H.h2("Kapitel 4 · Daten, die bleiben"),

  H.h3("Drehen 1"),

  H.p("Die zehn Beispiele sind wieder da, mit den Kennungen 1 bis 10, und alles, was man aufgenommen oder entfernt hatte, ist vergessen. Ohne Datei legt DB_OPEN eine neue, leere an, laden findet nichts, und die Mediathek füllt sich wie beim allerersten Start. Die Datei IST die Mediathek; wer sie sichert, sichert alles."),

  H.h3("Drehen 2"),

  H.p("Der erste Start läuft wie gewohnt. Beim zweiten gibt es die Tabelle schon, und CREATE TABLE ohne IF NOT EXISTS bricht ab: „DB_EXEC: table eintrag already exists“. Ein Programm, das mit einer Datei arbeitet, startet fast nie zum ersten Mal — IF NOT EXISTS sagt: Lege an, was fehlt, und lass stehen, was da ist."),

  H.h3("Drehen 3"),

  H.p("Die Tabelle beginnt alphabetisch: Abbey Road, Der kleine Prinz, Die Siedler von Catan … Das Feld mediathek ist in derselben Reihenfolge gefüllt, Zeile n ist also weiter Eintrag n, und das DELETE geht nach der Kennung, nicht nach der Position. Entfernen trifft darum das Richtige. Ein neu aufgenommener Eintrag kommt aber ans Ende, nicht an seinen Platz im Alphabet — bis zum nächsten Start."),

  H.h3("Erweitern 1: Ausleihen"),

  H.p("Eine zweite Tabelle, deren Schlüssel die Kennung des Eintrags ist; damit kann ein Eintrag höchstens einmal verliehen sein (code/loesungen/kap04_ausleihe.dh):"),

  ...neu("loesungen/kap04_ausleihe.dh").map((t) => H.code(t)),

  H.p("REFERENCES eintrag(id) sagt, dass in der Spalte eintrag die Kennung eines Eintrags steht. INSERT OR REPLACE legt eine Ausleihe an oder ersetzt die vorhandene — wer einen verliehenen Eintrag an jemand anderen gibt, braucht keinen eigenen Fall. Beim Entfernen eines Eintrags fällt seine Ausleihe in derselben Transaktion mit weg; sonst stünde in der Tabelle ausleihe eine Kennung, die es nicht mehr gibt. Die Namen liegen im Programm in einer MAP von der Kennung zum Namen, damit die fünfte Spalte beim Laden schnell gefüllt ist."),

  H.h3("Erweitern 2: Entfernen mit Nachfrage"),

  H.p("Ein Merker und zwei kleine SUBs; die Schleife fragt den Merker, bevor sie entfernt (code/loesungen/kap04_nachfrage.dh):"),

  ...neu("loesungen/kap04_nachfrage.dh").map((t) => H.code(t)),

  H.p("Die Frage steht im Knopf selbst, dort, wo man gerade hinsieht, und die Statuszeile sagt, wie viele es sind. Ein Klick in die Tabelle oder auf „Aufnehmen“ nimmt die Frage zurück — wer etwas anderes tut, meint das Entfernen nicht mehr. Einen richtigen Dialog, der wartet, bis man antwortet, zeigt Kapitel 6."),

  H.h2("Kapitel 5 · Formulare, die prüfen"),

  H.h3("Drehen 1"),

  H.p("Nein. Gespeichert wird ein Eintrag ohne Titel, und in der Tabelle steht eine Zeile, deren erste Zelle leer ist. NOT NULL verbietet „kein Wert“, aber ein leeres Feld ist der leere Text — ein Wert, nur ein leerer. Auch laenge 1 bis 200 hält ihn nicht auf, denn jede Regel außer pflicht lässt ein leeres Feld durch. Dass ein Feld ausgefüllt sein muss, sagt allein pflicht."),

  H.h3("Drehen 2"),

  H.p("Erst beim Klick auf „Speichern“: Dann prüft GUI_VALIDATE alle Felder, und das Jahr bekommt seinen Rahmen und seine Meldung. Mit GUI_VALIDATE_LIVE kommt die Meldung schon, wenn man das Feld verlässt — dann, wenn man noch weiß, was man gerade getippt hat."),

  H.h3("Drehen 3"),

  H.p("Tetris wird geladen, in beiden Fällen. GUI_FORM_CHANGED vergleicht die Werte mit dem Stand nach dem Laden, nicht die Tastendrücke: Wer ein Zeichen tippt und wieder löscht, hat danach denselben Urheber wie vorher, und das Formular gilt als unverändert."),

  H.h3("Erweitern 1: die ISBN"),

  H.p("Eine Spalte, ein Feld mit Regel und eine Funktion für die Prüfziffer (code/loesungen/kap05_isbn.dh):"),

  ...neu("loesungen/kap05_isbn.dh").map((t) => H.code(t)),

  H.p("ALTER TABLE fügt die Spalte in eine Datenbank ein, die es schon gibt — die aus Kapitel 4 hat sie noch nicht. Gibt es sie schon, meldet SQLite einen Fehler, den das leere CATCH schluckt: Das ist der eine Fehler, der hier erwartet ist. Die Regel muster lässt nur Ziffern und Striche durch; die Prüfziffer prüft das Programm nach GUI_VALIDATE und meldet sie mit GUI_SET_ERROR. Ein leeres Feld ist erlaubt. Zum Ausprobieren: 978-3-16-148410-0 hat die richtige Prüfziffer, mit einer 1 am Ende nicht."),

  H.h3("Erweitern 2: ein Buch braucht einen Urheber"),

  H.p("Drei Zeilen nach GUI_VALIDATE (code/loesungen/kap05_urheber.dh):"),

  ...neu("loesungen/kap05_urheber.dh").map((t) => H.code(t)),

  H.p("Als Regel ginge das nicht, denn pflicht gälte für jede Art. GUI_SET_ERROR setzt die Meldung so, wie eine Regel es täte: roter Rahmen, Text in der Fehlerzeile. Ändert man danach die Art auf Film, verschwindet die Meldung beim nächsten Speichern, denn GUI_VALIDATE prüft alle Felder neu."),

  H.h2("Kapitel 6 · Menüs, Kürzel, Dialoge"),

  H.h3("Drehen 1"),

  H.p("Gespeichert wird einmal, und danach steht „Ohne Titel geht es nicht.“ unter dem Formular. Die gui hat das Kürzel des Menüeintrags erkannt, GUI_CLICKED(miSpeichern) war wahr, und formular_speichern hat gespeichert und das Formular geleert. Dann kam die eigene Abfrage, sah dieselbe Taste und rief formular_speichern ein zweites Mal — mit einem leeren Formular, das die Regel für den Titel aufhält. Ohne die Regel stünde jetzt ein leerer Eintrag in der Mediathek."),

  H.h3("Drehen 2"),

  H.p("„Ja“ und „Nein“, in dieser Reihenfolge: Ja ist 1, Nein ist 2 — die Auswertung in der Schleife bleibt richtig. Nur muss man jetzt die Frage lesen, um zu wissen, was „Ja“ tut. „Entfernen“ sagt es selbst."),

  H.h3("Drehen 3"),

  H.p("Ein Zeichen im Titelfeld, das hinter der Schreibmarke. Ein Kürzel ohne Strg oder Alt gehört dem Textfeld, in dem die Schreibmarke steht — sonst ließe sich in keinem Feld mehr ein Zeichen löschen. Steht der Fokus in der Tabelle, fragt Entf, ob die gewählten Einträge entfernt werden sollen."),

  H.h3("Erweitern 1: Entfernen rückgängig"),

  H.p("auswahl_entfernen merkt sich, was es entfernt, und ein neuer Menüeintrag fügt es wieder ein (code/loesungen/kap06_zurueck.dh):"),

  ...neu("loesungen/kap06_zurueck.dh").map((t) => H.code(t)),

  H.p("Die Einträge kommen mit ihrer alten Kennung zurück — INSERT mit der Spalte id —, und weil laden nach Kennung sortiert, stehen sie danach wieder an ihrem alten Platz. Der Eintrag hat kein Kürzel: Strg+Z gehört den Textfeldern, die damit das letzte Tippen zurücknehmen; ein Menüeintrag mit demselben Kürzel nähme es ihnen weg. GUI_MENU_ENABLE in jedem Bild hält ihn gesperrt, solange es nichts zurückzuholen gibt."),

  H.h3("Erweitern 2: die Filterzeile im Menü"),

  H.p("Ein drittes Menü mit einem Häkchen-Eintrag (code/loesungen/kap06_filter.dh):"),

  ...neu("loesungen/kap06_filter.dh").map((t) => H.code(t)),

  H.p("GUI_MENU_CHECK macht den Eintrag zu einem mit Häkchen und setzt es; danach kippt jeder Klick es selbst. Das Programm liest mit GUI_MENU_CHECKED, wie es steht, und stellt die Tabelle danach ein. IIF wählt zwischen zwei Werten: 1, wenn das Häkchen gesetzt ist, sonst 0."),

  H.h2("Kapitel 8 · Das erste Projekt"),

  H.h3("Drehen 1"),

  H.p("Wie in Kapitel 1, Drehen 1: Der Rahmen um den Knopf ist weg, und Enter im Feld tut nichts mehr. Im erzeugten Code fehlt die Zeile mit GUI_WINDOW_DEFAULT — der Inspektor schreibt sie, wenn ein Enter-Knopf gewählt ist."),

  H.h3("Drehen 2"),

  H.p("Der Text erscheint unten in der Leiste Ausgabe, und zwar bevor das Fenster aufgeht: Die Laufzeit ruft erst Geoeffnet der App, dann zeigt sie das Startfenster. Darum gehört in die App, was vor jedem Fenster geschehen muss — eine Datenbank öffnen, die Sprache wählen, Einstellungen laden."),

  H.h3("Erweitern 1: ein Info-Fenster"),

  H.pmix(["Das Projekt code/loesungen/kap08_info hat ein zweites Fenster InfoFenster mit einer Beschriftung und einem Knopf knopfZu, der im Inspektor Enter- und Esc-Knopf zugleich ist. Sein Ereignis schließt es:"]),

  ...neu("loesungen/kap08_info/InfoFenster.dh").map((t) => H.code(t)),

  H.p("Im Hauptfenster kommt ein Knopf knopfInfo dazu und sein Ereignis:"),

  ...neu("loesungen/kap08_info/Hauptfenster.dh").map((t) => H.code(t)),

  H.p("NEW InfoFenster() legt ein neues Fenster an, ZeigenModal() zeigt es und kehrt erst zurück, wenn es zu ist. Solange es offen ist, nimmt das Hauptfenster keine Klicks an. Ohne „Modal“ — mit Zeigen() — kehrte der Aufruf sofort zurück, und beide Fenster wären gleichzeitig bedienbar; für ein Info-Fenster ist Warten das Erwartete."),

  H.h3("Erweitern 2"),

  H.p("Das Programm beginnt mit dem Info-Fenster. Wer es schließt, beendet das Programm, denn das Startfenster zu schließen beendet immer das ganze Programm. Das Hauptfenster bekommt man so nie zu sehen: Es wird nur geöffnet, wenn jemand es öffnet, und das tut hier niemand. Projekt → Als Startfenster auf dem Hauptfenster stellt es wieder her."),
];
