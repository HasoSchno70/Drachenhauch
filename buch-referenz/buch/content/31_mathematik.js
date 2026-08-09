module.exports = (H) => [
  H.chapter("Mathematik"),
  H.p("Drachenhauch bringt eine vollständige Mathe-Bibliothek mit – von einfachen Funktionen wie Betrag und Wurzel bis zu Trigonometrie und praktischen Spiel-Helfern. Alle hier vorgestellten Funktionen sind direkt verfügbar, ohne IMPORT. Winkel werden, wie in der Mathematik üblich, im Bogenmaß (Radiant) angegeben – mit DEG und RAD rechnest du nach Grad um."),

  H.h2("Grundfunktionen"),
  H.cmd("ABS · SQR · POW · HYPOT", "ABS(x)   SQR(x)   POW(b, e)   HYPOT(x, y)",
    "ABS liefert den Betrag (das Vorzeichen fällt weg), SQR die Quadratwurzel (x ≥ 0), POW die Potenz (b hoch e), HYPOT die Länge der Hypotenuse – also die Wurzel aus x²+y², ohne Überlauf. (SQRT ist ein erlaubter Zweitname für SQR.)",
    [
      'PRINT ABS(-7)',
      'PRINT POW(2, 10)',
      'PRINT HYPOT(3.0, 4.0)',
    ],
    { out: ["7", "1024", "5.0"] }),
  H.cmd("EXP · LOG", "EXP(x)   LOG(x[, basis])",
    "EXP ist e hoch x, LOG der Logarithmus (ohne Basis: der natürliche Logarithmus zur Basis e; mit zweitem Argument zu beliebiger Basis). LOG10 gibt es als Kurzform für den Zehnerlogarithmus.",
    [
      'PRINT ROUND(EXP(1), 4)',
      'PRINT ROUND(LOG(EXP(1)), 4)',
    ],
    { out: ["2.7183", "1.0"] }),

  H.h2("Runden"),
  H.cmd("FLOOR · CEIL · ROUND", "FLOOR(x)   CEIL(x)   ROUND(x[, stellen])",
    "FLOOR rundet ab (zur nächstkleineren ganzen Zahl), CEIL auf, ROUND kaufmännisch zur nächsten ganzen Zahl. Mit einem zweiten Argument rundet ROUND auf so viele Nachkommastellen.",
    [
      'PRINT FLOOR(3.7); " "; CEIL(3.2); " "; ROUND(3.5)',
      'PRINT ROUND(3.14159, 2)',
    ],
    { out: ["3 4 4", "3.14"] }),

  H.h2("Vergleichen & Begrenzen"),
  H.cmd("MIN · MAX", "MIN(a, b, ...)   MAX(a, b, ...)",
    "Liefern den kleinsten bzw. größten der übergebenen Werte. Du darfst beliebig viele angeben.",
    [
      'PRINT MIN(5, 2, 9)',
      'PRINT MAX(5, 2, 9)',
    ],
    { out: ["2", "9"] }),
  H.cmd("CLAMP · SIGN", "CLAMP(v, lo, hi)   SIGN(x)",
    "CLAMP zwängt einen Wert in den Bereich von lo bis hi (kleinere werden zu lo, größere zu hi). SIGN liefert das Vorzeichen: -1 bei negativen, 0 bei null, +1 bei positiven Zahlen. (CLAMP01(v) ist die Kurzform für CLAMP(v, 0, 1).)",
    [
      'PRINT CLAMP(150, 0, 100)',
      'PRINT SIGN(-3); " "; SIGN(0); " "; SIGN(8)',
    ],
    { out: ["100", "-1 0 1"] }),

  H.h2("Interpolieren & Umskalieren"),
  H.p("Diese Funktionen sind das Rückgrat vieler Animationen und Bewegungen – etwa um einen Wert sanft von A nach B laufen zu lassen oder einen Messwert in einen anderen Bereich zu übertragen."),
  H.cmd("LERP · REMAP", "LERP(a, b, t)   REMAP(v, von_lo, von_hi, nach_lo, nach_hi)",
    "LERP („lineare Interpolation“) liefert den Wert, der zum Anteil t zwischen a und b liegt: t=0 ergibt a, t=1 ergibt b, t=0.5 die Mitte. REMAP überträgt einen Wert aus einem Bereich linear in einen anderen.",
    [
      'PRINT LERP(0.0, 10.0, 0.25)',
      'PRINT REMAP(5, 0, 10, 0, 100)',
    ],
    { out: ["2.5", "50.0"] }),

  H.h2("Trigonometrie"),
  H.cmd("SIN · COS · TAN · ATAN2", "SIN(x)   COS(x)   TAN(x)   ATAN2(y, x)",
    "Die Winkelfunktionen erwarten den Winkel im Bogenmaß. ATAN2(y, x) liefert umgekehrt den Winkel (im Bogenmaß) zum Punkt (x, y) – ideal, um die Richtung von einem Punkt zu einem anderen zu bestimmen. (Ebenfalls vorhanden: ASIN, ACOS, ATAN.)",
    [
      'PRINT ROUND(SIN(PI / 2), 4)',
      'PRINT ROUND(DEG(ATAN2(4.0, 3.0)), 1)',
    ],
    { out: ["1.0", "53.1"] }),
  H.cmd("DEG · RAD", "DEG(bogenmaß)   RAD(grad)",
    "Rechnen zwischen den Winkeleinheiten um: DEG wandelt Bogenmaß in Grad, RAD Grad in Bogenmaß. Praktisch, weil man Winkel oft in Grad denkt, die Funktionen aber Bogenmaß brauchen.",
    [
      'PRINT DEG(PI)',
      'PRINT ROUND(RAD(180.0), 4)',
    ],
    { out: ["180.0", "3.1416"] }),

  H.h2("Spiel-Helfer"),
  H.cmd("WRAP · PINGPONG · MOVETOWARD", "WRAP(v, lo, hi)   PINGPONG(t, länge)   MOVETOWARD(akt, ziel, maxd)",
    "WRAP faltet einen Wert zyklisch in den Bereich [lo, hi) – gut für umlaufende Winkel oder Indizes (370° werden zu 10°). PINGPONG pendelt im Bereich [0, länge] hin und her (eine Dreieckswelle, gut für Hin-und-Her-Bewegungen). MOVETOWARD schiebt den aktuellen Wert um höchstens maxd in Richtung Ziel, ohne darüber hinauszuschießen.",
    [
      'PRINT WRAP(370, 0, 360)',
      'PRINT PINGPONG(2.5, 2)',
      'PRINT MOVETOWARD(0, 10, 3)',
    ],
    { out: ["10.0", "1.5", "3.0"] }),

  H.h2("Konstanten"),
  H.p("Zwei Kreiszahl-Konstanten stehen bereit: PI (3,14159…) und TAU (= 2·PI, der volle Kreis im Bogenmaß). Eine Konstante E gibt es bewusst NICHT, weil e ein beliebter Name für die Fehlervariable in CATCH ist – nutze stattdessen EXP(1)."),
  H.code([
    'PRINT ROUND(PI, 5)',
    'PRINT ROUND(TAU, 5)',
  ]),
  H.code(["3.14159", "6.28319"], { out: true }),
  H.h2("Vorzeichen, Nachkommateil, Fast-Gleichheit"),
  H.cmd("SGN · FRAC", 'SGN(zahl)   FRAC(x)',
    "SGN liefert das Vorzeichen als -1, 0 oder +1 – praktisch, um eine Richtung aus einer Geschwindigkeit zu gewinnen. FRAC gibt den Nachkommateil zurück (FRAC(3.75) = 0.75).",
    [
      'PRINT SGN(-12.5)',
      'PRINT FRAC(3.75)',
    ], { out: ["-1", "0.75"] }),
  H.cmd("APPROX", 'APPROX(a, b[, epsilon])',
    "Prüft, ob zwei Kommazahlen NAHEZU gleich sind. Das ist bei Kommazahlen fast immer die richtige Frage: 0.1 + 0.2 ergibt intern nicht exakt 0.3, ein Vergleich mit = schlägt also fehl. Ohne epsilon wird eine kleine Standardtoleranz benutzt.",
    [
      'PRINT 0.1 + 0.2 = 0.3',
      'PRINT APPROX(0.1 + 0.2, 0.3)',
    ], { out: ["FALSE", "TRUE"] }),
  H.cmd("FBM3", 'FBM3(x, y, z, oktaven)',
    "Dreidimensionales, geschichtetes Rauschen (fractal brownian motion): mehrere Rausch-Ebenen übereinander ergeben natürlicher wirkende Strukturen als eine einzelne. Mit der dritten Achse als Zeit bekommst du Rauschen, das sich langsam verändert – etwa ziehende Wolken.",
    [
      'DIM wolke AS FLOAT',
      'wolke = FBM3(x * 0.02, y * 0.02, GET_TIME() * 0.1, 4)',
    ]),
  H.tip("Zufall und Rauschen", "Für Zufallszahlen gibt es ein eigenes Kapitel („Zufall“). Für sanftes, zusammenhängendes „Rauschen“ – etwa zur Gelände- oder Wolkenerzeugung – stehen NOISE, NOISE2, NOISE3 und FBM bereit; mehr dazu bei den Spiel-Helfern in Teil IV."),
];
