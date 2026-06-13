module.exports = (H) => [
  H.chapter("Installation, Editor & Programme starten"),
  H.p("Bevor wir Code schreiben, sorgen wir dafür, dass du ihn auch ausführen kannst. Das ist schnell erledigt."),

  H.h2("Was du brauchst"),
  H.p("GameBasic besteht aus zwei Teilen: der Laufzeit „gbrt“, die deine Programme ausführt, und einer Sammlung von Editoren, mit denen du Code, Grafik, Musik und mehr erstellst. Auf einem eingerichteten System liegt alles bereit; du startest die Werkzeuge über kurze Befehle in der Eingabeaufforderung."),

  H.h2("Der Code-Editor"),
  H.p("Den Programm-Editor öffnest du mit dem Befehl gb (ohne Argumente erscheint ein kleines Auswahlfenster, in dem du den Code-Editor wählst). Der Editor färbt deinen Code ein, schlägt Befehle vor und zeigt Fehler an, noch bevor du startest. Mit der Taste F5 läuft dein Programm sofort los."),
  H.bulletRich("Neues Programm: ", "Datei → Neu, dann lostippen."),
  H.bulletRich("Starten: ", "Taste F5 – ein Fenster (oder die Konsole) geht auf und führt dein Programm aus."),
  H.bulletRich("Speichern: ", "Strg+S. GameBasic-Programme haben die Endung .gb."),

  H.h2("Ein Programm von Hand starten"),
  H.p("Du kannst ein gespeichertes Programm auch direkt starten, ohne den Editor. In der Eingabeaufforderung, im Projektordner:"),
  H.code(['gbrun.py mein_programm.gb']),
  H.p("Das ist der empfohlene Weg, weil gbrun.py ins Verzeichnis deiner Datei wechselt – wichtig, sobald dein Programm Bilder oder Klänge aus Unterordnern lädt. Direkt geht es ebenso:"),
  H.code(['gbrt run mein_programm.gb']),

  H.h2("Konsole oder Fenster?"),
  H.p("GameBasic-Programme gibt es in zwei Geschmacksrichtungen. Solange du nur mit PRINT Text ausgibst und mit INPUT etwas einliest, läuft alles in der Konsole – einem schlichten Textfenster. Sobald du den Befehl SCREEN benutzt, öffnet sich ein echtes Grafikfenster, in dem du zeichnen, Tasten abfragen und Töne abspielen kannst. Beide Welten lernst du in diesem Buch kennen, und wir fangen ganz bewusst mit der Konsole an: Sie lenkt nicht ab und zeigt das Wesentliche."),
  H.note("Die Beispiele in den ersten Kapiteln sind Konsolen-Programme – sie geben Text aus, den du sofort siehst. Ab Teil IV kommt die Grafik dazu."),
];
