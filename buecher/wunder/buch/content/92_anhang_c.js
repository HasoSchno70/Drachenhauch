// Anhang C: Farben, Tasten, Formeln. Die Farbfelder werden aus denselben
// Zahlen gerechnet, die daneben stehen -- HSV wie in der Laufzeit.
function hex(r, g, b) {
  return [r, g, b].map((v) => Math.round(Math.max(0, Math.min(255, v)))
    .toString(16).padStart(2, "0")).join("").toUpperCase();
}
function hsv(h, s, v) {
  const c = v * s, x = c * (1 - Math.abs(((h / 60) % 2) - 1)), m = v - c;
  const [r, g, b] = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x]
    : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return hex((r + m) * 255, (g + m) * 255, (b + m) * 255);
}

module.exports = (H) => [
  H.chapter("C · Farben, Tasten, Formeln"),

  H.p("Was man beim Bauen immer wieder nachschlägt: wie man eine Farbe schreibt, wie eine Taste heißt, und die Handvoll Formeln, die in fast jedem Kapitel vorkam."),

  H.h2("Farben schreiben"),

  H.table([
    [{ text: "RGB(255, 150, 40)", mono: true, swatch: hex(255, 150, 40) }, "Rot, Grün, Blau, je 0 bis 255 — Licht, kein Malkasten: Rot und Grün ergeben Gelb"],
    [{ text: "RGBA(255, 150, 40, 128)", mono: true }, "dazu die Deckkraft: 0 durchsichtig, 255 deckend"],
    [{ text: "&HFF9628", mono: true, swatch: "FF9628" }, "dieselbe Farbe als eine Zahl, je zwei Stellen für Rot, Grün, Blau"],
    [{ text: "HSV(30, 0.85, 1)", mono: true, swatch: hsv(30, 0.85, 1) }, "Farbton in Grad, Sättigung und Helligkeit je 0 bis 1 — gut, um durch Farben zu wandern"],
    [{ text: "COLOR_LERP(a, b, t)", mono: true }, "zwischen zwei Farben mischen, t von 0 (a) bis 1 (b)"],
    [{ text: "RED(c)  GREEN(c)  BLUE(c)", mono: true }, "die Anteile einer Farbe zurück"],
  ], { headers: ["Schreibweise", "Bedeutung"], widths: [3400, 5626], mono: [0] }),

  H.h2("Der Farbkreis"),

  H.p("HSV ordnet die Farben im Kreis an. Wer einen Farbton mit der Zeit oder mit einer Nummer wachsen lässt, wandert einmal herum — so leuchten der Farbkreis aus Kapitel 0, das Plasma und die Spektrumbalken."),

  H.table([
    [{ text: "0", swatch: hsv(0, 0.75, 1) }, "Rot", { text: "180", swatch: hsv(180, 0.75, 1) }, "Türkis"],
    [{ text: "30", swatch: hsv(30, 0.75, 1) }, "Orange", { text: "210", swatch: hsv(210, 0.75, 1) }, "Himmelblau"],
    [{ text: "60", swatch: hsv(60, 0.75, 1) }, "Gelb", { text: "240", swatch: hsv(240, 0.75, 1) }, "Blau"],
    [{ text: "90", swatch: hsv(90, 0.75, 1) }, "Gelbgrün", { text: "280", swatch: hsv(280, 0.75, 1) }, "Violett"],
    [{ text: "120", swatch: hsv(120, 0.75, 1) }, "Grün", { text: "320", swatch: hsv(320, 0.75, 1) }, "Pink"],
  ], { headers: ["HSV(h, 0.75, 1)", "", "HSV(h, 0.75, 1)", ""], widths: [2100, 2413, 2100, 2413] }),

  H.h2("Farben aus dem Buch"),

  H.table([
    [{ text: "RGB(14, 16, 28)", mono: true, swatch: hex(14, 16, 28) }, "Nachthimmel hinter dem Wetterkreis (Kapitel 22)"],
    [{ text: "RGB(6, 6, 18)", mono: true, swatch: hex(6, 6, 18) }, "Sternenhimmel der Demo (Kapitel 26)"],
    [{ text: "RGB(185, 210, 235)", mono: true, swatch: hex(185, 210, 235) }, "Himmel und Dunst an der Mühle (Kapitel 24)"],
    [{ text: "RGB(110, 170, 80)", mono: true, swatch: hex(110, 170, 80) }, "Wiese (Kapitel 24)"],
    [{ text: "&H3E7A3A", mono: true, swatch: "3E7A3A" }, "Wald (Kapitel 25)"],
    [{ text: "&H9A948E", mono: true, swatch: "9A948E" }, "Fels (Kapitel 25)"],
    [{ text: "RGB(255, 200, 120)", mono: true, swatch: hex(255, 200, 120) }, "warmes Licht: Titel und Überschriften der Demo (Kapitel 26)"],
    [{ text: "RGB(255, 150, 120)", mono: true, swatch: hex(255, 150, 120) }, "freundliches Rot für Meldungen (Kapitel 23)"],
  ], { headers: ["Farbe", "Wo"], widths: [3400, 5626], mono: [0] }),

  H.h2("Tasten und Maus"),

  H.table([
    [{ text: "KEY_DOWN(taste)", mono: true }, "ist die Taste gerade gedrückt? — für alles, was gehalten wird: Lenken, Gas"],
    [{ text: "KEY_HIT(taste)", mono: true }, "wurde sie in diesem Bild gedrückt? — genau einmal je Druck: Umschalten, Springen"],
    [{ text: "KEY_RELEASED(taste)", mono: true }, "wurde sie in diesem Bild losgelassen?"],
    [{ text: "KEY_LEFT  KEY_RIGHT  KEY_UP  KEY_DOWN", mono: true }, "die Pfeiltasten"],
    [{ text: "KEY_SPACE  KEY_ENTER  KEY_ESCAPE", mono: true }, "Leertaste, Eingabe, Escape"],
    [{ text: "KEY_A … KEY_Z,  KEY_0 … KEY_9", mono: true }, "Buchstaben und Ziffern"],
    [{ text: "MOUSE_X()  MOUSE_Y()", mono: true }, "wo die Maus ist, in Fensterpunkten"],
    [{ text: "MOUSE_DOWN(0)  MOUSE_HIT(0)", mono: true }, "linke Taste gehalten / in diesem Bild gedrückt (1 ist rechts)"],
    [{ text: "MOUSE_RELEASED(0)", mono: true }, "linke Taste in diesem Bild losgelassen"],
    [{ text: "MOUSE_WHEEL()", mono: true }, "wie weit das Rad gedreht wurde"],
    [{ text: "MOUSE_DELTA_X()  MOUSE_DELTA_Y()", mono: true }, "wie weit die Maus seit dem letzten Bild gewandert ist"],
  ], { headers: ["Befehl", "Bedeutung"], widths: [4000, 5026], mono: [0] }),

  H.h2("Winkel und Kreise"),

  H.code(["' Ein Punkt auf einem Kreis um (mx, my) mit Radius r. Der Winkel",
          "' ist im Bogenmass: eine ganze Runde ist 2 * PI.",
          "x = mx + COS(winkel) * r",
          "y = my + SIN(winkel) * r",
          "' Der Winkel von (x1, y1) zu (x2, y2) -- zum Zielen, Drehen.",
          "winkel = ATAN2(y2 - y1, x2 - x1)",
          "' Zwischen Grad und Bogenmass.",
          "PRINT RAD(180); DEG(PI / 2)      ' 3.14159... und 90"]),

  H.p("Weil y auf dem Bildschirm nach unten wächst, läuft ein wachsender Winkel im Uhrzeigersinn (Kapitel 0, 4). In 3D sind die Winkel von MODEL_EX in Grad, die von m3d im Bogenmaß (Kapitel 24)."),

  H.h2("Abstände und Richtungen"),

  H.code(["' Wie weit ist (x2, y2) von (x1, y1) entfernt?",
          "abstand = SQR((x2 - x1) ^ 2 + (y2 - y1) ^ 2)",
          "abstand = HYPOT(x2 - x1, y2 - y1)     ' dasselbe",
          "' Eine Richtung der Laenge 1: durch die eigene Laenge teilen.",
          "rx = (x2 - x1) / abstand",
          "ry = (y2 - y1) / abstand",
          "' Mit Vektoren aus Kapitel 9 in einer Zeile:",
          "richtung = VEC2_NORMALIZE(ziel - ort)"]),

  H.h2("Sanft ankommen"),

  H.code(["' Mischen: t = 0 ist a, t = 1 ist b, dazwischen ein Stueck des Wegs.",
          "x = LERP(a, b, t)",
          "' Sanft anfahren und abbremsen (Kapitel 4 und 23).",
          "t2 = t * t * (3 - 2 * t)",
          "' Schnell los, sanft ankommen.",
          "t2 = 1 - (1 - t) ^ 2",
          "' Einem Ziel folgen, gleich schnell bei jeder Bildrate",
          "' (Kapitel 19): in s Sekunden zwei Drittel des Wegs.",
          "wert = wert + (ziel - wert) * (1 - EXP(-DELTA() / s))",
          "' In Grenzen halten.",
          "x = CLAMP(x, 0, 800)"]),

  H.h2("Zufall"),

  H.code(["RANDOMIZE(42)                     ' jedes Mal dieselbe Folge",
          "z = RND()                         ' 0 bis unter 1",
          "z = von + RND() * (bis - von)     ' zwischen von und bis",
          "n = INT(RND() * 6) + 1            ' ein Wuerfel: 1 bis 6",
          "h = FBM(x * 0.02, y * 0.02, 6)    ' Rauschen (Kapitel 14)"]),
];
