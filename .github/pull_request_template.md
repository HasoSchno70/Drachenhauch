## Worum es geht

<!-- Was aendert sich, und warum? Der Titel sagt das WAS -- hier steht das WARUM.
     Bei einem Fund: was war kaputt, und woran hat man es gemerkt? -->

## Wie geprueft

<!-- Nicht "Tests laufen", sondern was tatsaechlich gemessen wurde. Faustregel
     aus diesem Repo: gegen den AERMSTEN Bau pruefen, nicht gegen die eigene
     Maschine -- die hat Grafik, Hardware-Features und einen nachsichtigen
     Linker. Wo ein Fund behauptet wird, gehoert die Gegenprobe dazu: schlaegt
     der neue Test gegen den alten Stand wirklich fehl? -->

- [ ] `pytest tests/ -q -n auto --dist loadfile -m "not seriell"` + `-m seriell`
- [ ] `rust/build_runtime.py` (bei Aenderungen an `rust/`)
- [ ] Doku-Pruefer, wenn `docs/`, `CLAUDE.md` oder ein Buch betroffen ist:
      `tools/pruef_docs.py`, `tools/pruef_doku_aussagen.py`, `tools/pruef_meldungen.js`

## Grenzen

<!-- Was ist NICHT abgedeckt? Was konnte hier nicht geprueft werden (macOS,
     Linux, Hardware, Netz)? Lieber hier benennen als in der CI entdecken. -->
