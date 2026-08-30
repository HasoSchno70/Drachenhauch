// Anhang B -- Tastencodes (KEY_* / JOY_*). Werte aus drachenhauch/graphics.py KEYS,
// gegen dhrt verifiziert.
module.exports = (H) => {
  const row = (name, code, note) => [{ text: name, mono: true }, { text: String(code), mono: true }, note || ""];
  const T = (rows) => H.table(rows, {
    headers: ["Konstante", "Code", "Bedeutung"],
    widths: [3000, 1800, 4226],
  });

  return [
    H.chapter("Anhang B — Tastencodes"),
    H.p("KEYPRESSED(code) und die input-Befehle erwarten einen Tastencode. Statt die Zahlen auswendig zu lernen, nutzt du die benannten Konstanten – sie sind ohne IMPORT überall verfügbar. Diese Tabelle zeigt sie alle samt Zahlenwert. (Mehr dazu in Kapitel „Eingabe“.)"),

    H.h2("Sonder- und Steuertasten"),
    T([
      row("KEY_ESCAPE", 27, "Esc – oft zum Beenden/Pausieren"),
      row("KEY_RETURN", 13, "Eingabetaste (= KEY_ENTER)"),
      row("KEY_ENTER", 13, "gleicher Code wie KEY_RETURN"),
      row("KEY_SPACE", 32, "Leertaste – beliebt für Springen/Schießen"),
      row("KEY_TAB", 9, "Tabulator"),
      row("KEY_BACKSPACE", 8, "Rücktaste"),
    ]),

    H.h2("Pfeiltasten"),
    T([
      row("KEY_LEFT", 1073741904, "Pfeil links"),
      row("KEY_RIGHT", 1073741903, "Pfeil rechts"),
      row("KEY_UP", 1073741906, "Pfeil hoch"),
      row("KEY_DOWN", 1073741905, "Pfeil runter"),
    ]),

    H.h2("Buchstaben und Ziffern"),
    H.p("Die Buchstabentasten heißen KEY_A bis KEY_Z, die Zifferntasten KEY_0 bis KEY_9. Ihre Codes laufen lückenlos durch, sodass du sie auch berechnen kannst (z. B. KEY_1 + i für die Tasten 1–8)."),
    T([
      row("KEY_A … KEY_Z", "97 … 122", "fortlaufend: KEY_B = 98, KEY_C = 99 …"),
      row("KEY_0 … KEY_9", "48 … 57", "fortlaufend: KEY_1 = 49, KEY_2 = 50 …"),
    ]),

    H.h2("Funktionstasten"),
    T([
      row("KEY_F1", 1073741882, "Funktionstaste 1"),
      row("KEY_F2 … KEY_F11", "1073741883 …", "fortlaufend aufsteigend"),
      row("KEY_F12", 1073741893, "Funktionstaste 12"),
    ]),

    H.h2("Gamepad (Modul input)"),
    H.p("Diese Konstanten bindest du mit INPUT_BIND an Aktionen (Kapitel „Eingabe“). Die negativen Codes kennzeichnen Gamepad-Eingaben gegenüber den Tastatur-Codes."),
    T([
      row("JOY_BUTTON_A", -100, "untere Taste (Bestätigen/Springen)"),
      row("JOY_BUTTON_B", -101, "rechte Taste (Abbrechen)"),
      row("JOY_BUTTON_X", -102, "linke Taste"),
      row("JOY_BUTTON_Y", -103, "obere Taste"),
      row("JOY_BUTTON_LB", -104, "Schultertaste links"),
      row("JOY_BUTTON_RB", -105, "Schultertaste rechts"),
      row("JOY_BUTTON_BACK", -106, "Back/Select"),
      row("JOY_BUTTON_START", -107, "Start/Menü"),
      row("JOY_BUTTON_LSTICK", -108, "linker Stick gedrückt"),
      row("JOY_BUTTON_RSTICK", -109, "rechter Stick gedrückt"),
      row("JOY_DPAD_UP", -200, "Steuerkreuz hoch"),
      row("JOY_DPAD_DOWN", -201, "Steuerkreuz runter"),
      row("JOY_DPAD_LEFT", -202, "Steuerkreuz links"),
      row("JOY_DPAD_RIGHT", -203, "Steuerkreuz rechts"),
    ]),

    H.note("Für die analogen Sticks gibt es keine festen Codes – ihre Achsen liest du mit INPUT_JOY_AXIS(slot, \"left_x\") als Wert zwischen -1 und +1 (siehe Kapitel „Eingabe“)."),
  ];
};
