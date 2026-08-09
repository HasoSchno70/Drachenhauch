module.exports = (H) => [
  H.chapter("Zufall"),
  H.p("Zufall macht Spiele lebendig: Würfel, zufällige Gegner-Positionen, gemischte Karten, abwechslungsreiche Beute. Drachenhauch bietet dafür mehrere Funktionen – von der einfachen Zufallszahl bis zum gewichteten Ziehen aus einer Liste. Weil die Werte naturgemäß bei jedem Aufruf anders ausfallen, zeigen die meisten Beispiele hier keine feste Ausgabe, sondern beschreiben den möglichen Bereich."),

  H.h2("Zufallszahlen"),
  H.cmd("RND", "RND()        RND(n)",
    "Ohne Argument liefert RND eine Kommazahl im Bereich von 0 (einschließlich) bis 1 (ausschließlich). Mit einem Argument n liefert RND eine ganze Zahl von 0 bis n-1 – ideal als Baustein, etwa für einen Würfel.",
    [
      'DIM augen AS INTEGER',
      'augen = RND(6) + 1      \' ganze Zahl 1..6',
      'PRINT "Du würfelst eine "; augen',
    ]),
  H.cmd("RANDINT · RANDF", "RANDINT(lo, hi)   RANDF(lo, hi)",
    "RANDINT liefert eine ganze Zufallszahl von lo bis hi – beide Grenzen eingeschlossen (anders als RND(n)). RANDF liefert eine Kommazahl von lo (einschließlich) bis hi (ausschließlich). Meist liest sich RANDINT(1, 6) angenehmer als RND(6) + 1.",
    [
      'PRINT RANDINT(1, 6)     \' Würfel: 1, 2, 3, 4, 5 oder 6',
      'PRINT RANDF(0.0, 1.0)   \' z. B. eine Wahrscheinlichkeit',
    ]),

  H.h2("Aus einer Liste ziehen"),
  H.cmd("CHOICE", "CHOICE(array)",
    "Liefert ein zufällig gewähltes Element eines 1D-Arrays – jedes mit gleicher Wahrscheinlichkeit.",
    [
      'DIM farben AS ARRAY OF STRING',
      'farben = SPLIT$("rot,grün,blau,gelb", ",")',
      'PRINT CHOICE(farben)    \' eine der vier Farben',
    ]),
  H.cmd("WEIGHTED_CHOICE", "WEIGHTED_CHOICE(werte, gewichte)",
    "Zieht ein Element aus werte, aber nicht gleichverteilt: Je höher das zugehörige Gewicht, desto wahrscheinlicher. werte und gewichte sind zwei gleich lange 1D-Arrays. Das klassische Werkzeug für Beute-Tabellen („selten, gelegentlich, häufig“).",
    [
      'DIM beute AS ARRAY OF STRING',
      'DIM chance AS ARRAY OF INTEGER',
      'beute  = SPLIT$("Gold,Trank,Schwert", ",")',
      'chance[0]=70 : chance[1]=25 : chance[2]=5   \' Schwert ist selten',
      'PRINT WEIGHTED_CHOICE(beute, chance)',
    ]),
  H.cmd("SHUFFLE", "SHUFFLE(array)",
    "Mischt ein 1D-Array zufällig durch – direkt an Ort und Stelle (das Array wird selbst verändert, es entsteht keine Kopie). Perfekt zum Mischen eines Kartenstapels.",
    [
      'DIM karten[5] AS INTEGER',
      'karten[0]=1 : karten[1]=2 : karten[2]=3',
      'karten[3]=4 : karten[4]=5',
      'SHUFFLE(karten)         \' nun in zufälliger Reihenfolge',
    ]),

  H.h2("Den Zufall steuern: RANDOMIZE"),
  H.p("Hinter den Zufallszahlen steckt ein berechneter Strom von Werten, der von einem Startwert – dem Seed – abhängt. Setzt du mit RANDOMIZE einen festen Seed, kommt danach IMMER dieselbe Folge heraus. Das klingt unzufällig, ist aber sehr nützlich: zum Testen (jeder Lauf ist gleich) oder für reproduzierbare Level („Welt Nr. 42“ sieht bei allen Spielern gleich aus)."),
  H.cmd("RANDOMIZE", "RANDOMIZE([seed])",
    "Setzt den Startwert des Zufalls. Mit einer festen Zahl wird die Zufallsfolge reproduzierbar; ohne Argument nimmt Drachenhauch einen unvorhersehbaren System-Seed (das ist der Normalfall für echten Zufall).",
    [
      'RANDOMIZE(42)           \' fester Seed',
      'PRINT RND(6) + 1',
      'PRINT RND(6) + 1',
    ],
    { out: ["2", "5"] }),
  H.note("Mit RANDOMIZE(42) erhältst du genau diese Folge (2, dann 5) – bei dir genauso. Lässt du das RANDOMIZE weg oder rufst es ohne Argument auf, ist die Folge bei jedem Programmstart anders."),
  H.tip("Wann Seed setzen?", "Für echten Zufall im fertigen Spiel brauchst du RANDOMIZE meist gar nicht – ohne Seed ist jede Runde anders. Greife zum festen Seed gezielt: beim Testen, für nachbaubare Zufalls-Level oder wenn ein Fehler nur bei bestimmten Zufallswerten auftritt und du ihn zuverlässig wiederholen willst."),
];
