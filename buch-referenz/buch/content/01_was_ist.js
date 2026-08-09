module.exports = (H) => [
  H.part("Teil I — Erste Schritte"),

  H.chapter("Was ist Drachenhauch?"),
  H.p("Drachenhauch ist eine Programmiersprache aus der ehrwürdigen BASIC-Familie. BASIC steht für „Beginner's All-purpose Symbolic Instruction Code“ – ein sperriger Name für eine freundliche Idee: eine Sprache, die bewusst so leicht lesbar ist, dass sie sich fast wie englische Sätze liest. Generationen von Programmierern haben in den achtziger Jahren auf Heimcomputern mit BASIC angefangen."),
  H.p("Drachenhauch nimmt diese alte, gute Idee und holt sie ins 21. Jahrhundert. Es ist von Grund auf für Spiele gemacht: Grafik, Sound, Eingabe und Spielablauf sind direkt eingebaut. Du musst dir nicht aus dem halben Internet Bibliotheken zusammensuchen, von denen die Hälfte nicht zusammenpasst. Du schreibst SCREEN, und ein Fenster geht auf. So soll es sein."),

  H.h2("Was Drachenhauch besonders macht"),
  H.bulletRich("Einfach zu lesen: ", "Befehle wie SCREEN, PLOT, DRAWIMAGE oder PLAYSOUND sagen, was sie tun."),
  H.bulletRich("Sicher durch Typen: ", "Jede Variable hat einen klaren Typ (INTEGER, FLOAT, STRING …). Das verhindert viele Anfängerfehler, bevor sie passieren."),
  H.bulletRich("Modern: ", "Klassen und Objekte, Funktionen, Module, sogar 3D – alles dabei, wenn du es brauchst, aber nie im Weg."),
  H.bulletRich("Schnell: ", "Dein Programm läuft direkt über die flinke Laufzeit „dhrt“ – und lässt sich auf Wunsch als fertige .exe weitergeben."),

  H.p("Genug der Theorie – schau dir an, wie wenig nötig ist, um etwas auf den Bildschirm zu bringen. Das hier ist ein vollständiges, lauffähiges Drachenhauch-Programm:"),
  H.code([
    'SCREEN(640, 480, "Hallo")',
    'TEXT(40, 40, "Hallo Welt!", RGB(255, 220, 0))',
    'FLIP()',
    'SLEEP(2000)',
  ]),
  H.p("Vier Zeilen. Die erste öffnet ein Fenster, die zweite schreibt in leuchtendem Gelb „Hallo Welt!“ hinein, die dritte zeigt das Gezeichnete an, die vierte wartet zwei Sekunden. Man muss kein Hellseher sein, um zu erraten, was hier passiert – und genau das ist der Punkt."),

  H.chapter("Was kann Drachenhauch alles?"),
  H.p("Erstaunlich viel. Hier ein Vorgeschmack, was eingebaut ist – jedes davon bekommt später sein eigenes Kapitel:"),
  H.bulletRich("2D-Grafik: ", "Linien, Rechtecke, Kreise, Farbverläufe, Splines, dicke Linien, Bilder und Sprites."),
  H.bulletRich("3D-Grafik: ", "Würfel, Kugeln, geladene 3D-Modelle, Licht, Schatten und Kameras."),
  H.bulletRich("Fertige Fenster-Oberflächen (GUI): ", "Buttons, Schieberegler, Checkboxen, Textfelder."),
  H.bulletRich("Sound & Musik: ", "Toneffekte erzeugen, Musik abspielen, eigene Arcade-Sounds synthetisieren."),
  H.bulletRich("Spiel-Bausteine: ", "Partikel-Effekte, Animationen, Kollisionen, Tilemaps, Pfadsuche, Physik und mehr."),
  H.p("Keine Sorge, wenn dir das gerade nach viel klingt. Wir gehen es Schritt für Schritt an, und du brauchst keinerlei Vorkenntnisse – wenn du einen Computer einschalten und Text tippen kannst, bist du qualifiziert."),
];
