# Drachenhauch 2026.26

*Die Notizen zu dieser Fassung. Einen Tag nach 2026.25, mit einem Thema:
das Symbol im Infobereich gibt es jetzt auf allen drei Systemen -- unter
Windows in der Taskleiste, unter macOS in der Menüleiste und unter Linux in
der Leiste des Desktops. Dazu ein Beispiel, das davon lebt.*

## Neu in dieser Fassung

### Das Tray-Symbol unter macOS und Linux

```basic
DIM bild AS INTEGER
bild = LOADIMAGE("symbol.png")
TRAY_SHOW(bild, "Mein Werkzeug")
TRAY_MENU("Fenster zeigen|-|Beenden")
DIM fertig AS BOOLEAN
fertig = FALSE
WHILE NOT fertig AND NOT QUITREQUESTED()
    IF TRAY_CLICKED() THEN WINDOW_SHOW()
    IF TRAY_MENU_CLICKED$() = "Beenden" THEN fertig = TRUE
    FLIP()
WEND
TRAY_HIDE()
```

Die `TRAY_*`-Befehle und `NOTIFY` gab es bisher nur unter Windows. Jetzt
gelten sie mit denselben Namen auch anderswo:

* **macOS:** das Symbol steht in der Menüleiste, auf 18 Punkte gebracht. Ein
  Klick ist `TRAY_CLICKED`, zwei schnell hintereinander zusätzlich
  `TRAY_DOUBLE_CLICKED`; die rechte Taste oder Ctrl+Klick öffnen das Menü.
* **Linux:** das Symbol meldet sich über D-Bus bei der Leiste an
  (*StatusNotifierItem*, samt Menü). KDE Plasma, Xfce, LXQt, Cinnamon und
  MATE zeigen es, **GNOME nur mit der Erweiterung „AppIndicator“**. Ohne
  Leiste bleibt das Symbol unsichtbar, `TRAY_SHOW` ist aber kein Fehler --
  startet die Leiste später, meldet es sich von selbst an. Ohne D-Bus-Sitzung
  (etwa über SSH) ist es ein Fehler. Einen Doppelklick kennt das Verfahren
  nicht.
* **Mitteilungen** (`NOTIFY`) gehen unter macOS über `osascript`, unter Linux
  über `notify-send`.
* `dhrt pruef tray` lässt das Symbol des Systems einmal echt durchlaufen --
  anlegen, zurücklesen, Klick und Menüwahl -- und sagt, was dabei herauskam.

### Beispiel 209: der Pausenwecker

Ein kleines Werkzeug, das im Infobereich lebt: ein Regler stellt die Minuten
bis zur nächsten Pause ein, der Hinweis am Symbol zählt herunter, und ist die
Zeit um, kommt eine Mitteilung des Systems. Das Kreuz versteckt das Fenster
nur, ein Klick auf das Symbol holt es zurück; beendet wird über das Menü.
Das Symbol zeichnet das Programm selbst. Es steht in der IDE unter
*Beispiele nach Themen* → *Fenster und System*.

## Unter der Haube

**Zahlen.** Es bleibt bei **2109** Befehlen in 48 Modulen; **225**
Beispiele (vorher 224). **4899 Fälle in 373 Prüfsammlungen** (vorher 4896)
und **533 Rust-Testfunktionen**.

* Unter macOS braucht das Symbol keine neue Bibliothek, es spricht direkt
  mit der Objective-C-Laufzeit. Unter Linux kommt zbus dazu -- in derselben
  Fassung, die die Barrierefreiheit ohnehin mitbringt.
* Die CI baut auf macOS und Linux ohne Grafik, dort gab es also nichts zum
  Laufen. `dhrt pruef tray` schließt die Lücke: unter Linux startet es einen
  eigenen D-Bus, stellt eine Leiste nach und liest das Symbol als fremder
  Teilnehmer zurück; unter macOS arbeitet es mit echten AppKit-Objekten. Die
  Prüfsammlung ruft es auf beiden Systemen auf.
* Unter Windows bedient die Prüfsammlung das **echte** Beispiel 209 über
  Nachrichten an das Symbol: verstecken, zurückholen, „Pause jetzt“,
  „Beenden“.

## Was offen bleibt

* Ob ein echter Mac und eine echte Linux-Leiste das Symbol auch **sehen**
  lassen, ist nicht ausprobiert -- geprüft ist es über AppKit und D-Bus, nicht
  am Bildschirm.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf, ein Index
  direkt auf dem Ergebnis eines Aufrufs (`reihe(5)[0].x`, eine Meldung);
  C++ und COM. `VALIST` und die Bitfelder auf Linux-ARM sind nach den Regeln
  gebaut, aber nicht ausprobiert.
* Die Formular-Fälle gegen die Datenbank-Server brauchen ein Fenster und
  sind nur lokal belegt.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt; der Installer ist nicht signiert.
