const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Plasma", 2),

  H.h2("Was wir bauen"),

  H.figure("kap02_2_klein.png", "Plasma: Farben, die über das ganze Bild wabern. Jeder Punkt hat seine eigene Farbe, berechnet aus drei Sinuswellen.", 480, 300),

  H.p("Farben, die wie flüssiges Glas über den Bildschirm fließen — das Plasma war in den Demos der Neunzigerjahre der Beweis, dass ein Rechner mehr kann als Tabellen. Es ist ein Bild, in dem jeder einzelne Punkt in jedem Bild neu berechnet wird. Darum handelt dieses Kapitel auch davon, wie schnell ein Programm ist, wie man das misst, und was man tut, wenn es zu langsam ist."),

  H.h2("Der Plan"),

  H.bulletRich("Für jeden Punkt ", "des Bildes eine Zahl v ausrechnen, aus seiner Stelle (x, y) und der Zeit t."),
  H.bulletRich("Aus v ", "eine Farbe machen: v bestimmt den Farbton."),
  H.bulletRich("Mit der Zeit ", "verschiebt sich alles ein wenig, und das Bild fließt."),

  H.p("Die Zahl v ist die Summe von drei Wellen. Eine Welle ist ein SIN: Er schwankt zwischen -1 und 1, und wenn x wächst, geht er auf und ab wie eine Wasserwelle. Drei Wellen in verschiedene Richtungen überlagern sich zu einem Muster aus Bergen und Tälern, und genau dieses Muster färbt das Plasma."),

  H.h2("Schritt 1: Punkt für Punkt"),

  H.p("800 mal 500 Punkte sind 400 000. Zwei Schleifen ineinander gehen sie alle durch, Zeile für Zeile:"),

  H.code(stueck("kap02/1_punktweise.dh", "DIM y AS INTEGER", "NEXT", 2)),

  H.pmix(["Die innere Schleife läuft für jedes y einmal ganz durch, also 500-mal 800 Durchgänge. Jede der drei Wellen hat eine eigene Richtung: ", ["SIN(x / 40 + t)", true], " ändert sich nur von links nach rechts, ", ["SIN(y / 30 - t)", true], " nur von oben nach unten, ", ["SIN((x + y) / 60 + t)", true], " schräg. Die Zahl unter dem Bruchstrich ist die Breite der Welle: Teilt man durch 40, dauert eine volle Welle 2 · π · 40 ≈ 250 Punkte. Und weil t in jedem Bild wächst, wandern alle drei Wellen weiter, die zweite andersherum."]),

  H.pmix(["v liegt zwischen -3 und 3. ", ["HSV(v * 60 + t * 40, 0.8, 1.0)", true], " macht daraus einen Farbton zwischen -180 und 180 Grad, also einmal den ganzen Farbkreis, und ", ["t * 40", true], " dreht alle Farben langsam weiter. ", ["PLOT(x, y, farbe)", true], " setzt einen einzelnen Punkt."]),

  H.pmix(["Noch ein Wort zu den Typen: x und y sind ganze Zahlen (INTEGER), denn sie zählen Punkte. ", ["x / 40", true], " ist trotzdem eine Kommazahl, weil / immer eine liefert (Kapitel 1) — genau das braucht SIN. v ist darum als Kommazahl (FLOAT) angelegt; in einer INTEGER-Variablen hätte das Ergebnis keinen Platz."]),

  H.figure("kap02_1_punktweise.png", "Der erste Stand: dasselbe Muster in voller Schärfe. Oben links steht, wie viele Bilder je Sekunde das Programm schafft.", 420, 262),

  H.h2("Messen statt raten"),

  H.p("Das Programm zeigt oben links, wie viele Bilder es je Sekunde schafft. Auf dem Rechner, mit dem dieses Buch entstand, waren es 16 — es ruckelt sichtbar. Bevor man etwas ändert, lohnt die Frage: Was genau ist langsam? Das Rechnen der Formel oder das Setzen der 400 000 Punkte? Ein kleines Programm ohne Fenster misst das Rechnen allein:"),

  H.code(ohneKopf("kap02/3_messen.dh")),

  H.pmix([["MILLIS()", true], " liefert die Millisekunden seit dem Start; die Differenz vorher und nachher ist die Zeit, die die Schleifen brauchen. Gemessen werden zehn Bilder, damit Zufälle sich ausgleichen. Das Ergebnis auf demselben Rechner: ", ["Ein Bild rechnen: 5 ms", true], ". Ein Bild hat bei sechzig Bildern je Sekunde 16 ms Zeit; die Formel braucht davon ein Drittel. Langsam ist also das Zeichnen: 400 000 einzelne PLOT, jedes ein Auftrag an die Grafikkarte."]),

  H.h3("Der Maschinencode"),

  H.p("Dass die Formel so schnell ist, verdankt sie dem Maschinencode. Drachenhauch übersetzt Schleifen, die viel rechnen, beim Laufen in die Sprache des Prozessors. Man merkt davon nichts, außer dass es schnell ist. Wie viel das ausmacht, lässt sich nachprüfen, denn der Maschinencode lässt sich abschalten. In der Eingabeaufforderung (PowerShell) geht das so:"),

  H.code(["$env:DHRT_JIT = \"aus\"",
          "dhrt run 3_messen.dh"]),

  H.pmix(["Ohne Maschinencode meldet dasselbe Programm ", ["Ein Bild rechnen: 106 ms", true], " — zwanzigmal so lange, mehr als sechs Bilder Zeit für die Rechnung eines einzigen. Auch das Plasma selbst fällt dann von 16 auf 6 Bilder je Sekunde. (", ["$env:DHRT_JIT = \"\"", true], " schaltet ihn wieder ein; ein neues Fenster der Eingabeaufforderung hat ihn ohnehin an.)"]),

  H.h2("Schritt 2: ein kleines Bild"),

  H.p("Wenn 400 000 Punkte zu viele sind, rechnet man weniger. Das Plasma ist ohnehin weich — benachbarte Punkte haben fast dieselbe Farbe. Also rechnet das Programm ein Bild, das in jeder Richtung viermal kleiner ist, 200 × 125 Punkte, und zeigt es viermal so groß. Das sind 25 000 Punkte statt 400 000, ein Sechzehntel."),

  H.code(stueck("kap02/2_klein.dh", "CONST MASSSTAB", "CONST KLEIN_H")),

  H.pmix(["Das kleine Bild ist ein Render-Ziel: eine Zeichenfläche, die nicht auf dem Bildschirm liegt. ", ["RENDERTARGET_NEW", true], " legt es einmal an. Zwischen ", ["RENDERTARGET_BEGIN", true], " und ", ["RENDERTARGET_END", true], " gehen alle Zeichenbefehle hinein statt auf den Bildschirm, und ", ["RENDERTARGET_DRAW", true], " zeigt es danach, mit dem Maßstab als viertem Wert:"]),

  H.code(stueck("kap02/2_klein.dh", "RENDERTARGET_END()", "RENDERTARGET_DRAW(klein, 0, 0, MASSSTAB)")),

  H.p("In der Schleife ändert sich nur eines: Damit das Muster genauso groß bleibt wie vorher, rechnet die Formel mit den Stellen im großen Bild — x · 4 und y · 4 —, setzt den Punkt aber im kleinen. Das ganze Programm:"),

  H.code(zeilen("kap02/2_klein.dh")),

  H.p("Jetzt sind es 59 Bilder je Sekunde, so viele, wie der Bildschirm zeigt — mit und ohne Maschinencode. Der Preis steht im Bild: Wer genau hinsieht, erkennt die Stufen, denn jeder gerechnete Punkt ist jetzt ein Quadrat von vier mal vier Bildpunkten. Beim Plasma fällt das kaum auf, und als Retro-Look ist es sogar gewollt."),

  H.h2("Warum so?"),

  H.h3("Erst einfach, dann messen, dann ändern"),

  H.p("Die erste Fassung ist absichtlich die naheliegende. Sie ist kurz, leicht zu verstehen und zeigt, ob das Muster überhaupt schön ist. Erst als sie zu langsam war, wurde gemessen — und die Messung hat eine Überraschung gebracht: Nicht die Rechnung mit dreimal SIN war das Problem, sondern das Zeichnen. Wer ohne Messung „die Formel schneller gemacht“ hätte, hätte an der falschen Stelle gearbeitet. Das ist die wichtigste Regel über Geschwindigkeit: Man rät fast immer falsch."),

  H.h3("Ein kleines Bild statt eines schnelleren Rechners"),

  H.p("Die schnellste Arbeit ist die, die man nicht tut. Ein Sechzehntel der Punkte heißt ein Sechzehntel der Arbeit, und das ist mehr, als jede Verbesserung an der Formel bringen könnte. Diese Abwägung — etwas weniger Schärfe gegen sehr viel weniger Arbeit — begegnet einem bei Grafik ständig. Es gibt noch einen schnelleren Weg: die Formel gar nicht im Programm rechnen, sondern auf der Grafikkarte, die Millionen Punkte gleichzeitig rechnet. Das ist ein Shader, und Kapitel 15 baut damit das Plasma noch einmal."),

  H.h3("CONST aus einer Rechnung"),

  H.pmix([["CONST KLEIN_B = 800 \\ MASSSTAB", true], " rechnet die Größe des kleinen Bildes aus dem Maßstab aus, statt sie hinzuschreiben. Wer MASSSTAB auf 2 ändert, bekommt ein Bild von 400 × 250, ohne eine zweite Zahl anfassen zu müssen. Der Rückstrich teilt ganzzahlig, denn ein Bild hat eine ganze Zahl von Punkten."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Das Ende vergessen"),

  H.code(zeilen("kap02/fehler_ohne_ende.dh")),
  H.code(["Laufzeitfehler in fehler_ohne_ende.dh:7: FLIP: ein Render-Ziel ist",
          "noch offen -- nach RENDERTARGET_BEGIN fehlt RENDERTARGET_END(). Bis",
          "dahin geht alles Gezeichnete ins Ziel statt auf den Bildschirm"], { out: true }),

  H.pmix(["Ohne ", ["RENDERTARGET_END()", true], " landet alles, was nach dem BEGIN gezeichnet wird — auch ", ["RENDERTARGET_DRAW", true], ", das das kleine Bild zeigen sollte —, im kleinen Bild selbst, und auf dem Bildschirm kommt nichts an. FLIP bemerkt das und hält das Programm mit dieser Meldung an. (Bis zur Fassung 2026.52 blieb das Bild einfach schwarz, ohne ein Wort — gefunden beim Schreiben dieses Kapitels.)"]),

  H.h3("Ein Plasma, das flimmert"),

  H.p("Wer die Zeit nicht aus DELTA() zusammenzählt, sondern sie direkt nimmt, schreibt vielleicht:"),

  H.code(["t = MILLIS()"]),

  H.p("Das Programm läuft, aber statt zu fließen, flackert das Bild wild. MILLIS zählt Millisekunden: Von einem Bild zum nächsten wächst t um etwa 16, und eine Welle braucht nur 2 · π ≈ 6,3, um einmal herum zu sein. Jede Welle macht also zwischen zwei Bildern mehr als zwei ganze Schwingungen, und das Auge sieht nur noch Zufall. t muss in Sekunden zählen; mit MILLIS() / 1000 ginge es auch."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze MASSSTAB auf 2, dann auf 8. Wie sieht es aus, und wie viele Bilder je Sekunde sind es?"),
  H.bulletRich("Drehen 2. ", "Ändere x / 40 in gx / 10 (im zweiten Programm). Welche der drei Wellen ändert sich, und wie?"),
  H.bulletRich("Drehen 3. ", "Setze die Sättigung in HSV von 0.8 auf 0.3. Woran erinnern die Farben?"),
  H.bulletRich("Erweitern 1. ", "Ein Plasma in Feuerfarben: nur Rot, Orange und Gelb, dunkel in den Tälern, hell auf den Bergen."),
  H.bulletRich("Erweitern 2. ", "Das Plasma nur in einem Kreis in der Mitte, außen dunkel."),
  H.bulletRich("Erfinden. ", "Lass die Maus mitspielen: Von ihr könnten Wellen ausgehen wie von einem Stein im Wasser, oder sie verzerrt das Muster, oder sie bestimmt die Farben. Anhang A zeigt die Wellen."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Grafik-Befehle → Render-Ziele: ", "RENDERTARGET_NEW und seine Verwandten, auch zum Speichern als Bild."),
  H.bulletRich("Handbuch, Grafik-Befehle: ", "PLOT, RECT, LINE und die anderen Grundformen."),
];
