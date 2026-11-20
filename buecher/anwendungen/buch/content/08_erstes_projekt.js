const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil II · Das Studio"),

  H.p("Im ersten Teil ist die Mediathek von Hand entstanden: jede Lage eine Zahl im Code, jede Abfrage eine Zeile in der Schleife. Das hat funktioniert, und man weiß jetzt, wie es funktioniert. Es hat aber auch gezeigt, wo es mühsam wird: Ein Knopf, der zehn Punkte weiter rechts stehen soll, heißt Zahlen ändern, starten, ansehen, wieder ändern. Und je mehr Fenster es gibt, desto voller wird die eine Schleife."),

  H.p("Ab hier baut das Buch die Mediathek noch einmal, diesmal richtig: im Drachenhauch Studio. Fenster werden gezeichnet statt ausgerechnet, jedes Fenster ist eine Klasse mit eigenem Code, und die Schleife schreibt das Studio selbst."),

  H.chapter("Das erste Projekt", 8),

  H.h2("Was wir bauen"),

  H.figure("kap08_entwurf.png", "Das Studio mit dem Projekt dieses Kapitels: links der Navigator mit den Teilen des Projekts und die Bibliothek mit den Bausteinen, in der Mitte das Fenster im Entwurf, rechts der Inspektor mit seinen Eigenschaften.", 480, 300),

  H.p("Dieselbe Mediathek wie am Ende von Kapitel 1 — ein Feld, ein Knopf, eine Antwort —, aber als Projekt im Studio. Am Ende des Kapitels sieht das Programm genauso aus wie das von Hand gebaute. Was sich geändert hat, steht nicht im Fenster, sondern in den Dateien dahinter."),

  H.h2("Der Plan"),

  H.bulletRich("Ein neues Projekt anlegen ", "und sichern — das Studio legt einen Ordner mit lesbaren Dateien an."),
  H.bulletRich("Das Fenster entwerfen: ", "vier Bausteine aus der Bibliothek hineinziehen, im Inspektor benennen und beschriften."),
  H.bulletRich("Dem Knopf ein Ereignis geben ", "und in seine Methode den Code schreiben, der in Kapitel 1 in der SUB aufnehmen stand."),
  H.bulletRich("Starten ", "mit F5."),

  H.h2("Schritt 1: ein neues Projekt"),

  H.p("Das Studio startet man aus der IDE (Werkzeuge → Drachenhauch Studio), über den Startbildschirm („Modern – das Studio“) oder im Startmenü. Strg+Umschalt+N fragt, womit das neue Projekt anfangen soll:"),

  H.figure("kap08_neu.png", "Neues Projekt: vier Vorlagen, jede ein fertiges Projekt, das sofort mit F5 läuft. Für dieses Kapitel: Leer.", 460, 290),

  H.p("Für die Mediathek ist „Leer“ richtig: die App und ein leeres Fenster. Strg+S sichert und fragt beim ersten Mal nach einem Ordner — am besten einen neuen, leeren mit dem Namen mediathek. Darin liegen danach diese Dateien:"),

  H.table([
    ["projekt.dhstudio", "Name des Projekts, das Startfenster und die Liste seiner Teile"],
    ["App.dh", "die Klasse App — das Programm als Ganzes"],
    ["Hauptfenster.dhform", "das Aussehen des Fensters: welche Bausteine wo liegen"],
    ["Hauptfenster.dh", "der Code des Fensters"],
    [".gitignore", "sagt einer Versionsverwaltung, dass _studio/ nicht dazugehört"],
  ], { headers: ["Datei", "was darin steht"], widths: [2600, 6426], mono: [0] }),

  H.p("Alle Dateien sind Text. Die .dhform ist JSON, die .dh ist Drachenhauch; man kann beides in jedem Editor öffnen, und eine Versionsverwaltung zeigt bei jeder Änderung, was genau anders ist."),

  H.h2("Schritt 2: das Fenster entwerfen"),

  H.p("Ein Klick auf „Hauptfenster“ im Navigator öffnet das Fenster im Entwurf. Ohne Auswahl zeigt der Inspektor rechts die Eigenschaften des Fensters selbst; dort bekommt es den Titel Mediathek und die Größe 520 × 330."),

  H.p("Dann kommen die Bausteine. In der Bibliothek links unten stehen sie in Gruppen; ein Baustein wird ins Fenster gezogen oder angeklickt und dann ins Fenster geklickt. Für die Mediathek braucht es vier Arten, fünf Stück:"),

  H.table([
    ["Beschriftung", "beschriftungTitel", "Text: Titel"],
    ["Textfeld", "feldTitel", "Platzhalter: z. B. Momo"],
    ["Knopf", "knopfAufnehmen", "Text: Aufnehmen"],
    ["Beschriftung", "status", "Text: Noch nichts aufgenommen."],
    ["Beschriftung", "liste", "Text leer, Umbruch an"],
  ], { headers: ["Baustein", "Name", "Eigenschaften"], widths: [2000, 2600, 4426], mono: [1] }),

  H.p("Jeder Baustein bekommt im Inspektor einen Namen, der sagt, was er ist und wozu er dient. Das Studio würde sie sonst Knopf1, Textfeld1 und so weiter nennen, und spätestens im Code weiß dann niemand mehr, welches Textfeld gemeint ist. Die Lage zieht man mit der Maus zurecht; Hilfslinien zeigen, wann zwei Bausteine an einer Kante fluchten."),

  H.p("Zum Schluss wählt man im Inspektor des Fensters knopfAufnehmen als Enter-Knopf. Das ist GUI_WINDOW_DEFAULT aus Kapitel 1, nur angeklickt statt getippt."),

  H.h2("Schritt 3: das Ereignis"),

  H.p("Ein Doppelklick auf den Knopf im Entwurf fragt, welches Ereignis er bekommen soll. Ein Knopf hat nur eines: Klick. Danach öffnet sich ein Reiter mit dem Code der Methode knopfAufnehmen_Klick. Die Kopfzeile darüber schreibt das Studio selbst; in das Feld darunter kommt der Rumpf:"),

  H.figure("kap08_code.png", "Die Methode im Code-Reiter. Über dem Code steht ihre Kopfzeile und, wozu sie da ist; im Navigator links ist das Ereignis Klick unter dem Knopf markiert.", 480, 300),

  H.p("Der Rumpf ist die SUB aufnehmen aus Kapitel 1, mit einem Unterschied: Vor jedem Baustein steht Self. — er ist jetzt ein Feld des Fensters. Dazu braucht das Fenster einen Ort für die Titel. Ein Klick auf „Eigenschaften“ unter dem Hauptfenster im Navigator öffnet, was außerhalb der Methoden steht; dorthin kommt das DIM. Und damit die Schreibmarke beim Start im Feld steht, bekommt das Fenster selbst ein Ereignis: Doppelklick auf eine freie Stelle im Entwurf, Ereignis Geoeffnet. Was dabei herauskommt, steht in Hauptfenster.dh:"),

  H.code(zeilen("kap08/mediathek/Hauptfenster.dh")),

  H.pmix(["Das ist die ganze Datei. Die Kopfkommentare sind von Hand; der Rest hat die Form, die das Studio schreibt: eine Klasse ", ["Hauptfenster", true], ", die von ", ["Hauptfenster_Basis", true], " erbt, ihre Felder oben, darunter je Ereignis eine Methode."]),

  H.h2("Schritt 4: starten"),

  H.p("F5 sichert das Projekt, prüft es, erzeugt den Code, der fehlt, und startet das Programm. Was es mit PRINT ausgibt, erscheint unten in der Leiste Ausgabe; findet die Prüfung einen Fehler, steht er unter Probleme, an der Methode und Zeile, in der er liegt."),

  H.figure("kap08_laeuft.png", "Das Programm aus dem Studio, nach einem Klick mit leerem Feld. Es ist von dem aus Kapitel 1 nicht zu unterscheiden.", 440, 280),

  H.h2("Warum so?"),

  H.h3("Ein Fenster ist eine Klasse"),

  H.pmix(["In Kapitel 1 waren ", ["feld", true], ", ", ["knopf", true], " und ", ["titel", true], " Variablen des Programms. Bei einem Fenster geht das gut. Bei zweien beginnt der Ärger: Beide haben ein Feld und einen Knopf, und die Namen müssen sich unterscheiden (", ["feld2", true], ", ", ["knopfInfo", true], "), obwohl sie nichts miteinander zu tun haben. Und ein Fenster, das zweimal offen sein soll — zwei Einträge zum Bearbeiten nebeneinander —, ist mit Variablen gar nicht zu machen."]),

  H.pmix(["Als Klasse hat jedes Fenster seine eigenen Daten (die Bausteine, die Titel) und sein eigenes Verhalten (die Methoden). ", ["NEW Hauptfenster()", true], " macht ein neues, mit eigenen Feldern; zwei davon kommen sich nicht in die Quere. Das ist dieselbe Regel wie beim Funken in „Kleine Wunder“: Was zusammengehört, soll im Programm auch zusammenstehen."]),

  H.h3("Was das Studio erzeugt"),

  H.pmix(["Beim Starten schreibt das Studio einen Ordner ", ["_studio", true], " ins Projekt. Darin steht unter anderem ", ["Hauptfenster_basis.dh", true], ", die Klasse, von der das Fenster erbt. Sie legt für jeden Baustein ein Feld an, baut das Fenster aus der .dhform und fragt in jedem Bild ab, was passiert ist. Für den Knopf steht dort genau eine Zeile:"]),

  H.code(["IF GUI_CLICKED(Self.knopfAufnehmen) THEN Self.knopfAufnehmen_Klick()"]),

  H.p("Das ist die Abfrage aus der Schleife von Kapitel 1. Das Studio schreibt sie für jedes Ereignis, das man anlegt, und die Schleife, die sie Bild für Bild aufruft, steht in der Laufzeit des Studios. Darum braucht man das Handwerk aus Teil I: Wer weiß, dass hinter „Ereignis Klick“ ein GUI_CLICKED in jedem Bild steckt, wundert sich nicht, dass ein Klick genau einmal ankommt, und weiß, wo er nachsehen kann, wenn nicht."),

  H.p("_studio gehört nicht ins Projekt. Es entsteht bei jedem Start neu aus den eigenen Dateien; wer darin etwas ändert, verliert es beim nächsten F5. Darum steht es in der .gitignore."),

  H.h3("Das Aussehen als Datei, nicht als Code"),

  H.p("In Kapitel 1 stand die Lage jedes Bausteins als Zahl im Programm. Im Studio steht sie in der .dhform, und das Programm baut sein Fenster beim Start daraus. Der Entwurf im Studio und das laufende Programm lesen dieselbe Datei — darum sieht das Programm genau so aus wie der Entwurf, auf den Punkt. Und der Code bleibt frei von Zahlen, die nur sagen, wo etwas liegt: In Hauptfenster.dh steht nur noch, was passiert."),

  H.h3("Self. oder nicht"),

  H.pmix(["In einer Methode sieht man die Felder der Klasse auch ohne ", ["Self.", true], ": ", ["GUI_TEXT(feldTitel)", true], " fände dasselbe Feld. Das Studio und dieses Buch schreiben es trotzdem hin, aus zwei Gründen. Man sieht sofort, dass ein Baustein des Fensters gemeint ist und keine lokale Variable. Und hinter ", ["Self.", true], " öffnet das Studio die Vorschläge von selbst — mit allen Bausteinen, Feldern und Methoden des Fensters."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein vertippter Name"),

  H.pmix(["Wer statt ", ["Self.feldTitel", true], " einmal ", ["Self.feldTitle", true], " schreibt, bekommt unter Probleme einen Befund an der Methode:"]),

  H.code(["HAUPTFENSTER hat kein Mitglied 'feldtitle' -- weder HAUPTFENSTER",
          "selbst noch eine ihrer Eltern- oder Kindklassen. Dieses Feld gibt",
          "es nicht, der Zugriff bricht beim Laufen ab. Meintest du",
          "'feldtitel'?"], { out: true }),

  H.p("Die Namen stehen klein, weil Drachenhauch Groß- und Kleinschreibung nicht unterscheidet; der Vorschlag am Ende ist meist schon die Lösung. Ein Klick auf den Befund öffnet die Methode an der Stelle."),

  H.h3("Ein Baustein umbenannt"),

  H.p("Benennt man einen Baustein im Inspektor um, ziehen seine Ereignis-Methoden mit (aus knopf1_Klick wird knopfAufnehmen_Klick), und im Code des Fensters ersetzt das Studio den alten Namen als ganzes Wort. Was es nicht ändern kann, sind Texte in Anführungszeichen und Code außerhalb des Fensters. Darum lohnt es sich, die Bausteine gleich beim Anlegen zu benennen, bevor Code an ihnen hängt."),

  H.h3("Das Startfenster zumachen"),

  H.pmix([["Self.Zumachen()", true], " schließt ein Fenster. Ist es das Startfenster, endet damit das ganze Programm — so, wie es das Kreuz in der Titelleiste tut. Jedes andere Fenster zu schließen, beendet nichts; es ist dann einfach weg, bis jemand es wieder zeigt."]),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Stelle im Inspektor des Fensters den Enter-Knopf auf „(keiner)“ und starte. Was ändert sich?"),
  H.bulletRich("Drehen 2. ", "Schreibe in die App ein Ereignis Geoeffnet mit PRINT \"Mediathek startet\". Wo erscheint der Text, und wann — vor oder nach dem Fenster?"),
  H.bulletRich("Erweitern 1. ", "Ein zweites Fenster InfoFenster (Strg+N) mit einem Satz über das Programm und einem Knopf „Schließen“. Ein Knopf „Über …“ im Hauptfenster öffnet es. Hinweis: NEW InfoFenster() und ZeigenModal()."),
  H.bulletRich("Erweitern 2. ", "Mache das InfoFenster zum Startfenster (Projekt → Als Startfenster) und starte. Was passiert, wenn du es schließt? Warum?"),
  H.bulletRich("Erfinden. ", "Baue dein Fenster aus der Erfinden-Übung von Kapitel 1 noch einmal im Studio. Zähle die Zeilen, die du selbst geschrieben hast, hier und dort."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Das Projekt: ", "die Dateien eines Projekts, die Vorlagen, Sichern und Umbenennen."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Code und Ereignisse: ", "alle Ereignisse je Baustein und die Handgriffe im Code-Reiter."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Starten und Bauen: ", "was in _studio/ steht und wie aus dem Projekt ein Programm zum Weitergeben wird."),
];
