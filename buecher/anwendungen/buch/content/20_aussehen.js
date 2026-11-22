const { stueck, zeilen } = require("../code_lesen");

const M = "kap20/mediathek/";

module.exports = (H) => [
  H.chapter("Aussehen", 20),

  H.h2("Was wir bauen"),

  H.figure("kap20_aussehen.png", "Die Mediathek mit Thema und Stildatei: „Entfernen“ ist rot, die Statuszeile leise, und rechts zeigt eine Kachel den gewählten Eintrag mit Cover.", 480, 320),

  H.p("Die Mediathek tut inzwischen, was sie soll. In diesem Kapitel bekommt sie ein Gesicht: ein Thema, das zu einem Programm von heute passt, eine Stildatei, die sagt, was wie aussieht, und rechts neben der Tabelle eine Kachel mit dem Cover des gewählten Eintrags — ein eigenes Control, das man wie einen Knopf ins Fenster legt. Im Menü Ansicht schaltet „Helles Aussehen“ zwischen dunkel und hell."),

  H.h2("Der Plan"),

  H.p("Das Aussehen hat drei Ebenen, von grob nach fein:"),

  H.bulletRich("Das Thema ", "färbt alles: Fenster, Felder, Knöpfe, Auswahl. GUI_THEME_PRESET wählt eines; modern_dark und modern_light bringen dazu runde Ecken und Schatten mit."),
  H.bulletRich("Die Stildatei ", "sagt per Regel, was anders aussieht als das Thema: nach Art (button), Klasse (.gefahr), Name (#status) und Zustand (:hover) — wie CSS auf Webseiten."),
  H.bulletRich("Ein eigenes Control ", "bündelt mehrere Teile und ihren Code zu einem Baustein, den man mehrmals verwenden kann."),

  H.h2("Schritt 1: das Thema"),

  H.p("Das Thema wird gesetzt, bevor das erste Fenster aufgeht — in App.Geoeffnet:"),

  H.code(stueck(M + "App.dh", "CLASS App", "END CLASS")),

  H.p("Im Menü Ansicht kommt ein Eintrag „Helles Aussehen“ dazu, im Menü-Editor mit „Mit Haken“. Ein Klick kippt den Haken selbst; die Methode muss nur nachsehen, wie er steht:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB miHell_Aktion", "END SUB")),

  H.h2("Schritt 2: die Stildatei"),

  H.p("Eine Stildatei ist eine Textdatei mit der Endung .dhstil im Projektordner. Beim Starten und Bauen kopiert das Studio sie nach assets/, und StilLaden(\"mediathek.dhstil\") in App.Geoeffnet lädt sie. Die der Mediathek:"),

  H.code(zeilen(M + "mediathek.dhstil"), { out: true }),

  H.p("Jede Regel nennt vorn, wen sie trifft, und in geschweiften Klammern, was gilt. button trifft jeden Knopf; button.gefahr nur die Knöpfe mit der Klasse gefahr, und :hover nur, solange die Maus darauf steht. Eine Farbe ist #RRGGBB oder der Name einer Farbe des Themas wie accent — dann folgt sie dem Thema, wenn es wechselt."),

  H.p("Die Klassen bekommt ein Widget im Inspektor unter Kennung → Klassen, neben seinem Namen: „Entfernen“ bekommt gefahr, die Statuszeile leise. Eine Klasse sagt, was ein Widget ist — gefährlich, nebensächlich —, nicht wie es aussieht. Wie das aussieht, steht an einer Stelle: in der Stildatei. Der Entwurf zeigt sie nicht, denn dort gilt die Stildatei des Studios selbst; F5 zeigt sie."),

  H.figure("kap20_hell.png", "Dieselbe Mediathek nach Ansicht → Helles Aussehen. Das Thema ist hell geworden; „Entfernen“ und die Statuszeile haben feste Farben aus der Stildatei und bleiben.", 480, 320),

  H.h2("Schritt 3: ein eigenes Control"),

  H.p("Projekt → „Eigenes Control hinzufügen …“ legt ein Element an, das wie ein Fenster entworfen wird, nur ohne Titel und Menü. Die Cover-Kachel besteht aus einem Teil: einer Kachel aus der Bibliothek, so groß wie das ganze Control, verankert an allen vier Seiten. Eine Kachel zeigt ein Bild, einen Titel und eine Beschreibung, und die ganze Fläche ist ein Knopf."),

  H.p("Im Inspektor des Controls steht unter „Eigenes Control“ das Feld Ereignisse; dort trägt man Angeklickt ein. Der Code des Controls holt sich, was er zeigen soll, selbst aus der Datenbank:"),

  H.code(stueck(M + "CoverKachel.dh", "SUB Zeigen", "END SUB")),

  H.p("Das Cover liegt als Bytes in der Bildspalte; IMAGE_FROM_BUFFER macht daraus ein Bild, GUI_SET_IMAGE gibt es der Kachel. Die Kachel merkt sich das Bild und gibt es frei, bevor sie das nächste nimmt — sonst bliebe bei jedem Klick in die Tabelle ein Bild mehr im Speicher. Und den Klick meldet sie weiter:"),

  H.code(stueck(M + "CoverKachel.dh", "SUB kachel_Klick", "END SUB")),

  H.p("In der Bibliothek steht die Cover-Kachel jetzt unter Projekt. Ins Hauptfenster gelegt, heißt sie kachel1, 160 × 250 groß, rechts oben verankert; die Tabelle wird dafür schmaler. Das Fenster sagt ihr, wenn sich die Auswahl ändert, und nimmt ihren Klick wie „Bearbeiten …“:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB tabelle_Geaendert", "END SUB")),

  H.code(stueck(M + "Hauptfenster.dh", "SUB kachel1_Angeklickt", "END SUB")),

  H.h2("Warum so?"),

  H.h3("Warum Farben in die Stildatei gehören"),

  H.p("GUI_SET_COLOR(Self.knopfEntfernen, \"bg\", RGB(176, 58, 46)) täte dasselbe wie die Regel button.gefahr — für diesen einen Knopf. Ein zweiter Knopf, der etwas löscht, im Ausleihfenster etwa, bräuchte dieselbe Zeile noch einmal, und wer das Rot ändern will, sucht sie in allen Fenstern. Mit der Stildatei bekommt der neue Knopf die Klasse gefahr, und das Rot steht an genau einer Stelle."),

  H.p("Dazu kommt, dass eine Regel den Zustand kennt: button.gefahr:hover wird heller, wenn die Maus darauf steht, ohne eine Zeile Code. Und wer das Aussehen ändern will, muss den Code nicht verstehen — die Stildatei ist eine eigene Datei, und die Klassen sagen, was gemeint ist."),

  H.h3("Warum ein eigenes Control"),

  H.p("Die Kachel hätte man auch direkt ins Hauptfenster legen können, mit ihrem Code dort. Als eigenes Control weiß sie aber selbst, wie sie einen Eintrag zeigt; das Fenster sagt nur „Zeigen(id)“. Wer die Kachel im Ausleihfenster auch haben will, legt sie dort hinein — der Code kommt mit."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Tippfehler in der Stildatei"),

  H.p("Eine Farbe mit fünf Ziffern, eine Eigenschaft, die es nicht gibt: Die Stildatei wird nicht still übergangen, sondern StilLaden hält das Programm mit der Zeile an — „GUI_STYLESHEET_LOAD (assets/mediathek.dhstil): Zeile 5: '#B03A2' ist keine Farbe“. Sonst suchte man im Programm, warum ein Knopf nicht rot ist."),

  H.h3("Feste Farben in jedem Thema"),

  H.p("#B03A2E ist auf Dunkel wie auf Hell ein deutliches Rot. Eine feste Farbe kann aber in einem Thema gut aussehen und im anderen schlecht zu lesen sein — ein helles Grau, das auf Schwarz leise wirkt, verschwindet auf Weiß. Wer zwei Themen anbietet, sieht sich beide an; wie man nachmisst, ob Text gut genug zu lesen ist, zeigt Kapitel 21."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lass in der Stildatei eine Ziffer von #B03A2E weg. Was geschieht beim Start?"),
  H.bulletRich("Drehen 2. ", "Schalte Ansicht → Helles Aussehen ein. Was ändert sich, was bleibt — und warum?"),
  H.bulletRich("Drehen 3. ", "Nimm „Entfernen“ im Inspektor die Klasse gefahr. Wie sieht der Knopf aus?"),
  H.bulletRich("Erweitern 1. ", "Ein dritter Eintrag im Menü Ansicht: „Wie das System“ — hell oder dunkel, wie das Betriebssystem eingestellt ist."),
  H.bulletRich("Erweitern 2. ", "Ein eigenes Control Sterne: fünf Sterne für eine Bewertung, im Bearbeitungsfenster, gespeichert in einer neuen Spalte bewertung."),
  H.bulletRich("Erfinden. ", "Gib deiner Sammlung aus Kapitel 13 eine Stildatei. Welche Klassen braucht sie — was ist dort gefährlich, was nebensächlich, was wichtig?"),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul gui → Aussehen ändern: ", "alle Themen, die Stildatei mit Selektoren, Eigenschaften, Vorrang, Regeln nach Zeit und Lage."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Eigene Controls: ", "Eigenschaften, Ereignisse, Controls in Controls."),
];
