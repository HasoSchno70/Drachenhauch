module.exports = (H) => [
  H.chapter("Farben"),
  H.p("Eine Farbe ist in GameBasic eine ganze Zahl, die Rot-, Grün- und Blauanteil zusammenfasst (das Format 0xRRGGBB). Du musst diese Zahl aber nie selbst ausrechnen: RGB(r, g, b) baut sie aus drei Werten von 0 bis 255, und für die gängigen Farben gibt es fertige Konstanten. Dieses Kapitel zeigt, wie du Farben erzeugst, mischst und zerlegst."),
  H.figure("44_farben.png", "Farbkonstanten, ein HSV-Regenbogen, ein COLOR_LERP-Verlauf und eine Helligkeitsreihe."),

  H.h2("Farben bauen: RGB"),
  H.cmd("RGB", "RGB(rot, grün, blau)",
    "Baut eine Farbe aus den drei Kanälen Rot, Grün und Blau (jeweils 0 bis 255). 0,0,0 ist Schwarz, 255,255,255 Weiß, 255,0,0 reines Rot.",
    [
      'PRINT RGB(255, 128, 0)      \' ein Orange',
      'PRINT HEX$(RGB(255, 128, 0))',
    ], { out: ["16744448", "FF8000"] }),
  H.note("Hexadezimale Farb-Literale schreibst du in GameBasic mit &H statt 0x: &HFF8000 ist dasselbe wie RGB(255, 128, 0). Am lesbarsten bleibt für Einsteiger aber RGB(...).") ,

  H.h2("Fertige Farbkonstanten"),
  H.p("Für die häufigsten Farben gibt es Konstanten, die du direkt als Farbe einsetzt – kürzer als RGB(...). Verfügbar sind unter anderem: BLACK, WHITE, GRAY, LIGHTGRAY, DARKGRAY, RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA, ORANGE, PURPLE, BROWN, PINK sowie DARKRED, DARKGREEN, DARKBLUE."),
  H.cmd("Farbkonstanten", "BLACK · WHITE · RED · GREEN · BLUE · YELLOW · …",
    "Vordefinierte Farbwerte. Du kannst sie überall verwenden, wo eine Farbe erwartet wird.",
    [
      'PRINT HEX$(RED)',
      'PRINT HEX$(WHITE)',
      'CLS(BLACK)',
      'CIRCLE(160, 120, 40, YELLOW)',
      'FLIP()',
    ], { out: ["FF0000", "FFFFFF"] }),

  H.h2("Farben zerlegen: RED, GREEN, BLUE"),
  H.cmd("RED · GREEN · BLUE", "RED(farbe)   GREEN(farbe)   BLUE(farbe)",
    "Holen den einzelnen Kanal (0 bis 255) aus einer Farbe heraus. Praktisch, um eine Farbe abzudunkeln oder zwei Farben zu vergleichen. (Mit Klammern sind es Funktionen; ohne Klammern ist RED dagegen die rote Farbkonstante – GameBasic unterscheidet das anhand der Klammern.)",
    [
      'DIM c AS INTEGER',
      'c = RGB(255, 128, 0)',
      'PRINT RED(c); " "; GREEN(c); " "; BLUE(c)',
    ], { out: ["255 128 0"] }),

  H.h2("Farben über HSV: HSV"),
  H.p("Manchmal denkt man Farben lieber als „Farbton, Sättigung, Helligkeit“ statt als Rot/Grün/Blau. Genau das ist HSV: Der Farbton (Hue, 0 bis 360 Grad) läuft einmal durch den Regenbogen, die Sättigung (0 bis 1) reicht von grau bis kräftig, der Wert (0 bis 1) von dunkel bis hell. HSV ist ideal, um Regenbogen-Effekte zu erzeugen oder eine Farbe nur heller/dunkler zu machen."),
  H.cmd("HSV", "HSV(farbton, sättigung, wert)",
    "Erzeugt eine Farbe aus Farbton (0–360 Grad), Sättigung (0–1) und Wert/Helligkeit (0–1). HSV(120, 1, 1) ist sattes Grün.",
    [
      'PRINT HSV(120.0, 1.0, 1.0)     \' Grün',
      '\' Regenbogen: Farbton mit der Zeit durchlaufen lassen',
      '\' farbe = HSV(WRAP(t, 0, 360), 1.0, 1.0)',
    ], { out: ["65280"] }),

  H.h2("Farben mischen: COLOR_LERP"),
  H.cmd("COLOR_LERP", "COLOR_LERP(farbe1, farbe2, t)",
    "Mischt zwei Farben kanalweise. t = 0 ergibt farbe1, t = 1 ergibt farbe2, t = 0.5 die Mitte. Perfekt für weiche Farbübergänge (Tag→Nacht, Energiebalken grün→rot).",
    [
      'PRINT COLOR_LERP(BLACK, WHITE, 0.5)   \' mittleres Grau',
      'DIM gruenZuRot AS INTEGER',
      'gruenZuRot = COLOR_LERP(GREEN, RED, 0.5)',
    ], { out: ["8421504"] }),
  H.tip("Energiebalken einfärben", "Ein hübsches Muster: Färbe einen Lebensbalken je nach Füllstand. anteil = hp / max_hp (0..1), dann farbe = COLOR_LERP(RED, GREEN, anteil) – voll ist grün, leer ist rot, dazwischen fließend."),
];
