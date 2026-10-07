# Module `cloud`

Cloud save + leaderboard against a small, self-hostable reference
server (`cloudserver/server.dh`, Drachenhauch + SQLite). Two resources: an arbitrary
save blob per player ID, and named leaderboards (one highscore per
name). HTTP runs natively over `ureq` (feature `http`, standard build).

```basic
IMPORT "cloud"
CLOUD_CONFIGURE("http://localhost:8787", "dein-api-key")
```

`CLOUD_CONFIGURE` must come before any other `CLOUD_*`/`LEADERBOARD_*` call,
otherwise you get a clear error ("CLOUD_CONFIGURE muss zuerst
aufgerufen werden" — CLOUD_CONFIGURE must be called first). Setting up the
server: see `cloudserver/README.md` (quick start, configuration, deployment,
**security model — be sure to read it before you run this publicly**).

## Cloud save

| Function | Effect |
|---|---|
| `CLOUD_CONFIGURE(base_url$, api_key$)` | set the server address + shared secret (once) |
| `CLOUD_SAVE(player_id$, data$)` → BOOLEAN | stores `data$` in full (overwrites a previous state) |
| `CLOUD_LOAD(player_id$)` → STRING | loads the stored string — **empty** if nothing has been saved yet OR on an error |
| `CLOUD_LAST_ERROR$()` → STRING | empty after success; otherwise the last error message (network/auth/server) |

`data$` is an arbitrary string — usually JSON from the `json` module
(`JSON_STRINGIFY`) or assembled by hand. `CLOUD_LOAD` deliberately throws
**no** runtime error on an empty result: "no saved game yet" (new player)
and "server unreachable" both look like an empty string to the caller at
first. You can tell them apart with `CLOUD_LAST_ERROR$()` — empty on a
genuine first visit, filled on a genuine error:

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

| Function | Effect |
|---|---|
| `LEADERBOARD_SUBMIT(board$, name$, score[, best_low])` → BOOLEAN | submit a score; `best_low` = TRUE for "lower is better" (e.g. speedrun times), default FALSE = "higher is better". Returns TRUE if it was a new best |
| `LEADERBOARD_FETCH(board$, n[, ascending])` → ARRAY OF TUPLE | the best `n` entries as `(name$, score)` tuples, sorted (descending by default) |

Per `board$` there is **one** entry per `name$` — submitting again with
`SUBMIT` only overwrites it if the new value is better (otherwise the previous
best is kept). Read the result of `LEADERBOARD_FETCH` like any
`ARRAY OF TUPLE` (index `[0]`/`[1]` or `FOR EACH`):

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

## The reference server

`cloudserver/server.dh` is a small Drachenhauch program of its own
(`dhrt run cloudserver/server.dh`, modules `httpd` + `db`) that you host
yourself — until 2026-09-15 it was a Flask process. No external cloud
service, no account management. A single shared API key secret protects
against random bots, but **not** against a player who extracts the key
embedded in their compiled game. For a small hobby/niche game with "take
your progress between computers" + a leaderboard for bragging rights that is
enough; for a game with serious competitive ranking it is not. Details,
configuration and deployment notes:
[`cloudserver/README.md`](../../cloudserver/README.md).

## Related

- `save` (chapter/module for **local** saved games) — `cloud` is its
  network counterpart, not a replacement: a local save as a fallback (see the
  example above) makes the game robust against "server not reachable right
  now".
- `NUMFMT$` (core built-in, no IMPORT needed) — formats big numbers readably
  (`1234567` → `"1.23M"`), a thematic fit for idle/incremental games with
  cloud save + leaderboard.
- Demo: [`examples/146_cloud_idle.dh`](../../examples/146_cloud_idle.dh).
