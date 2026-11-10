# Contributing to Drachenhauch

*Deutsch weiter unten.*

Thank you for wanting to help. Bug reports, examples, documentation fixes and
code are all welcome.

## Reporting a bug or wishing for something

Open an [issue](https://github.com/HasoSchno70/Drachenhauch/issues/new/choose).
For a bug, the most useful thing is a **small program that shows it**, plus the
output of `dhrt --version` and your operating system. Security problems do not
belong in a public issue, see [SECURITY.md](SECURITY.md).

## Changing code or documentation

1. Build the runtime as described under *Working from source* in the
   [README](README.md#working-from-source). Use `rust/build_runtime.py`;
   `cargo build` alone can leave stale results.
2. Work on a branch and open a pull request against `main`. The template asks
   what changed, why, and how you checked it.
3. Every change comes with a check in a test collection
   (`tests/pruef/<topic>/*.dhtest`, format in [docs/en/werkzeuge.md](docs/en/werkzeuge.md)).
   Make sure the new case **fails** without your change and passes with it.
4. Run the collections you touched, and before the PR the fast run:
   `dhrt test tests/pruef --schnell`. CI then runs everything on Windows,
   Linux and macOS.
5. If you change `docs/<file>.md`, change `docs/en/<file>.md` too; `dhrt pruef`
   checks code blocks, names, paths and that both languages match.

The rules that keep the project in shape (new built-ins, modules, the machine
code compiler, the GUI) are written down in [CLAUDE.md](CLAUDE.md). It is in
German; the short version of the most important ones:

- Run-time error messages are written in German; the English catalogue
  `daten/meldungen.en.txt` follows along (`dhrt pruef meldungen`).
- A new built-in needs an entry in `daten/builtin_index.json` with the correct
  signature and a line in `docs/module-*.md` and `docs/en/module-*.md`.
- `MID$`/`INSTR` count from 0, `/` always returns a FLOAT.

## License

By contributing you agree that your contribution is licensed under the
project's terms: MIT or Apache-2.0, at the user's choice.

---

# Mitmachen bei Drachenhauch

Danke, dass du helfen willst. Fehlerberichte, Beispiele, Korrekturen am
Handbuch und Code sind gleichermaßen willkommen.

## Fehler melden oder etwas wünschen

Ein [Issue](https://github.com/HasoSchno70/Drachenhauch/issues/new/choose)
öffnen. Bei einem Fehler hilft am meisten ein **kleines Programm, das ihn
zeigt**, dazu die Ausgabe von `dhrt --version` und das Betriebssystem.
Sicherheitslücken gehören nicht in ein öffentliches Issue, siehe
[SECURITY.md](SECURITY.md).

## Code oder Handbuch ändern

1. Die Laufzeit bauen wie im [README](README.de.md#aus-dem-quelltext-arbeiten)
   unter *Aus dem Quelltext arbeiten* beschrieben. Immer über
   `rust/build_runtime.py`; `cargo build` allein kann veraltete Ergebnisse
   liefern.
2. Auf einem eigenen Zweig arbeiten und einen Pull Request gegen `main`
   stellen. Die Vorlage fragt, was sich ändert, warum, und wie es geprüft ist.
3. Zu jeder Änderung gehört ein Fall in einer Prüfsammlung
   (`tests/pruef/<thema>/*.dhtest`, Format in [docs/werkzeuge.md](docs/werkzeuge.md)).
   Der neue Fall muss **ohne** die Änderung fehlschlagen und mit ihr grün sein.
4. Die berührten Sammlungen laufen lassen, vor dem PR den schnellen Lauf
   `dhrt test tests/pruef --schnell`. Die CI fährt danach alles auf Windows,
   Linux und macOS.
5. Wer `docs/<datei>.md` ändert, zieht `docs/en/<datei>.md` mit; `dhrt pruef`
   prüft Codeblöcke, Namen, Pfade und ob beide Sprachen zusammenpassen.

Die Regeln, die das Projekt in Form halten (neue Builtins, Module, der
Maschinencode, die gui), stehen in [CLAUDE.md](CLAUDE.md).

## Lizenz

Wer beiträgt, stimmt zu, dass sein Beitrag unter den Bedingungen des Projekts
steht: MIT oder Apache-2.0, nach Wahl des Nutzers.
