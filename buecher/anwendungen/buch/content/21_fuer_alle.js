const { stueck } = require("../code_lesen");

const M = "kap21/mediathek/";

module.exports = (H) => [
  H.chapter("Für alle", 21),

  H.h2("Was wir bauen"),

  H.p("Die Mediathek sieht gut aus — für jemanden, der gut sieht, eine Maus in der Hand hat und auf den Bildschirm schaut. In diesem Kapitel wird sie für alle anderen brauchbar: für jemanden, der nur die Tastatur nimmt, weil die Hand die Maus nicht mag; für jemanden, der sich den Bildschirm vorlesen lässt; für jemanden, der bei Sonne am Fenster sitzt und graue Schrift nicht mehr erkennt. Dazu kommt ein Menü Bearbeiten mit Kürzeln und im Menü Hilfe ein Eintrag, der das Fenster selbst prüft."),

  H.p("Zu sehen ist davon wenig. Das ist richtig so: Eine gute Tab-Folge, ein Name für die Tabelle und eine Ansage nach dem Entfernen fallen niemandem auf, der sie nicht braucht — und fehlen jedem, der sie braucht."),

  H.h2("Der Plan"),

  H.p("Vier Dinge, die eine Oberfläche für mehr Menschen öffnen:"),

  H.bulletRich("Tastatur überall. ", "Tab geht der Reihe nach durch das Fenster, wie man liest; jeder Befehl hat einen Menüeintrag, die häufigen ein Kürzel."),
  H.bulletRich("Namen. ", "Jedes Widget, das man bedienen kann, hat einen Namen, den ein Bildschirmleser vorlesen kann: seinen Text, eine Beschriftung daneben oder einen Hinweis."),
  H.bulletRich("Kontrast. ", "Schrift hebt sich vom Grund mindestens 4,5:1 ab — so verlangt es die WCAG, die Richtlinie für barrierefreie Oberflächen."),
  H.bulletRich("Ansagen. ", "Was sich nur in der Statuszeile zeigt, bekommt der Bildschirmleser zusätzlich als Satz."),

  H.h2("Schritt 1: prüfen lassen"),

  H.p("Bevor man etwas verbessert, misst man. GUI_A11Y_CHECK$ prüft ein Fenster mit denselben Farben und Lagen, die die gui zeichnet, und liefert eine Zeile je Befund. Im Menü-Editor bekommt die Mediathek ein Menü Hilfe mit „Barrierefreiheit prüfen …“:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB miPruefen_Aktion", "END SUB")),

  H.p("Mit dem Stand aus Kapitel 20, im hellen Aussehen, findet die Prüfung drei Dinge:"),

  H.figure("kap21_pruefen.png", "Hilfe → Barrierefreiheit prüfen an der Mediathek aus Kapitel 20, im hellen Aussehen: die Tabelle hat keinen Namen, die Statuszeile zu wenig Kontrast, und Tab springt zurück.", 480, 320),

  H.bulletRich("Die Tabelle hat keinen Namen. ", "Ein Knopf heißt wie sein Text; ein Feld nimmt die Beschriftung daneben. Eine Tabelle hat keinen eigenen Text, und „Suchen“ gehört dem Feld rechts davon. Ein Bildschirmleser sagt nur „Tabelle“."),
  H.bulletRich("Die Statuszeile ist zu blass. ", "Das feste Grau #8A93A6 aus der Stildatei hat auf dem hellen Grund nur 3,0:1. Im dunklen Aussehen fällt es nicht auf — die Prüfung findet es erst im hellen."),
  H.bulletRich("Tab springt zurück. ", "Von „Ausleihen …“ unten rechts springt Tab nach oben in die Cover-Kachel. Die Kachel ist ein eigenes Control; ihre Teile kommen erst ins Fenster, wenn alles andere schon steht, und ohne eigene Folge geht Tab in der Reihenfolge, in der die Widgets angelegt wurden."),

  H.h2("Schritt 2: die Tab-Folge"),

  H.p("Im Inspektor steht unter Verhalten das Feld Tab-Folge. Widgets mit einer Zahl kommen zuerst dran, aufsteigend; die ohne Zahl danach, in der Reihenfolge des Anlegens. Die Mediathek bekommt die Lesereihenfolge — oben, dann links nach rechts, dann unten:"),

  H.bulletRich("1 ", "feldSuche, 2 tabelle, 3 kachel1,"),
  H.bulletRich("4 ", "knopfNeu, 5 knopfBearbeiten, 6 knopfEntfernen, 7 knopfAusleihen."),

  H.p("Die Beschriftungen und die Statuszeile brauchen keine: Man kann sie nicht bedienen, Tab überspringt sie. Die Tab-Folge der Kachel gilt für die Teile darin."),

  H.h2("Schritt 3: ein Name für die Tabelle"),

  H.p("Unter Verhalten steht auch das Feld Hinweis. Die Tabelle bekommt „Bestand der Mediathek“. Der Hinweis erscheint, wenn die Maus eine Weile darauf steht, und ein Bildschirmleser liest ihn als Namen vor, wenn das Widget keinen anderen hat. Ein Name sagt, was es ist — nicht, wie man es bedient; das weiß der Leser selbst."),

  H.h2("Schritt 4: Kontrast"),

  H.p("Die Statuszeile hat die Klasse leise. Statt eines festen Graus nennt die Regel in der Stildatei jetzt eine Farbe des Themas:"),

  H.code([".leise              { fg: muted_fg }"], { out: true }),

  H.p("muted_fg ist die gedämpfte Schrift jedes Themas — die, mit der auch Platzhalter in leeren Feldern stehen. Jedes eingebaute Thema hält damit 4,5:1, auf Dunkel wie auf Hell. Eine feste Farbe kann nur in einem Thema richtig sein; ein Name folgt dem Thema."),

  H.h2("Schritt 5: Kürzel"),

  H.p("Wer ohne Maus arbeitet, will nicht jedes Mal mit Tab zu „Entfernen“ wandern. Ein neues Menü Bearbeiten nimmt die Befehle der Knöpfe auf, jeder mit einem Kürzel im Feld Kürzel des Menü-Editors: Neu … (Strg+N), Bearbeiten … (Strg+E), Entfernen (Entf) und Suchen (Strg+F). Die Methoden rufen nur, was die Knöpfe schon tun:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB miEntfernen_Aktion", "END SUB")),

  H.code(stueck(M + "Hauptfenster.dh", "SUB miSuchen_Aktion", "END SUB")),

  H.p("Strg+F setzt den Fokus ins Suchfeld — von dort ist es ein Tab bis zur Tabelle. Das Kürzel steht nur im Menü; das Programm fragt die Taste nicht noch einmal selbst ab, sonst liefe der Befehl zweimal. Und das Menü zeigt es an: Wer nachsehen will, wie etwas ohne Maus geht, findet es dort."),

  H.h2("Schritt 6: Ansagen"),

  H.p("Nach dem Entfernen steht in der Statuszeile eine Zahl weniger. Wer den Bildschirm sieht, merkt das; ein Bildschirmleser liest die Statuszeile nicht von selbst vor. GUI_ANNOUNCE gibt ihm einen Satz:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB knopfEntfernen_Klick", "END SUB")),

  H.p("Der Titel wird vor dem Entfernen gemerkt — danach gibt es den Eintrag nicht mehr. Der Leser spricht den Satz mit seiner Stimme, nach dem, was er gerade sagt. Ohne Bildschirmleser geschieht nichts. Ebenso sagen das Ausleihfenster nach Verleihen und Zurück und das Einlesen nach seiner Zusammenfassung an, was geschehen ist:"),

  H.code(stueck("kap21/mediathek/AusleihFenster.dh", "SUB knopfVerleihen_Klick", "END SUB")),

  H.p("Danach findet Hilfe → Barrierefreiheit prüfen nichts mehr — im dunklen wie im hellen Aussehen."),

  H.h2("Warum so?"),

  H.h3("Barrierefreiheit ist Tastatur-Tauglichkeit für alle"),

  H.p("Es klingt, als baue man für wenige. Aber wer nur die Tastatur nimmt, ist nicht nur, wer die Maus nicht bedienen kann. Es ist auch, wer schnell arbeiten will: Strg+F, tippen, Tab, Entf — das geht schneller als jeder Weg mit der Maus. Es ist, wer am Laptop ohne Maus sitzt, wer eine Sehnenscheidenentzündung auskuriert, wer bei Sonne auf den Bildschirm schaut und froh über Kontrast ist."),

  H.p("Und vieles davon kostet fast nichts, wenn man es gleich tut: eine Zahl im Feld Tab-Folge, ein Hinweis, ein Name statt einer festen Farbe. Teuer wird es erst, wenn man nachträglich durch dreißig Fenster geht. Darum gehört die Prüfung ins Menü, wo man sie nach jeder Änderung am Fenster aufruft — so wie man nach jeder Änderung am Code das Programm startet."),

  H.h2("Was schiefgehen kann"),

  H.h3("Keine Befunde heißt nicht fertig"),

  H.p("GUI_A11Y_CHECK$ misst, was sich messen lässt: Kontrast, fehlende Namen, zu kleine Ziele, eine Tab-Folge gegen die Lage. Ob ein Hinweis etwas Brauchbares sagt, ob eine Information nur in einer Farbe steckt, ob die Reihenfolge auch dem Sinn folgt — das weiß die Prüfung nicht. Den Rest findet, wer die Maus für eine Viertelstunde weglegt und das Programm benutzt."),

  H.h3("Nur in einem Aussehen geprüft"),

  H.p("Das Grau der Statuszeile war im dunklen Aussehen gut zu lesen; die Prüfung im dunklen Aussehen fand nichts. Wer zwei Aussehen anbietet, prüft beide."),

  H.h3("Zu viel ansagen"),

  H.p("Eine Ansage bei jedem Tastendruck im Suchfeld wäre gut gemeint und schwer zu ertragen: Der Leser redet, während man tippt, und das Wichtige geht im Geplapper unter. Angesagt wird das Ergebnis einer Handlung, die man nicht sieht — nicht jeder Zwischenstand."),

  H.h3("Ein Kürzel, das ein Feld braucht"),

  H.p("Entf entfernt den gewählten Eintrag — aber nicht, wenn man im Suchfeld steht: Dort löscht Entf ein Zeichen, wie es soll. Eine Taste, die ein Feld nimmt, ist für das Menü verbraucht. Ein Kürzel aus einem einzelnen Buchstaben ohne Strg wäre darum eine schlechte Wahl: In jedem Feld würde es zum Buchstaben."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setz in der Stildatei leise wieder auf #8A93A6 und prüfe im dunklen und im hellen Aussehen. Was findet die Prüfung wo?"),
  H.bulletRich("Drehen 2. ", "Gib „Ausleihen …“ die Tab-Folge 2 und zähle die übrigen ab der Tabelle eins weiter (3 bis 7). Was meldet die Prüfung?"),
  H.bulletRich("Drehen 3. ", "Mach „Entfernen“ nur 20 Punkte hoch, mittig zu den anderen Knöpfen. Ist das ein Befund?"),
  H.bulletRich("Erweitern 1. ", "Enter im Suchfeld sagt an, wie viele Einträge die Suche findet."),
  H.bulletRich("Erweitern 2. ", "Wer die Mediathek mit --gross startet, bekommt alles anderthalbmal so groß."),
  H.bulletRich("Erfinden. ", "Leg die Maus weg und bediene deine Sammlung aus Kapitel 13 eine Viertelstunde lang. Was geht nicht, was ist mühsam? Was meldet die Prüfung?"),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul gui → Barrierefreiheit: ", "was GUI_A11Y_CHECK$ prüft, Bildschirmleser, GUI_ANNOUNCE, das Thema contrast."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Eigene Controls: ", "wie die Tab-Folge einer Instanz für ihre Teile gilt."),
];
