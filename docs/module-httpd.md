# Modul `httpd`

Ein kleiner Webserver im Takt der Hauptschleife — für die Bedienoberfläche
einer Steuerung, ein Messwert-Blatt im Heimnetz, eine Fernbedienung fürs
eigene Programm.

```basic
IMPORT "httpd"
```

## Warum es ihn gibt

Die erste Allzweck-Roadmap hatte ihn ausdrücklich gestrichen: *„wer wirklich
einen Dienst braucht, stellt einen fertigen Server davor"*. Für einen
öffentlichen Dienst stimmt der Satz weiterhin. Vor dem Bastler-Leitbild sieht
es anders aus: Mit `mqtt`, `firmata`, `serial` und `net` an Bord fehlte für
„meine Heizungssteuerung hat eine kleine Weboberfläche" genau ein Baustein —
und es war der kleinste von allen, weil `NET_TCP_LISTEN` schon darunter liegt.

## Übersicht

| Funktion | Zweck |
|---|---|
| `HTTPD_START(port[, bind$])` → HTTPD | Server starten; `0` = freien Port wählen lassen |
| `HTTPD_PORT(s)` → INTEGER | der tatsächlich belegte Port |
| `HTTPD_ACCEPT(s)` → BOOLEAN | liegt eine Anfrage an? (kehrt sofort zurück) |
| `HTTPD_METHOD$(s)` → STRING | `GET`, `POST`, … |
| `HTTPD_PATH$(s)` → STRING | der angefragte Pfad, ohne `?…` |
| `HTTPD_QUERY$(s, name$)` → STRING | ein Wert aus `?a=1&b=2` (aufgelöst) |
| `HTTPD_HEADER$(s, name$)` → STRING | eine Kopfzeile (Groß-/Kleinschreibung egal) |
| `HTTPD_BODY$(s)` → STRING | der Rumpf (bei `POST`) |
| `HTTPD_SEND(s, code, typ$, inhalt$)` | antworten und schließen |
| `HTTPD_SEND_FILE(s, code, pfad$)` | eine bestimmte Datei ausliefern |
| `HTTPD_SEND_DIR(s, ordner$[, start$])` → BOOLEAN | den angefragten Pfad **sicher** im Ordner auflösen |
| `HTTPD_STOP(s)` | Server beenden |

## Das Muster

Ein Aufruf je Runde — dasselbe Muster wie `INPUT_UPDATE`, `TIMER_UPDATE` und
`MQTT_UPDATE`:

```basic
IMPORT "httpd"

DIM s AS HTTPD
DIM grad AS FLOAT
s = HTTPD_START(8080)
grad = 21.5

WHILE NOT QUITREQUESTED()
    IF HTTPD_ACCEPT(s) THEN
        IF HTTPD_PATH$(s) = "/setzen" THEN
            grad = VAL(HTTPD_QUERY$(s, "grad"))
            HTTPD_SEND(s, 200, "text/plain; charset=utf-8", "ok")
        ELSE
            HTTPD_SEND(s, 200, "text/html; charset=utf-8", _
                       "<h1>" + STR$(grad) + " Grad</h1>")
        END IF
    END IF
    SLEEP(10)
WEND
HTTPD_STOP(s)
```

`HTTPD_ACCEPT` **kehrt sofort zurück**, wenn niemand anklopft — die
Hauptschleife läuft also weiter, auch in einem Spiel mit 60 Bildern je
Sekunde.

## Dateien ausliefern

```basic
IF HTTPD_ACCEPT(s) THEN
    IF NOT HTTPD_SEND_DIR(s, "web") THEN
        PRINT "nicht gefunden: " + HTTPD_PATH$(s)
    END IF
END IF
```

`HTTPD_SEND_DIR` nimmt den angefragten Pfad, löst ihn **innerhalb** des
Ordners auf, rät den Inhaltstyp aus der Endung und antwortet. Ein Pfad auf
einen Ordner bekommt `index.html` (oder was als `start$` dasteht); eine
fehlende Datei wird zu `404` und liefert `FALSE` zurück — sie ist der Alltag
eines Servers und darf das Programm nicht anhalten.

**Nimm `HTTPD_SEND_DIR`, nicht Handarbeit.** Die naheliegende Zeile

```basic
HTTPD_SEND_FILE(s, 200, "web" + HTTPD_PATH$(s))     ' NICHT so
```

lässt sich mit `GET /../../geheim.txt` aus dem Ordner herausführen — und
liefert dann irgendeine Datei vom Rechner aus. `HTTPD_SEND_DIR` lehnt jeden
Pfadteil `..` ab (dazu Doppelpunkte und Backslashes, die unter Windows ein
Laufwerk oder einen alternativen Datenstrom meinen könnten) und antwortet mit
`403`. Es ist dieselbe Lehre wie bei der Zip-Slip-Prüfung in `ZIP_EXTRACT`.

`HTTPD_SEND_FILE` bleibt für den Fall, dass **das Programm** entscheidet,
welche Datei rausgeht — dort steht kein fremder Pfad im Spiel.

## Grenzen — und warum

* **Kein HTTPS.** Wer Verschlüsselung braucht, stellt einen Reverse-Proxy
  davor. Ein TLS-Server hier hieße Zertifikatsverwaltung im Programm.
* **Kein Keep-Alive**, eine Verbindung je Anfrage. Bei einer
  Bedienoberfläche kostet das nichts und erspart die halbe Zustandsverwaltung
  eines echten Servers.
* **Eine Anfrage je `HTTPD_ACCEPT`**, nacheinander. Zwei Browser gleichzeitig
  werden bedient, nur eben nach der Reihe.
* **Der Rumpf wird ganz in den Speicher gelesen**, Obergrenze 8 MiB. Zum
  Hochladen großer Dateien taugt der Server nicht.
* **Eine angefangene Anfrage darf 50 ms brauchen**, dann wird die Verbindung
  fallengelassen. Jede Millisekunde hier ist eine, die das Programm nicht
  zeichnet; ein Browser schickt seine Anfrage ohnehin in einem Stück.
* **Kein Cookie-, Sitzungs- oder Anmelde-Vokabular.** Wer eine Anmeldung
  braucht, liest `HTTPD_HEADER$(s, "Authorization")` selbst aus und vergleicht
  mit `SECURE_EQUALS`.

Alles zusammen heißt: **im Heimnetz ja, im offenen Netz nein.** So herum
stimmt der Satz aus der alten Roadmap.

## Beim Entwickeln

`HTTPD_START(0)` lässt das Betriebssystem einen freien Port wählen —
praktisch, wenn mehrere Programme gleichzeitig laufen. Den gewählten Port
meldet man am besten mit `EPRINT`: `PRINT` wird gepuffert und erscheint erst
am Programmende, `EPRINT` sofort.

```basic
s = HTTPD_START(0)
EPRINT("Läuft auf Port " + STR$(HTTPD_PORT(s)))
```

Beispiel: [examples/173_webserver.dh](../examples/173_webserver.dh).
