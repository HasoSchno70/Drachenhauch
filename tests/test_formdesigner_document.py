"""Tests fuer das Qt-freie Form-Designer-Modell: .gbform-Roundtrip, Laden in der
nativen Runtime (GUI_LOAD), Code-Generierung, Palette/Namen."""
import json

from drachenhauch.formdesigner import (
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


def test_resizable_window_roundtrips_in_runtime(run_gb, tmp_path):
    # FormDoc.resizable + min/max muessen durch GUI_LOAD -> GUI_TO_JSON ueberleben.
    # (min_w/min_h sind eindeutige Zahlen -> kein Quote-Escaping noetig.)
    doc = FormDoc(title="R", w=400, h=320)
    doc.resizable = True
    doc.min_w, doc.min_h = 211, 153
    doc.save(str(tmp_path / "r.gbform"))
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("r.gbform")\n'
        'DIM j AS STRING\nj = GUI_TO_JSON(frm)\n'
        'PRINT INSTR(j, "resizable") > 0\n'         # Feld vorhanden
        'PRINT INSTR(j, "211") > 0\n'               # min_w
        'PRINT INSTR(j, "153") > 0\n',              # min_h
        base=tmp_path)
    assert out.splitlines() == ["TRUE", "TRUE", "TRUE"]


def test_anchoring_reflows_in_runtime(run_gb, tmp_path):
    # Anchoring: beim Vergroessern des Fensters wandern/wachsen Controls je
    # nach Anker. Form 400x300 -> 500x400 (dx=100, dy=100).
    doc = FormDoc(title="A", w=400, h=300)
    doc.add("button", 10, 10)                       # idx0: Default "lt" -> bleibt
    b = doc.add("button", 300, 10); b.anchor = "rt" # idx1: rechts -> x += dx
    c = doc.add("button", 10, 260); c.w = 200; c.anchor = "lrtb"  # idx2: dehnt sich
    doc.save(str(tmp_path / "a.gbform"))
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("a.gbform")\n'
        'GUI_WINDOW_SET_BOUNDS(frm, 0, 0, 500, 400)\n'
        'PRINT GUI_GET_X(GUI_WINDOW_WIDGET(frm, 0))\n'   # lt: 10
        'PRINT GUI_GET_X(GUI_WINDOW_WIDGET(frm, 1))\n'   # rt: 300+100 = 400
        'PRINT GUI_GET_W(GUI_WINDOW_WIDGET(frm, 2))\n'   # lrtb breite: 200+100 = 300
        'PRINT GUI_GET_H(GUI_WINDOW_WIDGET(frm, 2))\n',  # lrtb hoehe: 28+100 = 128
        base=tmp_path)
    assert out.splitlines() == ["10", "400", "300", "128"]


def test_anchor_roundtrip_dict():
    doc = FormDoc()
    c = doc.add("button", 0, 0); c.anchor = "lrtb"
    d = doc.to_dict()["widgets"][0]
    assert d["anchor"] == "lrtb"
    assert "anchor" not in FormDoc().add("button", 0, 0).to_dict()   # Default weggelassen
    assert FormDoc.from_dict(doc.to_dict()).controls[0].anchor == "lrtb"


def test_gb_export_emits_window_resizable():
    doc = FormDoc(title="W")
    doc.resizable = True
    doc.min_w, doc.min_h = 200, 150
    src = doc.generate_gb_code(with_screen=False, with_loop=False)
    assert "GUI_WINDOW_RESIZABLE(frm, TRUE)" in src
    assert "GUI_WINDOW_SET_MIN_SIZE(frm, 200, 150)" in src


def test_new_widgets_load_in_runtime(run_gb, tmp_path):
    # Separator + GroupBox: neue Widget-Arten via .gbform -> GUI_LOAD.
    doc = FormDoc(title="N", w=300, h=200)
    doc.add("separator", 10, 40)
    doc.add("groupbox", 10, 60).text = "Gruppe"
    doc.save(str(tmp_path / "n.gbform"))
    out = run_gb(
        'IMPORT "gui"\nDIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("n.gbform")\n'
        'PRINT GUI_WINDOW_WIDGET_COUNT(frm)\n'
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(frm, 0))\n'
        'PRINT GUI_KIND(GUI_WINDOW_WIDGET(frm, 1))\n', base=tmp_path)
    assert out.splitlines() == ["2", "separator", "groupbox"]


def test_new_widgets_in_palette_and_export():
    kinds = {p.kind for p in PALETTE}
    assert "separator" in kinds and "groupbox" in kinds
    doc = FormDoc()
    doc.add("separator", 5, 5)
    doc.add("groupbox", 5, 30).text = "G"
    src = doc.generate_gb_code(with_screen=False, with_loop=False)
    assert "GUI_SEPARATOR(frm, 5, 5," in src
    assert 'GUI_GROUPBOX(frm, 5, 30,' in src and '"G"' in src


def test_gb_export_emits_anchor():
    doc = FormDoc()
    c = doc.add("button", 0, 0); c.anchor = "rb"
    src = doc.generate_gb_code(with_screen=False, with_loop=False)
    assert 'GUI_SET_ANCHOR(' in src and '"rb"' in src
    # Default "lt" wird NICHT emittiert
    d2 = FormDoc(); d2.add("button", 0, 0)
    assert "GUI_SET_ANCHOR" not in d2.generate_gb_code(with_screen=False, with_loop=False)


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


# --------------------------------------------------------------- Edit-Ops
def test_duplicate_offsets_and_unique_name():
    doc = FormDoc()
    b = doc.add("button", 10, 10)        # btn1
    d = doc.duplicate(b)
    assert d.kind == "button" and d.name == "btn2"
    assert (d.x, d.y) == (10 + 8, 10 + 8)    # GRID-Versatz
    assert len(doc.controls) == 2 and doc.controls[-1] is d


def test_clone_from_dict():
    doc = FormDoc()
    b = doc.add("button", 5, 5); b.text = "Hi"; b.on_click = "go"
    nc = doc.clone_from_dict(b.to_dict())
    assert nc.text == "Hi" and nc.name != b.name
    assert (nc.x, nc.y) == (13, 13)


def test_align_edges():
    doc = FormDoc()
    a = doc.add("button", 10, 10); a.w, a.h = 100, 20
    b = doc.add("button", 50, 80); b.w, b.h = 60, 30
    doc.align([a, b], "left")
    assert a.x == 10 and b.x == 10                 # an die linkeste Kante
    doc.align([a, b], "right")
    assert a.x + a.w == 110 and b.x + b.w == 110   # rechte Kanten buendig
    doc.align([a, b], "top")
    assert a.y == b.y == 10


def test_align_center_h():
    doc = FormDoc()
    a = doc.add("button", 0, 0); a.w = 100          # bbox 0..200, Mitte 100
    b = doc.add("button", 100, 40); b.w = 100
    doc.align([a, b], "center_h")
    assert a.x + a.w // 2 == 100 and b.x + b.w // 2 == 100


def test_same_size_to_reference():
    doc = FormDoc()
    a = doc.add("button", 0, 0); a.w, a.h = 120, 40   # Referenz
    b = doc.add("button", 0, 50); b.w, b.h = 60, 20
    doc.same_size([a, b], ref=a, dim="both")
    assert (b.w, b.h) == (120, 40) and (a.w, a.h) == (120, 40)
    c = doc.add("button", 0, 100); c.w, c.h = 30, 30
    doc.same_size([a, c], ref=a, dim="w")
    assert c.w == 120 and c.h == 30                  # nur Breite


def test_distribute_horizontal():
    doc = FormDoc()
    a = doc.add("button", 0, 0); a.w = 20
    b = doc.add("button", 30, 0); b.w = 20
    c = doc.add("button", 200, 0); c.w = 20         # span 0..220
    doc.distribute([a, b, c], "h")
    # span=220, sum(w)=60 -> gap=(220-60)/2=80; b.x = 0+20+80 = 100
    assert a.x == 0 and c.x == 200 and b.x == 100


def test_align_needs_two():
    doc = FormDoc()
    a = doc.add("button", 7, 7)
    doc.align([a], "left")                           # no-op bei < 2
    assert a.x == 7


def test_z_order_front_back():
    doc = FormDoc()
    a = doc.add("button", 0, 0)           # 100x28
    b = doc.add("button", 0, 0)           # ueberlappt, oben
    assert doc.control_at(5, 5) is b
    doc.to_front(a)
    assert doc.control_at(5, 5) is a      # a jetzt oben
    doc.to_back(a)
    assert doc.control_at(5, 5) is b      # b wieder oben


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


def test_window_props_roundtrip():
    doc = FormDoc(title="W", w=400, h=300)
    doc.resizable = True
    doc.min_w, doc.min_h = 200, 150
    doc.max_w, doc.max_h = 800, 600
    d = doc.to_dict()
    assert d["resizable"] is True and d["min_w"] == 200 and d["max_h"] == 600
    doc2 = FormDoc.from_dict(d)
    assert doc2.resizable and (doc2.min_w, doc2.min_h) == (200, 150)
    assert (doc2.max_w, doc2.max_h) == (800, 600)


def test_window_min_max_omitted_when_zero():
    d = FormDoc().to_dict()
    assert "min_w" not in d and "max_h" not in d        # 0 = nicht serialisiert
    assert d["resizable"] is False


def test_generate_runner_uses_stored_code():
    doc = FormDoc()
    doc.add("button", 0, 0).on_click = "on_ok"
    doc.code["on_ok"] = 'PRINT "stored"'
    src = doc.generate_runner("f.gbform")
    assert 'PRINT "stored"' in src and "SUB on_ok()" in src


# --------------------------------------------------------------- GB-Code-Export
def _parses(src):
    from drachenhauch.lexer import Lexer
    from drachenhauch.parser import Parser
    from drachenhauch.preprocess import process
    merged = process(src)
    if isinstance(merged, tuple):
        merged = merged[0]
    return Parser(Lexer(merged).tokenize()).parse()


def test_gb_export_explicit_construction_parses():
    doc = FormDoc(title="Login", w=320, h=220)
    b = doc.add("button", 20, 160); b.text = 'Sag "Hi"'; b.on_click = "on_ok"
    doc.code["on_ok"] = 'PRINT "ok"'
    dd = doc.add("dropdown", 20, 40); dd.items = ["A", "B", "C"]; dd.sel = 2; dd.on_change = "on_pick"
    doc.add("slider", 20, 80)
    src = doc.generate_gb_code()
    assert 'GUI_WINDOW("Login"' in src
    assert 'GUI_BUTTON(frm, "Sag ""Hi"""' in src       # String-Escaping
    assert "GUI_ON_CLICK(" in src and "GUI_DROPDOWN_SET_SELECTED(" in src
    assert "GUI_LOAD(" not in src                       # explizit, kein Load-Call
    assert _parses(src) is not None


def test_gb_export_image_skipped():
    doc = FormDoc()
    doc.add("image", 10, 10)
    src = doc.generate_gb_code()
    assert "GUI_IMAGE(" not in src and "uebersprungen" in src


def test_gb_export_runs_in_runtime(run_gb, tmp_path):
    # Die explizite Konstruktion muss in dhrt laufen (alle Builtin-Signaturen ok).
    doc = FormDoc(title="RT", w=300, h=220)
    doc.add("button", 10, 10).text = "Go"
    doc.add("label", 10, 40)
    doc.add("checkbox", 10, 70).checked = True
    doc.add("slider", 10, 100)
    doc.add("textinput", 10, 130)
    doc.add("radio", 10, 160).group = "g"
    doc.add("progress", 10, 190).value = 0.5
    dd = doc.add("dropdown", 150, 40); dd.items = ["Rot", "Gruen", "Blau"]; dd.sel = 1
    lb = doc.add("listbox", 150, 80); lb.items = ["x", "y"]
    doc.add("panel", 150, 140)
    doc.add("canvas", 150, 180)
    # Konstruktion ohne SCREEN/Loop -> headless ausfuehrbar; Ergebnis pruefen.
    body = doc.generate_gb_code(with_screen=False, with_loop=False)
    src = body + "\nPRINT GUI_WINDOW_WIDGET_COUNT(frm)\n" \
                 "PRINT GUI_DROPDOWN_TEXT(GUI_WINDOW_WIDGET(frm, 7))\n"
    out = run_gb(src, base=tmp_path)
    assert out.splitlines() == ["11", "Gruen"]         # 11 Widgets, sel=1 angewandt


# --------------------------------------------------------------- Codegen
def test_generated_runner_parses():
    from drachenhauch.lexer import Lexer
    from drachenhauch.parser import Parser
    from drachenhauch.preprocess import process

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


def test_runner_form_fills_os_window():
    # Die Form laeuft randlos auf dem OS-Fenster; resizable -> natives Resize.
    doc = FormDoc(title="App", w=400, h=300)
    doc.resizable = True
    doc.min_w, doc.min_h = 200, 150
    src = doc.generate_runner("f.gbform")
    assert "SCREEN(400, 300" in src                       # Fenster = Formulargroesse
    assert "WINDOW_RESIZABLE(TRUE)" in src                # OS-Fenster resizebar
    assert "WINDOW_MIN_SIZE(200, 150)" in src
    assert "GUI_WINDOW_CHROME(frm, FALSE)" in src         # randlos
    assert "GUI_WINDOW_SET_BOUNDS(frm, 0, 0, SCREENWIDTH(), SCREENHEIGHT())" in src
    assert _parses(src) is not None


def test_runner_non_resizable_no_window_flags():
    src = FormDoc(title="X").generate_runner("f.gbform")   # resizable=False
    assert "WINDOW_RESIZABLE(TRUE)" not in src             # OS-Fenster NICHT resizebar
    assert "GUI_WINDOW_CHROME(frm, FALSE)" in src          # randlos trotzdem


def test_generated_runner_with_bodies():
    doc = FormDoc()
    doc.add("button", 0, 0).on_click = "on_ok"
    src = doc.generate_runner("f.gbform", handler_bodies={"on_ok": 'PRINT "hi"'})
    assert 'PRINT "hi"' in src


# ----------------------------------------- Roundtrip-Treue gegenueber dhrt
# Der Designer kennt nur eine Teilmenge des Laufzeit-Formats (`gui.rs`). Alles
# Uebrige muss er unveraendert durchreichen, sonst zerstoert ein Oeffnen+
# Speichern im Designer Menues, Reiter, Tabellen und Baeume einer per GUI_SAVE
# erzeugten Form.
def test_unknown_widget_fields_survive_roundtrip():
    src = {"kind": "table", "x": 1, "y": 2, "w": 3, "h": 4,
           "tab_page": 1, "font": 7,
           "table": {"headers": ["A", "B"], "rows": [["1", "2"]],
                     "col_widths": [10, 20], "selected": -1}}
    c = Control.from_dict(src)
    assert c.extra["table"]["headers"] == ["A", "B"]
    out = c.to_dict()
    assert out["table"] == src["table"]
    assert out["tab_page"] == 1 and out["font"] == 7
    assert Control.from_dict(out).to_dict() == out          # stabil


def test_unknown_window_fields_survive_roundtrip(tmp_path):
    src = {"title": "T", "w": 100, "h": 80, "widgets": [],
           "chrome": False, "tabs": ["Eins", "Zwei"], "active_tab": 1,
           "menus": [{"label": "Datei", "in_bar": True,
                      "items": [{"label": "Beenden", "separator": False,
                                 "enabled": True}]}]}
    p = tmp_path / "m.gbform"
    p.write_text(json.dumps(src), encoding="utf-8")
    FormDoc.load(str(p)).save(str(p))
    back = json.loads(p.read_text(encoding="utf-8"))
    for k in ("chrome", "tabs", "active_tab", "menus"):
        assert back[k] == src[k], k


def test_clone_does_not_share_extra():
    c = Control.from_dict({"kind": "table", "table": {"rows": []}})
    d = c.clone()
    d.extra["table"]["rows"].append(["x"])
    assert c.extra["table"]["rows"] == []


def test_gui_save_form_survives_designer_roundtrip(run_gb, tmp_path):
    # Golden gegen die Laufzeit: eine von dhrt geschriebene Form durch den
    # Designer schleusen und pruefen, dass GUI_LOAD danach dasselbe sieht.
    run_gb(
        'IMPORT "gui"\n'
        'DIM w AS GUI_WINDOW\nw = GUI_WINDOW("RT", 10, 20, 400, 300)\n'
        'GUI_WINDOW_CHROME(w, FALSE)\n'
        'DIM m AS INTEGER\nm = GUI_MENU(w, "Datei")\n'
        'GUI_MENU_ITEM(m, "Beenden")\n'
        'DIM tb[2] AS STRING\ntb[0] = "Eins"\ntb[1] = "Zwei"\nGUI_TABS(w, tb)\n'
        'DIM t AS GUI_WIDGET\nt = GUI_TABLE(w, 10, 40, 200, 100)\n'
        'DIM b AS GUI_WIDGET\nb = GUI_BUTTON(w, "K", 10, 160, 80, 24)\n'
        'GUI_SET_TAB(b, 1)\nGUI_SAVE(w, "orig.gbform")\n',
        base=tmp_path)
    FormDoc.load(str(tmp_path / "orig.gbform")).save(str(tmp_path / "after.gbform"))
    out = run_gb(
        'IMPORT "gui"\n'
        'DIM a AS GUI_WINDOW\nDIM b AS GUI_WINDOW\n'
        'a = GUI_LOAD("orig.gbform")\nb = GUI_LOAD("after.gbform")\n'
        'PRINT GUI_TO_JSON(a) = GUI_TO_JSON(b)\n',
        base=tmp_path)
    assert out.strip() == "TRUE"


# ----------------------------------------- Identitaet statt Wertvergleich
def test_remove_hits_the_selected_control_not_an_equal_twin():
    doc = FormDoc()
    a = doc.add("button", 10, 10); b = doc.add("button", 10, 10)
    a.name = b.name = ""                 # feldgleich (wie bei GUI_SAVE-Formen)
    assert a == b and a is not b
    doc.remove(b)
    assert len(doc.controls) == 1 and doc.controls[0] is a


def test_to_front_does_not_duplicate_an_equal_twin():
    doc = FormDoc()
    a = doc.add("button", 10, 10); b = doc.add("button", 10, 10)
    a.name = b.name = ""
    doc.to_front(a)
    assert [c is a for c in doc.controls].count(True) == 1
    assert doc.controls[-1] is a and doc.controls[0] is b


def test_remove_prunes_orphaned_handler_bodies():
    doc = FormDoc()
    btn = doc.add("button", 0, 0)
    name = doc.ensure_handler(btn)
    doc.code[name] = 'PRINT "x"'
    doc.remove(btn)
    assert doc.code == {}
    assert doc.ensure_handler(doc.add("button", 0, 0)) == name   # kein "…2"


# ----------------------------------------- Tolerantes Laden (wie gui.rs)
def test_from_dict_tolerates_broken_json():
    c = Control.from_dict({"kind": "button", "x": None, "color": "#ff0000",
                           "items": None, "ov": {"bg": "blau"},
                           "checked": "ja", "text": 42})
    assert c.x == 0 and c.color == 0xFFFFFF and c.items == []
    assert c.ov == {} and c.checked is False and c.text == ""
    d = FormDoc.from_dict({"widgets": "keine liste", "code": None, "w": None})
    assert d.controls == [] and d.code == {} and d.w == 360
    assert FormDoc.from_dict(None).title == "Form1"


def test_closable_default_matches_runtime():
    # gui.rs::from_json defaultet auf false -- der Designer zeigte sonst ein
    # Schliessen-Kreuz an, das die laufende Form nicht hat.
    assert FormDoc.from_dict({"title": "X"}).closable is False
    assert FormDoc().closable is True                  # neue Formulare schon


# ----------------------------------------- GB-Emit-Helfer
def test_gb_num_never_uses_exponent_notation():
    from drachenhauch.formdesigner.document import _gb_num
    for v in (1e-5, 1e20, 1e-12, 0.0, -2.5):
        assert "e" not in _gb_num(v).lower(), v
        assert "." in _gb_num(v)
    assert _gb_num(float("inf")) == "0.0" and _gb_num(float("nan")) == "0.0"


def test_gb_ident_escapes_reserved_words():
    from drachenhauch.formdesigner.document import _gb_ident
    from drachenhauch.tokens import KEYWORDS
    for w in ("Print", "DATA", "to", "Step", "Next", "End", "True"):
        assert _gb_ident(w).lower() not in KEYWORDS, w
    assert _gb_ident("btnSave") == "btnSave"            # unveraendert
    assert _gb_ident("OK Knopf") == "OK_Knopf"


def test_ensure_handler_yields_a_valid_identifier():
    doc = FormDoc()
    b = doc.add("button", 0, 0); b.name = "OK Knopf"
    assert doc.ensure_handler(b) == "OK_KnopfClick"
    b2 = doc.add("button", 0, 0); b2.name = "Print"
    assert doc.ensure_handler(b2) == "PrintClick"       # Suffix -> kein Keyword


# ----------------------------------------- FormProject
def test_project_normalizes_paths():
    p = FormProject()
    p.add("forms/a.gbform"); p.add(r"forms\a.gbform"); p.add("./forms/a.gbform")
    assert p.forms == ["forms/a.gbform"]
    p.remove(r"forms\a.gbform")
    assert p.forms == []
    q = FormProject.from_dict({"forms": [r"sub\s.gbform"], "main": r"sub\s.gbform"})
    assert q.forms == ["sub/s.gbform"] and q.main == "sub/s.gbform"


def test_formdoc_load_rejects_a_project_manifest(tmp_path):
    # Sonst laedt `dhform Projekt.GBPROJ` das Manifest als leeres Formular --
    # und das naechste Strg+S ueberschreibt die Projektdatei.
    p = tmp_path / "Projekt.GBPROJ"
    FormProject(forms=["a.gbform"], main="a.gbform").save(str(p))
    try:
        FormDoc.load(str(p))
        assert False, "Manifest wurde als Formular akzeptiert"
    except ValueError as e:
        assert "gbproj" in str(e).lower() or "manifest" in str(e).lower()


# ------------------------- Export vs. GUI_LOAD: beide Wege muessen dasselbe bauen
def _export_vs_load(run_gb, tmp_path, doc, probes: str):
    """Dasselbe Formular einmal per GUI_LOAD und einmal per generiertem
    GB-Code aufbauen und dieselben Getter abfragen. Liefert (load, export)."""
    doc.save(str(tmp_path / "f.gbform"))
    load_out = run_gb('IMPORT "gui"\nDIM frm AS GUI_WINDOW\n'
                      'frm = GUI_LOAD("f.gbform")\n' + probes, base=tmp_path)
    src = doc.generate_gb_code(with_screen=False, with_loop=False) + "\n" + probes
    return load_out.strip().splitlines(), run_gb(src, base=tmp_path).strip().splitlines()


def test_export_keeps_label_and_slider_geometry(run_gb, tmp_path):
    # GUI_LABEL/GUI_SLIDER/GUI_SEPARATOR berechnen ihre Groesse selbst -- ohne
    # nachgereichtes GUI_SET_BOUNDS war ein 300x40-Label im Export 40x16.
    doc = FormDoc(title="G", w=420, h=300)
    lbl = doc.add("label", 10, 10); lbl.text = "Hi"; lbl.w, lbl.h = 300, 40
    sld = doc.add("slider", 10, 60); sld.h = 40
    sep = doc.add("separator", 10, 120); sep.h = 20
    probes = ("PRINT GUI_GET_W(GUI_WINDOW_WIDGET(frm, 0))\n"
              "PRINT GUI_GET_H(GUI_WINDOW_WIDGET(frm, 0))\n"
              "PRINT GUI_GET_H(GUI_WINDOW_WIDGET(frm, 1))\n"
              "PRINT GUI_GET_H(GUI_WINDOW_WIDGET(frm, 2))\n")
    load, exp = _export_vs_load(run_gb, tmp_path, doc, probes)
    assert load == ["300", "40", "40", "20"]
    assert exp == load


def test_export_progress_shows_same_fill_as_gbform(run_gb, tmp_path):
    # GUI_PROGRESS ist fest 0..1 und GUI_SET_VALUE clampt -- ein roher Wert 25
    # (bei max=100) wurde zu 1.0 = randvoll. Der Export normiert deshalb.
    doc = FormDoc(title="P", w=300, h=120)
    prg = doc.add("progress", 10, 10)
    prg.min, prg.max, prg.value = 0.0, 100.0, 25.0
    probes = "PRINT GUI_TO_JSON(frm)\n"
    doc.save(str(tmp_path / "f.gbform"))
    a = run_gb('IMPORT "gui"\nDIM frm AS GUI_WINDOW\n'
               'frm = GUI_LOAD("f.gbform")\n' + probes, base=tmp_path)
    b = run_gb(doc.generate_gb_code(with_screen=False, with_loop=False) + "\n" + probes,
               base=tmp_path)

    def fill(js):                      # so zeichnet gui.rs den Balken
        w = json.loads(js)["widgets"][0]
        span = w["max"] - w["min"]
        return 0.0 if span == 0 else (w["value"] - w["min"]) / span

    assert fill(a) == 0.25
    assert fill(b) == 0.25             # vorher 1.0 = randvoll


def test_export_window_is_closable_like_the_gbform(run_gb, tmp_path):
    # GUI_WINDOW legt closable=false an, GUI_LOAD liest den .gbform-Wert.
    doc = FormDoc(title="C", w=200, h=140)
    doc.closable = True
    probes = 'PRINT GUI_TO_JSON(frm)\n'
    doc.save(str(tmp_path / "f.gbform"))
    a = run_gb('IMPORT "gui"\nDIM frm AS GUI_WINDOW\nfrm = GUI_LOAD("f.gbform")\n'
               + probes, base=tmp_path)
    b = run_gb(doc.generate_gb_code(with_screen=False, with_loop=False) + "\n" + probes,
               base=tmp_path)
    assert json.loads(a)["closable"] is True
    assert json.loads(b)["closable"] is True


def test_runner_max_size_with_only_one_bound_set():
    doc = FormDoc(title="M", w=300, h=200)
    doc.resizable = True; doc.max_w = 900          # max_h bleibt 0
    src = doc.generate_runner("f.gbform")
    assert "WINDOW_MAX_SIZE(900, 0)" not in src    # 0 wuerde GLFW hart klemmen
    assert "WINDOW_MAX_SIZE(900, 32000)" in src


# --- Control umbenennen zieht die Handler mit ------------------------------

def test_umbenennen_zieht_abgeleiteten_handler_mit():
    """Der Handler-Name entsteht beim Anlegen aus dem Control-Namen. Wurde
    das Control spaeter umbenannt, blieb der alte Name stehen -- im
    Code-Fenster passierte scheinbar nichts."""
    d = FormDoc(title="T")
    c = d.add("button", 10, 10)
    name = d.ensure_handler(c)
    d.code[name] = 'PRINT "hallo"'
    assert name == "btn1Click"

    alt = c.name
    c.name = "StartKnopf"
    assert d.rename_control_handlers(c, alt) == [("btn1Click", "StartKnopfClick")]
    assert c.on_click == "StartKnopfClick"
    # Der Rumpf wandert mit, der alte Schluessel verschwindet -- sonst laege
    # der Code unerreichbar herum und der Export erzeugte nur ein TODO.
    assert d.code["StartKnopfClick"] == 'PRINT "hallo"'
    assert "btn1Click" not in d.code


def test_selbst_vergebener_handler_bleibt_beim_umbenennen():
    """Wer seinen Handler bewusst benannt hat, will ihn nicht verlieren."""
    d = FormDoc(title="T")
    c = d.add("button", 10, 10)
    c.on_click = "SpielStarten"
    d.code["SpielStarten"] = "x"
    alt = c.name
    c.name = "Anders"
    assert d.rename_control_handlers(c, alt) == []
    assert c.on_click == "SpielStarten"
    assert d.code["SpielStarten"] == "x"


def test_umbenennen_ohne_handler_tut_nichts():
    d = FormDoc(title="T")
    c = d.add("label", 10, 10)          # Label hat kein Event
    alt = c.name
    c.name = "Titel"
    assert d.rename_control_handlers(c, alt) == []


def test_umbenennen_auf_denselben_namen_ist_ein_no_op():
    d = FormDoc(title="T")
    c = d.add("button", 10, 10)
    d.ensure_handler(c)
    assert d.rename_control_handlers(c, c.name) == []


def test_umbenennen_kollidiert_nicht_mit_bestehendem_handler():
    """Traegt schon ein anderes Control den Zielnamen, muss der neue
    ausweichen -- sonst zeigten zwei Controls auf denselben Rumpf."""
    d = FormDoc(title="T")
    a = d.add("button", 10, 10)
    b = d.add("button", 10, 50)
    d.ensure_handler(a)                  # btn1Click
    d.ensure_handler(b)                  # btn2Click
    alt = b.name
    b.name = a.name                      # beide heissen jetzt btn1
    paare = d.rename_control_handlers(b, alt)
    assert paare, "Handler haette umbenannt werden muessen"
    assert b.on_click != a.on_click, "zwei Controls zeigen auf denselben Handler"
