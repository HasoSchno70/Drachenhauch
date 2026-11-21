const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Die Datenwerkstatt", 22),

  H.h2("Was wir bauen"),

  H.figure("kap22_2_jahr.png", "Ein Jahr Wetter als Kreis: jeder Tag ein Strahl von der tiefsten zur höchsten Temperatur, außen die Regentage.", 470, 294),

  H.p("Ein ganzes Jahr Wetter auf einem Bild. Jeder Tag ist ein Strahl, der bei der tiefsten Temperatur beginnt und bei der höchsten endet; der Januar steht oben, dann geht es im Uhrzeigersinn durchs Jahr. Die Farbe sagt, wie warm es war, und die blauen Punkte außen sind Regentage — je größer, desto mehr Regen. Links steht, was das Jahr ausmacht: der wärmste und der kälteste Tag, wie viel es geregnet hat und wie lange die Sonne schien. Die Daten kommen aus einer Datei mit 365 Zeilen. Wie man aus so einer Datei mit wenigen Zeilen Code Spalten, Höchstwerte und Mittel macht, darum geht es in diesem Kapitel — mit Comprehensions, einer kurzen Schreibweise für „nimm aus dieser Liste alles, was …, und mach daraus …“."),

  H.h2("Der Plan"),

  H.bulletRich("Die Daten: ", "ein erfundenes Jahr als CSV-Datei, eine Zeile je Tag."),
  H.bulletRich("Lesen und zerlegen: ", "die Datei als Zeilen, die Zeilen als Spalten — mit Comprehensions."),
  H.bulletRich("Auswerten: ", "der wärmste Tag, die Regentage, die Monate."),
  H.bulletRich("Zeichnen: ", "jeder Tag ein Strahl im Kreis, der sich in zwölf Sekunden selbst zeichnet."),

  H.h2("Schritt 1: ein Jahr Wetter"),

  H.code(zeilen("kap22/1_daten.dh")),
  H.code(["365 Tage geschrieben"], { out: true }),

  H.p("Echte Wetterdaten gibt es bei jedem Wetterdienst, aber jeder schreibt sie etwas anders auf. Darum erfinden wir uns ein Jahr, das echt genug aussieht. Die Jahreszeit ist eine Kosinuskurve: im Mittel 0 Grad Mitte Januar und 20 Grad Mitte Juli. Darauf kommt eine Abweichung, die nicht jeden Tag neu gewürfelt wird, sondern langsam wandert — jeden Tag behält sie 85 Prozent von gestern und bekommt ein wenig Zufall dazu. So entstehen warme und kalte Wochen wie im echten Wetter, statt dass es von Tag zu Tag hin und her springt. Regen fällt eher an kühlen Tagen, und wenn, dann meist wenig und selten viel; die Sonne scheint so lange, wie der Tag lang ist, aber weniger, wenn es regnet."),

  H.pmix(["Jeder Tag wird eine Zeile mit fünf Feldern, getrennt durch Semikolons. Der f-String schreibt die Zahlen gleich so, wie sie in die Datei sollen: ", ["{monat:02d}", true], " mit zwei Stellen und führender Null, ", ["{tief:.1f}", true], " mit einer Nachkommastelle. ", ["JOIN$", true], " fügt die Zeilen mit einem Zeilenumbruch zusammen, und ", ["FILE_WRITE", true], " schreibt alles auf einmal. Wegen ", ["RANDOMIZE(2025)", true], " entsteht bei jedem Lauf dieselbe Datei. So fängt sie an:"]),

  H.code(zeilen("kap22/wetter.csv").slice(0, 4), { out: true }),

  H.h2("Schritt 2: lesen und zerlegen"),

  H.code(stueck("kap22/2_jahr.dh", "' Die Datei lesen", "DIM anzahl")),

  H.pmix([["READLINES", true], " liefert die Datei als Feld von Zeilen. Dann kommt die erste Comprehension: ", ["[z FOR z IN zeilen WHERE …]", true], " heißt „jede Zeile z aus zeilen, bei der … gilt“. Übrig bleiben die Zeilen, die nicht leer sind und nicht mit „datum“ beginnen — also alle außer der Kopfzeile. Die zweite Comprehension macht aus jeder Zeile etwas anderes: ", ["SPLIT$(z, \";\")[0]", true], " zerlegt sie an den Semikolons und nimmt das erste Stück, das Datum. Vor dem FOR steht, was herauskommt; hinter dem IN, woher es kommt; hinter dem WHERE, was durchgelassen wird."]),

  H.pmix(["Die Zahlenspalten sind alle gleich gebaut, darum gibt es dafür eine Funktion: ", ["spalte(nr)", true], " nimmt aus jeder Zeile das nr-te Stück und macht mit ", ["VAL", true], " eine Zahl daraus. Vier Aufrufe, und aus 365 Zeilen Text sind vier Felder mit je 365 Zahlen geworden. Dass tief[7] und datum[7] zum selben Tag gehören, liegt daran, dass alle Spalten aus derselben Liste tage in derselben Reihenfolge entstanden sind."]),

  H.code(stueck("kap22/2_jahr.dh", "' Was das Jahr ausmacht", "DIM monate")),

  H.pmix(["Was das Jahr ausmacht, ist jetzt je eine Zeile. ", ["ARRAY_MAX(hoch)", true], " ist die höchste Temperatur des Jahres, und ", ["ARRAY_INDEXOF", true], " sagt, an welcher Stelle sie steht — das ist der Tag, an dem es so warm war. Die Regentage sind eine Comprehension mit WHERE: alle Regenmengen über null. Und die letzte Zeile hat geschweifte Klammern: ", ["{LEFT$(d, 7) FOR d IN datum}", true], " nimmt von jedem Datum Jahr und Monat, „2025-07“, aber jeden Wert nur einmal, in der Reihenfolge, in der er zuerst vorkommt. Aus 365 Tagen werden so zwölf Monate."]),

  H.h2("Schritt 3: der Kreis"),

  H.code(stueck("kap22/2_jahr.dh", "' Die Temperatur als Abstand", "END FUNCTION", 3)),

  H.p("Drei kleine Funktionen rechnen aus Daten Bildschirm. radius macht aus einer Temperatur einen Abstand von der Mitte: −10 Grad liegen 60 Punkte weit draußen, jedes Grad mehr 4 Punkte weiter, 30 Grad also bei 220. farbe wählt einen Farbton zwischen Blau und Rot — bis −5 Grad ganz blau, ab 30 ganz rot, dazwischen über Grün und Gelb. winkel verteilt die Tage gleichmäßig über den Kreis, den ersten oben."),

  H.code(stueck("kap22/2_jahr.dh", "zeit = zeit + DELTA()", "NEXT")),

  H.pmix(["In der Hauptschleife wächst ", ["zeit", true], " um DELTA(), und ", ["bis", true], " sagt, wie viele Tage schon zu sehen sind: nach zwölf Sekunden alle. Jeder Tag ist ein dicker Strich in seine Richtung, von radius(tief) bis radius(hoch), in der Farbe seiner Höchsttemperatur. Regnet es, kommt außen ein Punkt dazu; sein Radius wächst mit der Wurzel der Regenmenge, damit ein Wolkenbruch nicht alles zudeckt. Das Netz darunter — Gradkreise, Monatsstriche, Monatsnamen — zeichnet ", ["netz()", true], " vorher, die Gradzahlen ", ["skala()", true], " nachher, damit kein Strahl sie verdeckt. Das ganze Programm:"]),

  H.code(zeilen("kap22/2_jahr.dh")),

  H.h2("Warum so?"),

  H.h3("Warum Comprehensions"),

  H.p("Jede Comprehension in diesem Programm ließe sich auch als Schleife schreiben: ein leeres Feld anlegen, FOR EACH über die Zeilen, ein IF, ein ARRAY_PUSH. Das sind vier oder fünf Zeilen für etwas, das man in einem Satz sagt: „die Regenmengen über null“. Die Comprehension schreibt genau diesen Satz hin. Man liest, was herauskommt, und nicht, wie es Stück für Stück zusammengebaut wird. Bei Daten, die man in Spalten zerlegt, filtert und umrechnet, ist das fast immer so — darum heißt dieses Kapitel Datenwerkstatt."),

  H.h3("Wann eine Schleife besser ist"),

  H.code(zeilen("kap22/3_monate.dh")),
  H.code(["2025-01:   3.2   3.2",
          "2025-02:   6.3   6.3",
          "2025-03:   8.5   8.5"], { out: true }),

  H.pmix(["Das Mittel der Höchsttemperaturen je Monat, zweimal gerechnet. Die erste Fassung ist eine einzige Comprehension, ", ["{m: … FOR m IN monate}", true], " mit Doppelpunkt — das ergibt eine MAP, für jeden Monat sein Mittel. Darin steckt eine zweite Comprehension, die für den Monat m die passenden Tage heraussucht. Das stimmt, aber man muss es zweimal lesen. Und es geht für jeden der zwölf Monate alle 365 Tage durch, zusammen 4380-mal. Die Schleife darunter geht die Tage einmal durch und zählt dabei für jeden Monat mit: Summe und Anzahl, mit ", ["MAP_GET_OR", true], ", das 0 liefert, solange der Monat noch nicht vorkam. Beide Spalten der Ausgabe sind gleich. Faustregel: Eine Comprehension für einen Schritt — filtern, umrechnen, herausnehmen. Wo eine Comprehension in einer anderen steckt, ist eine Schleife oder eine Funktion mit gutem Namen meist klarer; die Lösung zum Erfinden zeigt so eine Funktion."]),

  H.h3("Warum die Datei Semikolons hat"),

  H.p("CSV heißt „durch Kommas getrennt“, aber in deutschen Dateien steckt das Komma schon in den Zahlen: 24,8 Grad. Darum trennen deutsche Programme die Felder meist mit Semikolons. Unsere Datei schreibt die Zahlen mit Punkt, wie Drachenhauch sie versteht, und trennt trotzdem mit Semikolons — so lässt sie sich auch in einer deutschen Tabellenkalkulation öffnen, ohne dass Spalten durcheinandergeraten. Was passiert, wenn eine fremde Datei doch Kommas in den Zahlen hat, steht unter „Was schiefgehen kann“."),

  H.h2("Was schiefgehen kann"),

  H.h3("Das deutsche Komma"),

  H.code(zeilen("kap22/fehler_komma.dh")),
  H.code(["24 FALSE",
          "24.8"], { out: true }),

  H.pmix([["VAL", true], " liest eine Zahl, so weit es geht, und hört beim ersten Zeichen auf, das nicht dazugehört. Aus „24,8“ wird so 24 — ohne Fehler, nur falsch. Darum fällt es nicht auf: Das Bild sieht fast richtig aus, nur sind alle Temperaturen ganzzahlig abgerundet. ", ["ISNUMERIC", true], " sagt ehrlich FALSE. Die Abhilfe ist ein ", ["REPLACE$", true], " vor dem VAL, das das Komma zum Punkt macht; die Lösung zu Erweitern 2 tut das für jede Zelle."]),

  H.h3("Ein Monat, den es so nicht gibt"),

  H.code(zeilen("kap22/fehler_monat.dh")),
  H.code(["Laufzeitfehler in fehler_monat.dh:7: ARRAY_AVG: Array ist leer"], { out: true }),

  H.p("Im Datum steht der Juli als „2025-07“, mit führender Null; gesucht wird „2025-7“. Kein Tag passt, das WHERE lässt nichts durch, und die Comprehension ist leer. Ein Mittel von nichts gibt es nicht. Eine Comprehension mit WHERE kann immer leer sein — wer danach ein Mittel, einen Höchstwert oder das erste Element nimmt, fragt vorher LEN. Beim Schreiben dieses Kapitels hieß die Meldung noch „nur ARRAY OF INTEGER/FLOAT“, was auf eine ganz falsche Spur führte; jetzt sagt sie, was los ist."),

  H.h3("Eine Comprehension ist ein Tupel"),

  H.pmix(["Was eine Comprehension in eckigen Klammern liefert, ist genau genommen kein Feld, sondern ein ", ["TUPLE", true], ": eine feste Folge von Werten. Beim Schreiben dieses Kapitels führte das zu zwei Fehlern, die es jetzt nicht mehr gibt: ", ["tage = [z FOR z IN zeilen …]", true], " an ein ARRAY OF STRING hieß „Erwartet ARRAY OF STRING, erhalten TUPLE“, und ", ["ARRAY_AVG([…])", true], " hieß „ARRAY_AVG erwartet ARRAY“. Jetzt wird das Tupel bei der Zuweisung zu einem Feld, und Befehle, die ein Feld nur lesen — ARRAY_SUM, ARRAY_AVG, ARRAY_MAX und ihre Geschwister —, nehmen eine Comprehension direkt. Befehle, die ein Feld verändern, wie ", ["SORT", true], ", brauchen weiter ein echtes: erst einer Variablen zuweisen, dann sortieren."]),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere in radius die 4 auf 2. Was passiert mit dem Kreis?"),
  H.bulletRich("Drehen 2. ", "Ändere bei nass das WHERE r > 0 auf WHERE r > 10. Welche Zahlen links ändern sich, welche nicht?"),
  H.bulletRich("Drehen 3. ", "Ändere in 1_daten.dh RANDOMIZE(2025) auf RANDOMIZE(7), lass es laufen und dann 2_jahr.dh. Was bleibt gleich?"),
  H.bulletRich("Erweitern 1. ", "Rechts neben dem Kreis je Monat ein Balken: das Mittel der Höchsttemperaturen."),
  H.bulletRich("Erweitern 2. ", "Eigene Daten: Eine CSV in derselben Form kommt als Argument beim Start oder wird aufs Fenster gezogen — auch eine mit Kommas in den Zahlen."),
  H.bulletRich("Erfinden. ", "Ein Diagramm, das man anklickt: je Monat zwei Balken für die mittlere tiefste und höchste Temperatur, und ein Klick auf einen Monat zeigt, was ihn ausmacht. Das Modul chart zeichnet die Balken. Oder: ein Kalender, in dem jeder Tag ein Kästchen in seiner Farbe ist; die Sonnenstunden als Kurve; zwei Jahre im Vergleich. Anhang A zeigt das Diagramm."),

  H.figure("kap22_diagramm.png", "Erfinden: Die Monatsmittel als Balken; der Juli ist angeklickt.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Comprehensions: ", "Feld, Menge und MAP, mit WHERE und verschachtelt, und was davon ein Tupel ist."),
  H.bulletRich("Handbuch, Standard-Built-ins → CSV: ", "CSV_LOAD und CSV_PARSE für Dateien mit Anführungszeichen und Trennzeichen in den Feldern."),
  H.bulletRich("Handbuch, Module → chart: ", "Balken, Linien, Kuchen und Tacho, mit Maus."),
  H.bulletRich("examples/169_csv.dh: ", "CSV lesen und schreiben mit dem eingebauten Modul."),
];
