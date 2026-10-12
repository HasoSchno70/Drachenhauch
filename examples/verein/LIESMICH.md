# Verein

Mitglieder, Beiträge und Zahlungen eines Vereins -- eine Geschäftsanwendung
mit Formular, Tabelle, Datenbank und PDF, an der mehrere Leute gleichzeitig
arbeiten dürfen.

```
dhrt run main.dh                                          (SQLite: verein.db daneben)
dhrt run main.dh -- postgres://nutzer:kennwort@server/verein
dhrt run main.dh -- mysql://nutzer:kennwort@server/verein
```

- Links die Mitglieder: ein Klick auf den Kopf sortiert, die Zeile darunter
  filtert. Ein Klick auf ein Mitglied lädt es ins Formular.
- Rechts das Formular (Strg+N neu, Strg+S speichern) und die Zahlungen des
  gewählten Mitglieds samt dem, was in diesem Jahr noch offen ist.
- Strg+P schreibt die Beitragsliste aller aktiven Mitglieder als PDF.

Die Tabellen legt das Programm beim ersten Start selbst an, auf allen drei
Datenbanken mit demselben Programm -- nur die Spalte `id` heißt je Server
anders (`SERIAL`, `AUTO_INCREMENT`, `AUTOINCREMENT`).

## Zu zweit an einem Server

Startet zwei Fenster mit demselben Ziel und bearbeitet dasselbe Mitglied.
Wer zuerst speichert, gewinnt; der andere wird gefragt: **Neu laden**
(seine Eingaben verwerfen), **Überschreiben** oder **Abbrechen**. Hat der
andere das Mitglied gelöscht, sagt es die Statuszeile, und Speichern legt
es neu an.

## Wie es gebaut ist

- **Datenbindung:** jedes Feld trägt den Namen seiner Spalte
  (`GUI_BIND`), `GUI_FORM_LOAD` und `GUI_FORM_SAVE` laden und speichern das
  ganze Formular -- es gibt keine Spaltenliste im Programm.
- **Gleichzeitig bearbeiten:** `GUI_FORM_VERSION(win, "stand", "m")`. Jedes
  Speichern zählt die Spalte `stand` hoch, und gespeichert wird nur, wenn
  sie noch den Stand vom Laden trägt. Sonst liefert `GUI_FORM_SAVE` 0, und
  `GUI_FORM_CONFLICT$` sagt `geaendert` oder `geloescht`.
- **Geld in Cent:** der Beitrag ist ein Geldfeld (`GUI_TEXTINPUT_SET(tf,
  "zahlen", 3)`) -- im Feld steht `48,00`, in der Datenbank `4800`. Summen
  rechnet die Datenbank, ohne Rundungsfehler.
- **Prüfung:** Pflichtfeld, Postleitzahl als Muster, E-Mail
  (`GUI_RULE`); falsche Felder werden rot umrandet.
- **Beitragsliste:** ein `LEFT JOIN` über die Zahlungen des Jahres, die
  Beträge rechtsbündig über `PDF_TEXT_WIDTH`.

Geprüft wird es in `tests/pruef/beispiel_verein.dhtest`: derselbe Ablauf
(anlegen, ein Kollege speichert dazwischen, überschreiben, neu laden, ein
Kollege löscht, Zahlung, Liste) gegen SQLite, PostgreSQL und MySQL mit
derselben Erwartung.
