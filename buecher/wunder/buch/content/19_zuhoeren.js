const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Ein Bild, das zuhört", 19),

  H.h2("Was wir bauen"),

  H.figure("kap19_2_zuhoeren.png", "Das Spektrum eines kleinen Stücks: tief links, hoch rechts. Gerade hat die Trommel geschlagen — der Hintergrund blitzt, der Bass ist ganz oben.", 470, 294),

  H.p("Ein Bild, das zur Musik tanzt. Das Programm spielt ein kleines Stück, das es selbst zusammensetzt — Trommel, Klatschen, Bass, Melodie —, und zeigt, was gerade klingt: als Balken, einer für jeden Bereich der Tonhöhe, tief links, hoch rechts. Weiße Striche merken sich die letzten Spitzen und sinken langsam. Oben rechts stehen Bass, Mitte und Höhen als drei Kreise, und bei jedem Schlag der Trommel blitzt der Hintergrund auf. Es geht um Felder als Messwerte, um Tupel — mehrere Werte als eine Rückgabe — und darum, warum man Werte über die Zeit glättet, aber nach der Uhr und nicht nach den Bildern."),

  H.h2("Der Plan"),

  H.bulletRich("Ein Stück aus Klängen: ", "einmal zusammengemischt, dann endlos gespielt."),
  H.bulletRich("Ein Feld als Spektrum: ", "AUDIO_FFT füllt jeden Platz eines Felds mit der Stärke eines Tonbereichs."),
  H.bulletRich("Ein Tupel: ", "Bass, Mitte und Höhen kommen aus einer einzigen Funktion zurück."),
  H.bulletRich("Glätten nach der Zeit: ", "Spitzen sinken langsam, ein Schnitt folgt dem Bass — gleich schnell auf jedem Bildschirm."),
  H.bulletRich("Ein Schlag: ", "Der Bass ist deutlich lauter als im Schnitt."),

  H.h2("Schritt 1: ein Ton wandert durchs Spektrum"),

  H.figure("kap19_1_gleiten.png", "Nach vier Sekunden ist der Ton bei gut 600 Hz angekommen — etwa in der Mitte der Balken.", 420, 262),

  H.code(zeilen("kap19/1_gleiten.dh")),

  H.pmix(["Der Ton ist ein Sinus, der in acht Sekunden von 55 Hz um 84 Halbtöne, sieben Oktaven, hinaufgleitet: Das letzte Argument von AUDIO_NOTE gleitet in Halbtönen. ", ["DIM sp[BAENDER] AS FLOAT", true], " ist ein Feld mit 48 Plätzen, und ", ["AUDIO_FFT(sp)", true], " füllt jeden mit einer Zahl zwischen 0 und 1: wie stark der Bereich gerade klingt. FFT heißt Fast Fourier Transform, schnelle Fourier-Zerlegung — ein Verfahren, das aus den letzten 1024 Abtastwerten ausrechnet, welche Frequenzen darin stecken."]),

  H.p("Startet man das Programm, wandert der Balken in gleichen Schritten nach rechts, obwohl sich die Frequenz in jeder Sekunde verdoppelt. AUDIO_FFT verteilt die Bänder nämlich wie der Theremin in Kapitel 18 seine Tasten: Jede Oktave bekommt gleich viele. Nach acht Sekunden, bei 7040 Hz, steht der Ton bei Band 38; die zehn Bänder rechts davon reichen bis 24 000 Hz, wo Musik kaum noch etwas hat. Dann beginnt er von vorn."),

  H.h2("Schritt 2: ein Stück aus Klängen"),

  H.code(stueck("kap19/2_zuhoeren.dh", "FUNCTION musik", "END FUNCTION")),

  H.pmix(["Wie in Kapitel 18 baut AUDIO_NOTE jeden Klang einzeln: Die Trommel ist ein Sinus, der in 140 ms zwei Oktaven tief fällt — so klingt eine elektronische Bassdrum —, Klatschen und Zischen sind Rauschen mit kurzer Hüllkurve. ", ["AUDIO_SOUND_NEW(8 * SCHLAG)", true], " ist eine leere Leinwand von vier Sekunden, und ", ["AUDIO_SOUND_MIX", true], " addiert einen Klang an einer Stelle hinein. Die Schleife geht die acht Schläge durch: auf jeden die Trommel, auf jeden zweiten das Klatschen, dazwischen das Zischen, je Schlag zwei Bassnoten und zwei Töne der Melodie. Danach spielt das Programm das fertige Stück endlos: ", ["AUDIO_PLAY(musik(), -1)", true], "."]),

  H.h2("Schritt 3: Bass, Mitte und Höhen als Tupel"),

  H.code(stueck("kap19/2_zuhoeren.dh", "' Der Mittelwert der", "END FUNCTION", 2)),

  H.pmix(["Eine Funktion gibt einen Wert zurück — will man drei, gibt sie ein ", ["Tupel", true], " zurück: ", ["RETURN (a, b, c)", true], " packt drei Werte in einen. Wer sie aufruft, packt sie gleich wieder aus:"]),

  H.code(stueck("kap19/2_zuhoeren.dh", "AUDIO_FFT(spektrum)", "(bass, mitte, hoehen)")),

  H.p("Links stehen drei Namen in Klammern, und jeder bekommt seinen Wert in der Reihenfolge, in der RETURN sie hingeschrieben hat. Der Bass ist das unterste Achtel der Bänder, bei 48 also die Bänder 0 bis 5 — dort liegt die Trommel, die bis 27 Hz hinunterfällt. Die Grenzen sind Anteile von BAENDER, nicht feste Nummern, damit das Programm auch mit mehr oder weniger Bändern läuft. Die Bassnoten liegen eine Oktave höher, zwischen 175 und 262 Hz, und zählen zur Mitte. Das ist Absicht: Beim ersten Versuch lagen sie tiefer, und dann hielt das Programm auch manche Bassnote für einen Schlag."),

  H.h2("Schritt 4: glätten und Schläge finden"),

  H.code(stueck("kap19/2_zuhoeren.dh", "FUNCTION glaetten", "END FUNCTION")),

  H.pmix(["glaetten lässt einen Wert einem anderen langsam folgen: In ", ["sekunden", true], " legt er etwa zwei Drittel des Wegs zurück. DELTA() ist die Zeit seit dem letzten Bild, und ", ["1 - EXP(-DELTA() / sekunden)", true], " ist der Anteil des Wegs, der in dieser Zeit dran ist — bei 60 Bildern je Sekunde und einer Sekunde 1,6 %, bei 30 Bildern 3,3 %. So kommt der Wert nach einer Sekunde gleich weit, wie oft auch gezeichnet wird. Damit findet das Programm die Schläge:"]),

  H.code(stueck("kap19/2_zuhoeren.dh", "seit_schlag = seit_schlag", "blitz = MAX")),

  H.p("bass_schnitt ist der Bass, über etwa eine Sekunde geglättet: wie laut er im Moment so ist. Ein Schlag ist, wenn der Bass 30 % über diesem Schnitt liegt — die Trommel springt hoch, der Schnitt kommt nicht so schnell hinterher. seit_schlag verhindert, dass ein Schlag doppelt zählt, solange die Trommel noch klingt. Gemessen fand das Programm in zehn Sekunden 20 Schläge, alle im Abstand von einer halben Sekunde, und keinen dazwischen. Der Blitz klingt in einer Viertelsekunde wieder ab."),

  H.h2("Schritt 5: das Bild"),

  H.code(stueck("kap19/2_zuhoeren.dh", "DIM balken AS FLOAT", "NEXT")),

  H.p("Jedes Band ist ein Kasten, darunter sein blasses Spiegelbild. Die Spitzen sinken eine halbe Bildhöhe je Sekunde — wieder mit DELTA() gerechnet, nicht je Bild. Das ganze Programm:"),

  H.code(zeilen("kap19/2_zuhoeren.dh")),

  H.h2("Warum so?"),

  H.h3("Warum man über die Zeit glättet"),

  H.p("Ein Bild dauert eine sechzigstel Sekunde. In so kurzer Zeit schwankt Musik heftig: Ein Ton hat Schwebungen, eine Trommel ist nach 50 ms schon fast vorbei, und die FFT sieht jedes Mal einen anderen Ausschnitt. Zeigte das Bild jeden Messwert, wie er ist, zuckten die Balken so schnell, dass das Auge nur Flimmern sieht. Glätten heißt: Jeder neue Wert zählt nur zum Teil, der alte klingt nach. Das Auge sieht dann Bewegung statt Rauschen. AUDIO_FFT glättet schon selbst ein wenig — ein Band fällt nicht sofort, sondern in jeder sechzigstel Sekunde auf 80 % —, das Programm glättet darüber hinaus, was es selbst ausrechnet."),

  H.h3("Warum nach der Uhr"),

  H.p("Nicht jeder Bildschirm zeigt 60 Bilder je Sekunde. Viele zeigen 120 oder 144, und ein langsamer Rechner schafft vielleicht 30. Wer je Bild einen festen Anteil des Wegs nimmt, glättet auf einem schnellen Bildschirm schneller und auf einem langsamen langsamer — das Programm verhält sich überall anders. Darum rechnen glaetten und die Spitzen mit DELTA(). Beim Ausprobieren für dieses Kapitel hat sich gezeigt, dass AUDIO_FFT selbst den Fehler hatte: Es ließ die Bänder je Aufruf fallen. Wer es alle 4 ms fragte, sah ein Band 50 ms nach dem Ton bei 0,03, wer es alle 16 ms fragte, bei 0,33. Jetzt rechnet es mit der Zeit seit dem letzten Aufruf."),

  H.h3("Warum die tiefen Balken in Gruppen stehen"),

  H.p("Links sind immer mehrere Balken gleich hoch, ganz links sogar sechs. Die FFT rechnet mit 1024 Abtastwerten, und daraus kann sie Frequenzen nur in Stufen von 47 Hz unterscheiden (48 000 durch 1024). Bei 50 Hz ist eine Stufe fast eine Oktave breit, bei 5000 Hz nur ein Hundertstel davon. Die untersten sechs Bänder fallen alle in dieselbe Stufe und zeigen denselben Wert. Mehr Abtastwerte würden feiner auflösen, aber länger zurückblicken: 4096 Werte sind schon fast eine Zehntelsekunde, und eine Trommel wäre im Bild verschmiert. 1024 ist ein Kompromiss zwischen genau und schnell."),

  H.h3("Warum ein Tupel"),

  H.p("Drei Werte aus einer Funktion zu holen, ginge auch mit drei Funktionen — dann rechnete das Programm aber dreimal über das Feld, und nichts sagte, dass die drei zusammengehören. Mit globalen Variablen ginge es auch, aber dann kann jede Stelle des Programms sie verändern. Ein Tupel hält zusammen, was zusammen entsteht: Bass, Mitte und Höhen desselben Augenblicks. Klassen wären die andere Möglichkeit; ein Tupel ist leichter, wenn die Werte gleich wieder ausgepackt werden."),

  H.h2("Was schiefgehen kann"),

  H.h3("Je Bild statt nach der Zeit"),

  H.code(zeilen("kap19/fehler_je_bild.dh")),
  H.code(["30 Bilder/s: je Bild 0.79, nach der Zeit 0.86",
          "60 Bilder/s: je Bild 0.96, nach der Zeit 0.86",
          "144 Bilder/s: je Bild 1.00, nach der Zeit 0.86"], { out: true }),

  H.p("Eine halbe Sekunde lang folgt ein Wert der 1. Wer je Bild zehn Prozent des Wegs nimmt, kommt bei 30 Bildern je Sekunde auf 0,79, bei 144 auf 1,00 — auf dem schnellen Bildschirm sind die Spitzen schon unten, wenn sie auf dem langsamen noch fallen. Nach der Zeit gerechnet sind es überall 0,86. Der Fehler fällt beim Schreiben nicht auf, denn auf dem eigenen Bildschirm sieht alles richtig aus."),

  H.h3("Ein Tupel, falsch ausgepackt"),

  H.code(zeilen("kap19/fehler_tupel.dh")),
  H.code(["Laufzeitfehler in fehler_tupel.dh:8: Tupel entpacken: links stehen 2",
          "Namen, das Tupel hat aber 3 Werte"], { out: true }),

  H.p("Links müssen so viele Namen stehen, wie das Tupel Werte hat. Bis zu diesem Kapitel hieß die Meldung „Tupel-Destructuring: 2 Ziele, aber Tupel hat 3 Element(e)“ — halb englisch und für Einsteiger ein Rätsel; das Buch hat sie umgeschrieben."),

  H.h3("Ein Feld ohne Plätze"),

  H.code(zeilen("kap19/fehler_leer.dh")),
  H.code(["Laufzeitfehler in fehler_leer.dh:3: AUDIO_FFT: das Feld hat keine",
          "Plaetze -- es bekommt so viele Werte, wie es Plaetze hat: DIM",
          "werte[64] AS FLOAT"], { out: true }),

  H.p("DIM spektrum AS ARRAY OF FLOAT legt ein leeres Feld an, und AUDIO_FFT füllt so viele Bänder, wie das Feld Plätze hat — also keines. Bisher geschah dann gar nichts: kein Balken, keine Meldung. Jetzt meldet es sich. Und noch einmal der Fall, der in diesem Buch nicht aufhört: Beim Schreiben hieß die Breite eines Balkens zuerst breite, neben CONST BREITE. Diesmal stand die Variable nicht in einer Funktion, sondern ganz oben im Programm, und da ist es kein stiller Fehler, sondern ein Abbruch beim Übersetzen: Namens-Kollision. Jetzt heißt sie balken."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze BAENDER auf 12 und auf 128. Was ändert sich links, was rechts?"),
  H.bulletRich("Drehen 2. ", "Lösche AND seit_schlag > 0.2 aus der Bedingung für einen Schlag. Wie oft zählt jetzt ein Schlag, und was sieht man?"),
  H.bulletRich("Drehen 3. ", "Lass die Spitzen mit 0.1 statt 0.5 je Sekunde sinken, dann mit 3."),
  H.bulletRich("Erweitern 1. ", "Kreis statt Balken: Die Bänder stehen als Strahlen um die Mitte, der Bass oben, und der innere Kreis atmet mit dem Bass."),
  H.bulletRich("Erweitern 2. ", "Farbe nach Bass: Je kräftiger der Bass im Schnitt, desto weiter drehen sich alle Farben im Farbkreis."),
  H.bulletRich("Erfinden. ", "Funken auf jedem Schlag, wie das Feuerwerk aus Kapitel 5. Oder: ein Gesicht, dessen Mund sich mit der Mitte öffnet; Sterne, die bei den Höhen funkeln; ein Visualisierer für die eigene Musik, die man aufs Fenster zieht. Anhang A zeigt die Funken."),

  H.figure("kap19_kreis.png", "Erweitern 1: Das Spektrum als Kranz, der Bass oben, auf beiden Seiten gespiegelt.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Module → audio: ", "AUDIO_FFT, AUDIO_PEAK, Mitschnitt, Klänge mischen (AUDIO_SOUND_NEW, AUDIO_SOUND_MIX)."),
  H.bulletRich("Handbuch, Module → Mikrofon: ", "MIC_FFT — dasselbe Spektrum vom Mikrofon, für ein Bild, das wirklich zuhört."),
  H.bulletRich("examples/115_modplayer.dh: ", "ein Spieler für Amiga-Module mit Spektrum, der Musikdateien annimmt, die man aufs Fenster zieht."),
];
