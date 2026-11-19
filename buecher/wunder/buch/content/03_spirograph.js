const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Spirograph", 3),

  H.h2("Was wir bauen"),

  H.figure("kap03_spirograph.png", "Der Spirograph mitten beim Zeichnen: Das kleine Rad rollt innen am großen Ring entlang, der weiße Punkt ist der Stift. Seine Bahn wird nach und nach zur Rosette.", 470, 295),

  H.p("Ein Spirograph ist ein Spielzeug aus Zahnrädern: Ein kleines Rad rollt innen in einem großen Ring, durch ein Loch im kleinen Rad steckt ein Stift, und der zeichnet Schleife um Schleife, bis sich ein Muster schließt. Unser Programm macht dasselbe auf dem Bildschirm. Es zeigt die Räder beim Rollen, zeichnet die Kurve Strich für Strich und beginnt, wenn sie geschlossen ist, mit anderen Rädern von vorn."),

  H.p("Die Sprache, um die es geht: SUB und FUNCTION — Unterprogramme, die einen Namen haben, Werte mitbekommen und einen zurückgeben. Und zwei kleine Annehmlichkeiten, die Aufrufe lesbar machen: Vorgaben und benannte Argumente."),

  H.h2("Der Plan"),

  H.p("Wo der Stift steht, hängt von nur einer Zahl ab: wie weit das Rad schon gerollt ist. Diese Zahl heißt t, ein Winkel. Die Mitte des kleinen Rads läuft auf einem Kreis um die Mitte des Rings; weil das Rad innen anliegt, ist dieser Kreis um den Radius des Rads kleiner als der Ring. Und während es herumrollt, dreht sich das Rad selbst — andersherum und schneller, je kleiner es ist. Der Stift sitzt auf dem Rad, also macht er beide Bewegungen zugleich."),

  H.bulletRich("Eine Formel ", "rechnet für jedes t die Stelle des Stifts aus."),
  H.bulletRich("Das Programm ", "lässt t wachsen und verbindet die Stellen mit kurzen Linien."),
  H.bulletRich("Wann die Kurve geschlossen ist, ", "lässt sich vorher ausrechnen."),

  H.h2("Schritt 1: eine Formel bekommt einen Namen"),

  H.p("Die Formel für die Stelle des Stifts ist nicht kurz, und das Programm braucht sie oft. Statt sie jedes Mal hinzuschreiben, bekommt sie einen Namen — als FUNCTION:"),

  H.code(stueck("kap03/1_eine_kurve.dh", "FUNCTION kurveX", "END FUNCTION")),

  H.pmix(["Die erste Zeile sagt, wie die Funktion heißt, welche Werte sie mitbekommt (die Parameter t, ring, rad und stift, alle Kommazahlen) und was sie zurückgibt (", ["AS FLOAT", true], " am Ende). Weil die Zeile sonst zu lang wäre, geht sie innerhalb der Klammern in der nächsten weiter — das darf man überall, wo eine Klammer offen ist. ", ["RETURN", true], " gibt den Wert zurück und beendet die Funktion. ", ["innen", true], " ist eine lokale Variable: Es gibt sie nur in dieser Funktion, und eine Variable gleichen Namens draußen bleibt unberührt."]),

  H.pmix(["In der Formel steckt der Plan: ", ["innen * COS(t)", true], " ist die Mitte des Rads auf ihrem Kreis, ", ["stift * COS(innen / rad * t)", true], " der Stift auf dem Rad, das sich ", ["innen / rad", true], "-mal so schnell dreht. kurveY ist dasselbe mit SIN, und mit einem Minus vor dem zweiten Teil, weil das Rad sich andersherum dreht."]),

  H.p("Gezeichnet wird in einem Zug, mit t von 0 bis zum Ende in kleinen Schritten:"),

  H.code(stueck("kap03/1_eine_kurve.dh", "DIM t AS FLOAT", "NEXT")),

  H.figure("kap03_1_eine_kurve.png", "Der erste Stand: Ring 220, Rad 84, Stift 60, in einem Zug gezeichnet.", 420, 262),

  H.h2("Warum 21?"),

  H.p("Die Schleife läuft bis 2 · π · 21, also über 21 volle Runden des Rads um den Ring. Warum gerade 21? Weil die Kurve dann zum ersten Mal wieder genau dort ankommt, wo sie angefangen hat. Man stelle sich die beiden als Zahnräder vor, das Rad mit 84 Zähnen, den Ring mit 220. Der Stift steht wieder am Anfang, wenn derselbe Zahn des Rads wieder in dieselbe Lücke des Rings greift. Abgerollt ist dann eine Zahl von Zähnen, die zugleich ein Vielfaches von 84 und von 220 ist; die kleinste solche Zahl ist 4620. Das sind 4620 / 220 = 21 Runden um den Ring. Ausrechnen lässt sich das als 84 / ggt(220, 84) — mit 4 als größtem gemeinsamem Teiler kommt wieder 21 heraus."),

  H.p("ggt ist der größte gemeinsame Teiler. Ihn auszurechnen ist eines der ältesten Programme der Welt: Euklid hat es vor über zweitausend Jahren beschrieben. Der größte gemeinsame Teiler von a und b ist derselbe wie der von b und dem Rest, der bleibt, wenn man a durch b teilt. Und wenn kein Rest mehr bleibt, ist man fertig. Als Funktion:"),

  H.code(ohneKopf("kap03/3_ggt.dh")),
  H.code(["ggt(220, 84)", "ggt(84, 52)", "ggt(52, 32)", "ggt(32, 20)", "ggt(20, 12)",
          "ggt(12, 8)", "ggt(8, 4)", "ggt(4, 0)", "Ergebnis: 4"], { out: true }),

  H.pmix(["Die Funktion ruft sich selbst auf — das heißt Rekursion. Das klingt nach einem Kreis ohne Ende, aber jeder Aufruf hat kleinere Zahlen als der vorige, und irgendwann ist b gleich 0. Dann gibt die innerste Funktion a zurück, und jede Runde reicht das Ergebnis nach außen weiter. ", ["a MOD b", true], " ist der Rest der Division. Das Ergebnis 4 ergibt 84 / 4 = 21 Runden — die Zahl aus der Schleife. (Die PRINT-Zeile steht nur in diesem Programm, damit man zusehen kann; im Spirographen fehlt sie.)"]),

  H.h2("Schritt 2: rollende Räder"),

  H.p("Das fertige Programm zeichnet die Kurve nicht in einem Zug, sondern so, wie der Stift sie zieht, und zeigt dabei die Räder. Dafür bekommen die beiden Funktionen eine Vorgabe:"),

  H.code(stueck("kap03/2_spirograph.dh", "FUNCTION kurveX", "END FUNCTION")),

  H.pmix([["ring AS FLOAT = RINGRADIUS", true], " heißt: Wer ring nicht angibt, bekommt 220. Weil der Ring sich im Programm nie ändert, wird er bei keinem Aufruf mehr hingeschrieben. Parameter mit Vorgabe müssen am Ende der Liste stehen; dahinter darf keiner ohne Vorgabe mehr kommen. Aufgerufen wird mit Namen:"]),

  H.code(["DIM sx AS FLOAT : sx = kurveX(t, rad: rad, stift: stift)"]),

  H.pmix([["rad: rad", true], " sieht zunächst doppelt aus, sagt aber, was gemeint ist: Der Parameter rad bekommt den Wert der Variablen rad. Interessant wird es, wenn Zahlen dastehen. ", ["kurveX(t, 84, 60)", true], " — welche ist das Rad, welche der Stift? ", ["kurveX(t, rad: 84, stift: 60)", true], " sagt es, und es dürfte sogar ", ["kurveX(t, stift: 60, rad: 84)", true], " heißen. Dasselbe geht bei eingebauten Befehlen: ", ["CIRCLE(x: 400, y: 250, r: 30)", true], "."]),

  H.p("Dann die Unterprogramme, die etwas tun, statt etwas auszurechnen. Sie heißen SUB und geben nichts zurück:"),

  H.code(stueck("kap03/2_spirograph.dh", "SUB neueKurve", "END SUB")),

  H.p("neueKurve würfelt ein Rad und einen Stift, rechnet mit ggt aus, wann die Kurve fertig ist, und leert die Leinwand. Die Leinwand ist ein Render-Ziel wie im Plasma, aber mit einem dritten Wert beim Anlegen:"),

  H.code(stueck("kap03/2_spirograph.dh", "' TRUE: das Bild bleibt", "leinwand = RENDERTARGET_NEW")),

  H.p("Ohne TRUE wird ein Render-Ziel zu Beginn jedes Bildes geleert, und das ist für das Plasma richtig, das ohnehin alles neu malt. Der Spirograph malt in jedem Bild nur das neue Stück der Kurve dazu; alles Alte muss stehen bleiben. RENDERTARGET_CLEAR leert es dann von Hand, wenn eine neue Kurve beginnt."),

  H.p("In der Schleife wächst t, und für jedes Stück wird eine kurze Linie in die Leinwand gezeichnet:"),

  H.code(stueck("kap03/2_spirograph.dh", "IF t < ende THEN", "END IF")),

  H.p("Ist die Kurve fertig, zählt pause zwei Sekunden herunter, dann beginnt die nächste. Danach kommt die Leinwand auf den Bildschirm und darüber die Räder, die raederZeichnen in jedem Bild neu malt — sie bewegen sich, also gehören sie nicht auf die Leinwand. Das ganze Programm:"),

  H.code(zeilen("kap03/2_spirograph.dh")),

  H.h2("Warum so?"),

  H.h3("FUNCTION für Rechnungen, SUB für Taten"),

  H.p("kurveX und ggt rechnen etwas aus und geben es zurück; sie ändern nichts außerhalb. neueKurve und raederZeichnen tun etwas: Sie setzen Variablen, leeren eine Leinwand, malen. Diese Trennung ist nicht nur Ordnung. Eine Funktion, die nur rechnet, kann man überall aufrufen, so oft man will, ohne dass etwas passiert — man kann sie sogar ausprobieren, wie ggt oben. Ein Unterprogramm, das etwas tut, ruft man genau dann auf, wenn es das tun soll."),

  H.h3("Variablen außen, Werte innen"),

  H.pmix(["Die SUBs benutzen Variablen, die außerhalb angelegt sind: rad, stift, t. Das ist bequem und hier vertretbar, denn das Programm ist klein und hat genau eine Kurve. Die Funktionen dagegen bekommen alles, was sie brauchen, als Parameter. Dadurch zeichnet ", ["kurveX(t, rad: 84, stift: 60)", true], " auch eine andere Kurve als die gerade laufende — im Anhang braucht die Lösung mit drei Rädern genau das. Wächst ein Programm, wandern auch die Variablen der SUBs in Parameter oder, ab Teil VI, in eine Klasse."]),

  H.h3("Kleine Schritte"),

  H.pmix(["Die Kurve wird aus geraden Stücken zusammengesetzt, je 0,02 lang in t. Größere Schritte wären schneller, aber an engen Schleifen sähe man die Ecken. Und ", ["bis = MIN(t + TEMPO * dt, ende)", true], " sorgt dafür, dass die letzte Linie genau am Ende aufhört — sonst zeichnete der Stift ein Stück über den Anfang hinaus, und an der Naht wäre eine Linie doppelt."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Die Kurve verschwindet"),

  H.p("Fehlt das TRUE bei RENDERTARGET_NEW, sieht man die Räder rollen, aber keine Kurve — höchstens einen winzigen Strich am Stift, der gleich wieder weg ist. Die Leinwand wird zu Beginn jedes Bildes geleert, und es bleibt nur, was in diesem Bild dazukam. Eine Meldung gibt es nicht, denn es ist kein Fehler: So ist ein Render-Ziel gemeint, wenn man nichts anderes sagt."),

  H.h3("Zu viele Argumente"),

  H.code(zeilen("kap03/fehler_argumente.dh")),
  H.code(["fehler_argumente.dh:7: Compile-Fehler: flaeche: zu viele Argumente (max 2)"], { out: true }),

  H.p("Bemerkenswert ist, was nicht passiert: Auch die drei richtigen PRINT davor laufen nicht. Drachenhauch übersetzt das ganze Programm, bevor es die erste Zeile ausführt, und ein Fehler beim Übersetzen hält alles an. In der IDE steht er schon vor dem Start in der Problemliste. Die drei anderen Aufrufe zeigen nebenbei, was erlaubt ist: beide Werte, nur den ersten (die Höhe ist dann 1,0), oder beide mit Namen in beliebiger Reihenfolge."),

  H.h3("Ein Name, den es nicht gibt"),

  H.code(zeilen("kap03/fehler_name.dh")),
  H.code(["fehler_name.dh:4: Compile-Fehler: flaeche: kein Parameter 'hohe'"], { out: true }),

  H.p("Ein vertippter Name eines Arguments fällt ebenso vor dem Start auf. Anders als bei einem vertippten Befehl schlägt die Meldung hier keinen ähnlichen Namen vor; ein Blick auf die erste Zeile der Funktion hilft."),

  H.h3("Ein Parameter verdeckt eine Konstante"),

  H.p("Der heimtückischste Fall bricht gar nicht ab:"),

  H.code(zeilen("kap03/fehler_verdeckt.dh")),
  H.code(["31.41592653589793"], { out: true }),

  H.p("Dazu, vor dem Start, diese Warnung (und eine zweite zur Schreibweise):"),

  H.code(["fehler_verdeckt.dh:2: Warnung: Parameter 'radius' verdeckt die",
          "gleichnamige Konstante -- Drachenhauch ignoriert",
          "Gross-/Kleinschreibung. Innerhalb dieser Funktion meint der Name den",
          "Parameter, nicht die Konstante."], { out: true }),

  H.p("Gemeint war vielleicht der Umfang des großen Kreises, 2 · π · 100 ≈ 628. Ausgegeben wird 31,4. Weil Drachenhauch Groß- und Kleinschreibung nicht unterscheidet, sind RADIUS und radius derselbe Name, und in der Funktion gewinnt der Parameter. Die Warnung sagt es; wer sie liest, benennt eines von beiden um. Darum heißt die Konstante im Spirographen RINGRADIUS und der Parameter ring."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Im ersten Programm: Setze den Stift auf 120, weiter vom Mittelpunkt weg, als das Rad groß ist. Was ändert sich an der Kurve? Und mit Stift 0?"),
  H.bulletRich("Drehen 2. ", "Setze das Rad auf 110 — halb so groß wie der Ring — und den Stift auf 60. Was zeichnet der Stift? Und mit Stift 110? Sage es vorher."),
  H.bulletRich("Drehen 3. ", "Setze im zweiten Programm die Schrittweite (t + 0.02) auf t + 0.5. Was siehst du? Ist die Kurve am Ende trotzdem geschlossen, und warum?"),
  H.bulletRich("Erweitern 1. ", "Die Kurve soll beim Zeichnen langsam die Farbe wechseln."),
  H.bulletRich("Erweitern 2. ", "Zwei Regler stellen Rad und Stift ein, jede Änderung fängt die Kurve neu an. Hinweis: das gui-Modul mit GUI_WINDOW, GUI_SLIDER, GUI_VALUE und GUI_CHANGED; GUI_UPDATE und GUI_DRAW gehören in die Schleife."),
  H.bulletRich("Erfinden. ", "Drei Räder statt zwei, eins auf dem anderen. Oder ein Rad, das außen am Ring rollt. Oder ein Spirograph, bei dem man den Stift mit der Maus setzt. Anhang A zeigt drei Räder."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Funktionen und Unterprogramme: ", "Parameter, Vorgaben, benannte Argumente, BYREF und Rekursion."),
  H.bulletRich("Handbuch, Grafik-Befehle → Render-Ziele: ", "keep, RENDERTARGET_CLEAR und Nachzieheffekte."),
];
