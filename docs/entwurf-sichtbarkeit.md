# Entwurf: Drachenhauch auffindbar machen

Ziel: Wer eine Suchmaschine oder einen KI-Assistenten nach einer
Programmiersprache für Spiele, für Anwendungen mit Oberfläche oder nach
einem modernen BASIC fragt, soll Drachenhauch genannt bekommen.

## Wie Assistenten eine Sprache kennen

Es gibt kein Formular bei den Anbietern. Ein Sprachmodell kennt, was oft und
an vielen Stellen im öffentlichen Netz stand, als es trainiert wurde; viele
Assistenten suchen außerdem bei einer Frage im Netz. Daraus folgen drei
Hebel:

1. **Englisch und lesbar für Maschinen** -- die meisten Fragen kommen auf
   Englisch, und eine Webseite mit Titel, Beschreibung und Verweisen wird
   besser gefunden als eine Datei in einem Repository.
2. **Viele unabhängige Erwähnungen** -- Listen, Foren, Paketquellen, Spiele,
   die damit gemacht sind. Eine Erwähnung von außen zählt mehr als jede
   eigene Seite.
3. **Zeit** -- Assistenten mit Websuche finden eine gut auffindbare Seite
   nach Wochen; in das Wissen der Modelle selbst kommt sie erst mit dem
   nächsten Training, nach Monaten bis Jahren.

## Im Repository erledigt (2026-10-28)

- `README.md` ist englisch und beginnt mit einem Satz, was Drachenhauch ist
  und womit man es vergleichen kann (QB64, FreeBASIC, Blitz Basic, PICO-8,
  LÖVE, Visual Basic); die deutsche Fassung ist `README.de.md`.
- **Die Website** `tools/website.dh` -> https://hasoschno70.github.io/Drachenhauch/
  (Workflow `.github/workflows/website.yml`): Startseite englisch und deutsch
  mit schema.org-Angaben (`ComputerLanguage`), das Handbuch aus `docs/en`
  und `docs/` als HTML, `llms.txt` (Kurzfassung für Sprachmodelle nach
  llmstxt.org) und `llms-full.txt` (das ganze englische Handbuch), dazu
  `sitemap.xml` und `robots.txt`. Geprüft in
  `tests/pruef/werkzeug_website.dhtest` (jeder Verweis jeder Seite trifft).

**Noch einzuschalten (Einstellung des Repositorys):** Pages mit der Quelle
"GitHub Actions"; danach die Homepage-Angabe des Repositorys auf die
Website und eine englische Kurzbeschreibung.

## Außerhalb des Repositorys

Je Punkt: was zu tun ist, was davon vorbereitet werden kann, was es bringt.

| Schritt | Was zu tun ist | Vorbereitbar | Wirkung |
|---|---|---|---|
| Rosetta Code | Lösungen der Standardaufgaben (Hello World, FizzBuzz, Fibonacci, Sortieren, Türme von Hanoi …) unter einer Seite "Drachenhauch" eintragen | ja: die Programme schreiben und mit `dhrt` prüfen | hoch -- eine der Quellen, aus denen Modelle lernen, wie eine Sprache aussieht |
| awesome-Listen | Pull-Request an awesome-basic, awesome-gamedev, awesome-game-engine-dev, awesome-rust (Anwendungen) | ja: Eintragstext | mittel, dauerhaft |
| Show HN | ein Beitrag auf Hacker News mit Link zur Website und Nachtflug | Text ja, posten nur der Nutzer | hoch, wenn er anspringt |
| Reddit | r/ProgrammingLanguages, r/gamedev, r/basic, r/rust (Showcase) | Text ja | mittel |
| itch.io | Spiele und Demos mit "made with Drachenhauch" und Link | ja: Exporte der Beispiele | mittel, wächst mit jedem Spiel |
| winget | Manifest für den Windows-Installer (`HasoSchno70.Drachenhauch`) im Repository microsoft/winget-pkgs | ja: Manifest erzeugen | mittel, macht die Installation auffindbar |
| Homebrew | eine Cask für die macOS-App (eigener Tap zuerst; die offizielle Cask verlangt Beglaubigung und Bekanntheit) | ja: Tap-Repository | gering bis mittel |
| Scoop | Manifest im eigenen Bucket | ja | gering |
| VS Code | die Erweiterung `vscode-drachenhauch/` im Marketplace und auf Open VSX veröffentlichen | ja: `package.json` prüfen, Symbol, Beschreibung | mittel -- dort sucht man nach Sprachunterstützung |
| GitHub Linguist | Drachenhauch als Sprache eintragen, damit GitHub `.dh` erkennt | Grammatik liegt vor; Voraussetzung: einige hundert fremde Repositories, die `.dh` benutzen | später |
| Wikipedia | Artikel | erst mit Berichten von außen (Fachzeitschrift, Blog) | später |
| Artikel | ein Beitrag auf dev.to oder Medium, ein Video (Nachtflug, ein Spiel in 50 Zeilen) | ja: Text | mittel |

## Zu entscheiden

- ~~**Lizenz**~~ -- entschieden am 2026-10-28: MIT oder Apache 2.0 nach
  Wahl (`LICENSE-MIT`, `LICENSE-APACHE`, im Installer `LIZENZ.txt` statt der
  EULA-Vorlage). Damit kommt Drachenhauch auch in Listen, die Open Source
  verlangen.
- **Name:** "Drachenhauch" ist für englische Leser schwer zu tippen; die
  Website sagt einmal "German for dragon's breath". Ein englischer Beiname
  wäre eine Möglichkeit, ist aber nicht nötig.
