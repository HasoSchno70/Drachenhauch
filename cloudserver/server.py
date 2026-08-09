"""Minimaler Cloud-Save- + Leaderboard-Server fuer Drachenhauch (Modul `cloud`).

Ein einzelner Flask-Prozess + eine SQLite-Datei. Zwei Ressourcen:

  - Save-Blobs:   ein beliebiger String pro player_id (z.B. JSON aus dem
                  json-Modul), komplett ueberschrieben bei jedem Speichern.
  - Leaderboards: benannte Bestenlisten (board), ein Highscore pro Name.

Auth ist ein einziges geteiltes API-Key-Secret (Header `X-Api-Key`), das
Server und Spiel gemeinsam kennen -- kein Pro-Spieler-Login. Das reicht, um
zufaellige Bots draussen zu halten, schuetzt aber NICHT vor einem Spieler,
der seinen eigenen (im Spiel eingebetteten) Key extrahiert und damit fremde
Saves ueberschreibt oder falsche Scores einreicht. Siehe README.md
"Sicherheitsmodell" fuer Details und Haerte-Optionen.

Start (Entwicklung):
    pip install -r requirements.txt
    set GB_CLOUD_API_KEY=dein-geheimes-schluesselwort
    python server.py

Start (Produktion): siehe README.md (waitress/gunicorn statt app.run()).
"""
from __future__ import annotations

import hmac
import os
import re
import sqlite3
import time
from contextlib import closing
from pathlib import Path

from flask import Flask, g, jsonify, request

app = Flask(__name__)

DB_PATH = os.environ.get("GB_CLOUD_DB", str(Path(__file__).parent / "cloud.db"))
API_KEY = os.environ.get("GB_CLOUD_API_KEY", "")
MAX_SAVE_BYTES = int(os.environ.get("GB_CLOUD_MAX_SAVE_BYTES", "65536"))
MAX_LEADERBOARD_N = 200

# player_id / board / name: konservativ begrenzt (keine Pfad-Sonderzeichen
# noetig -- landen nur als SQL-Parameter, aber eine Laengen-/Zeichen-Grenze
# haelt kaputte/absichtlich riesige Requests fern).
_ID_RE = re.compile(r"^[A-Za-z0-9_\-\.]{1,128}$")


def _valid_id(s: str) -> bool:
    return bool(_ID_RE.match(s))


def get_db() -> sqlite3.Connection:
    if "db" not in g:
        g.db = sqlite3.connect(DB_PATH)
        g.db.row_factory = sqlite3.Row
    return g.db


@app.teardown_appcontext
def close_db(_exc) -> None:
    db = g.pop("db", None)
    if db is not None:
        db.close()


def init_db() -> None:
    with closing(sqlite3.connect(DB_PATH)) as db:
        db.execute(
            """CREATE TABLE IF NOT EXISTS saves (
                player_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                updated_at REAL NOT NULL
            )"""
        )
        db.execute(
            """CREATE TABLE IF NOT EXISTS scores (
                board TEXT NOT NULL,
                name TEXT NOT NULL,
                score REAL NOT NULL,
                updated_at REAL NOT NULL,
                PRIMARY KEY (board, name)
            )"""
        )
        db.execute("CREATE INDEX IF NOT EXISTS idx_scores_board_score ON scores(board, score DESC)")
        db.commit()


def require_api_key():
    if not API_KEY:
        # Kein Key konfiguriert -> Server bewusst offen (nur fuer lokales
        # Ausprobieren gedacht). Deutlich im README als unsicher markiert.
        return None
    supplied = request.headers.get("X-Api-Key", "")
    if not hmac.compare_digest(supplied, API_KEY):
        return jsonify(error="unauthorized"), 401
    return None


@app.before_request
def _auth_gate():
    if request.path == "/health":
        return None
    return require_api_key()


@app.after_request
def _cors(resp):
    # Permissiv, damit z.B. ein dhrt-WASM-Build im Browser den Server
    # erreichen kann. Fuer einen oeffentlichen Server ggf. auf die eigene
    # Domain einschraenken.
    resp.headers["Access-Control-Allow-Origin"] = "*"
    resp.headers["Access-Control-Allow-Headers"] = "X-Api-Key, Content-Type"
    resp.headers["Access-Control-Allow-Methods"] = "GET, POST, OPTIONS"
    return resp


@app.route("/health", methods=["GET"])
def health():
    return jsonify(ok=True)


# --- Save-Blobs --------------------------------------------------------------

@app.route("/save/<player_id>", methods=["POST"])
def save_upload(player_id: str):
    if not _valid_id(player_id):
        return jsonify(error="invalid_player_id"), 400
    body = request.get_json(silent=True) or {}
    data = body.get("data")
    if not isinstance(data, str):
        return jsonify(error="missing_data_field"), 400
    if len(data.encode("utf-8")) > MAX_SAVE_BYTES:
        return jsonify(error="save_too_large", max_bytes=MAX_SAVE_BYTES), 413
    db = get_db()
    db.execute(
        "INSERT INTO saves(player_id, data, updated_at) VALUES (?, ?, ?) "
        "ON CONFLICT(player_id) DO UPDATE SET data=excluded.data, updated_at=excluded.updated_at",
        (player_id, data, time.time()),
    )
    db.commit()
    return jsonify(ok=True)


@app.route("/save/<player_id>", methods=["GET"])
def save_download(player_id: str):
    if not _valid_id(player_id):
        return jsonify(error="invalid_player_id"), 400
    row = get_db().execute(
        "SELECT data, updated_at FROM saves WHERE player_id = ?", (player_id,)
    ).fetchone()
    if row is None:
        return jsonify(error="not_found"), 404
    return jsonify(data=row["data"], updated_at=row["updated_at"])


# --- Leaderboards --------------------------------------------------------------

@app.route("/leaderboard/<board>/submit", methods=["POST"])
def leaderboard_submit(board: str):
    if not _valid_id(board):
        return jsonify(error="invalid_board"), 400
    body = request.get_json(silent=True) or {}
    name = body.get("name")
    score = body.get("score")
    if not isinstance(name, str) or not _valid_id(name):
        return jsonify(error="invalid_name"), 400
    if not isinstance(score, (int, float)) or isinstance(score, bool):
        return jsonify(error="invalid_score"), 400
    # "high" (Standard) = groesser ist besser, "low" = kleiner ist besser
    # (z.B. Speedrun-Zeiten). Nur beim ERSTEN Eintrag eines Namens relevant --
    # danach bleibt der Bestwert erhalten, unabhaengig vom Modus.
    best_mode = body.get("best", "high")
    if best_mode not in ("high", "low"):
        return jsonify(error="invalid_best_mode"), 400
    db = get_db()
    existing = db.execute(
        "SELECT score FROM scores WHERE board = ? AND name = ?", (board, name)
    ).fetchone()
    if existing is None:
        db.execute(
            "INSERT INTO scores(board, name, score, updated_at) VALUES (?, ?, ?, ?)",
            (board, name, float(score), time.time()),
        )
        db.commit()
        return jsonify(ok=True, updated=True)
    is_better = score > existing["score"] if best_mode == "high" else score < existing["score"]
    if is_better:
        db.execute(
            "UPDATE scores SET score = ?, updated_at = ? WHERE board = ? AND name = ?",
            (float(score), time.time(), board, name),
        )
        db.commit()
    return jsonify(ok=True, updated=is_better)


@app.route("/leaderboard/<board>/top", methods=["GET"])
def leaderboard_top(board: str):
    if not _valid_id(board):
        return jsonify(error="invalid_board"), 400
    try:
        n = int(request.args.get("n", "10"))
    except ValueError:
        return jsonify(error="invalid_n"), 400
    n = max(1, min(n, MAX_LEADERBOARD_N))
    order = request.args.get("order", "desc")
    if order not in ("asc", "desc"):
        return jsonify(error="invalid_order"), 400
    rows = get_db().execute(
        f"SELECT name, score FROM scores WHERE board = ? ORDER BY score {order.upper()} LIMIT ?",
        (board, n),
    ).fetchall()
    entries = [{"name": r["name"], "score": r["score"]} for r in rows]
    return jsonify(entries=entries)


init_db()

if __name__ == "__main__":
    if not API_KEY:
        print("WARNUNG: GB_CLOUD_API_KEY nicht gesetzt -- Server laeuft OHNE Auth (nur zum lokalen Testen!)")
    host = os.environ.get("GB_CLOUD_HOST", "0.0.0.0")
    port = int(os.environ.get("GB_CLOUD_PORT", "8787"))
    app.run(host=host, port=port)
