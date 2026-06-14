"""B3: hilfreiche Fehlermeldung bei `DIM x AS <Modul-Typ>` ohne IMPORT.

Frueher: irrefuehrendes „Stufe 3e: DIM-Typ 'vec2' noch nicht unterstuetzt"
(Compiler-Interna + sachlich falsch -- der Typ existiert, nur das IMPORT fehlt).
Jetzt: „Unbekannter Typ 'vec2' -- fehlt IMPORT \"vec2\"?".
"""
import pytest
from gamebasic.errors import GameBasicError


def test_dim_external_type_without_import_hints_import(run_gb):
    with pytest.raises(GameBasicError, match=r'Unbekannter Typ .vec2.*IMPORT "vec2"'):
        run_gb("DIM v AS VEC2\n")


def test_dim_external_type_shared_lists_candidates(run_gb):
    # tiled_map kommt aus mehreren Modulen -> alle als Kandidaten nennen.
    with pytest.raises(GameBasicError, match=r'Unbekannter Typ .tiled_map.*IMPORT'):
        run_gb("DIM m AS TILED_MAP\n")


def test_dim_truly_unknown_type(run_gb):
    with pytest.raises(GameBasicError, match=r"Unbekannter Typ .foobar."):
        run_gb("DIM x AS FOOBAR\n")


def test_dim_array_element_external_type_without_import(run_gb):
    with pytest.raises(GameBasicError, match=r'Unbekannter Typ .vec2.*IMPORT "vec2"'):
        run_gb("DIM a[5] AS VEC2\n")


def test_no_stufe_3e_jargon_leaks(run_gb):
    # Die interne „Stufe 3e"-Phasenbezeichnung darf nicht mehr in der Meldung stehen.
    try:
        run_gb("DIM v AS VEC2\n")
    except GameBasicError as e:
        assert "Stufe 3" not in str(e)
    else:
        pytest.fail("erwartete einen Fehler")


def test_dim_external_type_with_import_works(run_gb):
    out = run_gb('IMPORT "vec2"\n'
                 'DIM v AS VEC2\n'
                 'v = VEC2_NEW(3.0, 4.0)\n'
                 'PRINT VEC2_LENGTH(v)\n')
    assert out == "5.0\n"
