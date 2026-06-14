// Anhang D -- Fehlermeldungen verstehen. Alle Meldungs-Wortlaute gegen gbrt verifiziert.
module.exports = (H) => {
  const err = (msg, fix) => [{ text: msg, mono: true, size: 17 }, fix];
  const T = (rows) => H.table(rows, {
    headers: ["Meldung (sinngemäß)", "Ursache und Lösung"],
    widths: [4100, 4926],
  });

  return [
    H.chapter("Anhang D — Fehlermeldungen verstehen"),
    H.p("Fehler gehören zum Programmieren dazu. GameBasic meldet sie in einer festen Form, die dir genau sagt, wo und was schiefging. Dieser Anhang erklärt den Aufbau und die häufigsten Meldungen."),

    H.h2("So liest du eine Fehlermeldung"),
    H.pmix([
      "Ein Laufzeitfehler sieht so aus: ",
      ['Laufzeitfehler in spiel.gb:42: Variable \'x\' nicht deklariert', true],
      ". Darin steckt alles Wichtige: die Datei (",
      ['spiel.gb', true],
      "), die Zeilennummer (",
      ['42', true],
      ") und die Beschreibung. Springe zuerst zu genau dieser Zeile – dort (oder kurz davor) liegt die Ursache.",
    ]),
    H.bulletRich("Parse-Fehler", " erscheinen schon BEIM EINLESEN, bevor das Programm startet – meist ein Tippfehler in der Syntax (fehlendes END, falsche Klammer)."),
    H.bulletRich("Laufzeitfehler", " treten WÄHREND des Laufs auf – etwa ein Zugriff außerhalb eines Arrays. Sie lassen sich mit TRY/CATCH abfangen (Kapitel 24)."),

    H.h2("Häufige Laufzeitfehler"),
    T([
      err("Variable 'x' nicht deklariert (DIM fehlt?)",
        "Die Variable wurde nie mit DIM angelegt – oder ihr Name ist vertippt. Auch IS_NIL(NIL) löst das aus, weil NIL kein schreibbares Literal ist: Wende IS_NIL auf eine echte Variable an."),
      err("Index 5 ausserhalb [0..2] in Dimension 0",
        "Du greifst auf einen Array-/String-Platz zu, den es nicht gibt. Gültig sind die Indizes 0 bis LEN(...) - 1. Schleifengrenzen und Bedingungen prüfen."),
      err("Zuweisung an global: Erwartet STRING, erhalten INTEGER",
        "Der zugewiesene Wert passt nicht zum deklarierten Typ der Variable. Wandle ihn um (STR$/VAL) oder deklariere die Variable mit dem richtigen Typ."),
      err("Integer-Division durch 0   ·   MOD durch 0",
        "Division (\\) oder Rest (MOD) durch null. Prüfe den Teiler vorher mit IF teiler <> 0."),
      err("MAPGET: Schluessel 'fehlt' nicht in Map",
        "Der abgefragte Schlüssel ist nicht in der Map. Prüfe vorher mit MAPHAS oder nutze MAPGETOR(map, key, default)."),
      err("Index-Zuweisung an Nicht-Array (TUPLE)",
        "Tupel (und Strings) sind unveränderlich – du kannst t[0] = ... nicht schreiben. Baue stattdessen ein neues Tupel."),
      err("Tupel-Destructuring: 2 Ziele, aber Tupel hat 3 Element(e)",
        "Beim Zerlegen (a, b) = ... muss die Anzahl der Ziele exakt zur Tupel-Länge passen."),
      err("AUDIO_SFX: erwartet STRING",
        "Ein Befehl bekam ein Argument vom falschen Typ (hier eine Zahl statt eines Strings). Vergleiche den Aufruf mit der Signatur in Anhang A."),
      err("Builtin 'WIFI_AVAILABLE' gehoert zum Hardware-Modul 'wifi', das in diesem gbrt-Build fehlt",
        "Die Hardware-Module (serial/usb/wifi/bt) sind nur im Spezial-Build enthalten. Baue mit: python rust\\build_runtime.py --hardware (siehe Kapitel 76)."),
    ]),

    H.tip("Schritt für Schritt zur Ursache", "Wenn du eine Meldung nicht sofort verstehst: (1) Geh zur genannten Zeile. (2) Lies, welcher Befehl oder welche Variable gemeint ist. (3) Prüfe deren Werte – oft hilft ein zusätzliches PRINT direkt davor, um zu sehen, was wirklich drinsteht. (4) Schlage bei Befehlen die erwarteten Argumente in Anhang A nach. Die meisten Fehler sind nach diesen vier Schritten klar."),

    H.p("Damit endet das Lehrbuch. Du kennst nun die Sprache, die eingebauten Befehle, die Module und hast ein vollständiges Spiel gebaut. Der beste nächste Schritt ist immer derselbe: ein eigenes kleines Projekt anfangen und ausprobieren. Viel Freude beim Programmieren mit GameBasic!"),
  ];
};
