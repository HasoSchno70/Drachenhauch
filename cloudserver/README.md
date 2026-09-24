# Drachenhauch Cloud Server

Ein minimaler, selbst hostbarer Server für das Drachenhauch-Modul `cloud`:
Speicherstände in der Cloud ablegen und Highscore-Listen (Leaderboards)
führen. Ein Drachenhauch-Programm + eine SQLite-Datei — kein Account-System,
keine externen Cloud-Dienste, kein Python, nichts, was du nicht selbst
kontrollierst.

## Schnellstart (lokal ausprobieren)

```
set DH_CLOUD_API_KEY=test-schluessel                # Windows (PowerShell: $env:DH_CLOUD_API_KEY="test-schluessel")
dhrt run cloudserver/server.dh
```

Der Server läuft dann auf `http://localhost:8787`. In deinem Drachenhauch-Programm:

```basic
IMPORT "cloud"
CLOUD_CONFIGURE("http://localhost:8787", "test-schluessel")
CLOUD_SAVE("spieler1", !"{\"gold\": 1200}")
PRINT CLOUD_LOAD("spieler1")
```

Bis 2026-09-15 war der Server ein Flask-Programm (`server.py`). Pfade,
Antworten, Fehlercodes und das Datenbankschema sind geblieben — eine
vorhandene `cloud.db` läuft ohne Umbau weiter.

## Konfiguration (Umgebungsvariablen)

| Variable | Standard | Bedeutung |
|---|---|---|
| `DH_CLOUD_API_KEY` | *(leer)* | Geteiltes Secret, das Server und Spiel kennen. **Leer = kein Auth-Schutz** — nur für lokales Testen! |
| `DH_CLOUD_DB` | `cloud.db` neben `server.dh` | Pfad zur SQLite-Datei |
| `DH_CLOUD_MAX_SAVE_BYTES` | `65536` | Maximale Größe eines einzelnen Save-Blobs |
| `DH_CLOUD_HOST` | `0.0.0.0` | Bind-Adresse |
| `DH_CLOUD_PORT` | `8787` | Port (`0` = freien Port wählen; der Server nennt ihn als `PORT=...` auf der Fehlerausgabe) |

Die Namen `GB_CLOUD_*` aus der GameBasic-Zeit gelten weiter, wenn der `DH_`-Name fehlt.

## REST-API

Alle Endpunkte außer `/health` (und der CORS-Vorabfrage `OPTIONS`) erwarten
den Header `X-Api-Key: <dein-secret>`.

| Methode | Pfad | Body | Antwort |
|---|---|---|---|
| GET | `/health` | — | `{"ok": true}` |
| POST | `/save/<player_id>` | `{"data": "<string>"}` | `{"ok": true}` |
| GET | `/save/<player_id>` | — | `{"data": "...", "updated_at": <sekunden>}` oder 404 |
| POST | `/leaderboard/<board>/submit` | `{"name": "...", "score": <zahl>[, "best": "high"\|"low"]}` | `{"ok": true, "updated": bool}` |
| GET | `/leaderboard/<board>/top?n=10&order=desc` | — | `{"entries": [{"name": "...", "score": <zahl>}, ...]}` |

`player_id`, `board` und `name` sind auf `[A-Za-z0-9_.-]{1,128}` begrenzt.
Ein Leaderboard-Eintrag ist **ein Highscore pro Name** — ein erneutes
`submit` überschreibt ihn nur, wenn der neue Wert besser ist (`best: "high"`
= größer gewinnt, Standard; `best: "low"` = kleiner gewinnt, z. B.
Speedrun-Zeiten). `n` wird auf 1..200 begrenzt.

Fehler kommen als JSON `{"error": "..."}`: `unauthorized` (401),
`not_found` (404), `method_not_allowed` (405), `save_too_large` (413, mit
`max_bytes`), sonst 400 mit `invalid_player_id`, `missing_data_field`,
`invalid_board`, `invalid_name`, `invalid_score`, `invalid_best_mode`,
`invalid_n` oder `invalid_order`.

**Zwei Unterschiede zur Flask-Fassung:** `updated_at` ist eine ganze Zahl
(Sekunden seit 1970, wie `ZEIT_JETZT()`) statt einer Kommazahl. Und die
CORS-Vorabfrage (`OPTIONS`) wird vor der Schlüsselprüfung beantwortet — ein
Browser schickt dabei keine eigenen Kopfzeilen mit, die Flask-Fassung wies
sie mit gesetztem Schlüssel ab, und ein Spiel im Browser erreichte den
Server nicht.

## Deployment (Produktion)

Der Server bedient eine Anfrage nach der anderen (`httpd`-Modul, siehe
[`docs/module-httpd.md`](../docs/module-httpd.md)) — für ein Hobby-Spiel
mit einer Handvoll gleichzeitiger Spieler reicht das. Für "immer
erreichbar" einen Reverse-Proxy (nginx/Caddy) mit TLS davorsetzen und den
Prozess per systemd, Windows-Dienst oder Docker am Laufen halten.

## Sicherheitsmodell — bitte lesen

Das ist bewusst **minimal**, nicht enterprise-grade:

- **Ein geteiltes API-Key-Secret**, kein Pro-Spieler-Login. Jeder, der den
  im Spiel eingebetteten Key extrahiert (bei einem kompilierten `.exe` mit
  angehängtem Bytecode ist das machbar), kann fremde Spielstände
  überschreiben und falsche Highscores einreichen. Für ein kleines
  Hobby-/Nischen-Spiel meist ein akzeptables Risiko; für ein Spiel mit
  echtem kompetitivem Leaderboard nicht ausreichend. Verglichen wird der
  Schlüssel in konstanter Zeit (`SECURE_EQUALS`).
- **Keine Rate-Limits** über die reine Payload-Größenbegrenzung hinaus —
  setz bei Bedarf einen Reverse-Proxy mit Rate-Limiting davor.
- **`player_id` ist der Schlüssel, keine Authentifizierung** — wer die
  player_id eines anderen Spielers kennt (oder errät), kann dessen
  Save-Blob lesen/überschreiben. Für mehr Sicherheit einen langen,
  zufälligen `player_id` (z. B. eine beim ersten Start generierte UUID,
  lokal gespeichert) statt eines erratbaren Namens verwenden.
- **HTTPS ist deine Aufgabe** — der Server selbst spricht nur HTTP. Für
  alles außer localhost gehört TLS (Reverse-Proxy) davor, sonst geht der
  API-Key im Klartext übers Netz.

Kurz: reicht locker für "meine Idle-Game-Fans sollen ihren Fortschritt
zwischen Rechnern mitnehmen und sich in einer Bestenliste sehen können" —
nicht gedacht für ein Spiel, bei dem Cheating in einem echten Leaderboard
ein Problem wäre.

## Tests

```
dhrt test tests/pruef/werkzeug_cloudserver.dhtest
```

Startet den Server als eigenen Prozess mit einer frischen SQLite-Datei je
Fall und spricht über HTTP mit ihm — auch über das `cloud`-Modul selbst.
