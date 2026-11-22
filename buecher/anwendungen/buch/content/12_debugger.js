const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Der Debugger", 12),

  H.h2("Was wir bauen"),

  H.figure("kap12_fehler.png", "Metropolis ist eben entfernt worden. In der Tabelle ist Tetris gewählt, aber das Ausleihfenster sagt „Kein Eintrag gewählt.“ — und Verleihen tut nichts.", 440, 278),

  H.p("Diesmal entsteht nichts Neues. Die Mediathek aus Kapitel 11 hat einen Fehler, und dieses Kapitel findet ihn, ohne zu raten. Er ist leicht nachzustellen: Metropolis wählen, „Ausleihen …“ öffnen, im Hauptfenster „Entfernen“ drücken und die Rückfrage bestätigen. Danach ist in der Tabelle Tetris gewählt, die Zeile, die nachgerückt ist. Das Ausleihfenster aber sagt „Kein Eintrag gewählt.“, und wer jetzt einen Namen einträgt und „Verleihen“ drückt, verleiht nichts. Erst ein Klick auf eine Zeile bringt beide wieder zusammen."),

  H.h2("Der Plan"),

  H.p("Der Debugger lässt das Programm an einer Stelle anhalten, die man vorher markiert hat — einem Haltepunkt. Dort steht es still, und man kann nachsehen, welchen Wert jede Variable gerade hat, wer die Methode aufgerufen hat, und dann Zeile für Zeile weitergehen. Alles steht im Menü Ausführen:"),

  H.table([
    ["F9", "Haltepunkt an der Zeile an oder aus — ein roter Punkt; ebenso ein Klick in die Spalte mit den Zeilennummern"],
    ["F7", "Debuggen: das Programm starten, sodass es an Haltepunkten hält"],
    ["F5", "Weiter: bis zum nächsten Haltepunkt"],
    ["F10", "Schritt: eine Zeile weiter; ein Aufruf darin läuft ganz"],
    ["F11", "Schritt hinein: in die Methode, die die Zeile aufruft"],
    ["Umschalt+F11", "Schritt heraus: bis die Methode zurückkehrt"],
    ["Umschalt+F5", "Anhalten: das Programm beenden"],
  ], { headers: ["Taste", "was sie tut"], widths: [2000, 7026] }),

  H.p("Hält das Programm, öffnet das Studio den Code der Methode und markiert die Zeile, die als Nächste dran ist. Unten in der Leiste Debugger stehen links die Variablen, rechts der Aufrufstapel: die Methode, in der das Programm steht, darunter die, die sie gerufen hat, und so weiter bis zum Anfang."),

  H.h2("Schritt 1: eine Vermutung"),

  H.p("Wer einen Fehler sucht, fängt mit einer Vermutung an, wo er steckt, und prüft sie. Das Ausleihfenster zeigt, was es zuletzt über folgen bekommen hat:"),

  H.code(stueck("kap11/mediathek/AusleihFenster.dh", "SUB folgen", "END SUB")),

  H.p("Also bekommt es nach dem Entfernen etwas Falsches. Aufgerufen wird folgen an drei Stellen im Hauptfenster: in tabelle_Geaendert, nach Bearbeiten und nach Entfernen. Der Fehler tritt nach Entfernen auf — der Haltepunkt kommt dorthin, wo knopfEntfernen_Klick dem Ausleihfenster Bescheid gibt:"),

  H.code(stueck("kap11/mediathek/Hauptfenster.dh", "SUB knopfEntfernen_Klick", "END SUB")),

  H.p("Im Navigator ein Klick auf Klick unter knopfEntfernen, die Schreibmarke in die letzte Zeile, F9. Dann F7 statt F5, und im Programm dieselben Handgriffe wie oben."),

  H.h2("Schritt 2: anhalten und nachsehen"),

  H.figure("kap12_halt.png", "Der Debugger hält in knopfEntfernen_Klick vor Zeile 10. Links unten Self mit seinen Feldern — eintraege hat schon nur noch vier Einträge —, rechts der Aufrufstapel. Darunter in der Liste stehen z = 1, t = Metropolis und antwort = 1.", 480, 266),

  H.p("Das Programm steht vor Zeile 10; sie ist noch nicht gelaufen. Die Leiste Debugger zeigt, was die Methode weiß. Ganz oben Self, das Fenster selbst, und eingerückt darunter seine Felder: eintraege hat nur noch vier Einträge — Zeile 8 hat Metropolis schon herausgenommen —, ausleihe ist das Ausleihfenster. Dann kommen die Controls; ein Control ist für das Programm eine Nummer, darum steht dort tabelle = 0. Am Ende der Liste die Variablen der Methode: z = 1, t = Metropolis, antwort = 1."),

  H.p("z ist die Zeile, die gewählt war, als „Entfernen“ gedrückt wurde. tabelleFuellen hat sie in Zeile 9 wieder gewählt — und in Zeile 1 steht jetzt Tetris. Die Tabelle zeigt also einen Eintrag, und Zeile 10 schickt dem Ausleihfenster NIL."),

  H.p("Der Aufrufstapel rechts sagt, wie das Programm hierher kam: knopfEntfernen_Klick wurde von studioTakt in Hauptfenster_basis.dh gerufen — der Basisklasse, die das Studio erzeugt —, die von studioBild und so weiter. Was dort steht, schreibt man nicht selbst; es zeigt aber, dass ein Ereignis eine gewöhnliche Methode ist, die von irgendwo gerufen wird."),

  H.h2("Schritt 3: Schritt für Schritt"),

  H.p("F11, Schritt hinein: Das Studio öffnet folgen im Ausleihfenster und hält vor seiner ersten Zeile. Die Leiste zeigt jetzt die Variablen von folgen — Self ist hier das Ausleihfenster, und e = NIL. Mit F10 geht es Zeile für Zeile weiter: Self.eintrag wird NIL, anzeigen schreibt „Kein Eintrag gewählt.“. Damit ist die Vermutung bestätigt. Umschalt+F5 beendet das Programm."),

  H.p("F10 und F11 unterscheiden sich nur an einer Zeile mit Aufruf. F10 lässt den Aufruf ganz laufen und hält in der nächsten Zeile derselben Methode; F11 geht hinein. Wer sich hineinverirrt hat, kommt mit Umschalt+F11 heraus und landet wieder auf der Zeile des Aufrufs."),

  H.h2("Schritt 4: die Korrektur"),

  H.p("Das Ausleihfenster soll der Tabelle folgen, wie bei jedem Klick auf eine Zeile. Genau das tut tabelle_Geaendert schon — Zeile 10 ruft es, statt NIL zu schicken (code/kap12/mediathek):"),

  H.code(stueck("kap12/mediathek/Hauptfenster.dh", "SUB knopfEntfernen_Klick", "END SUB")),

  H.p("Nach dem Entfernen von Metropolis sagt das Ausleihfenster jetzt „Tetris ist bei Lena.“. Und wer den letzten Eintrag entfernt, bekommt keine gewählte Zeile und im Ausleihfenster „Kein Eintrag gewählt.“ — beides passt zusammen."),

  H.h2("Warum so?"),

  H.h3("Der Debugger statt PRINT"),

  H.p("Man hätte dasselbe mit PRINT herausfinden können: in folgen ausgeben, was ankommt, in knopfEntfernen_Klick ausgeben, was z ist, starten, nachstellen, lesen, die Zeilen wieder entfernen. Bei einem Fehler, den man schon fast kennt, ist das schnell. Der Debugger lohnt sich, wenn man noch nicht weiß, wonach man fragen soll: Er zeigt alles, was die Methode in dem Moment weiß, auch das, woran man nicht gedacht hat — dass eintraege schon kürzer ist, wer die Methode gerufen hat. Und er ändert den Code nicht."),

  H.h3("Den Haltepunkt dorthin, wo Daten weitergegeben werden"),

  H.p("Ein Fenster zeigt etwas Falsches. Der Fehler liegt dann fast immer nicht dort, wo es angezeigt wird, sondern dort, wo die Daten ankommen oder abgeschickt werden: an einem Aufruf von einem Fenster ins andere, an einem Parameter, an einem Feld, das gesetzt wird. Ein Haltepunkt dort zeigt beide Seiten — was geschickt wird und was der Absender in dem Moment wusste."),

  H.h2("Was schiefgehen kann"),

  H.h3("Der Haltepunkt hält nie"),

  H.p("Ein Haltepunkt hält nur, wenn das Programm die Zeile erreicht. Liegt er in einem Ereignis, das nicht kommt — einem Knopf, der nicht gedrückt wird, einem Zweig eines IF, der nicht läuft —, läuft das Programm einfach durch. Ein Haltepunkt auf einer Kommentar- oder Leerzeile rutscht zur nächsten Zeile mit Code. Und F5 startet ohne Debugger: Dann hält das Programm an keinem Haltepunkt. Gestartet wird mit F7."),

  H.h3("Ein Laufzeitfehler hält auch"),

  H.p("Läuft das Programm unter dem Debugger auf einen Laufzeitfehler, hält es an der Fehlerstelle, statt sich zu beenden. Das Ausleihfenster aus Kapitel 11 ohne die Abfrage, ob es offen ist, meldet beim ersten Klick auf Momo:"),

  H.code(["Laufzeitfehler in AusleihFenster.anzeigen, Zeile 4: GUI_SET_TEXT:",
          "erwartet INTEGER (Arg 1), erhalten NIL -- das Widget oder Fenster",
          "gibt es noch nicht (ein Fenster aus dem Studio legt seine Controls",
          "erst mit Zeigen an)"], { out: true }),

  H.p("Diesmal steht das Programm aber noch: Unter Self sieht man gewaehlt = NIL und offen = FALSE. Die Meldung sagt, was fehlt, die Variablen sagen, warum."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze den Haltepunkt nicht in knopfEntfernen_Klick, sondern in die erste Zeile von folgen. Wann hält das Programm überall?"),
  H.bulletRich("Drehen 2. ", "Am Halt in Zeile 10: Wie viele Einträge hat eintraege, und warum nicht mehr fünf?"),
  H.bulletRich("Drehen 3. ", "Entferne mit der Korrektur den letzten Eintrag, Emil und die Detektive. Welche Zeile ist danach gewählt, und was sagt das Ausleihfenster?"),
  H.bulletRich("Erweitern 1. ", "Wer den letzten Eintrag entfernt, soll auf dem davor landen statt auf keinem. Sieh dir mit einem Haltepunkt an, welchen Wert z danach hat."),
  H.bulletRich("Erweitern 2. ", "„Bearbeiten …“ und „Entfernen“ ohne gewählte Zeile tun nichts. Sie sollen in der Statuszeile sagen: „Bitte erst einen Eintrag wählen.“"),
  H.bulletRich("Erfinden. ", "Setze in deinem Programm aus Kapitel 11 in jedes Ereignis einen Haltepunkt und bediene es einmal ganz. Hält es überall, wo du es erwartest — und nirgends sonst?"),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Was ein Programm braucht → Debugger: ", "alle Handgriffe in einer Tabelle."),
  H.bulletRich("Handbuch, Die IDE → Debugger: ", "dieselben Haltepunkte und Schritte für Programme ohne Studio, dazu Bedingungen und Überwachen."),
];
