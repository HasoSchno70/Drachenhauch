module.exports = (H) => [
  H.chapter("Selbst malen"),

  H.p("Dieses Kapitel enthält keine einzige Zeile Code. Es geht um ein Werkzeug, das schon auf deinem Rechner liegt, und um eine Fertigkeit, die überraschend schnell zu lernen ist."),

  H.p("Pixelgrafik hat einen unschlagbaren Vorteil für Anfänger: Man braucht kein bisschen Zeichentalent. Ein Sprite von 16 mal 16 Punkten ist ein Karopapier mit 256 Kästchen, und mehr als „welches Kästchen bekommt welche Farbe“ ist nicht zu entscheiden."),

  H.figure("kap20_raster_schiff.png", "Das Schiff aus dem letzten Kapitel — 16 mal 16 Kästchen. Mehr steckt nicht dahinter.", 380, 380),

  H.p("Sieh es dir an: Das ist dasselbe Raumschiff, das du im letzten Kapitel gesteuert hast. Ein weißer Rand, ein hellblauer Rumpf, ein dunkleres Fenster, drei orangefarbene Düsen. Etwa hundert gefärbte Kästchen."),

  H.h2("Den Editor starten"),

  H.p("Drachenhauch bringt einen Pixel-Editor mit. Du startest ihn so:"),

  H.code(["dhsprites"]),

  H.p("Du bekommst eine leere Fläche von 32 mal 32 Punkten. Eine bestehende Datei öffnest du, indem du sie dahinterschreibst:"),

  H.code(["dhsprites schiff.png"]),

  H.note("Von diesem Editor gibt es in diesem Buch keine Abbildung — ein Bildschirmfoto eines Programmfensters altert schneller als alles andere, und du hast das Fenster ja vor dir. Die Beschreibungen hier nennen die Tasten; alles Übrige findest du in zwei Minuten selbst."),

  H.h2("Die Werkzeuge, die du wirklich brauchst"),

  H.table([
    [{ text: "B", mono: true }, "Stift", "Punkte setzen. Linke Maustaste malt die Vordergrundfarbe, rechte die Hintergrundfarbe."],
    [{ text: "E", mono: true }, "Radierer", "Punkte wieder durchsichtig machen"],
    [{ text: "G", mono: true }, "Farbeimer", "eine zusammenhängende Fläche füllen"],
    [{ text: "I", mono: true }, "Pipette", "eine Farbe aus dem Bild aufnehmen"],
    [{ text: "L", mono: true }, "Linie", "gerade Strecken"],
    [{ text: "R", mono: true }, "Rechteck", "gefülltes Rechteck"],
    [{ text: "O", mono: true }, "Ellipse", "gefüllter Kreis"],
  ], { headers: ["Taste", "Werkzeug", "Wofür"], widths: [900, 1800, 6326] }),

  H.p("Mit den Tasten 1 bis 4 stellst du die Stiftbreite ein, mit X vertauschst du Vorder- und Hintergrundfarbe. Speichern ist Strg+S, ein neues Bild anlegen Strg+N — dort wirst du nach Größe und Anzahl der Bilder gefragt."),

  H.tip("Der eine Kniff, der alles leichter macht", "Drück Strg+Shift+X. Damit ist die Spiegelung eingeschaltet: Was du links malst, erscheint automatisch rechts. Für Raumschiffe, Gesichter, Käfer und fast alle Spielfiguren ist das die halbe Arbeit — und das Ergebnis sieht sofort ordentlich aus, weil es wirklich symmetrisch ist."),

  H.h2("Fünf Ratschläge für lesbare Figuren"),

  H.bulletRich("Klein anfangen. ", "16 mal 16 reicht für fast alles in diesem Buch. Größer heißt nicht besser, sondern nur mehr Kästchen, bei denen man sich vertun kann."),
  H.bulletRich("Wenige Farben. ", "Drei bis fünf je Figur. Der Rumpf, ein dunklerer Ton für Schatten, ein heller für Kanten — mehr braucht es selten."),
  H.bulletRich("Eine Umrandung ziehen. ", "Eine dunkle oder helle Linie rings um die Figur hebt sie vom Hintergrund ab. Ohne sie verschwindet ein grünes Männchen auf einer grünen Wiese."),
  H.bulletRich("Die Silhouette zuerst. ", "Male erst den Umriss in einer Farbe und sieh ihn dir klein an. Erkennt man, was es sein soll? Dann erst Farben hineinlegen. Erkennt man es nicht, hilft auch schönes Ausmalen nicht."),
  H.bulletRich("Ausprobieren, wie es im Spiel wirkt. ", "Eine Figur, die im Editor riesig auf dem Bildschirm steht, ist im Spiel 32 Punkte groß. Zeig sie dir immer wieder in der wirklichen Größe an."),

  H.h2("Mehrere Bilder in einer Datei"),

  H.p("Der Editor kann von Anfang an mehrere Bilder auf einmal — er nennt sie Frames. Rechts stehen sie als kleine Vorschauen untereinander; mit den Tasten F2 bis F9 springst du direkt hin, und Strg+P zeigt dir die Folge als laufende Animation."),

  H.p("Beim Speichern als PNG entsteht daraus genau das, was du im letzten Kapitel schon benutzt hast: ein Streifen, in dem die Bilder nebeneinanderliegen."),

  H.figure("kap21_zwei_frames.png", "Zwei Frames desselben Gegners. Nur Fühler und Beine unterscheiden sich — und genau daraus entsteht im nächsten Kapitel Bewegung.", 460, 240),

  H.p("Damit schließt sich der Kreis zum letzten Kapitel: DRAWIMAGEPART schneidet aus so einem Streifen ein einzelnes Feld heraus. Was der Editor speichert, liest dein Programm."),

  H.tip("Zwiebelhaut", "Für Animationen gibt es eine Hilfe, die man einmal gesehen haben muss: das Onion-Skinning. Ist es eingeschaltet, siehst du blass hinter dem aktuellen Bild die Nachbarbilder durchscheinen — vorherige blau, nächste rot. Damit malst du das zweite Bild passgenau über das erste, statt hin- und herzuklicken und zu raten."),

  H.h2("Ein sinnvoller Ablauf"),

  H.p("Wenn du deine erste eigene Figur malst, geh am besten so vor:"),

  H.bullet("Strg+N, Größe 16 mal 16, ein Bild."),
  H.bullet("Spiegelung einschalten (Strg+Shift+X), falls die Figur symmetrisch werden soll."),
  H.bullet("Mit dem Stift den Umriss ziehen. Nicht ausmalen, nur die Form."),
  H.bullet("Kurz zurücklehnen. Erkennt man es? Wenn nicht, jetzt ändern und nicht später."),
  H.bullet("Mit dem Farbeimer die Fläche füllen, dann Schatten und helle Kanten setzen."),
  H.bullet("Als PNG neben deinem Programm speichern und im Programm ansehen."),

  H.p("Der vierte Punkt ist der wichtigste und wird am häufigsten übersprungen."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Das Bild hat einen schwarzen Hintergrund", "Es wurde mit dem Stift statt mit dem Radierer freigemacht. Radierer macht durchsichtig, der Stift malt eine Farbe."],
    ["Im Spiel ist die Figur winzig", "Der Editor zeigt stark vergrößert. Ein 16er-Sprite ist im Fenster wirklich 16 Punkte groß — im Zweifel doppelt so groß malen."],
    ["Die Figur verschwindet vor dem Hintergrund", "Es fehlt die Umrandung."],
    ["Das Programm findet die Datei nicht", "Sie muss neben dem Programm liegen, nicht dort, wo der Editor sie zuletzt gespeichert hat."],
    ["Nur das erste Bild erscheint", "Mehrere Frames werden als Streifen gespeichert. Im Programm braucht es DRAWIMAGEPART, um ein einzelnes Feld zu zeigen."],
    ["Die Spiegelung malt an der falschen Stelle", "Sie spiegelt an der Mitte der Fläche. Bei ungerader Breite gibt es keine saubere Mitte — nimm gerade Maße."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Male dein eigenes Raumschiff, 16 mal 16, mit eingeschalteter Spiegelung. Lade es in das Programm aus Kapitel 19."),
  H.bullet("Male eine zweite Fassung desselben Schiffs mit längerer Flamme und wechsle im Programm zwischen beiden hin und her."),
  H.bullet("Male einen Gegner in zwei Frames und benutze die Zwiebelhaut dabei."),
  H.bullet("Male eine Münze in vier Frames, die sich zu drehen scheint. Der Trick ist, sie von Bild zu Bild schmaler werden zu lassen."),
  H.bullet("Nimm eine deiner Figuren und male sie noch einmal mit nur drei Farben. Vergleiche, welche besser lesbar ist."),
  H.bullet("Male ein Sprite, das nach rechts schaut. Im Programm zeigst du es mit DRAWIMAGEFLIPPED auch nach links — so brauchst du nur eine Richtung zu malen."),

  H.p("Jetzt hast du Figuren. Im nächsten Kapitel bringen wir sie zum Laufen."),
];
