"""Tests fuer das Qt-freie Form-Designer-Modell: .gbform-Roundtrip, Laden in der
nativen Runtime (GUI_LOAD), Code-Generierung, Palette/Namen."""
import json

from gamebasic.formdesigner import (
    Control, FormDoc, FormProject, History, PALETTE, palette_spec, GRID,
    HANDLES, snap, resize_rect,
)


# --------------------------------------------------------------- Modell
def test_add_and_unique_names():
    doc = FormDoc()
    b1 = doc.add("button", 10, 10)
    b2 = doc.add("button", 10, 50)
    lbl = doc.add("label", 10, 90)
    assert b1.name == "btn1" and b2.name == "btn2" and lbl.name == "lbl1"
    assert b1.kind == "button" and b1.text == "Button"   # has_text -> Default-Text
    assert len(doc.controls) == 3


def test_palette_covers_runtime_kinds():
    kinds = {p.kind for p in PALETTE}
    for k in ("button", "label", "checkbox", "radio", "slider", "textinput",
              "dropdown", "listbox", "progress", "image", "canvas", "panel"):
        assert k in kinds
    assert "on_click" in palette_spec("button").events
    assert "on_change" in palette_spec("dropdown").events


def test_control_at():
    doc = FormDoc()
    a = doc.add("button", 10, 10)        # 100x28
    b = doc.add("button", 10, 10)        # ueberlappt -> spaeter = oben
    assert doc.control_at(20, 20) is b
    assert doc.control_at(500, 500) is None


def test_roundtrip_dict():
    doc = FormDoc(title="Login", w=400, h=300)
    btn = doc.add("button", 20, 200)
    btn.on_click = "on_ok"
    dd = doc.add("dropdown", 20, 40)
    dd.items = ["A", "B", "C"]; dd.sel = 2; dd.on_change = "on_pick"
    cb = doc.add("checkbox", 20, 80); cb.checked = True; cb.enabled = False
    d = doc.to_dict()
    doc2 = FormDoc.from_dict(d)
    assert doc2.title == "Login" and doc2.w == 400
    assert [c.kind for c in doc2.controls] == ["button", "dropdown", "checkbox"]
    assert doc2.controls[0].on_click == "on_ok"
    assert doc2.controls[1].items == ["A", "B", "C"] and doc2.controls[1].sel == 2
    assert doc2.controls[2].checked is True and doc2.controls[2].enabled is False


def test_handler_names_unique_in_order():
    doc = FormDoc()
    doc.add("button", 0, 0).on_click = "a"
    doc.add("checkbox", 0, 30).on_change = "b"
    doc.add("button", 0, 60).on_click = "a"   # Duplikat
    assert doc.handler_names() == ["a", "b"]


# --------------------------------------------------------------- Geometrie
def test_snap_to_grid():
    assert GRID == 8
    assert snap(0) == 0
    assert snap(3) == 0                  # naeher an 0
    assert snap(4) == 8                  # Mittelpunkt -> auf (round-half-up)
    assert snap(11) == 8 and snap(12) == 16
    assert snap(100) == 104             # 12.5*8 -> 13*8
    assert snap(5, grid=1) == 5         # grid<=1 -> passthrough


def test_resize_east_grows_width_only():
    # Ost-Griff: nur Breite, x fix.
    assert resize_rect(10, 20, 100, 40, "e", 200, 999) == (10, 20, 190, 40)


def test_resize_west_moves_x_keeps_right_edge():
    # West-Griff: rechte Kante (x+w=110) bleibt fix.
    x, y, w, h = resize_rect(10, 20, 100, 40, "w", 30, 999)
    assert (x, w) == (30, 80) and x + w == 110
    assert (y, h) == (20, 40)            # vertikal unberuehrt


def test_resize_corner_se():
    assert resize_rect(0, 0, 50, 50, "se", 80, 90) == (0, 0, 80, 90)


def test_resize_respects_min_size():
    # Ueber die gegenueberliegende Kante hinaus -> auf Mindestgroesse geklemmt.
    x, y, w, h = resize_rect(10, 10, 100, 100, "nw", 999, 999, min_w=8, min_h=8)
    assert w == 8 and h == 8
    assert x == 110 - 8 and y == 110 - 8   # rechte/untere Kante bleiben fix


def test_handles_cover_eight_directions():
    assert set(HANDLES) == {"nw", "n", "ne", "e", "se", "s", "sw", "w"}


# --------------------------------------------------------------- Undo/Redo
def _snap_doc(*kinds):
    d = FormDoc()
    for k in kinds:
        d.add(k, 0, 0)
    return d


def test_history_empty_state():
    h = History()
    assert not h.can_undo and not h.can_redo


def test_history_undo_redo_roundtrip():
    h = History()
    s0 = FormDoc().to_dict()                       # leeres Formular
    s1 = _snap_doc("button").to_dict()             # 1 Control
    h.push(s0)                                      # Checkpoint vor dem Add
    assert h.can_undo and not h.can_redo
    # Undo: aktueller Zustand (s1) wandert auf Redo, s0 kommt zurueck.
    restored = h.undo(s1)
    assert restored == s0
    assert not h.can_undo and h.can_redo
    # Redo: s1 wieder her.
    again = h.redo(s0)
    assert again == s1
    assert h.can_undo and not h.can_redo


def test_history_push_clears_redo():
    h = History()
    h.push({"a": 1})
    h.undo({"a": 2})                               # jetzt liegt was auf Redo
    assert h.can_redo
    h.push({"a": 3})                               # neue Mutation killt Redo
    assert not h.can_redo


def test_history_limit_drops_oldest():
    h = History(limit=3)
    for i in range(5):
        h.push({"i": i})
    # Nur die letzten 3 bleiben -> 3x Undo moeglich, danach leer.
    cur = {"i": 99}
    seen = []
    while h.can_undo:
        cur = h.undo(cur)
        seen.append(cur["i"])
    assert seen == [4, 3, 2]                        # 0 und 1 wurden verworfen


# --------------------------------------------------------------- .gbform IO
def test_save_load_file(tmp_path):
    doc = FormDoc(title="T")
    doc.add("button", 10, 10).on_click = "go"
    p = tmp_path / "x.gbform"
    doc.save(str(p))
    raw = json.loads(p.read_text(encoding="utf-8"))
    assert raw["widgets"][0]["kind"] == "button"
    assert raw["widgets"][0]["on_click"] == "go"
    assert raw["widgets"][0]["name"] == "btn1"     # Designer-Metadaten
    doc2 = FormDoc.load(str(p))
    assert doc2.controls[0].name == "btn1"


def test_gbform_loads_in_runtime(run_gb, tmp_path):
    # Das vom Designer geschriebene .gbform muss GUI_LOAD direkt verstehen.
    doc = FormDoc(title="RT", x=50, y=40, w=320, h=200)
    doc.add("label", 20, 20)
    dd = doc.add("dropdown", 20, 60); dd.items = ["Rot", "Gruen", "Blau"]; dd.sel = 1
    cb = doc.add("checkbox", 20, 100); cb.checked = True
    doc.save(str(tmp_path / "f.gbform"))
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("f.gbform")\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(frm)\n'
        'PRINT GUI_DROPDOWN_TEXT(GUI_WINDOW_WIDGET(frm, 1))\n'
        'PRINT GUI_CHECKED(GUI_WINDOW_WIDGET(frm, 2))\n',
        base=tmp_path)
    assert out.splitlines() == ["3", "Gruen", "TRUE"]


def test_gbform_with_code_loads_in_runtime(run_gb, tmp_path):
    # `code` (Handler-Koerper) ist Designer-Metadaten -- GUI_LOAD muss es ignorieren.
    doc = FormDoc(title="C", w=200, h=120)
    doc.add("button", 10, 10).on_click = "on_ok"
    doc.code["on_ok"] = 'PRINT "x"'
    doc.save(str(tmp_path / "c.gbform"))
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("c.gbform")\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(frm)\n',
        base=tmp_path)
    assert out.splitlines() == ["1"]


# --------------------------------------------------------------- FormProject
def test_project_add_sets_first_as_main():
    p = FormProject()
    p.add("main.gbform")
    p.add("settings.gbform")
    assert p.forms == ["main.gbform", "settings.gbform"]
    assert p.main == "main.gbform"        # erstes wird Startformular


def test_project_add_dedup():
    p = FormProject()
    p.add("a.gbform"); p.add("a.gbform")
    assert p.forms == ["a.gbform"]


def test_project_remove_repoints_main():
    p = FormProject()
    p.add("a.gbform"); p.add("b.gbform")
    p.remove("a.gbform")                  # war main -> faellt auf b
    assert p.forms == ["b.gbform"] and p.main == "b.gbform"
    p.remove("b.gbform")
    assert p.forms == [] and p.main == ""


def test_project_roundtrip(tmp_path):
    p = FormProject(forms=["a.gbform", "b.gbform"], main="b.gbform")
    fp = tmp_path / "proj.gbproj"
    p.save(str(fp))
    q = FormProject.load(str(fp))
    assert q.forms == ["a.gbform", "b.gbform"] and q.main == "b.gbform"


def test_project_from_dict_fixes_dangling_main():
    p = FormProject.from_dict({"forms": ["a.gbform"], "main": "ghost.gbform"})
    assert p.main == "a.gbform"           # main muss Mitglied sein
    empty = FormProject.from_dict({"forms": [], "main": "x"})
    assert empty.main == ""


# --------------------------------------------------------------- Handler/Code
def test_primary_event():
    doc = FormDoc()
    assert doc.primary_event(doc.add("button", 0, 0)) == "on_click"
    assert doc.primary_event(doc.add("slider", 0, 0)) == "on_change"
    assert doc.primary_event(doc.add("label", 0, 0)) is None    # kein Event


def test_ensure_handler_generates_name_and_code_entry():
    doc = FormDoc()
    b = doc.add("button", 0, 0)        # name btn1
    name = doc.ensure_handler(b)
    assert name == "btn1Click"
    assert b.on_click == "btn1Click"
    assert doc.code["btn1Click"] == ""   # leerer Koerper angelegt


def test_ensure_handler_keeps_existing_name():
    doc = FormDoc()
    b = doc.add("button", 0, 0)
    b.on_click = "on_save"
    name = doc.ensure_handler(b)
    assert name == "on_save" and "on_save" in doc.code


def test_ensure_handler_unique_names():
    doc = FormDoc()
    a = doc.add("button", 0, 0)        # btn1 -> btn1Click
    b = doc.add("button", 0, 30)       # btn2 -> btn2Click
    assert doc.ensure_handler(a) == "btn1Click"
    assert doc.ensure_handler(b) == "btn2Click"
    # Kollision erzwingen: zweites Control bekommt denselben Basisnamen
    c = doc.add("button", 0, 60)
    c.name = "btn1"
    assert doc.ensure_handler(c) == "btn1Click2"


def test_ensure_handler_none_for_eventless():
    doc = FormDoc()
    assert doc.ensure_handler(doc.add("label", 0, 0)) is None


def test_code_roundtrip():
    doc = FormDoc()
    b = doc.add("button", 0, 0); b.on_click = "on_ok"
    doc.code["on_ok"] = 'PRINT "ok"'
    d = doc.to_dict()
    assert d["code"] == {"on_ok": 'PRINT "ok"'}
    doc2 = FormDoc.from_dict(d)
    assert doc2.code["on_ok"] == 'PRINT "ok"'


def test_empty_code_not_serialized():
    doc = FormDoc()
    assert "code" not in doc.to_dict()


def test_generate_runner_uses_stored_code():
    doc = FormDoc()
    doc.add("button", 0, 0).on_click = "on_ok"
    doc.code["on_ok"] = 'PRINT "stored"'
    src = doc.generate_runner("f.gbform")
    assert 'PRINT "stored"' in src and "SUB on_ok()" in src


# --------------------------------------------------------------- Codegen
def test_generated_runner_parses():
    from gamebasic.lexer import Lexer
    from gamebasic.parser import Parser
    from gamebasic.preprocess import process

    doc = FormDoc(title="App")
    doc.add("button", 20, 200).on_click = "on_save"
    doc.add("slider", 20, 40).on_change = "on_vol"
    src = doc.generate_runner("forms/app.gbform", screen_title="App")
    assert 'GUI_LOAD("forms/app.gbform")' in src
    assert "SUB on_save()" in src and "SUB on_vol()" in src
    # Muss sauber durch Preprocess + Lexer + Parser laufen.
    merged = process(src)
    if isinstance(merged, tuple):
        merged = merged[0]
    prog = Parser(Lexer(merged).tokenize()).parse()
    assert prog is not None


def test_generated_runner_with_bodies():
    doc = FormDoc()
    doc.add("button", 0, 0).on_click = "on_ok"
    src = doc.generate_runner("f.gbform", handler_bodies={"on_ok": 'PRINT "hi"'})
    assert 'PRINT "hi"' in src
