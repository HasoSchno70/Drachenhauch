"""Tests fuer server.py via Flask-Test-Client (kein echter Netzwerk-Server,
kein echtes dhrt noetig -- prueft nur die REST-API-Semantik)."""
import os
import tempfile

os.environ["GB_CLOUD_API_KEY"] = "test-key"
os.environ["GB_CLOUD_DB"] = os.path.join(tempfile.mkdtemp(), "test_cloud.db")
os.environ["GB_CLOUD_MAX_SAVE_BYTES"] = "64"

import server  # noqa: E402  (Umgebungsvariablen muessen vor dem Import stehen)

import pytest


@pytest.fixture()
def client():
    server.app.config["TESTING"] = True
    with server.app.test_client() as c:
        yield c


AUTH = {"X-Api-Key": "test-key"}


def test_health_no_auth(client):
    r = client.get("/health")
    assert r.status_code == 200
    assert r.get_json()["ok"] is True


def test_save_requires_auth(client):
    r = client.post("/save/p1", json={"data": "x"})
    assert r.status_code == 401


def test_save_roundtrip(client):
    r = client.post("/save/p1", json={"data": '{"gold": 42}'}, headers=AUTH)
    assert r.status_code == 200
    assert r.get_json()["ok"] is True

    r = client.get("/save/p1", headers=AUTH)
    assert r.status_code == 200
    body = r.get_json()
    assert body["data"] == '{"gold": 42}'
    assert "updated_at" in body


def test_save_not_found(client):
    r = client.get("/save/nobody", headers=AUTH)
    assert r.status_code == 404
    assert r.get_json()["error"] == "not_found"


def test_save_overwrite(client):
    client.post("/save/p2", json={"data": "v1"}, headers=AUTH)
    client.post("/save/p2", json={"data": "v2"}, headers=AUTH)
    r = client.get("/save/p2", headers=AUTH)
    assert r.get_json()["data"] == "v2"


def test_save_too_large_rejected(client):
    big = "x" * 1000  # ueber dem in setUp gesetzten Limit von 64 Bytes
    r = client.post("/save/p3", json={"data": big}, headers=AUTH)
    assert r.status_code == 413


def test_save_invalid_player_id_rejected(client):
    r = client.post("/save/../etc", json={"data": "x"}, headers=AUTH)
    # Flask normalisiert den Pfad schon selbst; falls nicht, muss unser
    # Regex-Check greifen -- so oder so kein 200/500.
    assert r.status_code in (400, 404)


def test_leaderboard_submit_and_top(client):
    client.post("/leaderboard/b1/submit", json={"name": "anna", "score": 100}, headers=AUTH)
    client.post("/leaderboard/b1/submit", json={"name": "bert", "score": 250}, headers=AUTH)
    client.post("/leaderboard/b1/submit", json={"name": "carla", "score": 50}, headers=AUTH)

    r = client.get("/leaderboard/b1/top?n=2", headers=AUTH)
    assert r.status_code == 200
    entries = r.get_json()["entries"]
    assert entries == [{"name": "bert", "score": 250.0}, {"name": "anna", "score": 100.0}]


def test_leaderboard_only_keeps_best_score_high(client):
    client.post("/leaderboard/b2/submit", json={"name": "anna", "score": 100}, headers=AUTH)
    r = client.post("/leaderboard/b2/submit", json={"name": "anna", "score": 50}, headers=AUTH)
    assert r.get_json()["updated"] is False  # 50 ist schlechter als 100 -> verworfen

    r = client.post("/leaderboard/b2/submit", json={"name": "anna", "score": 300}, headers=AUTH)
    assert r.get_json()["updated"] is True

    r = client.get("/leaderboard/b2/top", headers=AUTH)
    assert r.get_json()["entries"] == [{"name": "anna", "score": 300.0}]


def test_leaderboard_best_low_mode(client):
    client.post("/leaderboard/speedrun/submit", json={"name": "anna", "score": 61.2, "best": "low"}, headers=AUTH)
    r = client.post("/leaderboard/speedrun/submit", json={"name": "anna", "score": 58.9, "best": "low"}, headers=AUTH)
    assert r.get_json()["updated"] is True

    r = client.get("/leaderboard/speedrun/top?order=asc", headers=AUTH)
    assert r.get_json()["entries"][0]["score"] == 58.9


def test_leaderboard_invalid_score_rejected(client):
    r = client.post("/leaderboard/b3/submit", json={"name": "anna", "score": "viel"}, headers=AUTH)
    assert r.status_code == 400


def test_wrong_api_key_rejected(client):
    r = client.get("/save/p1", headers={"X-Api-Key": "falsch"})
    assert r.status_code == 401
