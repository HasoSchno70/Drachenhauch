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

  H.p("MAPGETOR liefert die Vorgabe, solange es die Datei oder den Eintrag nicht gibt — beim allerersten Start. Fenster und Spalte entstehen danach mit breite und hoehe statt mit 560 und 300. Wer das Fenster maximiert und dann beendet, findet es beim nächsten Start so groß wie den Bildschirm, aber nicht maximiert; wer das unterscheiden will, fragt WINDOW_MAXIMIZED() und legt es mit ab."),

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
