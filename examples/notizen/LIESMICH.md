# Notizen

Ein Notizzettel mit Fenster: links die Notizen mit einem Suchfeld darüber,
rechts Titel und Text. Fett, kursiv und unterstrichen gehen mit Strg+B, Strg+I
und Strg+U; gespeichert wird das als Markdown. Ohne Titel gilt die erste Zeile
des Textes.

- **Strg+N** neue Notiz, **Strg+S** sichern, **Strg+Entf** löschen
- **Strg+P** die Notiz als PDF sichern (Wörter werden an der Seitenbreite
  umgebrochen)
- **Strg+H** versteckt das Fenster; das Symbol im Infobereich holt es zurück,
  sein Menü legt auch eine neue Notiz an oder beendet
- Wer eine andere Notiz wählt, eine neue anlegt oder beendet, während etwas
  ungesichert ist, wird gefragt: Sichern, Verwerfen oder Abbrechen

Gespeichert wird in SQLite (`notizen.db` neben dem Programm, legt es beim
ersten Start selbst an). Das Fenster lässt sich größer ziehen, Liste und Text
wachsen mit.

Entstanden beim Dogfooding am 2026-10-09; dabei kamen `INSTRREV`,
`GUI_WINDOW_CONTENT_W/H`, `GUI_CLICKED` für Listen und zwei Korrekturen an der
gui heraus (Markieren beim Halten der Maustaste, umgebrochener Leer-Hinweis).
