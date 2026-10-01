# Haushaltsbuch

Einnahmen und Ausgaben in einer kleinen Anwendung mit Fenster: Datum aus dem
Kalender, Text, Betrag in Euro und ein Kästchen für Ausgaben. „Buchen“ legt
die Buchung an, die Tabelle zeigt alle nach Datum mit der Summe darunter,
Ausgaben rot. Ein Doppelklick auf eine Zeile holt sie zum Ändern in die
Felder, „Löschen“ entfernt die gewählte.

Gespeichert wird in SQLite (`haushalt.db` neben dem Programm, legt es beim
ersten Start selbst an). Beträge rechnet es in ganzen Cent, damit keine
Rundungsfehler entstehen.

In der IDE startet F5 das Programm. Entstanden ist es beim Ausprobieren der
IDE (Formular-Vorlage, Datumswähler, Tabelle, Datenbank mit `?`-Parametern).
