const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Der Inspektor", 9),

  H.h2("Was wir bauen"),

  H.figure("kap09_programm.png", "Das Fenster der Mediathek aus dem Studio: Formular oben, Tabelle, Knöpfe unten rechts. Es sieht aus wie das aus Teil I — gebaut ist es ohne eine Zeile Layout-Code.", 440, 324),

  H.p("Das Fenster der Mediathek, wie es am Ende von Teil I aussah: vier Felder, die Tabelle, drei Knöpfe und eine Statuszeile. Diesmal entsteht es im Entwurf des Studios, und alles, was in Teil I Code war — wohin ein Feld beim Vergrößern wächst, in welcher Reihenfolge Tab durch die Felder geht, welcher Knopf Enter bekommt —, ist eine Eigenschaft im Inspektor."),

  H.h2("Der Plan"),

  H.p("Der Inspektor ist die rechte Leiste des Studios. Er zeigt die Eigenschaften dessen, was gewählt ist: ohne Auswahl die des Fensters, sonst die des Controls. Was man dort einträgt, steht in der .dhform und gilt im laufenden Programm genauso. Für die Mediathek sind es vier Arten von Eigenschaften:"),

  H.bulletRich("Lage und Anker: ", "wo ein Control steht, wie groß es ist und an welchen Rändern des Fensters es klebt."),
  H.bulletRich("Tab-Folge: ", "in welcher Reihenfolge die Tabulatortaste durch die Controls geht."),
  H.bulletRich("Hinweis: ", "ein Text, der erscheint, wenn die Maus einen Augenblick über dem Control steht."),
  H.bulletRich("Knopfart: ", "wie ein Knopf aussieht — „primaer“ für den, den man am häufigsten braucht."),

  H.h2("Schritt 1: das Fenster entwerfen"),

  H.p("Ein neues Projekt „Leer“ wie in Kapitel 8. Im Inspektor des Fensters bekommt es den Titel Mediathek, die Größe 760 × 560 und „Größe änderbar“. Dann kommen die Controls aus der Bibliothek:"),

  H.table([
    [{ text: "beschriftungTitel …", mono: true }, "Beschriftung", "Titel, Urheber, Art, Erscheinungsjahr; links, 140 breit"],
    [{ text: "feldTitel, feldUrheber", mono: true }, "Textfeld", "ab Links 170, 570 breit"],
    [{ text: "wahlArt", mono: true }, "Klappliste", "Einträge: Buch; Film; Spiel; Musik"],
    [{ text: "feldJahr", mono: true }, "Textfeld", "80 breit"],
    [{ text: "tabelle", mono: true }, "Tabelle", "Spalten: Titel; Urheber; Art; Jahr"],
    [{ text: "status", mono: true }, "Beschriftung", "links unten"],
    [{ text: "knopfEntfernen, knopfNeu, knopfSpeichern", mono: true }, "Knopf", "rechts unten nebeneinander"],
  ], { headers: ["Name", "Baustein", "Eigenschaften"], widths: [3300, 1700, 4026] }),

  H.p("Die Felder beginnen bei 170, nicht bei 100: Die Beschriftung „Erscheinungsjahr“ braucht so viel Platz. Im Entwurf sieht man das sofort — ein Feld, das über einem Wort liegt, liegt dort auch im Programm. Im Inspektor des Fensters wird knopfSpeichern der Enter-Knopf."),

  H.h2("Schritt 2: Anker"),

  H.figure("kap09_anker.png", "Die Tabelle ist gewählt. Der Inspektor zeigt ihre Lage — Links 20, Oben 172, 720 × 300 — und darunter den Anker: alle vier Kästchen gesetzt.", 480, 263),

  H.p("Der Anker sagt, an welchen Rändern des Fensters ein Control klebt, wenn das Fenster größer oder kleiner wird. Klebt es links und rechts, wird es breiter; klebt es nur rechts, wandert es mit dem rechten Rand; klebt es oben und unten, wird es höher. Für die Mediathek:"),

  H.table([
    [{ text: "feldTitel, feldUrheber", mono: true }, "L O R", "werden breiter, bleiben oben"],
    [{ text: "tabelle", mono: true }, "L O R U", "wird breiter und höher — sie bekommt allen Zuwachs"],
    [{ text: "status", mono: true }, "L U", "bleibt links, wandert mit dem unteren Rand"],
    [{ text: "die drei Knöpfe", mono: true }, "R U", "wandern mit der rechten unteren Ecke"],
    [{ text: "alles andere", mono: true }, "L O", "die Vorgabe: bleibt, wo es ist"],
  ], { headers: ["Control", "Anker", "beim Vergrößern"], widths: [3000, 1600, 4426] }),

  H.p("Im Entwurf kann man das gleich ausprobieren: Wer an der rechten unteren Ecke des Fensters zieht, sieht die verankerten Controls mitgehen. Im laufenden Programm geschieht dasselbe, wenn man das Fenster größer zieht oder maximiert."),

  H.h2("Schritt 3: Tab-Folge, Hinweise, Knopfart"),

  H.p("Wer ein Formular mit der Tastatur ausfüllt, springt mit Tab von Feld zu Feld. Ohne Angabe geht Tab in der Reihenfolge, in der die Controls angelegt wurden — und das ist selten die, in der man sie ausfüllt. Die Eigenschaft „Tab-Folge“ in der Gruppe Verhalten legt sie fest: Titel 1, Urheber 2, Art 3, Jahr 4, dann Speichern 5, Neu 6, die Tabelle 7 und Entfernen 8. Wer das Jahr eingetippt hat, landet mit einem Tab auf „Speichern“ und mit der Leertaste ist der Eintrag drin."),

  H.p("„Hinweis“ in derselben Gruppe ist der Text, der über einem Control erscheint, wenn die Maus dort einen Augenblick stehen bleibt. Das Jahr bekommt „Vier Ziffern, z. B. 1973“, die Knöpfe sagen, was sie tun. Und „Speichern“ bekommt unter Aussehen die Knopfart „primaer“: Es ist farbig hervorgehoben, weil es der Knopf ist, den man am häufigsten drückt."),

  H.h2("Schritt 4: ein wenig Code"),

  H.p("Damit das Fenster etwas tut, bekommen die drei Knöpfe ihr Ereignis Klick, und das Fenster bekommt Geoeffnet. Der Code ist noch vorläufig — die Einträge stehen nur in der Tabelle; Kapitel 10 gibt ihnen ihre eigene Klasse:"),

  H.code(zeilen("kap09/mediathek/Hauptfenster.dh")),

  H.p("Mit F5 läuft es. Kein GUI_LAYOUT, kein GUI_SET_ANCHOR, keine Zahl für eine Lage: Das alles steht in der .dhform, und das Programm baut sein Fenster beim Start daraus."),

  H.h2("Warum so?"),

  H.h3("Anker statt Behälter — und wann doch Behälter"),

  H.p("In Kapitel 2 hat eine Spalte mit Zeilen und Gewichten das Formular verteilt. Das Ergebnis war dasselbe wie hier: Felder werden breiter, die Tabelle bekommt den Platz, die Knöpfe bleiben in der Ecke. Anker sind dafür der kürzere Weg, wenn man ein Fenster mit der Maus entwirft: Man stellt jedes Control dorthin, wo es stehen soll, und sagt nur noch, an welchen Rändern es hängt. Das reicht für die meisten Fenster einer Anwendung."),

  H.p("Was Anker nicht können, ist auf den Inhalt zu reagieren. Wird „Erscheinungsjahr“ in einer anderen Sprache länger, rückt kein Feld nach rechts — die Lage ist eine Zahl. Dafür hat das Studio die Layouts aus Kapitel 2 auch: In der Bibliothek unter Behälter stehen Zeile, Spalte und Raster, und was man hineinzieht, ordnen sie selbst. Kapitel 22 braucht sie, wenn die Mediathek zweisprachig wird."),

  H.h3("Warum die Tab-Folge eine Zahl ist"),

  H.p("Man könnte die Reihenfolge aus der Lage ableiten — von oben nach unten, von links nach rechts. Bei der Mediathek ginge das schief: „Neu“ steht links von „Speichern“, aber wer ein Formular ausgefüllt hat, will als Nächstes speichern. Die Reihenfolge der Bedienung ist eine Entscheidung, keine Geometrie; darum ist sie eine Eigenschaft."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Feld über der Beschriftung"),

  H.p("Lag das Feld für das Jahr bei 160 statt 170, schnitt es „Erscheinungsjahr“ ab — dieselbe Falle wie in Kapitel 2, Schritt 1. Der Entwurf zeigt es genauso, wie das Programm es zeigen wird; man muss nur hinsehen. Wer eine Beschriftung ändert, sieht danach im Entwurf nach, ob sie noch Platz hat."),

  H.h3("Ein Anker zu wenig"),

  H.p("Fehlt der Tabelle der Anker U, wird sie beim Vergrößern breiter, aber nicht höher. Die Knöpfe wandern trotzdem nach unten, und zwischen Tabelle und Knöpfen entsteht ein leerer Streifen. Fehlt einem Feld der Anker R, bleibt es so breit, wie es im Entwurf war, während das Feld darüber wächst. Eine Meldung gibt es in beiden Fällen nicht; man sieht es, wenn man das Fenster im Entwurf an der Ecke zieht."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Nimm der Tabelle den Anker U und maximiere das Programm. Was ändert sich, was nicht?"),
  H.bulletRich("Drehen 2. ", "Nimm dem Feld Urheber den Anker R. Was siehst du, wenn das Fenster wächst?"),
  H.bulletRich("Drehen 3. ", "Setze die Schreibmarke ins Jahr und drücke Tab. Welches Control bekommt den Fokus — und warum nicht „Neu“, das doch links daneben steht?"),
  H.bulletRich("Erweitern 1. ", "Ein Menü Ansicht mit dem Eintrag Vollbild und dem Kürzel F11 schaltet zwischen der Größe aus dem Entwurf und dem ganzen Bildschirm um. Das Fenster muss auf beiden gut aussehen."),
  H.bulletRich("Erweitern 2. ", "„Entfernen“ ist nur bedienbar, wenn in der Tabelle eine Zeile gewählt ist."),
  H.bulletRich("Erfinden. ", "Entwirf ein Fenster für etwas, das du kennst, nur im Inspektor: jedes Control mit Namen, Anker, Tab-Folge und einem Hinweis, wo einer hilft. Zieh das Fenster im Entwurf groß und klein, bis nichts mehr schief steht."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Der Inspektor: ", "alle Gruppen und Editoren, Enter- und Esc-Knopf."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Fenster entwerfen: ", "Ausrichtungslinien, Anordnen, Layouts und Abstandhalter."),
  H.bulletRich("Handbuch, Modul gui → GUI_SET_ANCHOR: ", "was ein Anker im laufenden Programm genau tut."),
];
