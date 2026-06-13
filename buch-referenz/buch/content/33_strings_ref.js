module.exports = (H) => [
  H.chapter("Zeichenketten-Funktionen"),
  H.p("Dieses Kapitel ist die kompakte Übersicht aller String-Funktionen zum Nachschlagen. Ausführlich erklärt und mit Beispielen versehen sind sie im Kapitel „Strings im Detail“ (Teil II) – dort findest du auch den wichtigen Hinweis, dass Zeichen-Positionen ab 0 gezählt werden. Funktionen mit $-Endung liefern einen String; das $ darfst du weglassen (UPPER$ ≡ UPPER)."),

  H.h2("Länge & Schreibweise"),
  H.bulletRich("LEN(s$)  ", "→ Anzahl der Zeichen."),
  H.bulletRich("UPPER$(s$) · LOWER$(s$)  ", "→ in Groß- bzw. Kleinbuchstaben."),
  H.bulletRich("REVERSE$(s$)  ", "→ Zeichen in umgekehrter Reihenfolge."),

  H.h2("Teile herausschneiden"),
  H.bulletRich("LEFT$(s$, n) · RIGHT$(s$, n)  ", "→ die ersten bzw. letzten n Zeichen."),
  H.bulletRich("MID$(s$, start[, n])  ", "→ ab Position start (ab 0!) bis zu n Zeichen, sonst bis zum Ende."),
  H.bulletRich("s[i]  ·  s[a:b]  ", "→ einzelnes Zeichen bzw. Ausschnitt (Slicing; untere Grenze dabei, obere nicht)."),

  H.h2("Suchen & Prüfen"),
  H.bulletRich("INSTR(s$, teil$[, start])  ", "→ Position von teil$ (ab 0) oder -1, wenn nicht enthalten; optional ab Position start."),
  H.bulletRich("CONTAINS(s$, teil$)  ", "→ TRUE, wenn teil$ enthalten ist (auch: teil$ IN s$)."),
  H.bulletRich("STARTSWITH(s$, präfix$) · ENDSWITH(s$, suffix$)  ", "→ TRUE, wenn s$ so beginnt bzw. endet."),

  H.h2("Ersetzen & Säubern"),
  H.bulletRich("REPLACE$(s$, alt$, neu$)  ", "→ alle Vorkommen von alt$ durch neu$ ersetzt."),
  H.bulletRich("TRIM$(s$)  ", "→ Leerraum vorne und hinten entfernt."),
  H.bulletRich("LTRIM$(s$) · RTRIM$(s$)  ", "→ Leerraum nur links bzw. nur rechts entfernt."),

  H.h2("Zerlegen & Zusammensetzen"),
  H.bulletRich("SPLIT$(s$, trenner$)  ", "→ zerlegt s$ am Trenner in ein ARRAY OF STRING."),
  H.bulletRich("JOIN$(array, trenner$)  ", "→ fügt ein String-Array mit dem Trenner zu einem String."),

  H.h2("Auffüllen & Wiederholen"),
  H.bulletRich("PADL$(s$, breite[, füll$]) · PADR$(...)  ", "→ auf Breite auffüllen, links (rechtsbündig) bzw. rechts (linksbündig)."),
  H.bulletRich("REPEAT$(s$, n)  ", "→ s$ n-mal aneinandergehängt (auch: \"-\" * n)."),

  H.h2("Praxis: alles zusammen"),
  H.p("Viele dieser Funktionen lassen sich auch als Methode mit dem Punkt aufrufen und so zu einer Kette verbinden – oft die lesbarste Form:"),
  H.code([
    'DIM eingabe AS STRING',
    'eingabe = "  Hallo Welt  "',
    'PRINT eingabe.trim().upper()',
    'PRINT eingabe.trim().replace("Welt", "GameBasic")',
  ]),
  H.code(["HALLO WELT", "Hallo GameBasic"], { out: true }),
  H.note("Zum Umwandeln zwischen Text und Zahl (STR$, VAL, CHR$, ASC) gibt es das eigene Kapitel „Typumwandlung & Prüfung“. Text bequem aus Werten zusammenbauen kannst du außerdem mit f-Strings (Kapitel „Ein- und Ausgabe“)."),
];
