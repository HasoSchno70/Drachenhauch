"""Tests fuer OutlinePanel._scan() (Review-Fund: der Scanner war weder
comment-/string-aware noch kannte er PROPERTY GET/SET -- eine Klasse mit
nur Properties zeigte gar nichts im Outline, obwohl dieselbe Property in
der Breadcrumb-Leiste (die symbols.scan_scopes nutzt) korrekt erschien)."""
from drachenhauch.editor_qt.outline_panel import OutlinePanel


def test_scan_sub_and_function():
    src = "SUB foo(a AS INTEGER)\nEND SUB\nFUNCTION bar() AS INTEGER\nEND FUNCTION"
    items = OutlinePanel._scan(src)
    kinds_names = [(k, n) for k, n, *_ in items]
    assert ("sub", "foo") in kinds_names
    assert ("function", "bar") in kinds_names


def test_scan_property_get_and_set():
    src = (
        "CLASS Player\n"
        "    DIM _hp AS INTEGER\n"
        "    PROPERTY GET hp() AS INTEGER\n"
        "        RETURN Self._hp\n"
        "    END PROPERTY\n"
        "    PROPERTY SET hp(value AS INTEGER)\n"
        "        Self._hp = value\n"
        "    END PROPERTY\n"
        "END CLASS\n"
    )
    items = OutlinePanel._scan(src)
    properties = [(kind, name, line, indent) for kind, name, line, indent, _params in items
                  if kind == "property"]
    assert ("property", "hp", 3, 1) in properties
    assert ("property", "hp", 6, 1) in properties


def test_scan_property_class_shows_something():
    """Eine Klasse mit NUR Properties (keine SUB/FUNCTION) darf im Outline
    nicht komplett leer erscheinen."""
    src = "CLASS Player\n    PROPERTY GET hp() AS INTEGER\n        RETURN 1\n    END PROPERTY\nEND CLASS\n"
    items = OutlinePanel._scan(src)
    assert len(items) == 2   # CLASS + die eine Property


def test_scan_ignores_class_keyword_inside_comment():
    src = "' SUB fake() sollte nicht als Opener zaehlen\nPRINT 1\n"
    items = OutlinePanel._scan(src)
    assert items == []


def test_scan_ignores_class_keyword_inside_string():
    src = 'PRINT "CLASS Foo sollte kein Opener sein"\n'
    items = OutlinePanel._scan(src)
    assert items == []


def test_scan_class_with_trailing_comment_still_recognized():
    src = "CLASS Player ' der Spieler\nEND CLASS\n"
    items = OutlinePanel._scan(src)
    assert items == [("class", "Player", 1, 0, "")]
