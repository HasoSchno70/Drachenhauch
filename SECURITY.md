# Security / Sicherheit

## Supported versions

Only the latest release ([releases](https://github.com/HasoSchno70/Drachenhauch/releases/latest))
and the current `main` branch receive fixes.

## Reporting a vulnerability

Please do **not** open a public issue. Use GitHub's private reporting instead:
on the [Security tab](https://github.com/HasoSchno70/Drachenhauch/security) click
**Report a vulnerability**. Describe what happens, which version (`dhrt --version`)
and system you used, and if possible a small program that shows it.

You will get an answer as soon as possible. Once a fix is released, the report
can be made public, with credit to you if you like.

Areas where a flaw matters most: the network modules (`httpd`, `websocket`,
`mqtt`, `net`), reading files from outside (`zip`, `json`, `xml`, `csv`, `db`),
`PROCESS_*`, and the foreign function interface (`DECLARE … LIB`). Note that
`DECLARE … LIB` and `PROCESS_*` intentionally give a program full access to the
system; a Drachenhauch program is as trustworthy as any other program you run.

---

## Unterstützte Fassungen

Fehlerbehebungen gibt es nur für die neueste Fassung und den aktuellen
Zweig `main`.

## Eine Lücke melden

Bitte **kein** öffentliches Issue. Stattdessen die private Meldung von GitHub:
im [Reiter Security](https://github.com/HasoSchno70/Drachenhauch/security) auf
**Report a vulnerability** klicken. Beschreiben, was passiert, mit welcher
Fassung (`dhrt --version`) und welchem System, und wenn möglich ein kleines
Programm mitschicken, das es zeigt.

Eine Antwort kommt so bald wie möglich. Ist die Behebung erschienen, darf
die Meldung öffentlich werden, auf Wunsch mit Nennung.
