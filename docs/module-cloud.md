# Modul `cloud`

Cloud-Save + Leaderboard gegen einen kleinen, selbst hostbaren Referenz-
Server (`cloudserver/`, Flask + SQLite). Zwei Ressourcen: ein beliebiger
Save-Blob pro Spieler-ID, und benannte Bestenlisten (ein Highscore pro
Name). HTTP läuft nativ über `ureq` (Feature `http`, Standard-Build).

```basic
IMPORT "cloud"
CLOUD_CONFIGURE("http://localhost:8787", "dein-api-key")
```

`CLOUD_CONFIGURE` muss vor jedem anderen `CLOUD_*`/`LEADERBOARD_*`-Aufruf
stehen, sonst kommt ein klarer Fehler ("CLOUD_CONFIGURE muss zuerst
aufgerufen werden"). Server aufsetzen: siehe `cloudserver/README.md`
(Schnellstart, Konfiguration, Deployment, **Sicherheitsmodell — unbedingt
lesen, bevor du das öffentlich betreibst**).

## Cloud-Save

| Funktion | Wirkung |
|---|---|
| `CLOUD_CONFIGURE(base_url$, api_key$)` | Server-Adresse + geteiltes Secret festlegen (einmalig) |
| `CLOUD_SAVE(player_id$, data$)` → BOOLEAN | Speichert `data$` komplett (überschreibt einen vorherigen Stand) |
| `CLOUD_LOAD(player_id$)` → STRING | Lädt den gespeicherten String — **leer**, wenn noch nichts gespeichert wurde ODER bei einem Fehler |
| `CLOUD_LAST_ERROR$()` → STRING | Leer nach Erfolg; sonst die letzte Fehlermeldung (Netzwerk/Auth/Server) |

`data$` ist ein beliebiger String — üblich ist JSON aus dem `json`-Modul
(`JSON_STRINGIFY`) oder von Hand zusammengebaut. `CLOUD_LOAD` wirft
absichtlich **keinen** Laufzeitfehler bei einem leeren Ergebnis: "noch kein
Spielstand" (neuer Spieler) und "Server nicht erreichbar" sehen für den
Aufrufer beide erstmal wie ein leerer String aus. Unterscheiden lässt sich
das über `CLOUD_LAST_ERROR$()` — leer bei einem echten Erstbesuch, gefüllt
bei einem echten Fehler:

```basic
DIM stand AS STRING
stand = CLOUD_LOAD(spieler_id)
IF stand = "" THEN
    IF CLOUD_LAST_ERROR$() <> "" THEN
        PRINT "Cloud nicht erreichbar -- lokalen Save als Fallback nutzen"
    ELSE
        PRINT "Neuer Spieler, frischer Start"
    END IF
END IF
```

## Leaderboard

| Funktion | Wirkung |
|---|---|
| `LEADERBOARD_SUBMIT(board$, name$, score[, best_low])` → BOOLEAN | Score einreichen; `best_low` = TRUE für "kleiner ist besser" (z. B. Speedrun-Zeiten), Standard FALSE = "größer ist besser". Liefert TRUE, wenn es ein neuer Bestwert war |
| `LEADERBOARD_FETCH(board$, n[, ascending])` → ARRAY OF TUPLE | Die besten `n` Einträge als `(name$, score)`-Tupel, sortiert (Standard absteigend) |

Pro `board$` gibt es **einen** Eintrag je `name$` — ein erneutes `SUBMIT`
überschreibt ihn nur, wenn der neue Wert besser ist (der bisherige Bestwert
bleibt sonst erhalten). Ergebnis von `LEADERBOARD_FETCH` auslesen wie jedes
`ARRAY OF TUPLE` (Index `[0]`/`[1]` oder `FOR EACH`):

```basic
IMPORT "cloud"
CLOUD_CONFIGURE("http://localhost:8787", "dein-api-key")

LEADERBOARD_SUBMIT("highscores", "Anna", 4200.0)

DIM top AS ARRAY OF TUPLE
top = LEADERBOARD_FETCH("highscores", 10)
DIM i AS INTEGER
FOR i = 0 TO LEN(top) - 1
    PRINT (i + 1); ". "; top[i][0]; " -- "; NUMFMT$(top[i][1])
NEXT
```

## Der Referenz-Server

`cloudserver/` ist ein eigener kleiner Flask-Prozess (nicht Teil von
`dhrt`), den du selbst hostest — kein externer Cloud-Dienst, keine
Account-Verwaltung. Ein einziges geteiltes API-Key-Secret schützt vor
zufälligen Bots, aber **nicht** vor einem Spieler, der den in seinem
kompilierten Spiel eingebetteten Key extrahiert. Für ein kleines Hobby-
/Nischenspiel mit "Fortschritt zwischen Rechnern mitnehmen" + einer
Bestenliste zum Angeben reicht das; für ein Spiel mit ernsthaftem
kompetitivem Ranking nicht. Details, Konfiguration und Deployment-Hinweise:
[`cloudserver/README.md`](../cloudserver/README.md).

## Verwandt

- `save` (Kapitel/Modul für **lokale** Speicherstände) — `cloud` ist das
  Netz-Pendant dazu, kein Ersatz: ein lokaler Save als Fallback (siehe
  Beispiel oben) macht das Spiel robust gegen "Server gerade nicht
  erreichbar".
- `NUMFMT$` (core-Builtin, kein IMPORT nötig) — grosse Zahlen lesbar
  formatieren (`1234567` → `"1.23M"`), passt thematisch zu Idle-/
  Incremental-Games mit Cloud-Save + Leaderboard.
- Demo: [`examples/146_cloud_idle.dh`](../examples/146_cloud_idle.dh).
