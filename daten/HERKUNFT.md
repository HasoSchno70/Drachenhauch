# Herkunft der Dateien in `daten/`

Was die Laufzeit und die IDE mitbringen, mit Herkunft und Lizenz.
Die Beispiel-Assets stehen in [examples/ASSET-CREDITS.md](../examples/ASSET-CREDITS.md).

| Datei | Herkunft | Lizenz |
|---|---|---|
| `video/vorspann.mp4`, `video/vorspann_2.mp4` | Vorspann der IDE, vom Projektautor mit einem KI-Werkzeug erzeugt | wie das Projekt (MIT oder Apache-2.0) |
| `bilder/logo.png` | Logo und Fenstersymbol, vom Projektautor mit einem KI-Werkzeug erzeugt | wie das Projekt |
| `bilder/schriftzug.png` | letztes Bild von `vorspann_2.mp4`, freigestellt mit `tools/schriftzug.dh` | wie das Projekt |
| `schriften/NotoMusic-Regular.ttf` | Noto Music, Google ([notofonts](https://github.com/notofonts/music)) | SIL Open Font License 1.1, Text in `schriften/OFL-NotoMusic.txt` |
| `*.json`, `meldungen.en.txt` | aus dem Handbuch und dem Quelltext erzeugt (`dhrt doku`) | wie das Projekt |

Die Schriften DejaVu (Modul `pdf` und `LOADFONT("sans")`) kommen über die Rust-Crate
`dejavu` in die Laufzeit (Bitstream Vera / DejaVu-Lizenz); `installer/lizenzen.dh`
sammelt die Lizenzen aller Crates beim Bau des Installers.

---

# Origin of the files in `daten/`

| File | Origin | License |
|---|---|---|
| `video/vorspann.mp4`, `video/vorspann_2.mp4` | IDE intro, made by the project author with an AI tool | same as the project (MIT or Apache-2.0) |
| `bilder/logo.png` | logo and window icon, made by the project author with an AI tool | same as the project |
| `bilder/schriftzug.png` | last frame of `vorspann_2.mp4`, cut out with `tools/schriftzug.dh` | same as the project |
| `schriften/NotoMusic-Regular.ttf` | Noto Music, Google | SIL Open Font License 1.1, see `schriften/OFL-NotoMusic.txt` |
| `*.json`, `meldungen.en.txt` | generated from the manual and the source (`dhrt doku`) | same as the project |
