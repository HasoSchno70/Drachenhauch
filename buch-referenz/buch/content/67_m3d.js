module.exports = (H) => [
  H.chapter("Modul: m3d"),
  H.p("Was vec2 für die Fläche ist, ist m3d für den Raum: 3D-Mathematik für die Arbeit mit dem g3d-Modul. Es bietet vier Werttypen – VEC3 (3D-Vektor), VEC4, QUAT (Quaternion für Drehungen) und MAT4 (4×4-Transformationsmatrix). Damit positionierst, drehst und skalierst du 3D-Objekte und reichst das Ergebnis als Matrix an den Renderer. Alle Werte sind unveränderlich, und +, -, * sind überladen."),
  H.figure("67_m3d.png", "Fünf Würfel, jeder mit MAT4_TRS aus Position, einer Quaternion-Drehung und Skalierung platziert und per MODEL_MATRIX gezeichnet."),

  H.h2("VEC3 – der 3D-Vektor"),
  H.cmd("VEC3_NEW · VEC3_X/Y/Z · Operatoren", 'VEC3_NEW(x, y, z)   VEC3_X(v)/Y/Z   v + w   v * skalar',
    "Erzeugt und liest 3D-Vektoren; die Operatoren rechnen wie bei vec2 (komponentenweise addieren/subtrahieren, mit Zahl skalieren).",
    [
      'IMPORT "m3d"',
      'DIM a AS VEC3',
      'a = VEC3_NEW(1.0, 2.0, 2.0)',
      'PRINT a + VEC3_NEW(1.0, 0.0, 0.0)',
      'PRINT VEC3_LENGTH(a)',
    ], { out: ["Vec3(2.0, 2.0, 2.0)", "3.0"] }),
  H.cmd("VEC3_DOT · VEC3_CROSS · VEC3_NORMALIZE", 'VEC3_DOT(a, b)   VEC3_CROSS(a, b)   VEC3_NORMALIZE(v)',
    "DOT ist das Skalarprodukt, CROSS das Kreuzprodukt (liefert einen zu beiden senkrechten Vektor – wichtig für Normalen), NORMALIZE bringt einen Vektor auf Länge 1.",
    [
      'PRINT VEC3_CROSS(VEC3_NEW(1.0,0.0,0.0), VEC3_NEW(0.0,1.0,0.0))',
    ], { out: ["Vec3(0.0, 0.0, 1.0)"] }),

  H.h2("QUAT – Drehungen ohne Stolpern"),
  H.p("Drehungen im Raum stellt man am besten mit Quaternionen dar – sie vermeiden das berüchtigte „Gimbal-Lock“ und lassen sich sauber ineinander überblenden. Du erzeugst eine Drehung meist aus einer Achse und einem Winkel."),
  H.cmd("QUAT_FROM_AXIS_ANGLE · QUAT_MUL · QUAT_SLERP", 'QUAT_FROM_AXIS_ANGLE(ax, ay, az, winkel)   QUAT_MUL(a, b)   QUAT_SLERP(a, b, t)',
    "FROM_AXIS_ANGLE dreht um die Achse (ax, ay, az) um den Winkel (Bogenmaß). QUAT_MUL verkettet zwei Drehungen, QUAT_SLERP blendet weich von einer Drehung zur anderen (für sanfte Kamera-/Objektdrehungen). Auch QUAT_FROM_EULER (Nick/Gier/Roll) ist verfügbar.",
    [
      'DIM rot AS QUAT',
      'rot = QUAT_FROM_AXIS_ANGLE(0.0, 1.0, 0.0, RAD(45.0))   \' 45° um Y',
    ]),

  H.h2("MAT4 – die Transformationsmatrix"),
  H.p("Eine 4×4-Matrix fasst Verschiebung, Drehung und Skalierung in einem Wert zusammen – genau das, was der Renderer braucht. Der bequemste Konstruktor ist MAT4_TRS: er baut „Translation · Rotation · Scale“ in einem Schritt aus einem Positions-Vektor, einer Dreh-Quaternion und einem Skalierungs-Vektor."),
  H.cmd("MAT4_TRS · MAT4_MUL · MAT4_IDENTITY", 'MAT4_TRS(pos, rot, scale)   MAT4_MUL(a, b)   MAT4_IDENTITY()',
    "MAT4_TRS erzeugt die fertige Transformation eines Objekts. MAT4_MUL verkettet zwei Matrizen (für Hierarchien gilt: Welt = MAT4_MUL(eltern, lokal)). MAT4_IDENTITY ist die „neutrale“ Matrix (keine Veränderung). Weiter gibt es MAT4_TRANSLATE/SCALE/INVERT/LOOKAT/PERSPECTIVE u. a.",
    [
      'DIM m AS MAT4',
      'm = MAT4_TRS(VEC3_NEW(0.0, 0.8, 0.0), rot, VEC3_NEW(1.0, 1.0, 1.0))',
    ]),

  H.h2("Rendern: MODEL_MATRIX"),
  H.cmd("MODEL_MATRIX", 'MODEL_MATRIX(modell, matrix [, tint])',
    "Zeichnet ein g3d-Modell (aus MESH_* oder LOADMODEL) mit der angegebenen Transformations-Matrix – so verbindest du m3d mit der 3D-Grafik. So entstand das Bild oben: pro Würfel eine MAT4_TRS, dann MODEL_MATRIX.",
    [
      'IMPORT "g3d"',
      'IMPORT "m3d"',
      'DIM box AS INTEGER',
      'box = MESH_CUBE(1.0, 1.0, 1.0)',
      '\' ... in der 3D-Szene (nach CAMERA3D): ...',
      'MODEL_MATRIX(box, m, RGB(224, 128, 64))',
    ]),
  H.note("Für SEHR viele gleiche Objekte (Wälder, Asteroidenfelder, Armeen) gibt es MODEL_INSTANCED(modell, mats): Es zeichnet ein Mesh mit hunderten Matrizen aus einem Array in EINEM Draw-Call – viel schneller als ein MODEL_MATRIX pro Objekt. Außerdem lassen sich mit CAMERA3D_VIEW/PROJECTION eigene Kamera-Matrizen setzen."),
  H.tip("m3d und g3d gehören zusammen", "g3d (voriges Teil-IV-Kapitel) liefert die Bausteine und Primitive, m3d die Mathematik, um sie präzise zu platzieren und zu drehen. Für einfache Szenen reichen die g3d-Befehle wie CUBE/MODEL; sobald du Hierarchien (Bones, Gelenke), eigene Kameras oder viele Instanzen brauchst, kommt m3d ins Spiel."),
];
