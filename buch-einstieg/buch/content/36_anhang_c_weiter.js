module.exports = (H) => [
  H.chapter("C · Wie es weitergeht"),

  H.p("Dieses Buch war der Anfang, nicht das Ganze. Es hat dir 113 Befehle gezeigt — Drachenhauch hat rund siebenhundertsechzig. Was jetzt kommt, hängt davon ab, wohin du willst."),

  H.h2("Wenn du nachschlagen willst"),

  H.p("„Das Lehrbuch“ ist der große Bruder dieses Bandes: vollständig, systematisch, jeder Befehl mit einem kleinen Beispielprogramm. Es geht den umgekehrten Weg — erst die Konsole, dann die Grammatik der Sprache, Grafik ab Teil IV. Wo dieses Buch fünf Zeilen zeigt und sagt „sieh hin“, sagt jenes „so ist es aufgebaut“."),

  H.p("Sechs Teile, fünfundsiebzig Kapitel: erste Schritte, die Sprache selbst, die eingebauten Befehle als Referenz, Grafik und Klang, die Module — und am Ende zwei Kapitel, die eine mitgelieferte Demo auseinandernehmen."),

  H.p("Es ist kein Buch zum Durchlesen, sondern eines zum Nachschlagen. Genau dafür hat es die Referenzteile."),

  H.h2("Wenn du Spiele bauen willst"),

  H.p("„Galaga“ baut in zwölf Kapiteln einen Arcade-Shooter: ein Schiff unten, eine Formation Gegner oben, die in geschwungenen Bahnen einfliegen, in Formation schweben, einzeln im Bogen herabstürzen und dabei Bomben werfen."),

  H.p("Das ist genau der nächste Schritt nach dem Arcade-Spiel aus Kapitel 23 — dieselbe Idee, aber mit allem, was fehlte: Einflugbahnen, Angriffsmuster, Gegner mit eigenem Verhalten. Und die Sprites zeichnest du selbst, in einem Pixel-Editor, der mitgeliefert wird."),

  H.h2("Wenn du Anwendungen bauen willst"),

  H.p("„Tippspiel“ baut in dreizehn Kapiteln eine Bundesliga-Tippanwendung: Spiele in einer Datenbank, Tipps eingeben, Ergebnisse eintragen, Punkte rechnen, Rangliste führen, Spielplan aus dem Netz holen."),

  H.p("Wer den Vokabeltrainer gebaut hat, erkennt fast alles wieder — Reiter, Datenbank, Netzabruf, drei Schichten von Daten bis Anzeige. Nur mit einer Frage mehr, die dort im Mittelpunkt steht: Wo leben die Daten, und wer darf sie ändern?"),

  H.table([
    ["Das Lehrbuch", "nachschlagen", "alle Befehle, die ganze Sprache, 75 Kapitel"],
    ["Galaga", "ein Spiel", "Arcade-Shooter in 12 Kapiteln, eigene Sprites"],
    ["Tippspiel", "eine Anwendung", "Datenbank, Netz, Rangliste in 13 Kapiteln"],
  ], { headers: ["Band", "Wofür", "Was drinsteht"], widths: [2000, 2000, 5026] }),

  H.h2("Werkzeuge, die schon da sind"),

  H.p("Zu Drachenhauch gehören ein paar Programme, die dieses Buch nicht gebraucht hat, weil es bei den Grundlagen blieb. Für das nächste Vorhaben lohnt sich ein Blick:"),

  H.table([
    [{ text: "dhrun.py --editor", mono: true }, "die IDE: Farben im Quelltext, Vervollständigung, klickbare Fehlermeldungen"],
    [{ text: "dhsprites", mono: true }, "Pixel-Editor für Sprites, exportiert PNG-Sheets und Animationen"],
    [{ text: "dhrt --export", mono: true }, "aus einem Programm eine eigenständige Datei machen"],
    [{ text: "dhrt --check", mono: true }, "ein Programm prüfen, ohne es zu starten"],
  ], { headers: ["Werkzeug", "Wofür"], widths: [2600, 6426], mono: [0] }),

  H.pmix(["Es gibt außerdem Editoren für Animationen, Partikel, Kachelkarten und Oberflächen. Sie stehen alle in der Dokumentation unter ", ["docs/", true], " — und keiner davon ist nötig, um ein Programm zu schreiben. Sie nehmen einem nur Fleißarbeit ab."]),

  H.h2("Der wichtigste Schritt"),

  H.p("Der ist keiner von den dreien. Er heißt: etwas bauen, das du selbst haben willst."),

  H.p("Ein Buch kann zeigen, wie ein Ball abprallt und wie eine Datenbank Fragen beantwortet. Was es nicht kann, ist, dir eine Idee zu geben, an der dir etwas liegt — und ohne die geht nichts weiter. Es muss nichts Großes sein. Ein Programm, das ausrechnet, wie lange du für den Schulweg brauchst. Ein Würfel für ein Brettspiel, dessen Würfel verloren ging. Eine Liste, die zählt, wie oft dein Lieblingsverein trotzdem verliert."),

  H.p("Was immer es ist: Fang klein an, sieh dir nach jeder Änderung an, was passiert, und glaub keiner Zeile, die du nicht laufen gesehen hast. Mehr Handwerk steckt nicht darin."),

  H.p("Viel Vergnügen."),
];
