"""Tests fuer den Fuzzy-Match-Scorer (Quick-Open + Command-Palette)."""
from drachenhauch.editor_qt.fuzzy import score


def test_empty_query_returns_zero():
    assert score("", "anything") == 0


def test_no_match_returns_none():
    assert score("xyz", "abc") is None


def test_query_longer_than_label_returns_none():
    assert score("hello", "hi") is None


def test_exact_match_scores_high():
    s = score("save", "save")
    assert s is not None and s > 0


def test_prefix_match_beats_middle_match():
    s_prefix = score("save", "save_as")
    s_middle = score("save", "abcsavexyz")
    assert s_prefix is not None and s_middle is not None
    assert s_prefix > s_middle


def test_consecutive_chars_score_higher():
    s_consecutive = score("abc", "abcdef")
    s_scattered = score("abc", "axbxcx")
    assert s_consecutive is not None and s_scattered is not None
    assert s_consecutive > s_scattered


def test_word_boundary_bonus():
    """Match nach Separator (`_`/` `/`/`) ist bevorzugt."""
    s_word = score("nt", "open_tab")          # `t` direkt nach `_`
    s_word_alt = score("ot", "open_tab")      # `o` am Anfang, `t` nach `_`
    s_no_word = score("ot", "boot_to")        # `o`,`t` keine Word-Boundaries
    assert s_word is not None and s_word_alt is not None and s_no_word is not None
    # Word-Boundary-Variante schlaegt das in-Wort-Match.
    assert s_word_alt >= s_no_word


def test_case_insensitive():
    s_upper = score("ABC", "abc")
    s_lower = score("abc", "ABC")
    assert s_upper is not None and s_lower is not None
    assert s_upper == s_lower


def test_short_path_beats_long_path():
    """Bei gleichem Match-Pattern gewinnt das kuerzere Label."""
    short = score("tetris", "tetris")
    long = score("tetris", "tetris_legacy_v2_archive")
    assert short is not None and long is not None
    assert short > long
