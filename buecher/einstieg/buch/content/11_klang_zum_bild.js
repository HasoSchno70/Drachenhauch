module.exports = (H) => [
  H.chapter("Klang zum Bild"),

  H.p("Ein Ton für sich ist eine Spielerei. Interessant wird Klang, wenn er etwas bedeutet — wenn er sagt: das war ein Treffer, das war die Wand, das war ein Punkt für den anderen."),

  H.p("Der Unterschied ist größer, als er klingt. Ein Spiel ohne Ton wirkt wie hinter Glas. Dieselben Programme mit Ton fühlen sich an, als würden sie antworten."),

  H.h2("Die eine Regel"),

  H.p("Bevor wir anfangen, die wichtigste Regel dieses Kapitels — und die einzige, die man wirklich falsch machen kann:"),

  H.p("Erzeuge einen Klang EINMAL, vor der Schleife. Spiel ihn dann so oft ab, wie du willst."),

  H.code([
    "DIM wand AS SOUND",
    'wand = AUDIO_TONE(660, 60, "square", 0.4)',
  ]),

  H.p("Diese zwei Zeilen gehören nach oben, zu den anderen Vorbereitungen. In die Schleife kommt nur noch das Abspielen:"),

  H.code([
    "PLAYSOUND(wand)",
  ]),

  H.warn("Wer AUDIO_TONE in die Schleife schreibt, baut sechzigmal je Sekunde einen neuen Klang. Das kostet Rechenzeit, und die fertigen Klänge sammeln sich im Speicher an — ein Programm, das nach zehn Minuten zäh wird, hat oft genau diese Ursache. Es gibt UNLOADSOUND, um einen Klang wieder freizugeben; einfacher ist, ihn gar nicht erst mehrfach zu bauen.", "Nicht in der Schleife bauen"),

  H.h2("Der Ball, den man hört"),

  H.p("Das ist der Ball im Kasten aus Kapitel 6, unverändert bis auf fünf Zeilen. Die Wände klingen hoch, Decke und Boden tief:"),

  H.code([
    'SCREEN(640, 400, "Treffer")',
    "",
    "DIM x AS FLOAT",
    "DIM y AS FLOAT",
    "DIM dx AS FLOAT",
    "DIM dy AS FLOAT",
    "DIM wand AS SOUND",
    "DIM decke AS SOUND",
    "",
    "x = 320",
    "y = 200",
    "dx = 4",
    "dy = 3",
    "",
    'wand = AUDIO_TONE(660, 60, "square", 0.4)',
    'decke = AUDIO_TONE(330, 70, "square", 0.4)',
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    CIRCLE(x, y, 18, RGB(120, 220, 255))",
    "    FLIP()",
    "",
    "    x = x + dx",
    "    y = y + dy",
    "",
    "    IF x < 18 OR x > 621 THEN",
    "        dx = -dx",
    "        PLAYSOUND(wand)",
    "    END IF",
    "",
    "    IF y < 18 OR y > 381 THEN",
    "        dy = -dy",
    "        PLAYSOUND(decke)",
    "    END IF",
    "WEND",
  ]),

  H.figure("kap11_1_treffer.png", "Zu sehen ist derselbe Ball wie in Kapitel 6. Der Unterschied steckt in den Lautsprechern.", 440, 280),

  H.pmix(["Die beiden Klänge sind ", ['"square"', true], ", also die harte Rechteckform aus dem letzten Kapitel. Für kurze Signaltöne ist sie fast immer die richtige Wahl: Ein weicher Sinus von sechzig Millisekunden geht im Bild unter, ein Rechteck sticht heraus."]),

  H.pmix(["Die Lautstärke steht auf ", ["0.4", true], ", also knapp der Hälfte. Signaltöne dürfen nicht laut sein — sie kommen oft und schnell hintereinander, und was beim ersten Mal knackig klingt, ist beim dreißigsten Mal eine Zumutung."]),

  H.p("Dass die Wände hoch und die Decke tief klingen, ist keine Verzierung. Du hörst mit geschlossenen Augen, wo der Ball angestoßen ist. Genau das leistet Klang in einem Spiel: Er sagt etwas, das man nicht ansehen muss."),

  H.h2("Schießen und treffen"),

  H.p("Zwei Klänge, zwei Ereignisse — und diesmal ist einer davon kein Ton, sondern Rauschen:"),

  H.code([
    'laser = AUDIO_TONE(900, 70, "square", 0.35)',
    "knall = AUDIO_NOISE(260, 0.5)",
  ]),

  H.pmix([["AUDIO_NOISE(dauer, lautstaerke)", true], " erzeugt weißes Rauschen — ein Zischen ohne Tonhöhe. Damit macht man alles, was keine Melodie hat: Explosionen, Schritte, Wind, Regen. Ein Knall ist physikalisch genau das: viele Frequenzen auf einmal."]),

  H.p("Das ganze Programm ist das Schiff aus Kapitel 7, ergänzt um ein Ziel, das getroffen werden kann:"),

  H.figure("kap11_2_laser.png", "Das Ziel oben, das Schiff unten. Der Rest ist Zuhören.", 440, 280),

  H.code([
    "IF KEYHIT(KEY_SPACE) AND NOT fliegt THEN",
    "    fliegt = TRUE",
    "    sx = x",
    "    sy = 344",
    "    PLAYSOUND(laser)",
    "END IF",
  ]),

  H.p("Und die Trefferprüfung, die nach demselben Muster gebaut ist wie die Schlägerprüfung in Pong — erst senkrecht, dann waagerecht:"),

  H.code([
    "IF fliegt AND sy < zy + 12 AND sy > zy - 12 THEN",
    "    IF sx > zx - 30 AND sx < zx + 30 THEN",
    "        fliegt = FALSE",
    "        punkte = punkte + 1",
    "        PLAYSOUND(knall)",
    "        zx = RANDINT(60, 580)",
    "        zy = RANDINT(50, 160)",
    "    END IF",
    "END IF",
  ]),

  H.p("Das vollständige Programm steht in code/kap11/2_laser.dh. Spiel es kurz und achte darauf, was der Klang mit dir macht: Ohne ihn ist ein Treffer eine Zahl, die sich ändert. Mit ihm ist es ein Ereignis."),

  H.h2("Pong, das antwortet"),

  H.p("Zum Schluss die Belohnung. Pong aus Kapitel 8 bekommt drei Klänge — und wird dadurch zu einem anderen Spiel."),

  H.code([
    "' Die drei Klaenge werden EINMAL gebaut, nicht in der Schleife.",
    'schlag = AUDIO_TONE(520, 55, "square", 0.45)',
    'rand = AUDIO_TONE(260, 55, "square", 0.35)',
    'verloren = AUDIO_TONE(140, 320, "saw", 0.4)',
  ]),

  H.bulletRich("schlag ", "— hell und kurz, wenn der Ball einen Schläger trifft. Der wichtigste Klang des Spiels: Er bestätigt, dass du rechtzeitig warst."),
  H.bulletRich("rand ", "— dieselbe Länge, aber eine Oktave tiefer, für Decke und Boden. Tiefer, weil es weniger bedeutet."),
  H.bulletRich("verloren ", "— lang, tief und schnarrend, wenn ein Punkt fällt. Die Sägezahnform macht ihn unangenehm, und das ist Absicht."),

  H.p("Diese drei Zeilen sind eine kleine Übung in Gestaltung. Die Tonhöhe sagt, wie wichtig etwas ist; die Länge, wie endgültig; die Form, ob es angenehm sein soll. Man kann darüber Bücher schreiben, aber die Grundregeln passen in drei Sätze."),

  H.figure("kap11_3_pong_mit_ton.png", "Dasselbe Bild wie in Kapitel 8 — und ein völlig anderes Spielgefühl.", 440, 280),

  H.p("Das fertige Programm steht in code/kap11/3_pong_mit_ton.dh. Es ist Zeile für Zeile das Pong aus Kapitel 8; hinzugekommen sind vier Klangzeilen oben und fünf PLAYSOUND-Aufrufe an den Stellen, wo ohnehin schon eine Entscheidung stand."),

  H.tip("Der beste Versuch dieses Kapitels", "Spiel eine Runde Pong mit Ton. Dann kommentier die fünf PLAYSOUND-Zeilen aus (ein Hochkomma davor genügt) und spiel noch eine Runde. Der Unterschied ist größer, als neun Zeilen Code vermuten lassen."),

  H.h2("Klang gestalten: eine kleine Sammlung"),

  H.table([
    [{ text: 'AUDIO_TONE(880, 50, "square", 0.3)', mono: true }, "Piepser, Menüwechsel, Punkt aufgesammelt"],
    [{ text: 'AUDIO_TONE(900, 70, "square", 0.35)', mono: true }, "Laser, Schuss"],
    [{ text: 'AUDIO_TONE(140, 320, "saw", 0.4)', mono: true }, "Fehler, verloren, Warnung"],
    [{ text: 'AUDIO_TONE(523, 500, "sine", 0.5)', mono: true }, "geschafft, Ebene abgeschlossen"],
    [{ text: "AUDIO_NOISE(260, 0.5)", mono: true }, "Explosion, Einschlag"],
    [{ text: "AUDIO_NOISE(60, 0.2)", mono: true }, "Schritt, Klicken, Rascheln"],
  ], { headers: ["Aufruf", "Wofür er sich eignet"], widths: [4200, 4826], mono: [0] }),

  H.p("Diese Zahlen sind kein Gesetz, sondern Ausgangspunkte. Dreh daran, bis es passt — genau so entstehen die Klänge in richtigen Spielen auch."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Der Klang kommt zu spät", "Er wird erst nach dem FLIP angestoßen und ist ein Bild zu spät. Bei sechzig Bildern je Sekunde merkt das kaum jemand — wenn doch, PLAYSOUND vor das FLIP ziehen."],
    ["Es klingt zerhackt", "Derselbe Klang wird in aufeinanderfolgenden Bildern immer wieder neu gestartet. Meist steckt eine Bedingung dahinter, die mehrere Bilder lang wahr bleibt — etwa ein Ball, der in der Wand klebt."],
    ["Nach ein paar Minuten wird es zäh", "AUDIO_TONE steht in der Schleife. Nach oben ziehen."],
    ["Ein Klang übertönt alle anderen", "Die Lautstärken sind nicht aufeinander abgestimmt. Signaltöne gehören auf 0.3 bis 0.5, nicht auf 1."],
    ["Beim Punktverlust hört man nichts", "Der Ball wird sofort zurückgesetzt, und die Bedingung war nur ein Bild lang wahr — das reicht. Wenn nichts kommt, steht PLAYSOUND hinter dem Zurücksetzen und wird nie erreicht."],
  ], { headers: ["Was du hörst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Gib dem Ball im Kasten für jede der vier Wände einen eigenen Ton. Du hörst dann mit geschlossenen Augen, wo er ist."),
  H.bullet("Lass den Schlagton in Pong höher werden, je schneller der Ball ist."),
  H.bullet("Bau Snake einen Ton ein: einen hellen beim Fressen, einen tiefen beim Sterben."),
  H.bullet("Gib dem Laser-Programm einen zweiten Klang für den Fehlschuss — wenn der Schuss oben hinausfliegt, ohne zu treffen."),
  H.bullet("Spiel beim Treffer nicht einen Klang, sondern zwei kurz hintereinander. Du brauchst dafür einen Zähler, der ein paar Bilder wartet."),
  H.bullet("Mach die Sirene aus Kapitel 10 zum Alarm für das Laser-Programm: Sie ertönt, solange kein Ziel getroffen wurde."),

  H.p("Du kannst jetzt Klänge bauen und sie an Ereignisse hängen. Im nächsten Kapitel gibst du die Kontrolle darüber ab — an deine Finger."),
];
