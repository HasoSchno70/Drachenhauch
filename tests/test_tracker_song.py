"""Tests fuer das Tracker-Song-Modell (Qt-frei).

Deckt Pattern/Order-Ops, JSON-Roundtrip und den GB-Code-Export ab
(Flatten + Kompilierbarkeit des erzeugten Programms)."""
from gamebasic.tracker import (
    CHANNELS, TONAL, Pattern, Song, midi_to_freq, note_name,
)


# --------------------------------------------------------------- Pattern

def test_new_pattern_empty():
    p = Pattern("P1", 16)
    assert p.rows == 16
    assert len(p.data) == CHANNELS
    assert all(v is None for col in p.data for v in col)


def test_pattern_set_get():
    p = Pattern("P1", 8)
    p.set(0, 3, 60)
    assert p.get(0, 3) == 60


def test_pattern_set_rows_grow_and_shrink():
    p = Pattern("P1", 4)
    p.set(0, 0, 60)
    p.set(1, 3, 62)
    p.set_rows(8)
    assert p.rows == 8
    assert p.get(0, 0) == 60 and p.get(1, 3) == 62
    assert p.get(0, 7) is None
    p.set_rows(2)                      # schrumpfen -> Reihe 3 faellt weg
    assert p.rows == 2
    assert p.get(0, 0) == 60
    assert all(len(col) == 2 for col in p.data)


def test_pattern_copy_is_independent():
    p = Pattern("P1", 4); p.set(0, 0, 60)
    q = p.copy("P2")
    q.set(0, 0, 72)
    assert p.get(0, 0) == 60 and q.get(0, 0) == 72
    assert q.name == "P2"


def test_note_helpers():
    assert note_name(60) == "C4"
    assert round(midi_to_freq(69)) == 440


# --------------------------------------------------------------- Song-Ops

def test_song_defaults():
    s = Song()
    assert s.bpm == 120
    assert len(s.patterns) == 1
    assert s.order == [0]
    assert len(s.waves) == TONAL


def test_add_and_duplicate_pattern():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    i = s.add_pattern("Bridge", 8)
    assert i == 1 and s.patterns[1].rows == 8
    j = s.duplicate_pattern(0)
    assert j == 2 and s.patterns[2].get(0, 0) == 60


def test_remove_pattern_fixes_order():
    s = Song()
    s.add_pattern(); s.add_pattern()      # patterns 0,1,2
    s.order = [0, 1, 2, 1]
    s.remove_pattern(1)                    # pattern 1 weg
    # Verweise auf 1 raus, 2 -> 1
    assert s.order == [0, 1]
    assert len(s.patterns) == 2


def test_remove_last_pattern_noop():
    s = Song()
    s.remove_pattern(0)
    assert len(s.patterns) == 1


def test_remove_pattern_empty_order_falls_back():
    s = Song()
    s.add_pattern()                       # 0,1
    s.order = [1]
    s.remove_pattern(1)
    assert s.order == [0]


def test_order_ops():
    s = Song()
    s.add_pattern()
    s.order_add(1)
    assert s.order == [0, 1]
    j = s.order_move(0, 1)
    assert j == 1 and s.order == [1, 0]
    s.order_remove(0)
    assert s.order == [0]
    s.order_remove(0)                     # letzter bleibt
    assert s.order == [0]


def test_row_ms():
    s = Song(); s.bpm = 120
    assert s.row_ms() == 125              # 60000/120/4


# --------------------------------------------------------------- Flatten

def test_flatten_concatenates_order():
    s = Song()
    s.patterns[0].set_rows(4)
    s.patterns[0].set(0, 0, 69)           # A4 = 440 Hz
    s.patterns[0].set(TONAL, 1, 60)       # Drum-Hit -> 1
    i = s.add_pattern("P2", 2)
    s.patterns[i].set(1, 0, 69)
    s.order = [0, 1, 0]
    total, ch = s.flatten()
    assert total == 4 + 2 + 4
    assert ch[0][0] == 440                # erste Reihe von P0, Kanal 0
    assert ch[TONAL][1] == 1              # Drum-Hit
    assert ch[1][4] == 440               # P2 beginnt bei Reihe 4, Kanal 1
    assert ch[0][6] == 440               # zweite P0-Instanz ab Reihe 6


def test_flatten_empty_song_min_one_row():
    s = Song()
    total, ch = s.flatten()
    assert total == 16                    # ein leeres 16-Reihen-Pattern
    assert all(v == 0 for v in ch[0])


# --------------------------------------------------------------- JSON

def test_json_roundtrip(tmp_path):
    s = Song()
    s.bpm = 140
    s.waves = ["sine", "triangle", "square"]
    s.patterns[0].set(0, 0, 60)
    s.add_pattern("Chorus", 8)
    s.patterns[1].set(2, 3, 64)
    s.order = [0, 1, 1, 0]
    path = str(tmp_path / "song.json")
    s.save_json(path)

    s2 = Song.load_json(path)
    assert s2.bpm == 140
    assert s2.waves == ["sine", "triangle", "square"]
    assert len(s2.patterns) == 2
    assert s2.patterns[0].get(0, 0) == 60
    assert s2.patterns[1].rows == 8 and s2.patterns[1].get(2, 3) == 64
    assert s2.order == [0, 1, 1, 0]


def test_from_dict_filters_bad_order_indices():
    s = Song.from_dict({"bpm": 100, "patterns": [Pattern("A").to_dict()],
                        "order": [0, 5, -1, 0]})
    assert s.order == [0, 0]              # 5 und -1 fallen raus


# --------------------------------------------------------------- GB-Code

def test_gb_code_compiles(tmp_path):
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.preprocess import process

    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set(TONAL, 4, 50)
    s.add_pattern("P2", 8)
    s.patterns[1].set(1, 2, 64)
    s.order = [0, 1, 0]
    code = s.gb_code()
    prepped, _ = process(code, tmp_path, file_label="<tracker>")
    ast = Parser(Lexer(prepped).tokenize()).parse()
    Compiler().compile(ast)               # wirft bei Fehler


def test_gb_code_has_expanded_rows():
    s = Song()
    s.patterns[0].set_rows(4)
    s.add_pattern("P2", 4)
    s.order = [0, 1, 0]                    # 12 Reihen total
    code = s.gb_code()
    assert "CONST TRK_ROWS = 12" in code


# --------------------------------------------------------------- Lautstaerke

def test_set_vol_requires_note():
    from gamebasic.tracker import VOL_MAX
    p = Pattern("P")
    # ohne Note ignoriert set_vol
    p.set_vol(0, 0, 8)
    assert p.get_vol(0, 0) is None
    # mit Note wird gesetzt
    p.set(0, 0, 60)
    p.set_vol(0, 0, 8)
    assert p.get_vol(0, 0) == 8
    # Clamping
    p.set_vol(0, 0, 999)
    assert p.get_vol(0, 0) == VOL_MAX
    # 0/None -> Standard (None)
    p.set_vol(0, 0, 0)
    assert p.get_vol(0, 0) is None


def test_clearing_note_clears_vol():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_vol(0, 0, 10)
    p.set(0, 0, None)                     # Note loeschen
    assert p.get_vol(0, 0) is None


def test_set_rows_keeps_vol():
    p = Pattern("P", 8)
    p.set(0, 2, 60)
    p.set_vol(0, 2, 5)
    p.set_rows(4)                         # 2 < 4 -> bleibt
    assert p.get_vol(0, 2) == 5
    p.set_rows(2)                         # 2 >= 2 -> Reihe 2 faellt weg
    assert p.rows == 2


def test_to_dict_omits_empty_vol():
    p = Pattern("P")
    assert "vol" not in p.to_dict()
    p.set(0, 0, 60)
    p.set_vol(0, 0, 7)
    assert "vol" in p.to_dict()


def test_vol_json_roundtrip(tmp_path):
    s = Song()
    s.patterns[0].set(1, 3, 64)
    s.patterns[0].set_vol(1, 3, 12)
    path = str(tmp_path / "vol.json")
    s.save_json(path)
    s2 = Song.load_json(path)
    assert s2.patterns[0].get_vol(1, 3) == 12


def test_vol_copy_independent():
    p = Pattern("P")
    p.set(0, 0, 60)
    p.set_vol(0, 0, 9)
    q = p.copy()
    q.set_vol(0, 0, 3)
    assert p.get_vol(0, 0) == 9          # Original unveraendert


def test_vol_to_pct_mapping():
    from gamebasic.tracker import VOL_MAX, vol_to_pct
    assert vol_to_pct(VOL_MAX) == 100
    assert vol_to_pct(1) >= 1            # nie 0 (reserviert fuer Standard)


def test_gb_code_with_volume_compiles(tmp_path):
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.compiler import Compiler
    from gamebasic.preprocess import process

    s = Song()
    s.patterns[0].set(0, 0, 60)
    s.patterns[0].set_vol(0, 0, 12)
    code = s.gb_code()
    assert "DIM trkV0[TRK_ROWS]" in code
    assert "FUNCTION TRACKER_AMP" in code
    prepped, _ = process(code, tmp_path, file_label="<tracker>")
    ast = Parser(Lexer(prepped).tokenize()).parse()
    Compiler().compile(ast)              # wirft bei Fehler


def test_gb_code_without_volume_has_no_amp_helper():
    s = Song()
    s.patterns[0].set(0, 0, 60)
    code = s.gb_code()
    assert "TRACKER_AMP" not in code     # ohne Lautstaerke kein Helfer/Overhead
