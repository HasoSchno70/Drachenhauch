const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Leuchten", 15),

  H.h2("Was wir bauen"),

  H.figure("kap15_2_neon.png", "Leuchtspuren: Fünf Lichter ziehen Bahnen über einen dunklen Schirm, glühen wie Neonröhren und verblassen hinter sich. Mit der Maus malt man selbst mit Licht.", 470, 294),

  H.p("Fünf Lichter ziehen Schleifen über einen dunklen Schirm. Sie glühen wie Neonröhren, ihr Schein fällt weich in die Umgebung, und hinter ihnen verblasst die Spur langsam. Mit gedrückter Maustaste malt man selbst mit Licht. Vorher bauen wir das Plasma aus Kapitel 2 noch einmal — so, wie Kapitel 2 es versprochen hat: gerechnet von der Grafikkarte."),

  H.h2("Der Plan"),

  H.bulletRich("Ein Shader ", "ist ein kleines Programm, das die Grafikkarte für jeden Bildpunkt einmal ausführt — für alle zugleich."),
  H.bulletRich("POSTFX ", "schickt jedes fertige Bild durch einen Shader, bevor es auf den Schirm kommt."),
  H.bulletRich("Ein Zeichenblatt, ", "das stehen bleibt und in jedem Schritt etwas dunkler wird, lässt Spuren nachziehen."),
  H.bulletRich("Ein zweiter Shader ", "lässt alles Helle glühen."),

  H.h2("Schritt 1: das Plasma auf der Grafikkarte"),

  H.p("Ein Shader wird nicht in Drachenhauch geschrieben, sondern in GLSL, der Sprache der Grafikkarten. Sie sieht anders aus, ist aber nicht schwer zu lesen. Hier ist die Plasma-Formel aus Kapitel 2 als Shader, in einer eigenen Datei plasma.fs:"),

  H.code(zeilen("kap15/plasma.fs")),

  H.p("Was neu ist:"),

  H.bulletRich("Jede Anweisung ", "endet mit einem Semikolon, und Blöcke stehen in geschweiften Klammern statt zwischen FUNCTION und END FUNCTION. Kommentare beginnen mit //."),
  H.bulletRich("Kommazahlen ", "schreibt man mit Punkt und Nachkommastelle: 40.0, nicht 40. GLSL ist noch strenger als Drachenhauch und rechnet eine ganze Zahl nicht mit einer Kommazahl zusammen."),
  H.bulletRich("vec2, vec3, vec4 ", "sind Paare, Dreier und Vierer von Zahlen, wie VEC2 aus Kapitel 9. Eine Farbe ist ein vec4: Rot, Grün, Blau und Deckkraft, jeweils von 0 bis 1."),
  H.bulletRich("main ", "läuft für jeden Bildpunkt einmal. fragTexCoord sagt, welcher es ist (x und y von 0 bis 1); was in finalColor steht, ist seine Farbe."),
  H.bulletRich("uniform ", "heißen die Werte, die das Programm von außen hineingibt — für alle Bildpunkte gleich. Hier die Zeit und die Größe des Bildes."),

  H.p("Und eine Eigenheit: Die Grafikkarte zählt y von unten nach oben. Darum rechnet der Shader 1.0 − fragTexCoord.y; sonst stünde das Plasma auf dem Kopf. Die Funktion hsv macht dasselbe wie HSV im Programm, nur in drei Zeilen Rechnung statt einem Befehl."),

  H.p("Das Programm dazu ist kurz:"),

  H.code(zeilen("kap15/1_plasma.dh")),

  H.figure("kap15_1_plasma.png", "Das Plasma aus Kapitel 2, diesmal in voller Schärfe: Jeder der 400 000 Bildpunkte bekommt in jedem Bild seine eigene Farbe.", 420, 262),

  H.pmix([["SHADER_LOAD", true], " liest die Datei und übersetzt den Shader für die Grafikkarte; die Zahl, die es liefert, ist sein Name für alle weiteren Befehle. ", ["SHADER_SET2", true], " gibt dem uniform groesse zwei Werte, ", ["SHADER_SET", true], " dem uniform zeit einen — das zweite in jedem Bild neu, darum bewegt sich das Plasma. ", ["POSTFX(plasma)", true], " sagt: Jedes fertige Bild geht ab jetzt durch diesen Shader. Gezeichnet wird nichts, CLS genügt. Der Shader schaut sich das Bild gar nicht an, er malt einfach jeden Punkt neu."]),

  H.pmix(["Wie schnell ist das? code/kap15/3_messen.dh zeichnet 300 Bilder, ohne auf den Bildschirm zu warten — ", ["SETFPS(0)", true], " schaltet das Warten ab:"]),

  H.code(stueck("kap15/3_messen.dh", "DIM start", "PRINT \"Bilder")),
  H.code(["Ein Bild: 0.50 ms", "Bilder je Sekunde: 2013"], { out: true }),

  H.p("Auf dem Rechner dieses Buchs braucht ein Bild mit Plasma 0,5 Millisekunden, ein leeres Bild ohne Shader 0,45. Die Rechnung selbst kostet also etwa 0,05 Millisekunden. In Kapitel 2 brauchte dieselbe Formel 5 Millisekunden — nur fürs Rechnen, mit Maschinencode — und mit den 400 000 PLOT schaffte das Programm 16 Bilder je Sekunde. Eine Grafikkarte in einem einfachen Laptop ist langsamer als die dieses Rechners, aber auch dort ist das Plasma eine Kleinigkeit."),

  H.h2("Schritt 2: Spuren, die nachziehen"),

  H.p("Jetzt die Leuchtspuren. Die Lichter zeichnen in ein Render-Ziel, wie die Spirographen in Kapitel 3 — ein Zeichenblatt, das zwischen den Bildern stehen bleibt:"),

  H.code(stueck("kap15/2_neon.dh", "' Ein Zeichenblatt", "DIM blatt")),

  H.p("Beim Spirographen sollte alles für immer stehen bleiben. Hier soll es verblassen: In jedem Schritt wird das ganze Blatt etwas dunkler. Dafür gibt es einen Mischmodus, der nicht übermalt, sondern malnimmt:"),

  H.code(stueck("kap15/2_neon.dh", "' Je Schritt bleibt", "END SUB")),

  H.pmix([["BLEND_MODE(\"mult\")", true], " heißt: Die Farbe, die jetzt gezeichnet wird, wird mit der malgenommen, die schon da ist. Ein Kasten in RGB(251, 251, 251) nimmt jede Farbe mit 251/255 mal, also etwa 0,985 — ein wenig dunkler. Wie viel genau, rechnet faktor aus: Nach 60 Schritten, einer Sekunde, soll REST übrig sein. 0,4 hoch 1/60 ist die Zahl, die sechzigmal mit sich selbst malgenommen 0,4 ergibt. Das zweite, ", ["BLEND_MODE(\"subtract\")", true], ", zieht noch 1 von jeder Farbe ab; warum, steht unter „Was schiefgehen kann“. ", ["BLEND_MODE(\"alpha\")", true], " stellt danach wieder normales Malen ein."]),

  H.p("Verblasst wird in festen Schritten, wie der Sand in Kapitel 7 und die Kiste in Kapitel 10 rechnen:"),

  H.code(stueck("kap15/2_neon.dh", "RENDERTARGET_BEGIN(blatt)", "WEND")),

  H.p("Auf einem Bildschirm mit 144 Bildern je Sekunde würde das Blatt sonst mehr als doppelt so oft dunkler, und die Spuren wären kürzer. So läuft das Verblassen 60-mal je Sekunde, egal wie schnell der Rechner zeichnet. Dann kommen die Lichter dazu, jedes als Strich von dort, wo es vorhin war, bis dorthin, wo es jetzt ist:"),

  H.code(stueck("kap15/2_neon.dh", "' Die Lichter:", "NEXT")),

  H.p("bahn_x und bahn_y sind Lissajous-Figuren: zwei Sinusschwingungen mit verschiedenen Takten, eine für x, eine für y. Jedes Licht hat etwas andere Takte; darum laufen sie nie im Gleichschritt."),

  H.h2("Schritt 3: Glühen"),

  H.figure("kap15_ohne_gluehen.png", "Dieselben Spuren ohne Glühen (Taste G): scharfe, flache Striche.", 420, 262),

  H.p("Ohne Shader sind die Spuren saubere Striche. Leuchten tun sie nicht — dazu fehlt der Schein, den eine echte Leuchtröhre in ihre Umgebung wirft. Den rechnet ein zweiter Shader, glow.fs:"),

  H.code(zeilen("kap15/glow.fs")),

  H.pmix(["Neu ist ", ["texture0", true], ": das fertige Bild, so wie das Programm es gezeichnet hat. ", ["texture(texture0, ort)", true], " liest die Farbe an einer Stelle. Der Shader liest für jeden Bildpunkt seine 81 Nachbarn im Abstand von je 3 Bildpunkten und mittelt sie — die nahen zählen mehr, die fernen weniger; exp sorgt dafür, dass das Gewicht nach außen weich abnimmt. Das Ergebnis ist ein weichgezeichnetes Bild. Das kommt, mal staerke, zum scharfen Bild dazu: Wo nichts ist, bleibt es dunkel; neben einer hellen Linie wird es hell."]),

  H.p("Für jedes Bild liest die Grafikkarte damit 400 000 · 81 = 32 Millionen Mal eine Farbe. Im Programm wäre das nicht zu machen; für die Grafikkarte ist es Alltag. Das Programm lädt den Shader wie das Plasma:"),

  H.code(stueck("kap15/2_neon.dh", "DIM glow AS INTEGER", "POSTFX(glow)")),

  H.pmix(["Die Taste G schaltet ihn um: ", ["POSTFX(IIF(gluehen, glow, -1))", true], " — −1 heißt kein Shader. Das ganze Programm:"]),

  H.code(zeilen("kap15/2_neon.dh")),

  H.h2("Warum so?"),

  H.h3("Viele kleine Rechner"),

  H.p("Ein Prozessor hat einige wenige Kerne, jeder sehr schnell und sehr klug: Er kann springen, warten, Entscheidungen treffen und sich alles merken. Eine Grafikkarte hat Tausende Rechenwerke, jedes einzelne langsamer und einfacher, aber alle zugleich. Sie bekommen dasselbe Programm und jeder einen anderen Bildpunkt. Für das Plasma ist das ideal: Jeder Punkt wird nach derselben Formel gerechnet, und keiner braucht das Ergebnis eines anderen."),

  H.h3("Was ein Shader nicht kann"),

  H.p("Diese Arbeitsteilung hat einen Preis. Ein Shader kann nur seinen eigenen Punkt schreiben, nicht den daneben. Er kann lesen, was im Bild steht, aber nicht, was er selbst gerade rechnet. Und er vergisst alles, sobald das Bild fertig ist. Das Spiel des Lebens aus Kapitel 6 oder der Sand aus Kapitel 7 lassen sich darum nicht einfach als Shader schreiben: Dort hängt jeder Punkt vom vorigen Zustand seiner Nachbarn ab, und der Sand fällt Korn für Korn in einer Reihenfolge. Das Gedächtnis der Leuchtspuren — das Blatt, das langsam verblasst — liegt deshalb im Programm, in einem Render-Ziel. Der Shader macht nur das, was er am besten kann: auf jedes Bild dieselbe Rechnung anwenden."),

  H.h3("Werte von außen"),

  H.p("Ein uniform ist für alle Bildpunkte gleich und ändert sich höchstens einmal je Bild. Genau das ist die Zeit, die Größe des Fensters, die Stärke des Glühens. Alles, was sich von Punkt zu Punkt ändert, rechnet der Shader selbst aus fragTexCoord. Diese Trennung — wenige Werte vom Programm, die Fleißarbeit auf der Grafikkarte — ist der Grund, warum Shader so schnell sind."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Name in anderer Schreibweise"),

  H.code(zeilen("kap15/fehler_uniform.dh")),
  H.code(["fehler_uniform.dh:8: Warnung: SHADER_SET: der Shader hat kein",
          "uniform 'Zeit' -- der Wert kommt nicht an. Meintest du zeit? Im",
          "Shader zaehlt Gross/klein."], { out: true }),

  H.p("In Drachenhauch ist Zeit dasselbe wie zeit. In GLSL nicht. Das Programm läuft trotzdem, aber das Plasma steht still, denn sein uniform zeit bekommt nie einen Wert. Beim Schreiben dieses Kapitels gab es dazu noch keine Meldung — man sah nur ein Bild, das sich nicht bewegte. Seitdem warnt SHADER_SET einmal je Name. Ähnlich bei einem vertippten Dateinamen: SHADER_LOAD(\"plasm.fs\") ist ein Laufzeitfehler, der den Namen und den Ordner nennt, in dem gesucht wurde."),

  H.h3("Ein Fehler im Shader"),

  H.code(zeilen("kap15/fehler_glsl.fs")),
  H.code(zeilen("kap15/fehler_glsl.dh")),
  H.code(["WARNING: SHADER: [ID 4] Failed to compile fragment shader code",
          "WARNING: SHADER: [ID 4] Compile error: 0(8) : error C0000: syntax",
          "error, unexpected identifier, expecting ',' or ';' at token",
          "\"finalColor\"",
          "SHADER_LOAD lieferte -1"], { out: true }),

  H.p("In Zeile 7 fehlt das Semikolon. Den Shader übersetzt nicht Drachenhauch, sondern der Treiber der Grafikkarte; die Meldung kommt von ihm, auf Englisch, und sieht auf jeder Grafikkarte etwas anders aus. Diese hier stammt von einer NVIDIA-Karte. 0(8) heißt Zeile 8: Der Übersetzer merkt das fehlende Semikolon erst beim nächsten Wort, finalColor. SHADER_LOAD liefert dann −1 statt eines Shaders, und POSTFX(−1) schaltet nur ab — das Programm läuft mit einem Bild ohne Shader weiter. Darum prüft 1_plasma.dh, ob das Laden geklappt hat."),

  H.h3("Spuren, die nie ganz verschwinden"),

  H.figure("kap15_geist.png", "Verblassen nur mit „mult“: Jede Bahn, die die Lichter einmal gezogen haben, bleibt als dunkelgraue Spur stehen, und das Glühen macht sie noch deutlicher.", 420, 262),

  H.p("Ohne die Zeile mit subtract sieht das Bild nach einer Weile so aus. Der Grund ist die Rundung: Eine Farbe wird als ganze Zahl von 0 bis 255 gespeichert. 31 · 251/255 ist 30,5 und wird wieder 31. Ab da wird die Spur nicht mehr dunkler, egal wie oft man malnimmt — gemessen blieb sie bei 31 von 255 stehen, 12 Prozent Helligkeit. 1 abziehen bringt sie über diese Schwelle: Was fast schwarz ist, wird ganz schwarz."),

  H.p("Auch dazu gab es beim Schreiben einen Fund: subtract rechnete früher Farbe minus Hintergrund statt Hintergrund minus Farbe und machte das Blatt dabei durchsichtig. Jede Spur war nach einem Bild weg. Seitdem zieht es ab, was man erwartet."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze REST auf 0.9 und dann auf 0.05."),
  H.bulletRich("Drehen 2. ", "Setze staerke (im Programm) auf 0 und dann auf 8. Ändere in glow.fs die 3.0 vor punkt auf 1.0 und auf 8.0."),
  H.bulletRich("Drehen 3. ", "Schreibe in plasma.fs fragTexCoord.y statt 1.0 − fragTexCoord.y. Was ändert sich, und woran sieht man es?"),
  H.bulletRich("Erweitern 1. ", "Ein alter Röhrenbildschirm: ein Shader, der das Bild leicht wölbt, jede zweite Zeile dunkler macht und zu den Ecken hin abdunkelt. G schaltet zwischen Glühen und Röhre um."),
  H.bulletRich("Erweitern 2. ", "Ein Glühen im Takt: Eine tiefe Trommel schlägt zweimal je Sekunde, und mit jedem Schlag glühen die Spuren auf. AUDIO_NOTE macht den Ton, PLAYSOUND spielt ihn, und AUDIO_PEAK() sagt, wie laut es gerade ist — ein Vorgriff auf die Kapitel über Klang."),
  H.bulletRich("Erfinden. ", "Ein Shader, der Bilder nicht erzeugt, sondern umfärbt: Die Lichter malen weiß, und der Shader färbt jeden hellen Punkt mit dem Plasma ein. Oder: ein Kaleidoskop, das das Bild sechsfach spiegelt; Wasser, das das Bild wellt; ein Bild in nur vier Farben. Anhang A zeigt das Plasma."),

  H.figure("kap15_plasmaspur.png", "Eine Möglichkeit für die Erfinden-Übung: Die Spuren sind weiß gemalt, die Farbe kommt aus dem Plasma darunter.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Grafik-Befehle → Shader und Post-Processing: ", "alle SHADER_SET-Befehle, POSTFX mit kleinerer Auflösung für schwache Grafikkarten, ein zweites Bild für den Shader (SHADER_SET_TEXTURE)."),
  H.bulletRich("Handbuch, Grafik-Befehle → Render-Ziele und Blend-Modi: ", "keep, RENDERTARGET_CLEAR und die vier Mischmodi."),
  H.bulletRich("examples/86_postfx_shaders.dh: ", "drei fertige Shader — Röhre, Bloom (das Glühen mit Schwelle) und Vignette — über einer 3D-Szene."),
];
