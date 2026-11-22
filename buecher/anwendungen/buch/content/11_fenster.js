const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Mehrere Fenster", 11),

  H.h2("Was wir bauen"),

  H.figure("kap11_bearbeiten.png", "Metropolis wird bearbeitet. Das Fenster „Eintrag“ ist modal: Das Hauptfenster dahinter ist abgedunkelt und nimmt keine Klicks an, bis OK oder Abbrechen gedrückt ist.", 440, 278),

  H.p("Bisher hatte die Mediathek ein Fenster, und das Formular stand über der Tabelle. Jetzt bekommt jede Aufgabe ihr eigenes: Das Hauptfenster zeigt nur noch die Tabelle. Ein Eintrag wird in einem eigenen Fenster bearbeitet, das wartet, bis man fertig ist. Ausgeliehen wird in einem dritten Fenster, das offen bleiben darf, während man in der Tabelle weiterblättert."),

  H.h2("Der Plan"),

  H.p("Drei Fenster, zwei Arten, sie zu öffnen:"),

  H.table([
    ["Hauptfenster", "das Startfenster", "die Tabelle und die Knöpfe Neu …, Bearbeiten …, Entfernen, Ausleihen …"],
    ["EintragFenster", "modal", "ein Eintrag zum Bearbeiten; OK oder Abbrechen"],
    ["AusleihFenster", "nicht modal", "wer den gewählten Eintrag hat; folgt der Auswahl"],
  ], { headers: ["Fenster", "Art", "zeigt"], widths: [2200, 1700, 5126] }),

  H.p("Ein modales Fenster öffnet man mit ZeigenModal. Der Aufruf kehrt erst zurück, wenn das Fenster wieder zu ist; solange nimmt kein anderes Fenster des Programms Klicks an. Ein nicht modales öffnet man mit Zeigen. Der Aufruf kehrt sofort zurück, und beide Fenster sind bedienbar. Die drei Punkte hinter „Neu …“ sind eine alte Abmachung: Dieser Knopf öffnet ein Fenster, statt gleich etwas zu tun."),

  H.p("Für die Daten, die zwischen den Fenstern wandern, gilt dieselbe Regel wie für jede andere Klasse: Hinein und heraus geht es über Felder. Das Hauptfenster gibt dem Bearbeitungsfenster einen Eintrag mit und liest danach ab, ob mit OK geschlossen wurde."),

  H.h2("Schritt 1: ein zweites Fenster"),

  H.p("Projekt → Fenster hinzufügen (Strg+N) legt ein neues Fenster an. Im Inspektor bekommt es den Namen EintragFenster — das ist auch der Name seiner Klasse —, den Titel „Eintrag“ und die Größe 440 × 270. Die vier Beschriftungen und Felder aus Kapitel 9 ziehen hierher um. Unten rechts kommen zwei Knöpfe dazu, knopfOk und knopfAbbrechen. Im Inspektor des Fensters wird knopfOk der Enter-Knopf und knopfAbbrechen der Esc-Knopf."),

  H.p("Im Hauptfenster bleibt die Tabelle. Sie wird größer, und sie hat eine fünfte Spalte „Verliehen an“. Unten stehen die Statuszeile und vier Knöpfe: knopfNeu, knopfBearbeiten, knopfEntfernen, knopfAusleihen. Die Klasse Eintrag bekommt dafür ein fünftes Feld, verliehen, und zellen liefert es als fünfte Zelle. Im Modul Beispiele ist Tetris an Lena verliehen, damit man gleich etwas sieht."),

  H.h2("Schritt 2: das Bearbeitungsfenster"),

  H.p("Das Fenster hat zwei Felder, über die es mit dem redet, der es öffnet: eintrag, der Eintrag zum Bearbeiten (NIL für einen neuen), und ok, ob mit OK geschlossen wurde. Der Code:"),

  H.code(zeilen("kap11/mediathek/EintragFenster.dh")),

  H.p("Geoeffnet füllt die Felder aus dem Eintrag. knopfOk_Klick prüft den Titel, schreibt die Felder zurück — in den mitgegebenen Eintrag oder einen neuen —, setzt ok und macht das Fenster zu. Abbrechen macht es nur zu. Esc und das Kreuz in der Titelleiste tun dasselbe wie Abbrechen; ok bleibt dann FALSE."),

  H.h2("Schritt 3: öffnen und abwarten"),

  H.p("Das Hauptfenster bekommt eine Methode, die einen Eintrag bearbeiten lässt und das Ergebnis liefert:"),

  H.code(stueck("kap11/mediathek/Hauptfenster.dh", "FUNCTION bearbeitenLassen", "END FUNCTION")),

  H.p("Jedes Mal entsteht mit NEW ein frisches Fenster. ZeigenModal baut es auf, ruft sein Geoeffnet und wartet. Wenn die nächste Zeile läuft, ist das Fenster schon wieder zu, und in f.ok steht, wie es ausging. Neu und Bearbeiten werden damit kurz:"),

  H.code(stueck("kap11/mediathek/Hauptfenster.dh", "SUB knopfNeu_Klick", "END SUB")),

  H.code(stueck("kap11/mediathek/Hauptfenster.dh", "SUB knopfBearbeiten_Klick", "END SUB")),

  H.p("Bearbeiten gibt den Eintrag aus dem Feld selbst mit, keine Kopie. Was das Fenster bei OK hineinschreibt, steht darum schon in Self.eintraege; das Hauptfenster muss die Tabelle nur neu füllen. Bei Abbrechen hat das Fenster nichts geschrieben, und es gibt nichts zu tun."),

  H.h2("Schritt 4: ein Fenster, das offen bleibt"),

  H.figure("kap11_ausleihen.png", "Tetris ist gewählt, das Fenster „Ausleihen“ ist offen. Es ist nicht modal: Ein Klick auf eine andere Zeile der Tabelle wählt sie, und das Fenster zeigt den neuen Eintrag.", 440, 278),

  H.p("Das Ausleihfenster (380 × 190) hat eine Beschriftung gewaehlt, ein Feld feldAn und die Knöpfe knopfVerleihen und knopfZurueck. Es gibt nur eines davon. Das Hauptfenster legt es in Geoeffnet an und merkt es sich in einem Feld; das Ausleihfenster bekommt umgekehrt das Hauptfenster in sein Feld haupt:"),

  H.code(stueck("kap11/mediathek/Hauptfenster.dh", "SUB Geoeffnet", "END SUB")),

  H.p("NEW legt nur das Objekt an, nicht das Fenster auf dem Bildschirm. Das baut erst Zeigen, und erst dann gibt es seine Controls. Der Knopf „Ausleihen …“ zeigt es; ist es schon offen, holt Zeigen es nach vorn:"),

  H.code(stueck("kap11/mediathek/Hauptfenster.dh", "SUB knopfAusleihen_Klick", "END SUB")),

  H.p("Wählt man eine andere Zeile, sagt das Hauptfenster dem Ausleihfenster Bescheid:"),

  H.code(stueck("kap11/mediathek/Hauptfenster.dh", "SUB tabelle_Geaendert", "END SUB")),

  H.p("Das Ausleihfenster merkt sich den Eintrag und zeigt ihn, wenn es offen ist. Verleihen und Zurück ändern den Eintrag und bitten das Hauptfenster, die Tabelle neu zu füllen:"),

  H.code(zeilen("kap11/mediathek/AusleihFenster.dh")),

  H.p("Geoeffnet und Schliessen sind Ereignisse des Fensters; sie halten offen auf dem Laufenden. Wird das Fenster mit dem Kreuz geschlossen und eine andere Zeile gewählt, merkt folgen sich den Eintrag trotzdem. Beim nächsten „Ausleihen …“ zeigt Geoeffnet den richtigen."),

  H.h2("Warum so?"),

  H.h3("Wann ein Fenster modal sein muss — und wann es nervt"),

  H.p("Ein Fenster ist modal, wenn das Programm ohne die Antwort nicht weiterkann. Das Hauptfenster kann einen neuen Eintrag erst in die Tabelle schreiben, wenn es weiß, ob es einen gibt; dazwischen soll niemand die Zeile löschen, die gerade bearbeitet wird. Darum wartet bearbeitenLassen, und darum ist alles dahinter gesperrt. Dasselbe gilt für eine Rückfrage vor dem Löschen und für Einstellungen, die erst mit OK gelten."),

  H.p("Wo das Programm nicht auf eine Antwort wartet, nervt ein modales Fenster. Wer ausleiht, will zwischendurch nachsehen, was noch da ist, und mehrere Einträge nacheinander verleihen — mit einem modalen Fenster müsste er es für jeden schließen und wieder öffnen. Faustregel: Modal ist ein Fenster, das eine Frage stellt. Ein Fenster, das etwas zeigt oder ein Werkzeug ist, bleibt neben dem Hauptfenster offen."),

  H.h3("Wer kennt wen"),

  H.p("Das Bearbeitungsfenster kennt niemanden. Es bekommt einen Eintrag, gibt einen zurück und weiß nicht, ob es aus dem Hauptfenster, aus einem Menü oder aus einem Test geöffnet wurde. Das Ausleihfenster kennt das Hauptfenster, weil es ihm Bescheid geben muss, wenn sich etwas ändert. Je weniger ein Fenster von den anderen weiß, desto leichter lässt es sich woanders wiederverwenden. Ein modales Fenster kommt fast immer ohne aus: Wer es öffnet, wartet ja auf das Ergebnis und kann es selbst abholen."),

  H.h2("Was schiefgehen kann"),

  H.h3("Zeigen statt ZeigenModal"),

  H.p("Wer in bearbeitenLassen Zeigen schreibt, bekommt ein Fenster, das genauso aussieht — nur kehrt Zeigen sofort zurück. f.ok ist zu dem Zeitpunkt noch FALSE, bearbeitenLassen liefert NIL, und das Hauptfenster tut nichts. Drückt man danach im Fenster auf OK, ändert es den Eintrag trotzdem, denn es hat ihn ja. Die Tabelle erfährt davon nichts und zeigt weiter das Alte. Bei „Neu …“ ist es schlimmer: Das Fenster legt einen neuen Eintrag an, den niemand mehr abholt."),

  H.h3("Ein Control, das es noch nicht gibt"),

  H.p("folgen ruft anzeigen nur, wenn das Fenster offen ist. Ohne diese Abfrage hält das Programm beim ersten Klick in die Tabelle an, noch bevor jemand „Ausleihen …“ gedrückt hat:"),

  H.code(["Laufzeitfehler in projekt/AusleihFenster.dh:33: GUI_SET_TEXT: erwartet",
          "INTEGER (Arg 1), erhalten NIL -- das Widget oder Fenster gibt es noch",
          "nicht (ein Fenster aus dem Studio legt seine Controls erst mit Zeigen",
          "an) -- Aufruf: GUI_SET_TEXT(widget, text)"], { out: true }),

  H.p("Das Objekt AusleihFenster gibt es seit Geoeffnet des Hauptfensters, seine Controls aber erst, wenn es gezeigt wurde. Bis dahin ist Self.gewaehlt NIL. Ein Fenster, das von außen Bescheid bekommt, muss darum wissen, ob es offen ist — oder sich den Wert merken und ihn in Geoeffnet zeigen, wie es hier geschieht."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Wähle Metropolis, drücke „Bearbeiten …“ und klicke dann im Hauptfenster auf eine andere Zeile. Was geschieht — und was geschähe, wenn das ginge?"),
  H.bulletRich("Drehen 2. ", "Öffne das Ausleihfenster, schließe es mit dem Kreuz, wähle eine andere Zeile und öffne es wieder. Welchen Eintrag zeigt es, und welche Zeilen sorgen dafür?"),
  H.bulletRich("Drehen 3. ", "Drücke im Bearbeitungsfenster Esc. Was steht danach in f.ok, und was tut knopfBearbeiten_Klick?"),
  H.bulletRich("Erweitern 1. ", "Ein Menü Bearbeiten mit „Einstellungen …“ öffnet ein modales Fenster mit einem Kästchen „Verliehene Einträge farbig zeigen“. Mit OK wird es wirksam, mit Abbrechen nicht."),
  H.bulletRich("Erweitern 2. ", "Ein Menü Ansicht mit „Info zum Eintrag“ (Strg+I) öffnet ein kleines Fenster mit dem gewählten Eintrag in zwei Zeilen. Für jeden Eintrag ein eigenes — mehrere dürfen gleichzeitig offen sein."),
  H.bulletRich("Erfinden. ", "Gib deinem Programm aus Kapitel 10 ein zweites Fenster. Überlege vorher: Stellt es eine Frage (modal) oder zeigt es etwas (nicht modal)?"),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Was ein Programm braucht: ", "ZeigenModal und die Dialoge, die warten (Meldung, Frage, Eingabe, Auswahl)."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Starten und Bauen: ", "was Zeigen und Zumachen tun und was das Studio für jedes Fenster erzeugt."),
];
