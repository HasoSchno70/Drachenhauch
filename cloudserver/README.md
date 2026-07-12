# GameBasic Cloud Server

Ein minimaler, selbst hostbarer Server für das GameBasic-Modul `cloud`:
Speicherstände in der Cloud ablegen und Highscore-Listen (Leaderboards)
führen. Ein Flask-Prozess + eine SQLite-Datei — kein Account-System, keine
externen Cloud-Dienste, nichts, was du nicht selbst kontrollierst.

## Schnellstart (lokal ausprobieren)

```
cd cloudserver
python -m venv .venv
.venv\Scripts\pip install -r requirements.txt      # Windows
# source .venv/bin/activate && pip install -r requirements.txt   # Linux/macOS

set GB_CLOUD_API_KEY=test-schluessel                # Windows (PowerShell: $env:GB_CLOUD_API_KEY="test-schluessel")
.venv\Scripts\python server.py
```

Der Server läuft dann auf `http://localhost:8787`. In deinem GameBasic-Programm:

```basic
IMPORT "cloud"
CLOUD_CONFIGURE("http://localhost:8787", "test-schluessel")
CLOUD_SAVE("spieler1", "{\"gold\": 1200}")
PRINT CLOUD_LOAD("spieler1")
```

## Konfiguration (Umgebungsvariablen)

| Variable | Standard | Bedeutung |
|---|---|---|
| `GB_CLOUD_API_KEY` | *(leer)* | Geteiltes Secret, das Server und Spiel kennen. **Leer = kein Auth-Schutz** — nur für lokales Testen! |
| `GB_CLOUD_DB` | `cloud.db` neben `server.py` | Pfad zur SQLite-Datei |
| `GB_CLOUD_MAX_SAVE_BYTES` | `65536` | Maximale Größe eines einzelnen Save-Blobs |
| `GB_CLOUD_HOST` | `0.0.0.0` | Bind-Adresse |
| `GB_CLOUD_PORT` | `8787` | Port |

## REST-API

Alle Endpunkte außer `/health` erwarten den Header `X-Api-Key: <dein-secret>`.

| Methode | Pfad | Body | Antwort |
|---|---|---|---|
| GET | `/health` | — | `{"ok": true}` |
| POST | `/save/<player_id>` | `{"data": "<string>"}` | `{"ok": true}` |
| GET | `/save/<player_id>` | — | `{"data": "...", "updated_at": <unix-zeit>}` oder 404 |
| POST | `/leaderboard/<board>/submit` | `{"name": "...", "score": <zahl>[, "best": "high"\|"low"]}` | `{"ok": true, "updated": bool}` |
| GET | `/leaderboard/<board>/top?n=10&order=desc` | — | `{"entries": [{"name": "...", "score": <zahl>}, ...]}` |

`player_id`, `board` und `name` sind auf `[A-Za-z0-9_.-]{1,128}` begrenzt.
Ein Leaderboard-Eintrag ist **ein Highscore pro Name** — ein erneutes
`submit` überschreibt ihn nur, wenn der neue Wert besser ist (`best: "high"`
= größer gewinnt, Standard; `best: "low"` = kleiner gewinnt, z. B.
Speedrun-Zeiten).

## Deployment (Produktion)

`app.run()` (der Entwicklungs-Server in `server.py`) ist nicht für
Produktionslast gedacht. Für einen echten Server:

```
# waitress ist schon in requirements.txt (funktioniert auch unter Windows)
.venv\Scripts\waitress-serve --host=0.0.0.0 --port=8787 server:app
```

Auf Linux geht auch `gunicorn -w 2 -b 0.0.0.0:8787 server:app` (separat
installieren). Für "immer erreichbar" einen Reverse-Proxy (nginx/Caddy) mit
TLS davorsetzen und den Prozess per systemd/pm2/Docker am Laufen halten.

## Sicherheitsmodell — bitte lesen

Das ist bewusst **minimal**, nicht enterprise-grade:

- **Ein geteiltes API-Key-Secret**, kein Pro-Spieler-Login. Jeder, der den
  im Spiel eingebetteten Key extrahiert (bei einem kompilierten `.exe` mit
  angehängtem Bytecode ist das machbar), kann fremde Spielstände
  überschreiben und falsche Highscores einreichen. Für ein kleines
  Hobby-/Nischen-Spiel meist ein akzeptables Risiko; für ein Spiel mit
  echtem kompetitivem Leaderboard nicht ausreichend.
- **Keine Rate-Limits** über die reine Payload-Größenbegrenzung hinaus —
  setz bei Bedarf einen Reverse-Proxy mit Rate-Limiting davor.
- **`player_id` ist der Schlüssel, keine Authentifizierung** — wer die
  player_id eines anderen Spielers kennt (oder errät), kann dessen
  Save-Blob lesen/überschreiben. Für mehr Sicherheit einen langen,
  zufälligen `player_id` (z. B. eine beim ersten Start generierte UUID,
  lokal gespeichert) statt eines erratbaren Namens verwenden.
- **HTTPS ist deine Aufgabe** — der Flask-Server selbst spricht nur HTTP.
  Für alles außer localhost gehört TLS (Reverse-Proxy) davor, sonst geht
  der API-Key im Klartext übers Netz.

Kurz: reicht locker für "meine Idle-Game-Fans sollen ihren Fortschritt
zwischen Rechnern mitnehmen und sich in einer Bestenliste sehen können" —
nicht gedacht für ein Spiel, bei dem Cheating in einem echten Leaderboard
ein Problem wäre.

## Tests

```
cd cloudserver
python -m venv .venv && .venv\Scripts\pip install -r requirements.txt
.venv\Scripts\python -m pytest test_server.py -v
```

Nutzt Flasks Test-Client (kein echter Netzwerk-Server nötig) + eine
temporäre SQLite-Datei pro Testlauf.
