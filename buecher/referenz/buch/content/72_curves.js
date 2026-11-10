module.exports = (H) => [
  H.chapter("Modul: curves"),
  H.p("Das curves-Modul rechnet einzelne Punkte auf einer Kurve aus. Während das tween-Modul (Kapitel „Modul: tween“) einen Wert über die Zeit von A nach B animiert, beantwortet curves die Frage „welcher Wert liegt bei Fortschritt t auf dieser Kurve?“. Damit lässt du Objekte weiche Bögen fliegen, Kameras sanft schwenken oder Werte elegant einpendeln. Alle Befehle sind reine Funktionen – kein Zustand, kein Setup."),
  H.figure("72_curves.png", "Zwei Kurventypen durch dieselbe Gegend: Die Bezier-Kurve (gelb) wird von ihren Handles nur angezogen, die Catmull-Rom-Kurve (cyan) läuft direkt durch ihre Stützpunkte."),
  H.pmix([
    "Alle Kurven bekommen einen Parameter ",
    ['t', true],
    " (meist von 0 bis 1) und liefern den interpolierten Wert. Sie unterscheiden sich darin, ",
    [' wie ', true],
    "die Kontrollpunkte den Verlauf formen – von der geraden Linie bis zur frei geschwungenen Bahn.",
  ]),

  H.h2("Die einfachen: LERP, SMOOTHSTEP, SMOOTHERSTEP"),
  H.p("CURVE_LERP zieht eine gerade Linie zwischen zwei Werten. SMOOTHSTEP und SMOOTHERSTEP sind S-Kurven: sie starten und enden sanft (langsam beschleunigen, langsam abbremsen) – das typische „Ease-In-Out“-Gefühl. SMOOTHERSTEP ist die noch weichere Variante."),
  H.cmd("CURVE_LERP", 'CURVE_LERP(a, b, t)',
    "Lineare Interpolation: a + (b - a) * t. Bei t=0 kommt a heraus, bei t=1 kommt b heraus, bei t=0.5 die Mitte.",
    [
      'IMPORT "curves"',
      'PRINT CURVE_LERP(100.0, 200.0, 0.3)',
    ], { out: ["130.0"] }),
  H.cmd("SMOOTHSTEP · CURVE_SMOOTHSTEP · CURVE_SMOOTHERSTEP", 'SMOOTHSTEP(kante0, kante1, x)   CURVE_SMOOTHSTEP(kante0, kante1, x)   CURVE_SMOOTHERSTEP(kante0, kante1, x)',
    "Normalisiert x auf den Bereich [kante0, kante1] (geclamped auf 0..1) und legt eine S-Kurve darüber. Ideal für weiche Ein-/Ausblendungen. SMOOTHSTEP und CURVE_SMOOTHSTEP rechnen dasselbe – der Unterschied ist nur, dass der erste ein Kern-Befehl ist und ohne IMPORT auskommt.",
    [
      'IMPORT "curves"',
      'PRINT CURVE_SMOOTHSTEP(0.0, 1.0, 0.3)',
      'PRINT CURVE_SMOOTHERSTEP(0.0, 1.0, 0.3)   \' ~0.163, noch sanfter',
    ], { out: ["0.216", "0.16308000000000003"] }),

  H.h2("Bezier: von Handles geführt"),
  H.p("Eine kubische Bezier-Kurve hat vier Punkte. P0 ist der Start, P3 das Ende – die Kurve berührt beide. P1 und P2 sind „Handles“: Sie ziehen die Kurve in ihre Richtung, ohne dass sie hindurchläuft. So formst du Sprungbögen oder elegante Kamerafahrten. Im Bild oben sind die grauen Linien die Handles, die gelbe Bahn die Kurve."),
  H.cmd("CURVE_BEZIER", 'CURVE_BEZIER(t, p0, p1, p2, p3)',
    "Kubische Bezier-Kurve in einer Dimension. Liefert den interpolierten Skalar bei t.",
    [
      'IMPORT "curves"',
      'PRINT CURVE_BEZIER(0.5, 0.0, 0.0, 100.0, 100.0)',
    ], { out: ["50.0"] }),
  H.cmd("CURVE_BEZIER2", 'CURVE_BEZIER2(t, x0,y0, x1,y1, x2,y2, x3,y3)',
    "Bezier-Kurve in der Ebene: vier Punkte als (x, y), Rückgabe ist ein TUPLE (x, y). Perfekt, um ein Objekt eine Bahn entlang zu schicken.",
    [
      'IMPORT "curves"',
      'DIM p AS TUPLE',
      'p = CURVE_BEZIER2(0.5, 0.0,200.0, 100.0,50.0, 200.0,50.0, 300.0,200.0)',
      'PRINT p[0], p[1]',
    ], { out: ["150.0 87.5"] }),

  H.h2("Catmull-Rom: durch die Punkte"),
  H.p("Anders als Bezier läuft eine Catmull-Rom-Kurve direkt DURCH ihre mittleren Punkte (P1 und P2). P0 und P3 sind nur die Nachbarn, die für einen weichen Übergang sorgen. Das ist die erste Wahl, wenn ein Objekt eine Reihe vorgegebener Wegpunkte glatt abfahren soll – etwa eine Patrouille oder ein Rennstreckenverlauf."),
  H.cmd("CURVE_CATMULL · CURVE_CATMULL2", 'CURVE_CATMULL(t, p0, p1, p2, p3)   CURVE_CATMULL2(t, x0,y0, x1,y1, x2,y2, x3,y3)',
    "Catmull-Rom-Spline 1D bzw. 2D. Die Kurve verläuft von P1 nach P2 und berührt beide; P0/P3 bestimmen die Tangenten für einen sanften Anschluss an die Nachbarsegmente.",
    [
      'IMPORT "curves"',
      'PRINT CURVE_CATMULL(0.5, 0.0, 10.0, 20.0, 30.0)',
    ], { out: ["15.0"] }),
  H.note("Für einen Pfad aus vielen Wegpunkten fährst du die Segmente nacheinander ab: zwischen Punkt 1 und 2 (mit 0 und 3 als Nachbarn), dann zwischen 2 und 3 (mit 1 und 4 als Nachbarn) usw. – jedes Segment nutzt vier aufeinanderfolgende Punkte."),

  H.h2("Hermite: Tangenten von Hand"),
  H.cmd("CURVE_HERMITE", 'CURVE_HERMITE(t, p0, p1, m0, m1)',
    "Interpoliert von p0 nach p1 mit ausdrücklich vorgegebenen Tangenten (Geschwindigkeiten) m0 am Start und m1 am Ende. Beide Tangenten 0 ergibt eine sanfte S-Kurve wie SmoothStep.",
    [
      'IMPORT "curves"',
      'PRINT CURVE_HERMITE(0.5, 0.0, 100.0, 0.0, 0.0)',
    ], { out: ["50.0"] }),

  H.h2("curves und tween zusammen"),
  H.p("Beide Module ergänzen sich. tween animiert einen Wert zuverlässig über eine Dauer; curves verwandelt einen Fortschritt in eine Position auf einer Bahn. Lässt du ein tween den Parameter t von 0 nach 1 über zwei Sekunden laufen und steckst t in CURVE_BEZIER2, fliegt dein Objekt eine zeitlich exakte, weiche Kurve."),
  H.code([
    'IMPORT "curves"',
    '\' Sanftes Kamera-Nachziehen (jeden Frame 10 % naeher ans Ziel):',
    'cam_x = CURVE_LERP(cam_x, ziel_x, 0.1)',
    'cam_y = CURVE_LERP(cam_y, ziel_y, 0.1)',
  ]),
  H.tip("Das LERP-Nachzieh-Muster", "CURVE_LERP(jetzt, ziel, 0.1) jeden Frame aufzurufen, ist ein winziger Trick mit großer Wirkung: Der Wert nähert sich dem Ziel immer um denselben Bruchteil und bremst dadurch von selbst weich ab. So entstehen butterweiche Kamerafahrten, nachlaufende Lebensbalken und träge folgende Verfolger – mit einer einzigen Zeile pro Frame."),
];
