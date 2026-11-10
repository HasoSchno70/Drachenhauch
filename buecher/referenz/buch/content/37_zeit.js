module.exports = (H) => [
  H.chapter("Zeit & Datum"),
  H.p("Diese Befehle liefern die aktuelle Uhrzeit, das Datum und die seit Programmstart vergangene Zeit – nützlich für Zeitmessungen, Cooldowns oder einen Zeitstempel im Speicherstand. Weil die Werte vom Zeitpunkt der Ausführung abhängen, zeigen die Beispiele statt einer festen Ausgabe einen repräsentativen Wert als Kommentar."),

  H.h2("Vergangene Zeit messen"),
  H.cmd("MILLIS · TIMER", "MILLIS()   TIMER()",
    "Liefert die Anzahl der Millisekunden seit dem Programmstart (1000 ms = 1 Sekunde). Die absolute Zahl ist selten interessant – wertvoll ist die Differenz zweier MILLIS-Werte, um eine Dauer zu messen.",
    [
      'DIM start AS INTEGER',
      'start = MILLIS()',
      '\' ... etwas tun ...',
      'DIM dauer AS INTEGER',
      'dauer = MILLIS() - start',
      'PRINT "Das dauerte "; dauer; " ms"',
    ]),
  H.note("MILLIS eignet sich gut für eigene Zeitschranken („alle 500 ms etwas tun“). Für gezielte Aktionen nach Ablauf einer Zeit gibt es außerdem das timer-Modul mit COOLDOWN (Teil V)."),

  H.h2("Eine Pause einlegen"),
  H.cmd("SLEEP", "SLEEP(millisekunden)",
    "Hält das Programm für die angegebene Zeit an. In einem Grafik-Spiel mit laufendem Bild solltest du SLEEP meiden (es friert das Fenster ein) – dort steuerst du Tempo über die Bildrate. Für Konsolen-Programme ist es aber praktisch.",
    [
      'PRINT "Start"',
      'SLEEP(1000)        \' eine Sekunde warten',
      'PRINT "... eine Sekunde später"',
    ]),

  H.h2("Uhrzeit & Datum"),
  H.cmd("TIME$", "TIME$()",
    "Liefert die aktuelle Uhrzeit als String im Format \"HH:MM:SS\" (Stunden:Minuten:Sekunden).",
    [
      'PRINT "Es ist jetzt "; TIME$()',
      '\' z. B.:  Es ist jetzt 14:30:05',
    ]),
  H.cmd("DATE$", "DATE$()",
    "Liefert das aktuelle Datum als String im Format \"YYYY-MM-DD\" (Jahr-Monat-Tag).",
    [
      'PRINT "Heute ist der "; DATE$()',
      '\' z. B.:  Heute ist der 2026-06-13',
    ]),
  H.p("Beide eignen sich gut für einen Zeitstempel – etwa um festzuhalten, wann ein Spielstand gespeichert wurde:"),
  H.code([
    'DIM stempel AS STRING',
    'stempel = DATE$() + " " + TIME$()',
    'PRINT "Gespeichert am: " + stempel',
    '\' z. B.:  Gespeichert am: 2026-06-13 14:30:05',
  ]),
  H.tip("Tempo im Spiel", "Für framerate-unabhängige Bewegung im Spielfenster nimmst du nicht MILLIS, sondern DELTA() – die Sekunden seit dem letzten Bild. Das gehört zum Game-Loop und wird in Teil IV („Das Fenster“) behandelt."),
  H.note("TIMER liefert dieselbe Uhr wie MILLIS, nur in Sekunden als Kommazahl statt in ganzen Millisekunden. Für eine Bewegung, die mit der Zeit läuft, ist das die bequemere Form; für einen Vergleich auf ganze Millisekunden die andere. Beide zählen ab dem Programmstart und haben nichts mit der Uhrzeit zu tun – dafür ist das Modul zeit da.",
    "Zwei Namen für eine Uhr"),
];
