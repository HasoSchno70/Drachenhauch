const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Pixelkunst und Paletten", 16),

  H.h2("Was wir bauen"),

  H.figure("kap16_2_abendsee.png", "Ein Abend am See, 200 mal 125 Punkte in 18 Farben, viermal vergrößert. Der Himmel scheint viel mehr Farben zu haben — das macht die Rasterung.", 470, 294),

  H.p("Ein Bild wie aus einem Spiel der frühen Neunziger: ein Abend am See, ein Felsen, Berge, eine Sonne, die sich im Wasser spiegelt. Es hat nur 200 mal 125 Punkte und 18 Farben, und trotzdem glitzert der See, und mit der Taste N wird es langsam Nacht. Bewegt wird dabei kein einziger Punkt — nur Farben. Unterwegs lernen wir, warum eine Farbe eine Zahl ist und wie man sie in ihre Teile zerlegt."),

  H.h2("Der Plan"),

  H.bulletRich("Eine Farbe ist eine Zahl; ", "wer sie in Rot, Grün und Blau zerlegen kann, kann Farben mischen."),
  H.bulletRich("Jeder Punkt ", "speichert nicht seine Farbe, sondern nur ihre Nummer in einer Palette von 18 Farben."),
  H.bulletRich("Rasterung: ", "Wo eine Farbe in die nächste übergeht, mischt ein Muster die beiden — so wirken wenige Farben wie viele."),
  H.bulletRich("Farbzyklen: ", "Wer die Farben hinter einigen Nummern reihum wandern lässt, bewegt alle Punkte mit diesen Nummern."),

  H.h2("Schritt 1: eine Farbe zerlegen"),

  H.p("RGB(255, 160, 32) ist ein Orange. Für Drachenhauch ist es eine einzige Zahl. Dieses kleine Programm zeigt sie auf drei Arten und nimmt sie auseinander:"),

  H.code(zeilen("kap16/1_zahlen.dh")),
  H.code(["als Zahl: 16752672",
          "hex:      FFA020",
          "binaer:   111111111010000000100000",
          "rot 255, gruen 160, blau 32",
          "zusammen: FFA020 -- gleich? TRUE"], { out: true }),

  H.p("In hexadezimaler Schreibweise sieht man den Aufbau sofort: FF ist das Rot (255), A0 das Grün (160), 20 das Blau (32) — zwei Ziffern je Farbe. Im Binären sind es je acht Stellen: Die ersten acht Einsen und Nullen sind das Rot, die mittleren das Grün, die letzten das Blau. Darum kann man eine Farbe auch gleich als &HFFA020 schreiben; die Palette in Schritt 2 tut das."),

  H.pmix([["SHR 16", true], " schiebt alle Stellen um 16 nach rechts — die acht Stellen des Blaus und die acht des Grüns fallen hinten heraus, übrig bleibt das Rot. ", ["BAND 255", true], " behält nur die letzten acht Stellen und löscht alles davor: Bei (farbe SHR 8) BAND 255 bleibt so das Grün übrig, ohne das Rot davor. BAND vergleicht die Stellen paarweise: Nur wo in beiden Zahlen eine 1 steht, steht im Ergebnis eine 1, und 255 ist binär acht Einsen. Zurück geht es mit ", ["SHL", true], ", das nach links schiebt, und ", ["BOR", true], ", das die Einsen beider Zahlen vereint."]),

  H.h2("Schritt 2: eine Palette und ein Bild aus Nummern"),

  H.p("Das Bild bekommt eine Palette: ein Feld von 18 Zahlen, jede eine Farbe. Konstanten sagen, ab welcher Nummer welche Sorte Farbe beginnt:"),

  H.code(stueck("kap16/2_abendsee.dh", "' Die Nummern in der Palette.", "&H1E3A6A")),

  H.pmix(["Der Himmel hat sechs Farben von Nachtblau bis Gold, die Berge zwei, der Fels drei, das Gras zwei, der See vier. Das Bild selbst ist ein Feld, das für jeden Punkt nur eine Nummer speichert — ", ["DIM bild[0 TO B - 1, 0 TO H - 1] AS INTEGER", true], ". malen füllt es einmal beim Start. Für den Himmel zählt die Höhe: oben Nummer 0, am Horizont Nummer 5. Darüber kommen Sonne und Berge; die Höhe der Berge kommt aus dem Rauschen von Kapitel 14:"]),

  H.code(stueck("kap16/2_abendsee.dh", "' Himmel:", "END IF")),

  H.p("(x − 140)² + (y − 72)² < 14² ist der Satz des Pythagoras: Der Punkt liegt näher als 14 an der Mitte der Sonne. Der See darunter besteht aus kurzen Strichen in seinen vier Farben, je Zeile ein Stück verschoben; unter der Sonne liegt ihr Spiegelbild in jeder zweiten Zeile:"),

  H.code(stueck("kap16/2_abendsee.dh", "' Der See:", "IF ABS(x - 140)")),

  H.h2("Schritt 3: Rasterung"),

  H.figure("kap16_ohne_raster.png", "Derselbe Abend ohne Rasterung (Taste R): Der Himmel zerfällt in sechs harte Streifen.", 420, 262),

  H.p("Sechs Farben für einen ganzen Abendhimmel sind wenig: Ohne Hilfe wird er zu sechs Streifen. Die Hilfe heißt Rasterung. Wo der Himmel zwischen zwei Farben liegt, wird ein Teil der Punkte zur oberen und ein Teil zur unteren — je näher an der oberen, desto mehr. Aus der Entfernung mischt das Auge die beiden zu einem Ton dazwischen. Zeitungen drucken ihre Bilder so, nur mit Punkten verschiedener Größe."),

  H.code(stueck("kap16/2_abendsee.dh", "' Die Bayer-Matrix:", "bayer = ")),
  H.code(stueck("kap16/2_abendsee.dh", "FUNCTION gerastert", "END FUNCTION")),

  H.pmix(["wert ist etwa 2,3: Stufe 2 und ein Rest von 0,3. Ob der Punkt bei 2 bleibt oder zu 3 wird, entscheidet eine Schwelle, die von seinem Platz abhängt. Die 16 Schwellen der Bayer-Matrix gelten für ein Feld von 4 mal 4 Punkten, das sich über das ganze Bild wiederholt; ", ["(y MOD 4) * 4 + x MOD 4", true], " sucht die passende heraus. Bei einem Rest von 0,3 liegen 5 der 16 Schwellen darunter — rest · 16 = 4,8 ist größer als 0, 1, 2, 3 und 4 —, also werden 5 von 16 Punkten zur oberen Farbe. Die Zahlen der Matrix sind so verteilt, dass Punkte mit kleinen Schwellen nie nebeneinanderliegen; darum entsteht ein gleichmäßiges Muster statt Klumpen. Dieselbe Funktion rastert auch den Fels und die Berge."]),

  H.h2("Schritt 4: Farben, die wandern"),

  H.p("Jetzt bewegt sich der See — ohne dass ein einziger Punkt seine Nummer ändert. Gezeichnet wird mit einer Kopie der Palette, farben, in der die vier Farben des Sees reihum vertauscht sind:"),

  H.code(stueck("kap16/2_abendsee.dh", "SUB farben_stellen", "END SUB")),

  H.pmix(["Ist schritt 1, bekommt Nummer 14 die Farbe von 15, 15 die von 16, 16 die von 17 und 17 — dank ", ["MOD SEE_STUFEN", true], " — die von 14. Jeder Wellenstrich wechselt also seine Farbe, und weil benachbarte Striche benachbarte Nummern haben, sieht das aus, als wanderten die Wellen. Viermal je Sekunde geht es eine Stelle weiter: ", ["farben_stellen(FLOOR(zeit * 4), nacht)", true], ". Danach wird jeder Punkt mit seiner Farbe gezeichnet:"]),

  H.code(stueck("kap16/2_abendsee.dh", "RENDERTARGET_BEGIN(klein)", "RENDERTARGET_DRAW")),

  H.figure("kap16_nacht.png", "Die Taste N: Alle 18 Farben werden zu einem dunklen Blau hin gemischt. Die Punkte bleiben, wie sie sind.", 420, 262),

  H.p("Die Nacht macht dasselbe mit allen Farben: Jede wird ein Stück zu einem dunklen Blau hin gemischt. Dazu zerlegt mischen beide Farben in Rot, Grün und Blau, mischt jede für sich mit LERP aus Kapitel 14 und setzt das Ergebnis wieder zusammen:"),

  H.code(stueck("kap16/2_abendsee.dh", "FUNCTION mischen", "END FUNCTION")),

  H.p("Für die Nacht werden 18 Farben gemischt, nicht 25 000 Punkte. Das ist der Gedanke der Palette: Bild und Farben sind getrennt."),

  H.h2("Schritt 5: als Bild sichern"),

  H.pmix(["Die Taste S schreibt das Bild als PNG-Datei, in voller Größe. Dazu braucht das Programm das Modul imgfx, die Werkzeugkiste für Bilder: ", ["RENDERTARGET_IMAGE", true], " macht aus dem kleinen Zeichenblatt ein Bild, ", ["IMAGE_SCALE_NN", true], " vergrößert es, ", ["IMAGE_SAVE", true], " schreibt es."]),

  H.code(stueck("kap16/2_abendsee.dh", "IF KEYHIT(KEY_S)", "END IF")),

  H.p("NN steht für nearest neighbour, nächster Nachbar: Jeder neue Punkt nimmt einfach die Farbe des alten Punkts, auf den er fällt. Aus einem Punkt werden so 4 mal 4 gleiche. Das gewöhnliche IMAGE_SCALE glättet dagegen, damit Fotos beim Vergrößern nicht eckig werden — für Pixelkunst wäre das falsch, denn die Ecken sind gewollt. Das ganze Programm:"),

  H.code(zeilen("kap16/2_abendsee.dh")),

  H.h2("Warum so?"),

  H.h3("Warum eine Farbe eine Zahl ist"),

  H.p("Jeder Bildpunkt eines Bildschirms besteht aus drei winzigen Lichtern: rot, grün und blau. Jedes kann 256 Helligkeiten haben, von 0 bis 255, und 256 Möglichkeiten sind genau acht Binärstellen. Drei Lichter brauchen 24 Stellen, und die passen bequem in eine Zahl. 256 · 256 · 256 sind 16,7 Millionen Farben — mehr, als ein Auge unterscheiden kann. Hexadezimal ist für Farben die praktischste Schreibweise, weil jede der drei Farben genau zwei Ziffern hat. Drachenhauch benutzt die acht Stellen davor für die Deckkraft: RGBA(255, 160, 32, 128) ist ein halb durchsichtiges Orange."),

  H.h3("Warum BAND schwächer bindet als +"),

  H.p("Malnehmen geht vor Plus — das kennt jeder aus der Schule. Für BAND, BOR, SHL und SHR gibt es keine Schulregel; Drachenhauch macht es wie C, Java und Python: Sie binden schwächer als + und −. Wer a BAND b + c schreibt, bekommt a BAND (b + c). Das hat einen Grund: Bitoperationen arbeiten meist mit fertigen Zahlen — einer Farbe, einer Maske —, und die soll man ausrechnen können, bevor man sie zerlegt. Aber es ist eine Falle, und darum warnt Drachenhauch, wenn ein + oder − ohne Klammern neben einem BAND steht. Die einfachste Regel: Wer Bitoperationen mit Rechnung mischt, setzt Klammern."),

  H.h3("Warum Paletten"),

  H.p("Um 1990 hatte ein Rechner für das Bild 64 000 Bytes: 320 mal 200 Punkte, ein Byte je Punkt. Für 24 Stellen je Punkt reichte das nicht; ein Byte fasst aber eine Nummer von 0 bis 255. Die Grafikkarte schlug jede Nummer beim Anzeigen in einer Tabelle von 256 Farben nach. Änderte ein Programm eine Farbe in dieser Tabelle, änderten sich alle Punkte mit dieser Nummer — auf einen Schlag und ohne dass das Programm einen Punkt anfasste. So entstanden die glitzernden Wasserfälle, flackernden Feuer und pulsierenden Lichter der Spiele jener Zeit."),

  H.p("Heute ist es umgekehrt. Beim Abendsee braucht malen etwa 6,5 Millisekunden für alle 25 000 Punkte, das Zeichnen aus den Nummern aber 11 Millisekunden je Bild: Die 25 000 PLOT sind der teure Teil, nicht das Rechnen. Was die Palette heute bringt, ist die Trennung: Das Bild sagt, wo der See ist, die Palette, wie er aussieht. Nacht, Winter oder ein glitzernder See ändern 18 Zahlen und wissen nichts von Felsen und Bergen. Wer es schneller braucht, schickt die Nummern als Bild an einen Shader wie in Kapitel 15 und lässt die Grafikkarte nachschlagen — genau so machen es Emulatoren für alte Spiele."),

  H.h2("Was schiefgehen kann"),

  H.h3("BAND ohne Klammern"),

  H.code(zeilen("kap16/fehler_band.dh")),
  H.code(["Blau: 0"], { out: true }),
  H.code(["fehler_band.dh:3: Warnung: BAND bindet schwaecher als +: gerechnet",
          "wird a BAND (b + c). Ist das gemeint, die Klammern hinschreiben;",
          "sonst (a BAND b) + c"], { out: true }),

  H.p("Gemeint war das Blau (32) plus 20. Gerechnet wurde farbe BAND 275, und da 275 und 32 keine gemeinsame Binärstelle haben, kommt 0 heraus. Das Programm läuft trotzdem, nur mit der falschen Zahl. Bis zu diesem Kapitel gab es dazu keine Warnung; das Ausprobieren für das Buch hat sie gebracht. Richtig ist (farbe BAND 255) + 20."),

  H.h3("Eine Nummer ohne Farbe"),

  H.code(zeilen("kap16/fehler_palette.dh")),
  H.code(["Laufzeitfehler in fehler_palette.dh:5: Index 3 ausserhalb [0..2] in",
          "Dimension 1 -- Felder zaehlen ab 0: DIM a[3] hat a[0] bis a[2]"], { out: true }),

  H.p("Die Palette hat drei Farben, Nummer 0, 1 und 2. Wer eine neue Nummer einführt, muss auch eine Farbe ans Ende der Palette schreiben — die Übungen tun das jedes Mal. Die Meldung kommt erst, wenn ein Punkt mit dieser Nummer gezeichnet wird; ein Bild, in dem die Nummer nur selten vorkommt, läuft also eine Weile und bricht dann ab."),

  H.h3("Derselbe Name zweimal — noch einmal"),

  H.p("Beim Schreiben dieses Kapitels hieß die Höhe der Berge in malen zuerst berg, neben CONST BERG für ihre Nummer. Das ist die Falle aus Kapitel 14. Diesmal warnte Drachenhauch vor dem Start, dass die lokale Variable die Konstante verdeckt, und danach brach die Zeile nr = BERG + … ab: Gemeint war Nummer 7, gerechnet wurde mit der Höhe 64,9. Jetzt heißt die Höhe gipfel."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere die 4 in FLOOR(zeit * 4) auf 1 und auf 20."),
  H.bulletRich("Drehen 2. ", "Ersetze alle 16 Zahlen der Bayer-Matrix durch 8. Was wird aus dem Himmel?"),
  H.bulletRich("Drehen 3. ", "Speichere mit IMAGE_SCALE statt IMAGE_SCALE_NN und vergleiche die beiden Dateien."),
  H.bulletRich("Erweitern 1. ", "Eine eigene Palette: Die Taste P wechselt zwischen Abend und Winter. Kein Punkt des Bildes wird dafür neu gerechnet."),
  H.bulletRich("Erweitern 2. ", "Ein Wasserfall aus Farbzyklen: Vom Felsen stürzt Wasser herab. Vier neue Nummern, deren Farben schnell nach unten wandern."),
  H.bulletRich("Erfinden. ", "Sterne, die nachts funkeln und am Tag unsichtbar sind. Oder: ein Lagerfeuer am Ufer, dessen Flammen durch Farbzyklen flackern; ein Regenbogen; eine Palette, die mit der Tageszeit langsam von Morgen zu Abend wandert. Anhang A zeigt die Sterne."),

  H.figure("kap16_wasserfall.png", "Erweitern 2: Der Wasserfall besteht aus vier Nummern, deren Farben dreimal so schnell wie die des Sees nach unten wandern.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Standard-Built-ins → Bitwise: ", "BAND, BOR, BXOR, BNOT, SHL, SHR und ihr Vorrang."),
  H.bulletRich("Handbuch, Module → imgfx: ", "Bilder vergrößern, drehen, filtern, in Graustufen wandeln, ihre häufigsten Farben finden (IMAGE_PALETTE) und als PNG oder bewegtes GIF sichern."),
  H.bulletRich("examples/53_bitwise.dh: ", "alle Bitoperationen mit Beispielen."),
];
