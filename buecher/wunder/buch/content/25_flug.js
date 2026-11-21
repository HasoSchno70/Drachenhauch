const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Flug über eine Landschaft", 25),

  H.h2("Was wir bauen"),

  H.figure("kap25_1_flug.png", "Ein Flug über Berge aus Rauschen: Wald in den Tälern, Schnee auf den Gipfeln, die Ferne verschwimmt im Dunst unter einem blauen Himmel.", 470, 294),

  H.p("In Kapitel 14 haben wir aus Rauschen Landkarten gemacht, von oben gesehen. Jetzt fliegen wir hinein. Die Höhen aus dem Rauschen werden zu Bergen, über die man mit den Pfeiltasten steuert: links und rechts lenken, hoch und runter steigen und sinken, die Leertaste gibt Gas. In den Tälern wächst Wald, eintausendfünfhundert Tannen, auf den Gipfeln liegt Schnee, und weiter hinten verschwimmt alles im Dunst unter einem Himmel, der oben tiefblau ist. Neu sind drei Dinge: ein Gelände aus einem Bild, ein Himmel aus sechs Bildern — und eine Art, sehr viele gleiche Dinge mit einem einzigen Befehl zu zeichnen."),

  H.h2("Der Plan"),

  H.bulletRich("Die Höhen: ", "Rauschen, ein Wert je Punkt einer Karte."),
  H.bulletRich("Das Gelände: ", "die Höhen als Graubild, daraus ein Modell, darüber ein Farbbild."),
  H.bulletRich("Der Wald: ", "jeder Baum eine Matrix, alle zusammen ein Aufruf."),
  H.bulletRich("Der Himmel: ", "sechs Bilder um die Welt herum."),
  H.bulletRich("Der Flug: ", "ein Kurs, ein Tempo, und nie zu tief."),

  H.h2("Schritt 1: die Höhen"),

  H.code(stueck("kap25/1_flug.dh", "' Die Hoehen, ein Wert", "NEXT", 2)),

  H.pmix(["Wie in Kapitel 14 gibt ", ["FBM", true], " zu jedem Punkt einer Karte von 128 mal 128 eine Höhe aus mehreren Lagen Rauschen. Gemessen liegen die Werte hier etwa zwischen −0,3 und 0,3; das Programm schiebt sie auf 0 bis 1 und nimmt sie dann ins Quadrat. Kleine Werte werden dadurch noch kleiner, große bleiben groß: Die Täler werden breit und flach, die Gipfel spitz — so sehen Gebirge aus. Die Werte stehen in einem zweidimensionalen Feld ", ["hoehe", true], ", denn wir brauchen sie gleich noch zweimal: für das Gelände und für den Wald."]),

  H.h2("Schritt 2: das Gelände"),

  H.code(stueck("kap25/1_flug.dh", "' Zwei Bilder aus den Hoehen", "MODEL_TEXTURE")),

  H.pmix(["Aus den Höhen werden zwei Bilder. Im grauen ist jeder Punkt so hell, wie die Stelle hoch ist. ", ["MESH_HEIGHTMAP", true], " macht daraus ein Modell: Ein schwarzer Punkt liegt unten, ein weißer ", ["BERGE", true], " Einheiten hoch, und das ganze Bild wird ", ["WELT", true], " mal WELT Einheiten groß — von 0 bis 400 in x und in z, nicht um die Mitte. Das bunte Bild hat an jedem Punkt die Farbe seiner Höhe, von Wiesengrün über Waldgrün und Fels bis Schnee; ", ["MODEL_TEXTURE", true], " legt es über das Gelände wie eine Landkarte über ein Relief."]),

  H.code(stueck("kap25/1_flug.dh", "' Wie hoch der Boden", "END FUNCTION")),

  H.pmix(["Wie hoch der Boden zwischen den Kartenpunkten ist, rechnet ", ["boden", true], " aus: Die Stelle der Welt wird zur Stelle in der Karte, und zwischen den vier Punkten darum wird gemischt, erst in x, dann in z. Damit fliegt der Flug nie in einen Berg, und jeder Baum steht genau auf dem Boden."]),

  H.h2("Schritt 3: der Wald"),

  H.code(stueck("kap25/1_flug.dh", "' Baeume: wo es Wald ist", "DIM krone")),

  H.pmix(["Das Programm würfelt Plätze, bis es 1500 Bäume hat, aber nur dort, wo der Boden in der Waldhöhe liegt — nicht in der Talsohle, nicht im Fels. Jeder Baum wird keine Zeichnung, sondern eine ", ["Matrix", true], ": an seinen Platz geschoben und mit einer zufälligen Größe gestreckt. Stamm und Krone bekommen je eine Liste, und gezeichnet wird in der Hauptschleife mit nur zwei Befehlen:"]),

  H.code(stueck("kap25/1_flug.dh", "MODEL_INSTANCED(stamm", "MODEL_INSTANCED(krone")),

  H.pmix([["MODEL_INSTANCED", true], " zeichnet ein Modell an allen Plätzen einer Liste von Matrizen — 1500 Kronen in einem Aufruf. Warum das wichtig ist, steht unter „Warum so?“."]),

  H.h2("Schritt 4: der Himmel"),

  H.code(stueck("kap25/1_flug.dh", "' Der Himmel: sechs Bilder", "SKYBOX_IMAGE")),

  H.pmix(["Ein Himmel ist ein großer Würfel um die Kamera, und ", ["SKYBOX_IMAGE", true], " nimmt dafür ein Bild aus sechs Feldern nebeneinander, in der Reihenfolge rechts, links, oben, unten, hinten, vorn. Die vier Seiten bekommen mit ", ["IMAGE_GRADIENT", true], " einen Verlauf von Blau oben zur Farbe des Dunstes unten, die Decke ist ganz blau, der Boden ganz Dunst. Weil ", ["LIGHT_FOG", true], " dieselbe Farbe hat, gehen Berge und Himmel am Horizont nahtlos ineinander über. Der Himmel bewegt sich nicht mit: Wie weit man auch fliegt, er bleibt gleich weit weg."]),

  H.h2("Schritt 5: der Flug"),

  H.code(stueck("kap25/1_flug.dh", "DIM dt AS FLOAT", "py = MAX")),

  H.pmix(["Der Flug hat einen Ort — px, py, pz — und einen ", ["kurs", true], ", einen Winkel auf der Landkarte. Die Pfeiltasten drehen den Kurs oder ändern die Höhe, und in jedem Bild geht es ein Stück in Kursrichtung weiter: COS(kurs) in x, SIN(kurs) in z, mal Tempo mal DELTA. Am Rand der Welt kehrt der Flug um. Und die letzte Zeile hält ihn mindestens drei Einheiten über dem Boden — ", ["MAX", true], " nimmt den größeren der beiden Werte. Die Kamera steht am Ort des Flugs und blickt zehn Einheiten voraus, ein wenig nach unten. Das ganze Programm:"]),

  H.code(zeilen("kap25/1_flug.dh")),

  H.h2("Warum so?"),

  H.h3("Warum tausend Bäume ein Aufruf sind"),

  H.code(zeilen("kap25/2_messen.dh")),
  H.code(["je Baum ein MODEL:   7.1 ms je Bild",
          "ein MODEL_INSTANCED: 0.2 ms je Bild"], { out: true }),

  H.p("Dreitausend Bäume, einmal mit einem MODEL je Baum und einmal mit einem MODEL_INSTANCED für alle. Auf dem Rechner, an dem dieses Buch entstand, kostet die erste Art 7,1 Millisekunden je Bild, die zweite 0,2 — rund fünfunddreißigmal weniger. Bei sechzig Bildern je Sekunde hat ein Bild knapp 17 Millisekunden; mit MODEL wären die Bäume allein fast die Hälfte davon."),

  H.p("Der Grund ist nicht das Zeichnen selbst, das macht die Grafikkarte in beiden Fällen gleich schnell. Teuer ist das Bitten: Jeder MODEL-Aufruf schickt dem Grafikkartentreiber einen eigenen Auftrag mit allem, was dazugehört, und der Treiber prüft und richtet jedes Mal neu ein. Bei MODEL_INSTANCED gibt es einen einzigen Auftrag: hier das Modell, hier dreitausend Matrizen, zeichne sie alle. Darum zeichnet man viele gleiche Dinge — Bäume, Gras, Sterne, Steine, Bausteine — als Instanzen. Der Preis: Instanzen können keine Schatten werfen und nicht spiegeln wie MODEL_PBR; für eine einzelne Mühle nimmt man weiter MODEL."),

  H.h3("Warum die Höhen in einem Feld stehen"),

  H.p("Das Gelände könnte man auch direkt aus dem Rauschen in das Graubild malen. Aber dann wüsste das Programm später nicht mehr, wie hoch eine Stelle ist — und das braucht es ständig: für jeden Baum und in jedem Bild für den Flug. Ein Modell ist zum Zeichnen da, nicht zum Nachfragen. Also stehen die Zahlen einmal im Feld, und Bild, Wald und Flug fragen alle dieselbe Quelle. So passen sie immer zusammen."),

  H.h2("Was schiefgehen kann"),

  H.h3("Der Wald im Berg"),

  H.code(zeilen("kap25/fehler_massstab.dh")),

  H.figure("kap25_fehler_massstab.png", "Ein Hügel, und kein einziger Baum zu sehen.", 380, 238),

  H.p("Vierundzwanzig Bäume sollen im Kreis um den Gipfel stehen, und keiner ist zu sehen. Die Karte sagt, wie hoch eine Stelle ist, auf einer Skala von 0 bis 1 — der Hügel in der Welt ist aber BERGE, also 12 Einheiten hoch. Ohne das „* BERGE“ stehen die Bäume knapp über dem Grund, acht Einheiten tief im Hügel. Wer mit zwei Maßstäben rechnet — Karte und Welt —, rechnet am besten an genau einer Stelle um, wie boden es im Flug tut."),

  H.h3("Instanzen im Nebel"),

  H.p("Beim Schreiben dieses Kapitels standen die fernen Bäume scharf und dunkel vor Bergen, die schon im Dunst verschwunden waren: Der Befehl für Instanzen kannte LIGHT_FOG nicht. Jetzt verschwimmen Instanzen genauso wie Modelle."),

  H.h3("Ein Ring, der ein Punkt war"),

  H.p("Für die Übung mit den Ringen sollte MESH_TORUS(4, 0.5, …) einen Ring mit Radius 4 bauen — heraus kam ein Klümpchen, kaum zu sehen. Drachenhauch reichte die beiden Zahlen ungeändert an raylib weiter, die Bibliothek darunter, die sie ganz anders liest. Betroffen waren auch alle Beispiele mit Ring oder Knoten, und niemand hatte es bemerkt, denn ein kleiner Ring sieht auch aus wie ein Ring. Jetzt ist ein Ring so groß, wie man ihn bestellt."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere MASSSTAB von 0.02 auf 0.05. Was passiert mit den Bergen?"),
  H.bulletRich("Drehen 2. ", "Nimm die Höhen nicht ins Quadrat: hoehe[x, z] = t statt t * t."),
  H.bulletRich("Drehen 3. ", "Ändere in CAMERA3D die 60 auf 90 und die Leertaste auf ein Tempo von 120. Wie fühlt sich der Flug an?"),
  H.bulletRich("Erweitern 1. ", "Eine Wasserfläche: Wo das Land tief liegt, entstehen Seen."),
  H.bulletRich("Erweitern 2. ", "Tageszeit: Mit 1, 2 und 3 wird es Morgen, Mittag und Abend — Himmel, Dunst und Sonne zusammen."),
  H.bulletRich("Erfinden. ", "Ein Ziel, das man anfliegt: Ringe über dem Land, durch die man der Reihe nach fliegen muss. Oder: ein Schatten, der dem Flieger auf dem Boden folgt; Wolken als Billboards; eine Karte von oben in einer Ecke. Anhang A zeigt die Ringe."),

  H.figure("kap25_ringe.png", "Erfinden: Der nächste Ring leuchtet golden über einem Bergrücken.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Module → g3d: ", "MESH_HEIGHTMAP, MESH_CUBICMAP, SKYBOX_IMAGE und die anderen Formen."),
  H.bulletRich("Handbuch, Module → m3d → GPU-Instancing: ", "MODEL_INSTANCED, eine Farbe je Instanz, und was der Instancing-Weg nicht kann."),
  H.bulletRich("examples/89_heightmap.dh und 104_instancing.dh: ", "Gelände und viele Würfel auf einmal."),
];
