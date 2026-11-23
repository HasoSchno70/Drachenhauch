// Anhang D: von Hand oder Studio? Eine Entscheidungstafel. Jede Zeile
// stuetzt sich auf ein Kapitel dieses Buchs; was die beiden Werkzeuge
// koennen, steht in docs/ide.md und docs/studio.md.
module.exports = (H) => [
  H.chapter("D · Von Hand oder Studio?"),

  H.p("Dieses Buch hat dieselbe Mediathek zweimal gebaut: in Teil I von Hand, mit dem gui-Modul in der IDE, ab Teil II im Studio. Beide Wege führen zu einem Programm, das man weitergeben kann; die IDE exportiert es, das Studio baut es. Welcher Weg für das nächste Projekt der bessere ist, entscheiden ein paar Fragen."),

  H.h2("Die Tafel"),

  H.table([
    ["Das Programm zeichnet in jedem Bild selbst: ein Spiel, eine Demo, eine Simulation", "von Hand", "Die eigene Schleife bestimmt, was in jedem Bild geschieht; im Studio ginge es nur über eine Zeichenfläche."],
    ["Es gibt nur Text aus, kein Fenster", "von Hand", "Ein Programm aus dem Studio hat immer ein Startfenster."],
    ["Ein kleines Werkzeug mit einem Feld und einem Knopf", "beides", "Von Hand ist es eine kurze Datei (Kapitel 1); das Studio lohnt sich, sobald mehr dazukommt."],
    ["Mehrere Fenster, eines davon modal", "Studio", "Jedes Fenster ist eine Klasse mit Zeigen und ZeigenModal (Kapitel 11); von Hand fragt die Schleife jedes Fenster selbst ab."],
    ["Formulare, die an einer Tabelle der Datenbank hängen", "Studio", "Bindung, Laden und Sichern stellt man im Inspektor ein (Kapitel 14) statt mit GUI_BIND und GUI_FORM_SAVE (Kapitel 5)."],
    ["Abfragen, die man wiederverwendet", "Studio", "Das Datenbank-Element sammelt sie unter einem Namen (Kapitel 15); von Hand steht das SQL im Code."],
    ["Berichte zum Drucken", "Studio", "Den Bericht zeichnet man wie ein Fenster (Kapitel 17); von Hand setzt man die Seiten mit den Befehlen des Moduls pdf."],
    ["Eine zweite Sprache", "Studio", "Fenster, Menüs und Berichte übersetzt das Studio selbst; im Code genügt tr$ (Kapitel 22)."],
    ["Ein Fenster, das sich der Größe anpasst", "beides", "Anker im Inspektor (Kapitel 9) oder Behälter von Hand (Kapitel 2)."],
    ["Eine Datenbank auf einem Server", "beides", "Das Datenbank-Element arbeitet mit einer SQLite-Datei; PostgreSQL und MySQL spricht man von Hand an, auch im Code eines Studio-Projekts (Kapitel 24)."],
    ["Verstehen wollen, was eine Oberfläche tut", "von Hand", "Teil I schreibt von Hand, was das Studio später selbst erzeugt."],
  ], { headers: ["Wenn …", "dann", "weil"], widths: [3300, 1200, 4526] }),

  H.h2("Was beide gleich können"),

  H.bulletRich("Dieselbe Sprache: ", "Im Code eines Studio-Fensters gilt jeder Befehl, der in der IDE gilt — auch GUI_SET_TEXT, DB_QUERY und alle anderen aus Teil I."),
  H.bulletRich("Einen Debugger: ", "Haltepunkte, Schritt für Schritt, Variablen ansehen (Kapitel 12)."),
  H.bulletRich("Prüfen beim Tippen: ", "Vertippte Namen stehen unter Probleme, bevor das Programm läuft."),
  H.bulletRich("Ein eigenständiges Programm: ", "die IDE über den Export, das Studio über Bauen (Kapitel 23)."),

  H.h2("Umziehen"),

  H.p("Wer von Hand angefangen hat und ins Studio wechselt, nimmt nicht alles mit — und muss es auch nicht. Kapitel 7 hat die Zeilen der Mediathek aus Teil I gezählt: Der Aufbau der Fenster und die Abfragen in der Schleife fallen weg, weil man das Fenster zeichnet und das Studio die Ereignisse selbst abfragt. Laden, Sichern und Füllen übernimmt ab Teil III das Datenbank-Element. Was bleibt, ist das, was die Anwendung zu ihrer macht: die Prüfungen, die Rückfragen, was ganz oder gar nicht geschehen muss. Das wandert als Methode an den Knopf oder Menüeintrag, zu dem es gehört, oder in ein Modul, wenn mehrere Fenster es brauchen (Kapitel 10)."),

  H.p("Der umgekehrte Weg ist selten nötig. Wer wissen will, was das Studio aus einem Entwurf macht, liest es in _studio nach: Es ist gewöhnlicher Drachenhauch-Code, wie in Teil I (Kapitel 8)."),
];
