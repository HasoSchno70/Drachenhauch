# Feed-Leser

Nachrichten aus RSS- und Atom-Feeds lesen. Links stehen die Feeds mit der Zahl
ungelesener Einträge, rechts die Einträge, darunter die Vorschau. Beim ersten
Start stehen die Tagesschau und heise online in der Liste.

- **Strg+N** fügt einen Feed hinzu (RSS 2.0, RSS 1.0/RDF oder Atom),
  **Entf** entfernt den gewählten samt seiner Einträge
- **F5** ruft sofort ab, sonst geschieht das alle 15 Minuten von selbst
- **Strg+R** markiert alles als gelesen; ein Eintrag gilt als gelesen,
  sobald er in der Vorschau steht
- **Doppelklick** auf einen Eintrag oder der Verweis in der Vorschau öffnet
  ihn im Browser
- Neue Einträge meldet eine Mitteilung; das Symbol im Infobereich hat ein
  Menü (Fenster zeigen, Jetzt abrufen, Beenden)

Abgerufen wird im Hintergrund: `HTTP_GET_START` holt den Feed, ein Auftrag
(`TASK_START`) zerlegt das XML mit dem `xml`-Modul in einem eigenen Prozess,
und das Fenster bleibt die ganze Zeit bedienbar. Die Einträge liegen in
`feeds.db` (SQLite) neben dem Programm; doppelt kommt keiner hinein, weil die
Kennung je Feed eindeutig ist. Datumsangaben wie `Sat, 10 Oct 2026 08:30:00
+0200` liest `ZEIT_PARSE` und rechnet sie in Ortszeit um.

Entstanden beim Dogfooding am 2026-10-10. Dabei kamen heraus: `ZEIT_PARSE`
kannte weder Zeitzonen noch das Datum aus RSS und E-Mail, `TASK_START` gab
Argumente über die Kommandozeile weiter (lange Texte scheiterten), ein
falscher Thema-Schlüssel nannte die gültigen nicht, `€` und ähnliche Zeichen
in der eingebauten Schrift schalteten die ganze Zeile auf eine andere Schrift,
ein `DIM` in einer Schleife behielt den Wert der vorigen Runde, und
`HTML_TEXT` hielt jedes `<` für den Anfang eines Tags.
