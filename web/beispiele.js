// Beispiel-Galerie fuer den Web-Playground.
//
// Bewusste Regel: **kein Programm hier darf eine Datei laden.** Die Galerie
// laeuft in jedem Build, auch in einem ohne `assets/`-Paket -- ein
// LOADIMAGE/AUDIO_MUSIC_LOAD wuerde dort ins Leere greifen. Alles, was hier zu
// sehen ist, rechnet die Runtime selbst aus.
//
// Neues Beispiel: Eintrag mit `name` (Knopfbeschriftung), `name_en` (dieselbe
// auf Englisch) und `src` (Quelle) anhaengen. Kurz halten -- die Galerie soll zum Lesen einladen, nicht zum
// Scrollen.
window.DH_BEISPIELE = [
  {
    // Einziger Eintrag mit `datei` statt `src`: das Programm, mit dem der Build
    // gemacht wurde (`build_wasm.py <datei.dh>` legt es als program.dh ab).
    // NUR dieses Programm darf Dateien laden -- sein assets/-Ordner ist der,
    // der im Paket steckt.
    name: "▶ Demo",
    name_en: "▶ Demo",
    datei: "program.dh",
    titel: "Das Programm laden, mit dem dieser Build gemacht wurde",
    titel_en: "Load the program this build was made with",
  },
  // --- Die drei Startprojekte aus dem Schnellstart (docs/schnellstart.md).
  // `id` macht sie ueber `?beispiel=<id>` direkt aufrufbar, `src_en` ist
  // dieselbe Quelle mit englischen Kommentaren und Texten. String.raw, damit
  // ein `\` (Ganzzahl-Division) so ankommt, wie er dasteht.
  {
    id: "spiel",
    name: "★ Mini-Spiel",
    name_en: "★ Mini game",
    titel: "Startprojekt: Sterne sammeln -- TEMPO oder STERNE aendern",
    titel_en: "Starter project: collect stars -- change SPEED or STARS",
    src: String.raw`' Mini-Spiel: Sammle die Sterne! Pfeiltasten bewegen den Drachen.
' Zum Ausprobieren: TEMPO oder STERNE aendern und neu starten.
CONST TEMPO = 220.0
CONST STERNE = 5
SCREEN(480, 320, "Mini-Spiel", 2)

DIM x AS FLOAT = 240.0
DIM y AS FLOAT = 160.0
DIM sx[STERNE] AS INTEGER
DIM sy[STERNE] AS INTEGER
DIM punkte AS INTEGER = 0
DIM i AS INTEGER
FOR i = 0 TO STERNE - 1
    sx[i] = RANDINT(20, 460)
    sy[i] = RANDINT(40, 300)
NEXT

WHILE NOT QUITREQUESTED()
    DIM dt AS FLOAT : dt = DELTA()
    IF KEY_DOWN(KEY_LEFT) THEN x = x - TEMPO * dt
    IF KEY_DOWN(KEY_RIGHT) THEN x = x + TEMPO * dt
    IF KEY_DOWN(KEY_UP) THEN y = y - TEMPO * dt
    IF KEY_DOWN(KEY_DOWN) THEN y = y + TEMPO * dt
    x = CLAMP(x, 14.0, 466.0)
    y = CLAMP(y, 14.0, 306.0)
    FOR i = 0 TO STERNE - 1
        IF ABS(x - sx[i]) < 20 AND ABS(y - sy[i]) < 20 THEN
            punkte = punkte + 1
            sx[i] = RANDINT(20, 460)
            sy[i] = RANDINT(40, 300)
        END IF
    NEXT

    CLS(RGB(16, 22, 36))
    FOR i = 0 TO STERNE - 1
        CIRCLE(sx[i], sy[i], 7, RGB(255, 220, 80))
    NEXT
    CIRCLE(INT(x), INT(y), 14, RGB(255, 120, 40))
    CIRCLE(INT(x) + 5, INT(y) - 4, 3, RGB(255, 255, 255))
    TEXT(10, 8, "Punkte: " + STR$(punkte), RGB(230, 236, 248))
    FLIP()
WEND`,
    src_en: String.raw`' Mini game: collect the stars! Arrow keys move the dragon.
' Try it: change SPEED or STARS and run again.
CONST SPEED = 220.0
CONST STARS = 5
SCREEN(480, 320, "Mini game", 2)

DIM x AS FLOAT = 240.0
DIM y AS FLOAT = 160.0
DIM sx[STARS] AS INTEGER
DIM sy[STARS] AS INTEGER
DIM score AS INTEGER = 0
DIM i AS INTEGER
FOR i = 0 TO STARS - 1
    sx[i] = RANDINT(20, 460)
    sy[i] = RANDINT(40, 300)
NEXT

WHILE NOT QUITREQUESTED()
    DIM dt AS FLOAT : dt = DELTA()
    IF KEY_DOWN(KEY_LEFT) THEN x = x - SPEED * dt
    IF KEY_DOWN(KEY_RIGHT) THEN x = x + SPEED * dt
    IF KEY_DOWN(KEY_UP) THEN y = y - SPEED * dt
    IF KEY_DOWN(KEY_DOWN) THEN y = y + SPEED * dt
    x = CLAMP(x, 14.0, 466.0)
    y = CLAMP(y, 14.0, 306.0)
    FOR i = 0 TO STARS - 1
        IF ABS(x - sx[i]) < 20 AND ABS(y - sy[i]) < 20 THEN
            score = score + 1
            sx[i] = RANDINT(20, 460)
            sy[i] = RANDINT(40, 300)
        END IF
    NEXT

    CLS(RGB(16, 22, 36))
    FOR i = 0 TO STARS - 1
        CIRCLE(sx[i], sy[i], 7, RGB(255, 220, 80))
    NEXT
    CIRCLE(INT(x), INT(y), 14, RGB(255, 120, 40))
    CIRCLE(INT(x) + 5, INT(y) - 4, 3, RGB(255, 255, 255))
    TEXT(10, 8, "Score: " + STR$(score), RGB(230, 236, 248))
    FLIP()
WEND`,
  },
  {
    id: "formular",
    name: "★ Formular",
    name_en: "★ Form",
    titel: "Startprojekt: ein Fenster mit Eingabe, Knopf und Liste",
    titel_en: "Starter project: a window with input, button and list",
    src: String.raw`' Formular: ein Fenster mit Eingabe, Knopf und Liste.
' Zum Ausprobieren: den Gruss aendern oder einen zweiten Knopf dazubauen.
IMPORT "gui"
SCREEN(420, 300, "Formular", 1)

DIM w AS GUI_WINDOW : w = GUI_WINDOW("Gruesse", 10, 10, 400, 280)
GUI_LABEL(w, "Wie heisst du?", 20, 20)
DIM eingabe AS GUI_WIDGET : eingabe = GUI_TEXTINPUT(w, 20, 46, 240, 30, "Name")
DIM knopf AS GUI_WIDGET : knopf = GUI_BUTTON(w, "Gruessen", 270, 46, 110, 30)
DIM liste AS GUI_WIDGET : liste = GUI_LISTBOX(w, 20, 90, 360, 160, [])
GUI_LISTBOX_PLACEHOLDER(liste, "Noch niemand gegruesst.")

WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    IF GUI_CLICKED(knopf) OR GUI_ENTERED(eingabe) THEN
        IF GUI_GET_TEXT$(eingabe) <> "" THEN
            GUI_LISTBOX_ADD(liste, "Hallo, " + GUI_GET_TEXT$(eingabe) + "!")
            GUI_SET_TEXT(eingabe, "")
        END IF
    END IF
    CLS(RGB(30, 34, 44))
    GUI_DRAW()
    FLIP()
WEND`,
    src_en: String.raw`' Form: a window with an input, a button and a list.
' Try it: change the greeting or add a second button.
IMPORT "gui"
SCREEN(420, 300, "Form", 1)

DIM w AS GUI_WINDOW : w = GUI_WINDOW("Greetings", 10, 10, 400, 280)
GUI_LABEL(w, "What is your name?", 20, 20)
DIM nameInput AS GUI_WIDGET : nameInput = GUI_TEXTINPUT(w, 20, 46, 240, 30, "Name")
DIM button AS GUI_WIDGET : button = GUI_BUTTON(w, "Greet", 270, 46, 110, 30)
DIM list AS GUI_WIDGET : list = GUI_LISTBOX(w, 20, 90, 360, 160, [])
GUI_LISTBOX_PLACEHOLDER(list, "Nobody greeted yet.")

WHILE NOT QUITREQUESTED()
    GUI_UPDATE()
    IF GUI_CLICKED(button) OR GUI_ENTERED(nameInput) THEN
        IF GUI_GET_TEXT$(nameInput) <> "" THEN
            GUI_LISTBOX_ADD(list, "Hello, " + GUI_GET_TEXT$(nameInput) + "!")
            GUI_SET_TEXT(nameInput, "")
        END IF
    END IF
    CLS(RGB(30, 34, 44))
    GUI_DRAW()
    FLIP()
WEND`,
  },
  {
    id: "malen",
    name: "★ Malen",
    name_en: "★ Paint",
    titel: "Startprojekt: mit der Maus malen -- DICKE aendern",
    titel_en: "Starter project: paint with the mouse -- change SIZE",
    src: String.raw`' Malen: mit gedrueckter Maustaste zeichnen.
' LEER = neue Farbe, C = Blatt leeren. Zum Ausprobieren: DICKE aendern.
CONST DICKE = 8
SCREEN(480, 320, "Malen", 2)

DIM blatt AS INTEGER : blatt = RENDERTARGET_NEW(480, 320, TRUE)   ' TRUE: das Blatt behaelt, was darauf steht
RENDERTARGET_CLEAR(blatt, RGB(250, 248, 240))
DIM farbe AS INTEGER : farbe = RGB(220, 80, 40)
DIM altX AS INTEGER : altX = MOUSE_X()
DIM altY AS INTEGER : altY = MOUSE_Y()

WHILE NOT QUITREQUESTED()
    IF MOUSE_DOWN(0) THEN
        RENDERTARGET_BEGIN(blatt)
        LINEW(altX, altY, MOUSE_X(), MOUSE_Y(), DICKE, farbe)
        CIRCLE(MOUSE_X(), MOUSE_Y(), DICKE \ 2, farbe)
        RENDERTARGET_END()
    END IF
    altX = MOUSE_X()
    altY = MOUSE_Y()
    IF KEY_HIT(KEY_SPACE) THEN farbe = HSV(RANDINT(0, 359), 0.75, 0.9)
    IF KEY_HIT(KEY_C) THEN RENDERTARGET_CLEAR(blatt, RGB(250, 248, 240))

    CLS()
    RENDERTARGET_DRAW(blatt, 0, 0)
    BOX(8, 8, 28, 28, farbe)
    TEXT(36, 12, "LEER = Farbe, C = leeren", RGB(60, 60, 70))
    FLIP()
WEND`,
    src_en: String.raw`' Paint: draw while holding the mouse button.
' SPACE = new colour, C = clear the sheet. Try it: change SIZE.
CONST SIZE = 8
SCREEN(480, 320, "Paint", 2)

DIM sheet AS INTEGER : sheet = RENDERTARGET_NEW(480, 320, TRUE)   ' TRUE: the sheet keeps what is drawn on it
RENDERTARGET_CLEAR(sheet, RGB(250, 248, 240))
DIM colour AS INTEGER : colour = RGB(220, 80, 40)
DIM lastX AS INTEGER : lastX = MOUSE_X()
DIM lastY AS INTEGER : lastY = MOUSE_Y()

WHILE NOT QUITREQUESTED()
    IF MOUSE_DOWN(0) THEN
        RENDERTARGET_BEGIN(sheet)
        LINEW(lastX, lastY, MOUSE_X(), MOUSE_Y(), SIZE, colour)
        CIRCLE(MOUSE_X(), MOUSE_Y(), SIZE \ 2, colour)
        RENDERTARGET_END()
    END IF
    lastX = MOUSE_X()
    lastY = MOUSE_Y()
    IF KEY_HIT(KEY_SPACE) THEN colour = HSV(RANDINT(0, 359), 0.75, 0.9)
    IF KEY_HIT(KEY_C) THEN RENDERTARGET_CLEAR(sheet, RGB(250, 248, 240))

    CLS()
    RENDERTARGET_DRAW(sheet, 0, 0)
    BOX(8, 8, 28, 28, colour)
    TEXT(36, 12, "SPACE = colour, C = clear", RGB(60, 60, 70))
    FLIP()
WEND`,
  },
  {
    name: "Hallo",
    name_en: "Hello",
    src: [
      'PRINT "Hallo Welt!"',
      'DIM i AS INTEGER',
      'FOR i = 1 TO 5',
      '    PRINT STR$(i) + " zum Quadrat ist " + STR$(i * i)',
      'NEXT',
    ].join("\n"),
  },
  {
    name: "Plasma",
    name_en: "Plasma",
    src: [
      "' Farbverlauf pro Pixelblock -- klassisches Demo-Motiv.",
      'SCREEN(320, 200, "Plasma", 2)',
      "DIM t AS FLOAT",
      "DIM x AS INTEGER",
      "DIM y AS INTEGER",
      "DIM v AS FLOAT",
      "WHILE NOT QUITREQUESTED()",
      "    t = t + 0.03",
      "    FOR y = 0 TO 49",
      "        FOR x = 0 TO 79",
      "            v = SIN(x * 0.15 + t) + SIN(y * 0.19 - t) + SIN((x + y) * 0.11 + t)",
      "            BOX(x * 4, y * 4, x * 4 + 3, y * 4 + 3, RGB(128 + INT(SIN(v) * 127.0), 40, 128 + INT(COS(v * 1.3) * 127.0)))",
      "        NEXT",
      "    NEXT",
      "    FLIP()",
      "WEND",
    ].join("\n"),
  },
  {
    name: "Sternenflug",
    name_en: "Starfield",
    src: [
      "' PLOTS zeichnet alle Sterne in EINEM Aufruf statt in einer Schleife.",
      'SCREEN(480, 300, "Sterne", 1)',
      "CONST N AS INTEGER = 700",
      "DIM sx[N] AS FLOAT",
      "DIM sy[N] AS FLOAT",
      "DIM sz[N] AS FLOAT",
      "DIM px[N] AS INTEGER",
      "DIM py[N] AS INTEGER",
      "DIM farbe[N] AS INTEGER",
      "DIM i AS INTEGER",
      "DIM h AS INTEGER",
      "RANDOMIZE(7)",
      "FOR i = 0 TO N - 1",
      "    sx[i] = RND() * 2.0 - 1.0",
      "    sy[i] = RND() * 2.0 - 1.0",
      "    sz[i] = RND() * 0.9 + 0.1",
      "NEXT",
      "WHILE NOT QUITREQUESTED()",
      "    CLS(&H050810)",
      "    FOR i = 0 TO N - 1",
      "        sz[i] = sz[i] - 0.006",
      "        IF sz[i] < 0.05 THEN sz[i] = 1.0",
      "        px[i] = 240 + INT(sx[i] / sz[i] * 240.0)",
      "        py[i] = 150 + INT(sy[i] / sz[i] * 150.0)",
      "        h = 255 - INT(sz[i] * 130.0)",
      "        farbe[i] = RGB(h, h, 255)",
      "    NEXT",
      "    PLOTS(px, py, farbe, N)",
      "    FLIP()",
      "WEND",
    ].join("\n"),
  },
  {
    name: "3D + Licht",
    name_en: "3D + light",
    src: [
      "' Braucht WebGL 2 -- Beleuchtung und PBR laufen im Browser.",
      'IMPORT "g3d"',
      'SCREEN(480, 300, "3D", 1)',
      "LIGHT_ENABLE()",
      "LIGHT_AMBIENT(&H303848, 0.6)",
      "LIGHT_DIRECTIONAL(-0.5, -0.8, -0.4, &HFFE0B0)",
      "LIGHT_ENV(&H6090E0, &H201810, 1.0)",
      "DIM kugel AS INTEGER",
      "kugel = MESH_SPHERE(1.0, 24, 24)",
      "MODEL_LIT(kugel)",
      "MODEL_PBR(kugel, 0.9, 0.2)",
      "DIM t AS FLOAT",
      "WHILE NOT QUITREQUESTED()",
      "    t = t + 0.01",
      "    CLS(&H0A0E16)",
      "    CAMERA3D(SIN(t) * 6.0, 2.0, COS(t) * 6.0, 0.0, 0.0, 0.0, 45.0)",
      "    MODEL(kugel, -1.8, 0.0, 0.0, 1.0, &HFF7040)",
      "    MODEL(kugel, 1.8, 0.0, 0.0, 1.0, &H70A0FF)",
      "    CUBE(0.0, 0.0, 0.0, 1.2, 1.2, 1.2, &H80FF80)",
      "    GRID3D(12, 1.0)",
      "    FLIP()",
      "WEND",
    ].join("\n"),
  },
  {
    name: "Physik",
    name_en: "Physics",
    src: [
      "' Echte Starrkoerper-Physik (Rapier2D) -- Klick wirft einen Ball.",
      'IMPORT "physics2d"',
      'SCREEN(480, 320, "Physik", 1)',
      "DIM welt AS PHYS2D_WORLD",
      "welt = PHYS2D_NEW()",
      "PHYS2D_ADD_BOX(welt, 240.0, 310.0, 240.0, 10.0, FALSE, 0.4)",
      "PHYS2D_ADD_BOX(welt, 20.0, 160.0, 10.0, 160.0, FALSE, 0.4)",
      "PHYS2D_ADD_BOX(welt, 460.0, 160.0, 10.0, 160.0, FALSE, 0.4)",
      "DIM i AS INTEGER",
      "FOR i = 0 TO 11",
      "    PHYS2D_ADD_CIRCLE(welt, 120.0 + i * 20.0, 40.0 + i * 12.0, 10.0, TRUE, 0.6)",
      "NEXT",
      "WHILE NOT QUITREQUESTED()",
      "    IF MOUSE_HIT(1) THEN",
      "        PHYS2D_ADD_CIRCLE(welt, MOUSEX() * 1.0, MOUSEY() * 1.0, 10.0, TRUE, 0.7)",
      "    END IF",
      "    PHYS2D_STEP(welt, 1.0 / 60.0)",
      "    CLS(&H101820)",
      "    BOX(10, 300, 470, 320, &H30506A)",
      "    FOR i = 3 TO PHYS2D_COUNT(welt) - 1",
      "        CIRCLE(INT(PHYS2D_BODY_X(welt, i)), INT(PHYS2D_BODY_Y(welt, i)), 10, &HFFB040)",
      "    NEXT",
      "    TEXT(12, 12, \"Klick wirft einen Ball\", &HFFFFFF)",
      "    FLIP()",
      "WEND",
    ].join("\n"),
  },
  {
    name: "Klang + Spektrum",
    name_en: "Sound + spectrum",
    src: [
      "' Im Browser ist nichts zu HOEREN -- aber das Spektrum ist echt:",
      "' der Mix wird vollstaendig gerechnet, nur nicht ausgegeben.",
      'IMPORT "audio"',
      'SCREEN(480, 300, "Spektrum", 1)',
      "DIM ton AS SOUND",
      "DIM spek[32] AS FLOAT",
      "DIM i AS INTEGER",
      "DIM t AS INTEGER",
      "ton = AUDIO_TONE(220.0, 4000, \"saw\", 0.7)",
      "AUDIO_PLAY(ton, -1, 0.8)",
      "WHILE NOT QUITREQUESTED()",
      "    t = t + 1",
      "    IF t MOD 90 = 0 THEN",
      "        AUDIO_FILTER(\"sfx\", 300.0 + RND() * 4000.0, 2.0)",
      "    END IF",
      "    AUDIO_FFT(spek)",
      "    CLS(&H0A0E16)",
      "    FOR i = 0 TO 31",
      "        BOX(i * 15 + 2, 290 - INT(spek[i] * 260.0), i * 15 + 13, 290, RGB(60 + i * 6, 200 - i * 4, 255))",
      "    NEXT",
      "    TEXT(12, 12, \"AUDIO_FFT -- echtes Spektrum, stumm\", &HFFFFFF)",
      "    FLIP()",
      "WEND",
    ].join("\n"),
  },
];
