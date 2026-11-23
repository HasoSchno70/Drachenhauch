# Leitfaden: "Kleine Wunder" -> "Little Wonders" (English edition)

The German book teaches the Drachenhauch language (a BASIC dialect) through
programs that are fun to watch. The English edition is built from the SAME
chapter sources; every German text goes through a catalogue. You translate
text strings; code is never translated.

## Decisions made while translating Part I (2026-11-23)

- The module `zeit` is imported as `time` in the code; "the time module".
- Exercise levels: Tweak / Extend / Invent; "Zahlen drehen" -> "Tweak the numbers".
- Kommazahl -> decimal number, ganze Zahl -> whole number; 400 000 -> 400,000.
- Strahlensatz -> intercept theorem; Handgriffe -> moves; Leisten -> bars.
- Further-reading labels follow the headings of docs/en ("Manual, Language → Arrays: ").
- dhrt messages and output blocks stay German (they are not in the catalogue).
- A single lowercase word is taken for a code identifier and NOT translated
  (i18n.js, NICHT_PROSA) -- write table cells as a phrase ("das Programm
  starten", not "starten").

## Input / output format

- Input: `<datei>.de.json` -- a JSON array of German strings, in chapter order.
- Output: `<datei>.en.json` -- a JSON array of English strings, SAME LENGTH,
  SAME ORDER (element i translates element i). UTF-8, valid JSON.
- Never merge, split, drop or reorder entries.

## What the strings are

Paragraphs, headings (h2/h3), bullet labels and bodies, figure captions,
table cells and headers, and **comments from program code** (the text after
`'` in a code line -- translate it as a code comment: short, plain).
Bullet labels often end with a space or ": " -- keep that trailing
punctuation/space exactly.

## Must stay unchanged

- Drachenhauch commands and keywords: `SCREEN`, `CIRCLE`, `FLIP`, `DELTA()`,
  `QUITREQUESTED()`, `FOR`, `NEXT`, `DIM … AS FLOAT`, `RGB`, `HSV`, `KEY_ESCAPE` …
- Identifiers from the programs (they stay German in the code): `drehung`,
  `stern`, `funken`, variable/function/file names like `1_farbkreis.dh`,
  `fehler_tippfehler.dh`. Write them exactly as in the German text.
- Strings that the program shows (window titles like "Farbkreis") stay as
  they are, since the code is German.
- Numbers, units, formulas, inline code. But: German decimal comma becomes a
  decimal point in prose (0,017 -> 0.017; 2,5 -> 2.5).
- Error messages quoted from dhrt stay German (they are printed as dhrt
  prints them on a German system).
- Names of other books stay as titles in curly quotes: „Der Einstieg“ ->
  “Der Einstieg” (German beginners' book; you may add "(German)" once where
  it first appears). The Handbook = "Das Lehrbuch" -> "the Handbook".
- NO markdown: `*x*` or `**x**` is printed literally in the book.

## Terms (keep consistent across the book)

| German | English |
|---|---|
| Was wir bauen | What we build |
| Der Plan | The plan |
| Schritt für Schritt | Step by step |
| Warum so? | Why like this? |
| Was schiefgehen kann | What can go wrong |
| Übungen | Exercises |
| Drehen / Erweitern / Erfinden (exercise levels) | Tweak / Extend / Invent |
| Drehen 1. / Erweitern 2. / Erfinden. (bullet labels) | Tweak 1. / Extend 2. / Invent. |
| Zum Weiterlesen | Further reading |
| Lösungen in Anhang A | solutions in Appendix A |
| Handbuch (the online/docs manual) | the manual |
| Handbuch, Grafik-Befehle: | Manual, graphics commands: |
| Bild (one drawn frame) | frame |
| Bild (a picture/figure) | picture / figure |
| Fenster | window |
| Schleife | loop |
| Feld (ARRAY) | array |
| Punkte (screen units) | pixels / points (use "pixels" for screen sizes) |
| Schreibmarke | cursor |
| Meldung | message |
| Laufzeitfehler | run-time error |
| Übersetzen (compile) | compile |
| Strg / Umschalt / Entf | Ctrl / Shift / Del |
| Spielwiese | the playground |
| IDE, Studio, dhrt | unchanged |
| Zwischenstand | intermediate stage |

## Style

- British spelling (colour, centre, behaviour), as in the English Handbook.
- Address the reader as "you", warm and direct, like the German.
- Natural English, not word-for-word. Keep the meaning and every fact,
  number and claim exactly; do not add or drop content.
- German quotes „…“ become “…”. Dashes — stay as em dashes.
- Keep sentence-level structure roughly (one German paragraph = one English
  paragraph).
