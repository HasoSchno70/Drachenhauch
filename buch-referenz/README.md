# GameBasic – Das Lehrbuch

*Deutsch · [English overview](README.en.md)*

Ein vollständiges Lehr- und Referenzbuch für GameBasic: lehrt das Programmieren
von Grund auf **und** erklärt jeden Befehl mit kleinem Beispielprogramm.
Ausgabe: editierbares `.docx` zum Drucken.

Code wird durchgehend gut erkennbar dargestellt (Schreibmaschinenschrift im
grauen Kasten mit blauer Leiste), Programm-Ausgabe im grünen Kasten.

- Quelle & Build-Anleitung: [`buch/OUTLINE.md`](buch/OUTLINE.md) (Gliederung +
  Fortschritt + Architektur).
- Bauen: `cd buch && node build_book.js` → `GameBasic-Lehrbuch.docx`.
  Mit korrekten Inhaltsverzeichnis-Seitenzahlen: `python make_book.py`.
- Inhalt liegt modular in `buch/content/NN_*.js` – neues Kapitel = neue Datei.

Das Buch entsteht über mehrere Sitzungen (es behandelt alle ~760 Befehle +
die ganze Sprache). Der aktuelle Stand steht in `buch/OUTLINE.md`.
