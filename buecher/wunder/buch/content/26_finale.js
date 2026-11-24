const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Das große Finale — eine Demo", 26),

  H.h2("Was wir bauen"),

  H.figure("kap26_titel.png", "Die erste Szene der Demo: Funken steigen durch einen Sternenhimmel, der Titel glüht.", 470, 294),

  H.p("Zum Schluss alles zusammen. Eine Demo, wie es sie seit den Achtzigern gibt: ein kleines Programm, das nichts will, als eine Minute lang zu zeigen, was ein Rechner kann. Unsere hat drei Szenen. Erst steigen Funken durch einen Sternenhimmel, und ein Titel glüht auf. Dann fliegt die Kamera über eine Abendlandschaft mit Bergen und Wald. Zum Schluss zieht ein Abspann nach oben, und die Musik klingt aus. Dazwischen wird es kurz dunkel, und über allem liegt das Leuchten aus Kapitel 15."),

  H.p("Fast nichts daran ist neu — Partikel, Musik aus AUDIO_NOTE, ein Shader, Gelände und Instanzen kennen wir. Neu ist, wie man so etwas Großes baut, ohne den Überblick zu verlieren: in fünf Dateien, von denen jede eine Sache tut und von den anderen so wenig wie möglich weiß."),

  H.h2("Der Plan"),

  H.bulletRich("Der Taktgeber: ", "demo.dh weiß, welche Szene wann dran ist, und sonst fast nichts."),
  H.bulletRich("Die Musik: ", "musik.dh baut das Stück."),
  H.bulletRich("Die Szenen: ", "titel.dh, flug.dh und abspann.dh, jede für sich."),
  H.bulletRich("Das Leuchten: ", "ein Shader über allem, aus einem Ordner assets."),
  H.bulletRich("Ausliefern: ", "eine Datei, die man weitergeben kann."),

  H.h2("Schritt 1: ein Programm aus fünf Dateien"),

  H.code(["kap26/",
          "  demo.dh        der Taktgeber",
          "  musik.dh       das Stueck",
          "  titel.dh       Szene 1: Funken und Titel",
          "  flug.dh        Szene 2: Flug ueber die Berge",
          "  abspann.dh     Szene 3: der Abspann",
          "  assets/",
          "    glanz.fs     das Leuchten aus Kapitel 15"], { out: true }),

  H.pmix(["Mit ", ["IMPORT \"titel.dh\"", true], " haben wir schon in Kapitel 21 eine Datei eingebunden: Ihr Text wird an dieser Stelle eingefügt, als stünde er da. Für eine Demo aus fünf Teilen ist das zu wenig — jede Szene hat ihre eigenen Variablen, und schnell hießen zwei davon gleich. Darum bindet die Demo jede Datei mit ", ["AS", true], " ein: ", ["IMPORT \"titel.dh\" AS titel", true], ". Dann ist die Datei ein ", ["Namensraum", false], ". Was in ihr steht, heißt von außen ", ["titel.zeichnen", true], " oder ", ["titel.FUNKEN", true], ", innen aber weiter schlicht zeichnen. Zwei Szenen dürfen so beide ein zeichnen haben, ein BREITE, ein sx."]),

  H.p("Und noch etwas: Ein Namensraum sieht die Variablen und Konstanten des Hauptprogramms nicht. Das klingt nach einer Einschränkung und ist der eigentliche Gewinn. Was eine Szene braucht, steht in ihrer eigenen Datei oder kommt als Parameter herein — man kann sie lesen, ohne den Rest zu kennen, und sie in ein anderes Programm mitnehmen. Jede Szene bietet dieselben drei Dinge an: vorbereiten wird einmal am Anfang gerufen, zeichnen in jedem Bild mit den Sekunden seit dem Beginn der Szene, und wer etwas einschaltet, hat ein beenden, das es wieder ausschaltet."),

  H.h2("Schritt 2: der Taktgeber"),

  H.code(zeilen("kap26/demo.dh")),

  H.pmix(["Der Taktgeber kennt drei Zahlen: wie lange jede Szene dauert, im Feld ", ["dauer", true], ". In jedem Bild wächst zeit; ist die Szene vorbei, kommt die nächste, und zeit fängt bei null an. ", ["SELECT CASE", true], " ruft das zeichnen der Szene, die dran ist. Die Leertaste setzt zeit ans Ende — so springt man weiter. Und nach der letzten Szene verlässt ", ["EXIT WHILE", true], " die Schleife, das Programm endet."]),

  H.p("Die Blende ist eine einzige Rechnung. rest ist, wie weit wir vom nächsten Rand der Szene entfernt sind — vom Anfang oder vom Ende. Ist das weniger als eine Sekunde, wird ein schwarzes Rechteck über alles gelegt, umso dunkler, je näher der Rand. So blendet jede Szene von selbst ein und aus, ohne dass sie davon weiß."),

  H.pmix(["Das Leuchten ist der Shader aus Kapitel 15, als ", ["POSTFX", true], " über jedem Bild. Über der hellen Landschaft würde es blenden, darum stellt der Taktgeber es beim Szenenwechsel schwächer und danach wieder stärker. Und wenn der Abspann beginnt, lässt ", ["AUDIO_STOP(kanal, ms)", true], " die Musik in den 16 Sekunden des Abspanns ausklingen."]),

  H.h2("Schritt 3: die Musik"),

  H.code(zeilen("kap26/musik.dh")),

  H.pmix(["Die Musik ist wie in Kapitel 19 ein Stück aus AUDIO_NOTE, zusammengemischt mit ", ["AUDIO_SOUND_MIX", true], ": acht Takte zu je vier Schlägen, je Takt ein Akkord aus a-Moll, F, C und G als weicher Klangteppich, Trommel und Zischen für den Takt, darüber eine Melodie aus 32 Tönen. Neu ist nur ", ["PRIVATE", true], " vor ton: Diese Funktion braucht nur die Datei selbst, von außen ist sie nicht zu sehen. dauer dagegen ist öffentlich — 8 Takte mal 4 Schläge mal 600 Millisekunden, 19,2 Sekunden. Die Szenen der Demo sind genau so lang wie ein halbes und ein ganzes Stück, damit die Wechsel auf einen Takt fallen."]),

  H.h2("Schritt 4: die Szenen"),

  H.code(zeilen("kap26/titel.dh")),

  H.p("Die erste Szene hat alles, was sie braucht, in ihrer eigenen Datei: ihre Größe, ihre Sterne, ihre Funken. Die Felder stehen außerhalb der SUBs — so behalten sie ihren Wert von Bild zu Bild, gehören aber nur dem Namensraum. Die Funken steigen mit eigener Geschwindigkeit und fangen unten wieder an, wenn sie oben herausgeflogen sind; der Titel wird in vier Sekunden aus dem Dunkel hell."),

  H.code(stueck("kap26/flug.dh", "' Ein Bild der Szene", "END SUB", 2)),

  H.figure("kap26_flug.png", "Die zweite Szene: Die Kamera fliegt im Abendlicht über Berge und Wald.", 420, 262),

  H.pmix(["Die zweite Szene ist der Flug aus Kapitel 25, zusammengefasst: Gelände aus Rauschen, Wald als Instanzen, ein Himmel aus SKYBOX_IMAGE, alles in Abendfarben. Gesteuert wird nicht mehr; die Kamera zieht einen großen Bogen hoch über den Gipfeln. Wichtig ist das Ende der Datei: ", ["beenden", true], " schaltet aus, was die Szene eingeschaltet hat — den Himmel, ihre Sonne und den Dunst. Licht, Nebel und Himmel gelten nämlich für das ganze Programm, nicht für eine Szene, und eine Szene danach fände sonst noch das Abendlicht vor."]),

  H.code(stueck("kap26/abspann.dh", "' Ein Bild der Szene", "END SUB")),

  H.figure("kap26_abspann.png", "Die dritte Szene: Der Abspann zieht nach oben, die Überschriften warm, darunter blass.", 420, 262),

  H.p("Der Abspann ist die einfachste Szene: Jede Zeile steht 36 Punkte unter der vorigen, und alles wandert mit 40 Punkten je Sekunde nach oben. Gezeichnet wird nur, was gerade im Fenster ist. Eine Zeile direkt nach einer leeren ist eine Überschrift und wird warm."),

  H.h2("Schritt 5: ausliefern"),

  H.code(["dhrt --export demo.dh"]),
  H.code(["  1 referenzierte Datei(en) mitkopiert:",
          "    assets/glanz.fs",
          "Exportiert: demo_dist\\demo.exe"], { out: true }),

  H.p("Fertig ist ein Programm erst, wenn andere es starten können — ohne Drachenhauch. dhrt --export übersetzt die Demo samt allen eingebundenen Dateien und hängt sie an eine Kopie der Laufzeit; heraus kommt ein Ordner demo_dist mit einer demo.exe. Der Ordner assets wird mitkopiert: Darum liegt der Shader dort und nicht neben demo.dh, denn eine Datei, die das Programm erst beim Laufen lädt, kann der Export nicht von selbst finden. Die exportierte Demo läuft aus jedem Ordner — sie wechselt beim Start dorthin, wo sie liegt, und findet so ihr assets. Ausprobiert: Aus einem fremden Ordner gestartet, glüht der Titel wie in dhrt."),

  H.p("Der Export nimmt die kleinste Laufzeit, die alle Befehle der Demo kennt. Den Ordner kann man zippen und verschicken, auf einen USB-Stick legen oder auf eine Webseite stellen. In die Spielwiese im Browser passt die Demo dagegen nicht: Die nimmt ein Programm aus einem einzigen Textfeld und teilt es über einen Link, ohne weitere Dateien und ohne Shader. Für ein einzelnes Programm wie die aus den ersten Kapiteln ist sie der schnellste Weg, es zu zeigen; für eine Demo ist es die exportierte Datei."),

  H.h2("Warum so?"),

  H.h3("Warum ein großes Programm in kleine Teile"),

  H.p("Die Demo ist zusammen gut 300 Zeilen lang. In einer einzigen Datei wäre jede Zeile mit jeder anderen verbunden: Eine Variable sx, ein CONST N, eine Funktion boden — jede könnte überall gebraucht und überall verändert werden. Wer an der Landschaft etwas ändert, müsste wissen, ob der Abspann davon abhängt. Die Aufteilung schneidet diese Verbindungen ab. Was zwischen den Teilen noch fließt, steht sichtbar an einer Stelle: im Taktgeber, als Aufrufe wie flug.zeichnen(zeit). Man kann eine Szene lesen, ändern, ersetzen oder in ein anderes Programm mitnehmen, ohne die anderen anzufassen."),

  H.p("Der Schnitt geht dort, wo sich etwas unabhängig ändern lässt. Die Musik hat mit den Bildern nichts zu tun, außer ihrer Länge — also eine Datei, und dauer() als einzige Verbindung. Die Szenen haben miteinander nichts zu tun, nur denselben Aufbau. Was in fast jeder Szene vorkommt, die Blende, gehört dem Taktgeber. Eine gute Faustregel: Wenn man einem Teil einen Namen geben kann, der in drei Wörtern sagt, was er tut, ist er wahrscheinlich gut geschnitten."),

  H.h3("Warum jede Szene dieselbe Form hat"),

  H.p("vorbereiten, zeichnen, beenden — jede Szene sieht von außen gleich aus. Darum braucht der Taktgeber keine Sonderfälle: Eine vierte Szene ist eine Datei mit denselben drei SUBs und eine Zeile mehr im Taktgeber. Und weil die Form gleich ist, kann man die Szenen sogar in eine Liste legen — das zeigt die erste Übung."),

  H.h2("Was schiefgehen kann"),

  H.h3("Eine Szene, die das Hauptprogramm braucht"),

  H.code(zeilen("kap26/fehler_namensraum.dh")),
  H.code(zeilen("kap26/fehler_szene.dh")),

  H.p("Die Szene benutzt BREITE, das im Hauptprogramm steht — und sieht es nicht. Drachenhauch sagt es beim Übersetzen, noch bevor ein Fenster aufgeht: „breite kommt aus dem Hauptprogramm. Eine mit AS importierte Datei sieht dessen Globals nicht -- reiche den Wert als Parameter herein oder deklariere ihn in der Datei selbst.“ Darum hat jede Szene der Demo ihre eigenen CONST BREITE und HOEHE. Das sieht doppelt aus, macht die Szene aber unabhängig: In einem Programm mit anderem Fenster passt man genau die eine Datei an."),

  H.h3("Eine Szene, die nicht aufräumt"),

  H.p("Licht, Nebel und Himmel gelten für das ganze Programm. Ohne die zwei Zeilen in flug.beenden, die Sonne und Dunst ausschalten, scheint die Abendsonne in jede Szene danach. Ausprobiert mit der vierten Szene aus der Erfinden-Übung, dem Sonnensystem: Erde und Mars sind dann auch auf der Seite hell, die von ihrer Sonne abgewandt ist — die Abendsonne des Flugs scheint noch."),

  H.h3("Alles zusammen ist zu laut"),

  H.p("Die erste Fassung der Musik hatte Trommel, Teppich und Melodie je in der Lautstärke, die einzeln gut klang. Zusammen gemessen, an einer Aufnahme mit AUDIO_RECORD_START, erreichten die Spitzen in jeder Sekunde den Rand: 1,0, darüber wird abgeschnitten, und es klingt kratzig. Leiser gemischt liegt die lauteste Stelle des ganzen Stücks bei 0,78. Wer mischt, misst das Ganze, nicht die Teile."),

  H.h3("Eine Warnung, die nicht stimmte"),

  H.p("Beim Schreiben dieses Kapitels meldete jeder Lauf in flug.dh „'hoehe' ist als 'HOEHE' angelegt“ — nur weil das Hauptprogramm eine Konstante HOEHE hat, die der Namensraum gar nicht sieht. Die Werte stimmten, nur die Prüfung der Schreibweise schaute über die Grenze. Jetzt kennt sie die Namensräume."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere BLENDE von 1.0 auf 3.0. Was passiert, und was, wenn eine Szene kürzer ist als zwei Blenden?"),
  H.bulletRich("Drehen 2. ", "Ändere die Stärke des Leuchtens für Titel und Abspann von 1.2 auf 3."),
  H.bulletRich("Drehen 3. ", "Lass im Taktgeber die Zeile flug.vorbereiten() weg. Was passiert?"),
  H.bulletRich("Erweitern 1. ", "Die Szenen als Liste: Statt SELECT CASE steht jede Szene als Funktionsreferenz in einem Feld."),
  H.bulletRich("Erweitern 2. ", "Eine Pause: P hält Bild und Musik an, P lässt sie weiterlaufen."),
  H.bulletRich("Erfinden. ", "Eine vierte Szene — etwa das Sonnensystem aus Kapitel 24. Oder eine ganz eigene Demo: eine Geschichte aus Kapitel 21 mit Musik, ein Feuerwerk zum Takt, eine Reise durch alle Kapitel dieses Buchs. Anhang A zeigt das Sonnensystem als vierte Szene."),

  H.figure("kap26_vier.png", "Erfinden: Das Sonnensystem aus Kapitel 24 als vierte Szene, mit dem Leuchten der Demo.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Import: ", "wo IMPORT sucht, Namensräume mit AS, PRIVATE, Klassen über den Namensraum."),
  H.bulletRich("Handbuch, Schnellstart → Weitergeben: ", "dhrt --export, schlanke Laufzeiten und was sie können."),
  H.bulletRich("examples/nachtflug/: ", "eine größere Demo aus zwei Dateien, mit Musik, Partikeln und 3D."),
];
