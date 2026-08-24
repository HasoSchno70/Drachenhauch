# Modul `smtp`

E-Mail verschicken.

```basic
IMPORT "smtp"
```

Das Gegenstück zu [`xlsx`](module-xlsx.md) und [`pdf`](module-pdf.md): dort
entsteht die Auswertung, hier geht sie raus. Zusammen sind das die zwei
Hälften der Kette, die in Büroprogrammen am häufigsten verlangt wird —
*Bericht bauen und rausschicken*.

## Ein vollständiges Beispiel

```basic
IMPORT "smtp"

DIM m AS SMTP
m = SMTP_NEW()

SMTP_SERVER(m, "smtp.beispiel.de", 587)          ' 587 -> STARTTLS
SMTP_LOGIN(m, "hans@beispiel.de", "geheim")

SMTP_FROM(m, "hans@beispiel.de", "Abteilung Zahlen")
SMTP_TO(m, "chefin@beispiel.de")
SMTP_SUBJECT(m, "Auswertung August")
SMTP_TEXT(m, "Guten Tag," + CHR$(10) + CHR$(10) + "anbei die Zahlen.")
SMTP_ATTACH(m, "auswertung.xlsx")

SMTP_SEND(m)
SMTP_CLOSE(m)
```

## Übersicht

| Funktion | Zweck |
|---|---|
| `SMTP_NEW()` → SMTP | neue Nachricht |
| `SMTP_SERVER(m, host$, port [, sicherheit$])` | wohin eingeliefert wird |
| `SMTP_LOGIN(m, benutzer$, kennwort$)` | Anmeldung (weglassen = ohne) |
| `SMTP_FROM(m, adresse$ [, name$])` | Absender |
| `SMTP_TO(m, adresse$ [, name$])` | Empfänger — mehrfach aufrufbar |
| `SMTP_CC(m, adresse$ [, name$])` | Kopie |
| `SMTP_BCC(m, adresse$)` | Blindkopie |
| `SMTP_SUBJECT(m, betreff$)` | Betreff |
| `SMTP_TEXT(m, text$)` | Nachricht als reiner Text |
| `SMTP_HTML(m, html$)` | Nachricht als HTML |
| `SMTP_ATTACH(m, pfad$ [, name$])` | Datei anhängen — mehrfach aufrufbar |
| `SMTP_TIMEOUT(m, ms)` | Frist je Schritt (Vorgabe 30 000) |
| `SMTP_MESSAGE$(m)` → STRING | die fertige Nachricht, **ohne** zu senden |
| `SMTP_SEND(m)` | abschicken |
| `SMTP_CLOSE(m)` | Speicher freigeben |

**`SMTP_TEXT` und `SMTP_HTML` schließen sich nicht aus.** Wer beide setzt,
verschickt beide Fassungen (`multipart/alternative`); der Leser zeigt die an,
die er kann. Wer nur eine setzt, bekommt eine einteilige Nachricht — es wird
nichts künstlich verschachtelt.

## Verschlüsselung

Das dritte Argument von `SMTP_SERVER` ist die Sicherheit:

| Wert | Bedeutung | üblicher Port |
|---|---|---|
| `"starttls"` | erst im Klartext verbinden, dann hochschalten | 587 |
| `"tls"` | von der ersten Sekunde an verschlüsselt | 465 |
| `"keine"` | gar nicht | 25 |

**Ohne Angabe entscheidet der Port**: 465 → `tls`, 25 → `keine`, alles andere
→ `starttls`. Das trifft den Normalfall (Einlieferung über 587) und lässt
sich jederzeit ausdrücklich überschreiben.

**Ein Kennwort geht nicht unverschlüsselt ins Netz.** `SMTP_LOGIN` zusammen
mit `"keine"` ist ein Fehler — außer der Server ist `localhost`, denn ein
Relay auf demselben Rechner ohne TLS ist der Normalfall. Diese Regel kommt
aus dem Modul selbst, nicht vom Server: sonst gäbe ein vertippter Port das
Kennwort still preis.

Angemeldet wird mit `AUTH PLAIN`, sonst mit `AUTH LOGIN` — die zwei
Verfahren, die jeder Server kann. OAuth2 (`XOAUTH2`) gibt es **nicht**;
Anbieter, die es verlangen, brauchen stattdessen ein *App-Kennwort*.

## Was hinter den Kulissen passiert

* **Umlaute im Betreff und im Namen** werden nach RFC 2047 kodiert
  (`=?UTF-8?B?…?=`), sonst dürfte dort nur ASCII stehen. Lange Umlautfolgen
  werden dabei entlang der **Zeichen** getrennt, nicht der Bytes.
* **Der Rumpf ist base64**, auch reiner Text. SMTP hat zwei Fallen — Zeilen
  über 998 Zeichen und Zeilen, die mit einem Punkt anfangen — und base64
  schließt beide auf einmal.
* **Ein Zeilenumbruch in Betreff, Name oder Adresse ist ein Fehler.** Er wäre
  eine zusätzliche Kopfzeile; `"Rechnung\r\nBcc: fremd@example.com"` schickt
  sonst eine stille Kopie. Still entfernen wäre die schlechtere Antwort:
  dann verschwindet ein Stück Betreff, ohne dass es jemand merkt.
* **Die Blindkopie steht nur im Umschlag**, nicht in den Kopfzeilen — sonst
  wäre sie keine.
* **Die Art eines Anhangs** kommt aus der Dateiendung, aus derselben Tabelle,
  die auch das Modul [`httpd`](module-httpd.md) benutzt.

`SMTP_MESSAGE$` liefert genau die Zeichen, die sonst über die Leitung gingen.
Das ist der Weg, eine Nachricht zu prüfen, ohne einen Mailserver zu haben —
und der Weg, sie stattdessen in eine Datei zu schreiben.

## Grenzen

* **Nur senden.** Postfächer lesen (POP3/IMAP) gehört nicht dazu.
* **Kein OAuth2**, siehe oben.
* **Keine Zeitzone**: die `Date`-Kopfzeile steht in UTC (`+0000`). Ohne
  Zeitzonen-Datenbank ließe sich die Verschiebung des Rechners nicht
  benennen, und eine falsche Angabe wäre schlimmer als eine ehrliche.
* **Ein Anhang liegt vollständig im Speicher** (und base64-kodiert nochmal
  ein Drittel größer). Für Auswertungen und Berichte ist das kein Thema, für
  ein Videoarchiv schon.
* **`smtp` braucht das Feature `smtp`** (im Standard-Bau enthalten). Eine
  Fassung ohne sagt das beim Aufruf.

## In der nativen Runtime (dhrt)

`rust/drachenhauch_runtime/src/smtp.rs`, Feature `smtp`. TLS über `rustls`
mit `ring` — beides lag über `ureq` (Modul `html`) ohnehin schon im
Abhängigkeitsbaum, das Modul kostet also nichts Neues.

Der Aufbau der Nachricht (`nachricht`) ist von der Übertragung (`senden`)
getrennt; deshalb gibt es `SMTP_MESSAGE$`, und deshalb lässt sich alles
außer dem Netzweg ohne Mailserver prüfen. Antworten werden **ungepuffert**
gelesen: ein Puffer könnte über ein `STARTTLS` hinweg Daten aus der
unverschlüsselten Zeit mitschleppen, und genau daraus besteht eine bekannte
Lücke.

Beispiel: [examples/178_bericht_verschicken.dh](../examples/178_bericht_verschicken.dh)
— baut die Auswertung mit `xlsx` und schickt sie los.
