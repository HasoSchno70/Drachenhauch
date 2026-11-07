# Fenster, Tasten und Testlaeufe

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Klicks zwischen zwei Bildern, Tastennamen, Testfenster ohne Eingabe, Zeitgrenze je Fall.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Klicks und Tasten zwischen zwei Bildern (2026-09-30)

raylib liest Maus- und Tastentasten als ZUSTAND (Rueckruf schreibt
`current...State`, einmal je Bild nach `previous` kopiert). Kamen Druecken
UND Loslassen vor dem naechsten Bild an, sah niemand den Klick -- gemessen
mit echten Fensternachrichten: zusammen geschickt 0 von 20, bei 60 Bildern
je Sekunde unter ~16 ms ein grosser Teil, bei 15 Bildern je Sekunde noch
40 ms (Touchpad-Tippen, langsame Programme). `flanken.rs` haengt eigene
GLFW-Rueckrufe (Maus UND Tastatur) vor die von raylib und zaehlt; FLIP holt
die Zaehler nach `automation_tick` ab (`flanken::weiter`). Lag ein ganzer
Druck in der Luecke, gilt die Taste EIN Bild lang als gedrueckt und im
naechsten als losgelassen -- so sehen MOUSEBUTTON/MOUSE_HIT/KEYHIT und die gui
ihn ohne Aenderung. **Regel:** jede Tastenabfrage in graphics.rs geht ueber
`t_unten/t_neu/t_los` bzw. `mouse_button/mouse_hit/mouse_released`, nie direkt
an `self.rl.is_*` (Ausnahme: das Abholen im FLIP selbst). Die Wiedergabe
zaehlt ihre Maus- und Tastenereignisse mit, damit ein Druck im selben Bild
der Aufnahme dasselbe ist wie einer von aussen. Bekannte Grenze: zwei
verschluckte Klicks in DIREKT aufeinanderfolgenden Bildern sieht die gui als
einen (MOUSE_HIT zaehlt beide). Tests `tests/pruef/fenster/klick_im_selben_bild.dhtest`
(9; gegen 2026.18 fallen die 5 neuen, die 4 Gegenproben nicht; echte
Nachrichten ueber `fenstersender.ps1` `klicks:N:MS`/`tasten:N:MS`),
Rust-Tests in flanken.rs.

## Tastennamen ohne Tottasten-Zustand (2026-10-12)

`input_polish` "printable key uses the keyboard layout" fiel seit Fassungen
sporadisch ("Á" statt "A"), auch einzeln. Ursache: GLFW liest die Namen unter
Windows mit `ToUnicode`, und das veraendert den Tottasten-Zustand -- der gilt
NICHT je Prozess. Starten mehrere Programme zugleich, landet der Akzent der
deutschen ´-Taste (in GLFWs Reihenfolge vor A) bei einem anderen: gemessen
8 Fenster zugleich 6 von 48 falsch (`Á`, `Û`, `Ú`, Q als `^`), nacheinander
24 von 24 richtig. Betroffen waren `KEY_NAME$` UND die Belegung fuer
`KEYHIT("z")` und die Menue-Kuerzel (`belegung_auffrischen`). Jetzt
`Graphics::tastenname_roh`: unter Windows `MapVirtualKeyW` (VSC_TO_VK_EX, dann
VK_TO_CHAR; ohne Zustand, eine Tottaste liefert ihr eigenes Zeichen), der
Ziffernblock wie GLFW ueber feste VK_NUMPAD*, erweiterte Scancodes 0x100+x ->
0xE0xx; sonst weiter GLFW (X11 und macOS lesen ohne geteilten Zustand).
**Falle:** `#[link(name = "user32")]` fuer MapVirtualKeyW stellte user32.lib
vor raylib -- LNK2005 CloseWindow/ShowCursor; darum ueber das windows-Crate
(Feature `Win32_UI_Input_KeyboardAndMouse`). Danach 80 von 80 richtig. Test
"key names stay right when several programs start at once" in
`input_polish.dhtest` (faellt mit 2026.25 in 3 von 3 Laeufen).

## Testfenster ohne echte Eingabe (2026-10-06)

Die "wackelnden" IDE-Faelle waren nicht Last (fuenf Runden unter 16
Rechenprozessen gruen), sondern ECHTE Eingaben: wer waehrend eines Laufs
tippte, schrieb in das Fenster eines Falls ("BSCREEN" statt "SCREEN", ein
neuer Reiter mit nur "e"), eine Maus ueber dem Fenster nahm einem Fall den
Tooltip. `dhrt test` setzt jetzt jedem Kind `DHRT_OHNE_EINGABE=1`
(`stillfenster.rs`): GLFW_FOCUS_ON_SHOW aus, GLFW_MOUSE_PASSTHROUGH an
(nachgeschickte Nachrichten kommen trotzdem an), und unter Windows gibt
FLIP den Vordergrund an das Fenster zurueck, das ihn beim Anlegen hatte
(Maximieren und WINDOW_FOCUS aktivieren sonst). "0" schaltet ab.
`WINDOW_FOCUSED()` liefert dann TRUE -- ein Fall der IDE (Aufrufzaehler) hing
am echten Fokus und waere schon vorher gefallen, haette jemand woanders
hingeklickt. Ein `' --- seriell` als KOMMENTAR galt nie (`werkzeug_ide_karte`
und `_korrektur` liefen so parallel) und ist jetzt ein Fehler. Nebenfund:
die Schnellkorrektur der IDE merkte sich eine NUMMER in `probKorr`, die jede
Pruefung neu baut -- aenderte sich der Text bei offener Auswahl, brach Enter
mit einem JSON-Fehler ab; jetzt gilt ein Angebot nur fuer seinen Text
(`korrAngebote`/`korrText`). Tests `tests/pruef/laufzeit/stillfenster.dhtest`
(fremder Leser: Vordergrund, Treffer in der Mitte, WS_EX_TRANSPARENT; die
Vordergrund-Frage schwankt ohne den Schalter, darum die Gegenprobe am Stil)
und ein Fall in `werkzeug_ide_korrektur.dhtest`.

## Zeitgrenze je Fall in `dhrt test` (2026-10-11)

Ein haengender Fall hielt den ganzen Lauf an, ohne Meldung (zweimal gesehen,
in der Konsolen- und der Musik-Runde). Jetzt laeuft jeder Lauf eines Falls
ueber `main.rs::test_lauf` (Leser-Faeden fuer beide Kanaele, Warte-Faden +
`recv_timeout` gegen eine Frist): die Frist gilt dem GANZEN Fall (`--- vorher`,
Hauptlauf, `--- nochmal`, `--- zwischen`, `--- nachher`), Vorrang
`--- zeitgrenze N` im Fall > im Kopf der Sammlung > `dhrt test --zeitgrenze N`
(0 = aus) > `TEST_ZEITGRENZE` = 300 s; Pruefprogramme (`*_pruefung.dh`) haben
dieselbe Vorgabe. Bei Ablauf beendet `prozessbaum_beenden` den Fall samt
Nachkommen (Windows `taskkill /T /F`, sonst `pkill -KILL -P` + `kill -KILL`
-- bewusst OHNE eigene Prozessgruppe, damit Strg+C auf `dhrt test` die Faelle
weiter trifft), und der Fall ist FEHL mit "Zeitgrenze von N s ueberschritten
(Hauptlauf|--- nachher|...) -- abgebrochen; zuletzt: ..." (die letzten drei
Ausgabezeilen). **Die Nachkommen sind der Punkt:** in der Gegenprobe ohne `/T`
lebte das Kind des Falls weiter und hielt die geerbte Ausgabeleitung des
Laeufers offen -- die Pipeline hing, bis es von Hand beendet war. Gemessen:
ein voller Lauf mit `--zeitgrenze 60` hat keinen Fall darueber, die Vorgabe
laesst das Fuenffache. Tests `tests/pruef/werkzeugkette/dhrt_test_zeitgrenze.dhtest` (5,
seriell; die Kinder-Probe ueber eine Pulsdatei), Rust-Test
`zeitgrenze_im_kopf_und_je_fall`. Nebenbefund: `input_polish` "printable key
uses the keyboard layout" faellt auch EINZELN sporadisch ('Á' statt 'A', etwa
einer von drei Laeufen) -- nicht von der Zeitgrenze, noch offen.
