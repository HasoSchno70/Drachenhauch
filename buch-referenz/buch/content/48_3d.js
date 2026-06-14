module.exports = (H) => [
  H.chapter("3D-Grafik"),
  H.p("GameBasic kann nicht nur flache 2D-Bilder, sondern auch echte 3D-Welten zeichnen – mit dem Modul g3d. Dazu kommt eine dritte Koordinate hinzu: z (Tiefe). Eine virtuelle Kamera blickt von einem Punkt im Raum auf ein Ziel, und du platzierst Körper wie Würfel, Kugeln und Zylinder an beliebigen Stellen. Dieses Kapitel zeigt die Grundbausteine; 3D ist ein großes Thema, in das du hier nur den ersten Blick wirfst."),
  H.figure("48_3d.png", "Eine einfache 3D-Szene: Boden-Raster, Würfel (mit Drahtgitter), Kugel, Zylinder und Kegel."),
  H.note("3D läuft – wie der ganze Teil IV – nur im gbrt-Fenster. Der Aufbau ist wie in 2D: im Game-Loop CLS, dann zeichnen, dann FLIP. Nur kommt am Anfang jedes Frames die Kamera dazu, und alle Befehle stammen aus dem Modul g3d (IMPORT \"g3d\")."),

  H.h2("Die Kamera"),
  H.cmd("CAMERA3D", 'CAMERA3D(px, py, pz, tx, ty, tz, fov)',
    "Setzt die Kamera für diesen Frame: Sie steht am Punkt (px, py, pz) und blickt auf das Ziel (tx, ty, tz). fov ist der Blickwinkel in Grad (z. B. 45). Diesen Aufruf machst du als Erstes im 3D-Teil jedes Frames – noch vor den Körpern.",
    [
      'IMPORT "g3d"',
      'SCREEN(480, 320)',
      'WHILE NOT QUITREQUESTED()',
      '    CLS(RGB(16, 22, 34))',
      '    CAMERA3D(7.0, 5.5, 9.0,  0.0, 0.5, 0.0,  45.0)',
      '    \' ... 3D-Körper zeichnen ...',
      '    FLIP()',
      'WEND',
    ]),
  H.note("Eine kreisende Kamera baust du, indem du ihre Position über die Zeit mit SIN/COS berechnest: cx = COS(t) * radius, cz = SIN(t) * radius, und t pro Frame erhöhst. So umrundet der Blick das Geschehen."),

  H.h2("Orientierung: das Boden-Raster"),
  H.cmd("GRID3D", 'GRID3D(linien, abstand)',
    "Zeichnet ein Gitter in der Bodenebene – sehr hilfreich, um sich im Raum zu orientieren. linien ist die Anzahl der Linien pro Richtung, abstand ihr Abstand.",
    [
      'GRID3D(16, 1.0)        \' 16x16 Raster, 1 Einheit Abstand',
    ]),

  H.h2("Körper zeichnen"),
  H.p("Die Grundkörper bekommen ihre Position als (x, y, z) und ihre Maße, dann eine Farbe. Zu jedem gefüllten Körper gibt es meist eine Drahtgitter-Variante (…_WIRES), die nur die Kanten zeichnet – schön, um Konturen zu betonen."),
  H.cmd("CUBE · CUBE_WIRES", 'CUBE(x, y, z, breite, höhe, tiefe, farbe)   CUBE_WIRES(...)',
    "Zeichnet einen Quader am Mittelpunkt (x, y, z) mit den drei Kantenlängen. CUBE füllt, CUBE_WIRES zeichnet nur das Drahtgitter (gut kombinierbar: erst CUBE, dann CUBE_WIRES in dunkler Farbe).",
    [
      'CUBE(0.0, 1.0, 0.0,  2.0, 2.0, 2.0,  RED)',
      'CUBE_WIRES(0.0, 1.0, 0.0,  2.0, 2.0, 2.0,  BLACK)',
    ]),
  H.cmd("SPHERE · SPHERE_WIRES", 'SPHERE(x, y, z, radius, farbe)   SPHERE_WIRES(...)',
    "Zeichnet eine Kugel mit Mittelpunkt (x, y, z) und dem Radius.",
    [
      'SPHERE(4.0, 1.0, 0.0,  1.0,  BLUE)',
    ]),
  H.cmd("CYLINDER", 'CYLINDER(x, y, z, radius_oben, radius_unten, höhe, farbe)',
    "Zeichnet einen Zylinder. Mit unterschiedlichen Radien wird er ein Kegelstumpf; setzt du radius_oben auf 0, entsteht ein spitzer Kegel.",
    [
      'CYLINDER(-4.0, 0.0, 0.0,  0.8, 0.8, 2.5,  GREEN)   \' Zylinder',
      'CYLINDER(0.0, 0.0, -4.0,  0.0, 1.2, 2.2,  YELLOW)  \' Kegel',
    ]),
  H.cmd("PLANE · LINE3D · POINT3D", 'PLANE(x, y, z, größe_x, größe_z, farbe)   LINE3D(x1,y1,z1, x2,y2,z2, farbe)   POINT3D(x, y, z, farbe)',
    "PLANE zeichnet eine flache Fläche in der Bodenebene, LINE3D eine Linie zwischen zwei Raumpunkten, POINT3D einen einzelnen Punkt. Gut für Böden, Achsen und Markierungen.",
    [
      'PLANE(0.0, 0.0, 0.0,  10.0, 10.0,  RGB(40, 60, 40))',
      'LINE3D(0.0, 0.0, 0.0,  0.0, 3.0, 0.0,  WHITE)   \' Y-Achse',
    ]),

  H.h2("Mehr als Grundkörper"),
  H.p("Das g3d-Modul kann weit mehr, als hier gezeigt – das füllt eigene Bücher. Ein kurzer Ausblick, damit du weißt, was möglich ist:"),
  H.bulletRich("3D-Modelle:  ", "LOADMODEL lädt fertige Modelle (OBJ/GLTF), MESH_CUBE/SPHERE/TORUS u. a. erzeugen welche prozedural; gezeichnet mit MODEL. Sogar Skelett-Animationen sind möglich."),
  H.bulletRich("Beleuchtung:  ", "LIGHT_ENABLE, LIGHT_DIRECTIONAL/POINT und MODEL_LIT geben Modellen echtes Licht und Schatten (bis hin zu PBR-Materialien und Nebel)."),
  H.bulletRich("Kamera-Modi & Picking:  ", "CAMERA3D_UPDATE bietet fertige Steuerungen (frei, orbital, Ego-Perspektive); mit PICK_BOX/PICK_SPHERE wählst du Objekte per Mausklick aus."),
  H.tip("Klein anfangen", "3D ist mächtig, aber der Einstieg ist anspruchsvoller als 2D. Fang mit einer festen Kamera und ein paar Grundkörpern auf einem Raster an (wie im Bild oben), bring dann Bewegung in die Kamera, und taste dich erst danach an Modelle und Beleuchtung heran. Für die meisten 2D-Spiele brauchst du 3D übrigens gar nicht."),
];
