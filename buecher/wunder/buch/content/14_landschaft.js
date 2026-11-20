const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Wolken und Landschaften", 14),

  H.h2("Was wir bauen"),

  H.figure("kap14_2_karte.png", "Eine Karte aus Rauschen: Meer, Strände, Wiesen, Wälder, Fels und ein Gipfel mit Schnee — kein Punkt ist von Hand gesetzt, und die Taste R würfelt eine ganz neue Gegend.", 470, 294),

  H.p("Eine Landkarte, wie sie in Strategiespielen und Abenteuern vorkommt: Kontinente und Inseln, Buchten und Seen, Berge mit Schnee. Jeder Druck auf R erfindet eine neue. Dahinter steckt eine besondere Art Zufall, die nicht flimmert, sondern Formen bildet — Rauschen. Mit derselben Zutat entstehen Wolken, Marmor, Feuer und Wasserflächen."),

  H.h2("Der Plan"),

  H.bulletRich("Rauschen ", "liefert für jeden Punkt der Ebene eine Zahl; benachbarte Punkte bekommen ähnliche Zahlen."),
  H.bulletRich("Die Zahl ", "ist die Höhe des Geländes an diesem Punkt."),
  H.bulletRich("Ein Farbverlauf ", "macht aus der Höhe eine Farbe: unten Wasser, dann Strand, Wiese, Wald, Fels, Schnee."),

  H.h2("Schritt 1: Zufall, der zusammenhängt"),

  H.p("RND liefert bei jedem Aufruf eine neue Zahl, die mit der vorigen nichts zu tun hat. NOISE2 dagegen bekommt einen Punkt (x, y) und liefert eine Zahl zwischen etwa −1 und 1 — und für zwei Punkte, die nahe beieinander liegen, zwei ähnliche Zahlen. Das Programm zeigt beide nebeneinander, jeden Wert als Grau:"),

  H.code(stueck("kap14/1_rauschen.dh", "SUB rechnen", "END SUB")),

  H.figure("kap14_1_rauschen.png", "Links RND, jeder Punkt für sich: ein Flimmern wie auf einem Fernseher ohne Sender. Rechts NOISE2: weiche Hügel und Täler aus Grau.", 420, 262),

  H.pmix(["Der Faktor 0.04 ist der Maßstab: Er sagt, wie schnell man sich über die Rauschfläche bewegt. ", ["NOISE2(x * 0.04, y * 0.04)", true], " geht alle 25 Bildpunkte eine Einheit weiter, und eine Einheit ist etwa ein Hügel. NOISE2 ist außerdem verlässlich: Derselbe Punkt liefert immer dieselbe Zahl, heute und morgen, auf jedem Rechner."]),

  H.h2("Schritt 2: eine Karte"),

  H.p("Ein einziges Rauschen gibt sanfte, runde Hügel — zu glatt für eine Küste. FBM legt mehrere Rauschen übereinander: eines mit großen Formen, darüber eines halb so groß und halb so stark, darüber eines noch einmal halb so groß, und so weiter. So entstehen große Kontinente mit zerklüfteten Rändern. Die dritte Zahl sagt, wie viele Lagen:"),

  H.code(stueck("kap14/2_karte.dh", "' Eine Gegend:", "END SUB")),

  H.pmix(["Die Höhen kommen in ein Feld, ", ["gelaende", true], " — erst die Zahlen, dann das Bild. So muss das Programm die Höhen nicht noch einmal rechnen, wenn es sie anders zeichnen will, und eine Übung kann jede Höhe mit ihren Nachbarn vergleichen. Eine neue Gegend ist dieselbe Rauschfläche, an einer anderen Stelle angeschaut: R wählt einen zufälligen Versatz (ox, oy)."]),

  H.p("Dann der Farbverlauf. An bestimmten Höhen stehen bestimmte Farben — Stufen —, und dazwischen wird gemischt:"),

  H.code(stueck("kap14/2_karte.dh", "' Der Farbverlauf:", "stufe(0.45")),
  H.code(stueck("kap14/2_karte.dh", "' Die Farbe zu einer Hoehe", "END FUNCTION")),

  H.pmix(["farbe sucht die beiden Stufen, zwischen denen die Höhe liegt, und rechnet aus, wie weit sie zwischen ihnen liegt: t = 0 an der unteren Stufe, 1 an der oberen. ", ["LERP(a, b, t)", true], " mischt dann: a + (b − a) · t, für Rot, Grün und Blau einzeln. Bei 0 ist Strand, aber schon bei −0,02 flaches Wasser — darum ist die Küste ein schmaler heller Saum."]),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap14/2_karte.dh")),

  H.h2("Warum so?"),

  H.h3("Zufall, der zusammenhängt"),

  H.p("Eine Landschaft aus RND wäre das linke Bild aus Schritt 1: jeder Punkt eine eigene Höhe, kein Berg, kein Tal. Wirklichkeit hängt zusammen — neben einem hohen Punkt liegt ein fast so hoher. Rauschen hat genau diese Eigenschaft und ist trotzdem unvorhersehbar. Es wurde 1983 von Ken Perlin für Filmeffekte erfunden; seitdem steckt es in fast jeder Landschaft, jeder Wolke und jedem Feuer, das ein Rechner zeigt."),

  H.h3("Mehrere Lagen"),

  H.p("Echte Küsten sind auf jeder Größe zerklüftet: große Buchten, darin kleine Buchten, darin Felsen. FBM baut genau das nach — jede Lage halb so groß und halb so stark wie die vorige. Die großen Lagen bestimmen, wo Land ist; die kleinen machen die Ränder rau. Sechs Lagen reichen hier: Die siebte wäre kleiner als ein Bildpunkt."),

  H.h3("Erst die Zahlen, dann das Bild"),

  H.p("Das Programm hätte die Farbe auch direkt beim Rechnen setzen können. Mit dem Feld gelaende sind Gelände und Aussehen getrennt: dieselben Höhen können als Karte, als Höhenlinien oder als Graubild gezeichnet werden, und ein Spiel kann in ihnen nachschauen, ob ein Punkt im Wasser liegt. Das Feld ist ein Puffer — ein Stück Speicher, das ein Bild beschreibt, ohne eines zu sein."),

  H.h2("Was schiefgehen kann"),

  H.h3("Eine Kommazahl in einem INTEGER"),

  H.code(zeilen("kap14/fehler_ganzzahl.dh")),
  H.code(["Laufzeitfehler in fehler_ganzzahl.dh:4: Zuweisung an 'g': FLOAT",
          "143.875 passt nicht verlustfrei in INTEGER -- fuer ganzzahlige",
          "Division \\ statt / nehmen, sonst mit INT()/ROUND() runden"], { out: true }),

  H.p("Rauschen liefert Kommazahlen; ein Grauwert ist eine ganze Zahl. Drachenhauch rundet nicht heimlich, weil es nicht weiß, ob abrunden oder runden gemeint ist. Schon vor dem Start warnt es, und die Zeile bricht ab, sobald der Wert nicht ganz ist — bei manchen Punkten also, bei anderen nicht. Die Abhilfe steht in der Meldung: ROUND(128 + v * 127). (RGB dagegen nimmt Kommazahlen und rundet selbst; darum geht farbe ohne ROUND.)"),

  H.h3("Ein Name zweimal"),

  H.code(zeilen("kap14/fehler_hoehe.dh")),
  H.code(["fehler_hoehe.dh:4: Compile-Fehler: Namens-Kollision: 'hoehe' ist",
          "zugleich eine Variable und eine CONST -- das geht nicht, weil",
          "Drachenhauch Gross-/Kleinschreibung ignoriert. Benenne eines um."], { out: true }),

  H.p("HOEHE ist die Höhe des Fensters, hoehe sollte die Höhe des Geländes sein — für Drachenhauch ist es derselbe Name. Beim Schreiben dieses Kapitels ist genau das passiert; darum heißt das Feld jetzt gelaende. Eine Stufe stiller ist dieselbe Falle mit einem Parameter h neben CONST H: Er verdeckt die Konstante in seiner Funktion, ohne Fehler. Darum heißen die Parameter in 2_karte.dh wert."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze MASSSTAB auf 0.03 und dann auf 0.002."),
  H.bulletRich("Drehen 2. ", "Setze LAGEN auf 1 und dann auf 10."),
  H.bulletRich("Drehen 3. ", "Hebe das ganze Land um 0.1 an (in vermessen: … + 0.1). Was wird aus dem Meer?"),
  H.bulletRich("Erweitern 1. ", "Eine Insel: Zum Rand des Bildes hin soll das Land unter den Meeresspiegel sinken, so dass in der Mitte eine große Insel liegt."),
  H.bulletRich("Erweitern 2. ", "Höhenlinien: Wo zwischen zwei benachbarten Punkten eine bestimmte Höhe überschritten wird, ein dunkler Punkt."),
  H.bulletRich("Erfinden. ", "Wolken, die über die Karte ziehen. Oder: Flüsse, die vom Gebirge bergab ins Meer laufen; eine Karte, die mit der Maus verschoben wird; Marmor, Holzmaserung oder Feuer aus Rauschen. Anhang A zeigt die Wolken."),

  H.figure("kap14_wolken.png", "Eine Möglichkeit für die Erfinden-Übung: Wolken aus einem zweiten Rauschen ziehen durchsichtig über die Karte.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Standard-Built-ins → Math: ", "NOISE, NOISE2, NOISE3, FBM und FBM3 — mit NOISE3 und der Zeit als dritter Zahl wird ein Rauschen lebendig."),
];
