// Anhang C -- Farb-Konstanten. Werte aus gamebasic/graphics.py COLORS, gegen
// gbrt verifiziert (z. B. ORANGE = 16753920 = RGB(255,165,0) = &HFFA500).
module.exports = (H) => {
  // [Name, R, G, B, Hex]
  const COLORS = [
    ["BLACK", 0, 0, 0, "000000"],
    ["WHITE", 255, 255, 255, "FFFFFF"],
    ["GRAY", 128, 128, 128, "808080"],
    ["LIGHTGRAY", 192, 192, 192, "C0C0C0"],
    ["DARKGRAY", 64, 64, 64, "404040"],
    ["RED", 255, 0, 0, "FF0000"],
    ["GREEN", 0, 255, 0, "00FF00"],
    ["BLUE", 0, 0, 255, "0000FF"],
    ["YELLOW", 255, 255, 0, "FFFF00"],
    ["CYAN", 0, 255, 255, "00FFFF"],
    ["MAGENTA", 255, 0, 255, "FF00FF"],
    ["ORANGE", 255, 165, 0, "FFA500"],
    ["PURPLE", 128, 0, 128, "800080"],
    ["BROWN", 139, 69, 19, "8B4513"],
    ["PINK", 255, 192, 203, "FFC0CB"],
    ["DARKRED", 128, 0, 0, "800000"],
    ["DARKGREEN", 0, 128, 0, "008000"],
    ["DARKBLUE", 0, 0, 128, "000080"],
  ];

  const rows = COLORS.map(([name, r, g, b, hex]) => [
    { text: name, mono: true, swatch: hex },
    { text: r + ", " + g + ", " + b, mono: true },
    { text: "&H" + hex, mono: true, color: "555555" },
  ]);

  return [
    H.chapter("Anhang C — Farb-Konstanten"),
    H.p("Diese Farbnamen sind ohne IMPORT überall verfügbar – du schreibst einfach RED oder ORANGE statt einer Zahl. Jede Konstante entspricht einem RGB-Wert; die Tabelle zeigt die Rot-/Grün-/Blau-Anteile (0–255) und die Hexadezimal-Schreibweise. Eigene Farben mischst du mit RGB(r, g, b) (Kapitel 44)."),
    H.table(rows, {
      headers: ["Konstante", "R, G, B", "Hex"],
      widths: [3600, 2826, 2600],
    }),
    H.note("Das farbige Kästchen links zeigt die jeweilige Farbe (WHITE erscheint auf weißem Papier naturgemäß unsichtbar). Mit RGB(255, 165, 0) erzeugst du z. B. dasselbe Orange wie die Konstante ORANGE."),
  ];
};
