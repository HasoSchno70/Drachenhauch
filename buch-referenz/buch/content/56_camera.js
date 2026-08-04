module.exports = (H) => [
  H.chapter("Modul: camera"),
  H.p("Spielwelten sind oft größer als der Bildschirm. Damit du nur einen Ausschnitt zeigst – und ihn mitscrollen lässt, wenn die Figur läuft – gibt es das camera-Modul. Es verschiebt und zoomt die gesamte Zeichnung: Sobald eine Kamera gesetzt ist, gelten deine Koordinaten als Welt-Koordinaten, und die Kamera rechnet sie auf den Bildschirm um. Du zeichnest also weiter „an Weltpositionen“, und der sichtbare Ausschnitt folgt der Kamera."),
  H.figure("56_camera.png", "Eine 800×600-Welt, von der Kamera auf den Spieler (Weltposition 400, 300) zentriert – die HUD-Leiste oben steht fest (ohne Kamera)."),

  H.h2("Die Kamera setzen"),
  H.cmd("CAMERA_SET", 'CAMERA_SET(x, y[, zoom])',
    "Legt fest, welcher Welt-Punkt (x, y) oben links auf dem Bildschirm landet, und optional einen Zoom (1.0 = normal, 2.0 = doppelt so groß/näher, 0.5 = mehr Welt sichtbar). Ab jetzt sind alle Zeichen-Koordinaten Welt-Koordinaten.",
    [
      'IMPORT "camera"',
      'CAMERA_SET(100.0, 50.0)        \' Welt-Punkt (100,50) -> Bild oben links',
      'CAMERA_SET(0.0, 0.0, 2.0)      \' alles doppelt so groß',
    ]),
  H.cmd("CAMERA_FOLLOW", 'CAMERA_FOLLOW(ziel_x, ziel_y, bild_breite, bild_höhe)',
    "Setzt die Kamera so, dass das Ziel (ziel_x, ziel_y) im Bildschirm-Zentrum landet – ideal, um der Spielfigur zu folgen. Die Bildbreite/-höhe gibst du meist als SCREENWIDTH()/SCREENHEIGHT() an.",
    [
      'CAMERA_FOLLOW(spieler_x, spieler_y, 480.0, 320.0)',
    ]),
  H.cmd("CAMERA_RESET", 'CAMERA_RESET()',
    "Schaltet die Kamera wieder aus (zurück zu 0,0 und Zoom 1). Wichtig vor dem Zeichnen des HUD: Punkte, Leben, Menüs sollen fest am Bildschirm kleben und nicht mitscrollen.",
    [
      'CAMERA_RESET()',
      'TEXT(10, 10, "Punkte: 100")    \' fester HUD-Text',
    ]),

  H.h2("Bildschirm zurück in die Welt rechnen"),
  H.cmd("CAMERA_S2W_X · CAMERA_S2W_Y", 'CAMERA_S2W_X(sx)   CAMERA_S2W_Y(sy)',
    "Rechnen einen Bildschirm-Pixel zurück in eine Welt-Koordinate – unentbehrlich, um herauszufinden, WO in der Welt der Mauszeiger steht (etwa zum Anklicken eines Objekts).",
    [
      'DIM wx AS FLOAT',
      'DIM wy AS FLOAT',
      'wx = CAMERA_S2W_X(MOUSEX() * 1.0)   \' Maus -> Weltposition',
      'wy = CAMERA_S2W_Y(MOUSEY() * 1.0)',
    ]),

  H.h2("Rütteln: CAMERA_SHAKE"),
  H.cmd("CAMERA_SHAKE", 'CAMERA_SHAKE(stärke[, dauer_ms])',
    "Lässt die Kamera kurz wackeln – ein klassischer Wumms-Effekt bei Treffern oder Explosionen. Die Stärke ist in Welt-Pixeln, die Erschütterung klingt über dauer_ms (Standard 300) von selbst ab. Du rufst es einmalig auf; kein Pro-Frame-Code nötig.",
    [
      'IF getroffen THEN CAMERA_SHAKE(6.0, 250)   \' kurzer Wumms',
    ]),

  H.h2("Im Game-Loop"),
  H.p("Das übliche Muster: am Frame-Anfang der Figur folgen, die Welt in Welt-Koordinaten zeichnen, dann für das HUD die Kamera zurücksetzen:"),
  H.code([
    'IMPORT "camera"',
    'WHILE NOT QUITREQUESTED()',
    '    CAMERA_FOLLOW(px, py, SCREENWIDTH() * 1.0, SCREENHEIGHT() * 1.0)',
    '    CLS(RGB(18, 22, 36))',
    '    \' ... Welt zeichnen (Welt-Koordinaten) ...',
    '    CIRCLE(INT(px), INT(py), 16, YELLOW)   \' Spieler in der Welt',
    '',
    '    CAMERA_RESET()                          \' ab hier fester Bildschirm',
    '    TEXT(10, 10, "HUD")',
    '    FLIP()',
    'WEND',
  ]),
  H.note("Die Kamera wirkt auf alle Welt-Zeichenbefehle – BOX, CIRCLE, LINE, TEXT, DRAWIMAGE, DRAWTILEMAP, SPRITE_DRAW und PARTICLE_DRAW. Den Text-HUD also bewusst NACH CAMERA_RESET() zeichnen, damit er nicht mitwandert."),
  H.h2("Kamera abfragen und drehen"),
  H.cmd("CAMERA_X · CAMERA_Y · CAMERA_ZOOM", 'CAMERA_X()   CAMERA_Y()   CAMERA_ZOOM()',
    "Lesen die aktuelle Kamera-Position und den Zoom zurück – etwa nachdem CAMERA_FOLLOW sie weich nachgezogen hat, oder um den Stand im Spielstand zu sichern.",
    [
      'PRINT STR$(CAMERA_X()) + " / " + STR$(CAMERA_Y())',
    ]),
  H.cmd("CAMERA_SET_ROTATION · CAMERA_ROTATION", 'CAMERA_SET_ROTATION(grad)   CAMERA_ROTATION()',
    "Dreht die Weltansicht um die Bildschirmmitte – für ein Erdbeben, einen Sturz oder ein Spiel, in dem sich die Welt statt der Figur dreht.",
    [
      'CAMERA_SET_ROTATION(SIN(GET_TIME() * 8.0) * 3.0)   \' leichtes Wackeln',
    ]),
  H.warn("CAMERA_SET_ROTATION verschiebt, wohin ein Objekt gezeichnet wird – die Objekte selbst bleiben aufrecht. Ein Sprite, das sich mitdrehen soll, bekommt den Winkel zusätzlich selbst: DRAWIMAGEROT(bild, x, y, eigener_winkel + CAMERA_ROTATION()).", "Die Drehung dreht Positionen, nicht Formen"),
  H.cmd("CAMERA_ORBIT", 'CAMERA_ORBIT(zx, zy, zz, radius, gier, nick[, fov])',
    "Setzt die 3D-Kamera über Kugelkoordinaten: Sie schaut auf das Ziel (zx, zy, zz) und steht im Abstand radius, gedreht um gier (waagerecht) und nick (senkrecht, in Grad). Erspart die Trigonometrie, wenn die Kamera ein Objekt umkreisen soll.",
    [
      'CAMERA_ORBIT(0.0, 1.0, 0.0, 12.0, winkel, 25.0, 45.0)',
      'winkel = winkel + 0.5',
    ]),
  H.tip("Kamera-Tempo", "Statt die Kamera hart auf den Spieler zu setzen, kannst du ihr weich folgen lassen: die Kamera-Position pro Frame ein Stück Richtung Ziel bewegen (z. B. mit MOVETOWARD oder LERP). Das ergibt ein angenehmes, leicht nachziehendes Gefühl statt eines starren Klebens."),
];
