"""Form-Designer: Thema im Dokument, im erzeugten Programm und auf der
Entwurfsflaeche.

Der wichtigste Test hier ist `test_farben_stimmen_mit_der_laufzeit_ueberein`:
Der Designer zeichnet mit Qt und kann die Laufzeit nicht fragen, also ist
`FORM_THEME_COLORS` ein NACHBAU der Presets aus `rust/drachenhauch_runtime/src/gui.rs`.
Solche Doppelungen laufen still auseinander -- hier wird die Nachbildung
gegen die echte Laufzeit geprueft, damit ein Entwurf zeigt, was das Formular
spaeter wirklich tut.
"""
import pytest

from drachenhauch.formdesigner import (
    FORM_THEMES, FORM_THEME_COLORS, FormDoc, theme_colors,
)


# --- Dokument -------------------------------------------------------------

def test_thema_ueberlebt_speichern_und_laden():
    d = FormDoc(title="T")
    d.theme = "glas_dunkel"
    assert FormDoc.from_dict(d.to_dict()).theme == "glas_dunkel"


def test_ohne_thema_wird_kein_feld_geschrieben():
    """Sonst bekaeme jede alte Datei beim blossen Oeffnen+Speichern ein neues
    Feld -- unnoetige Aenderungen in der Versionsverwaltung."""
    assert "theme" not in FormDoc(title="T").to_dict()
    assert FormDoc.from_dict({"title": "T"}).theme == ""


def test_unbekanntes_thema_faellt_auf_die_vorgabe_zurueck():
    assert theme_colors("gibtsnicht") == theme_colors("")
    assert theme_colors("") == FORM_THEME_COLORS[""]


# --- Erzeugtes Programm ---------------------------------------------------

def test_erzeugtes_programm_setzt_das_thema():
    d = FormDoc(title="T")
    d.theme = "glas_hell"
    zeilen = d.generate_runner("a.gbform").split("\n")
    assert 'GUI_THEME_PRESET("glas_hell")' in zeilen
    # VOR dem Laden der Form: das Preset setzt auch Metriken (Eckenradius),
    # und die gehen in die Darstellung der Widgets ein.
    i_thema = zeilen.index('GUI_THEME_PRESET("glas_hell")')
    i_load = next(i for i, z in enumerate(zeilen) if "GUI_LOAD" in z)
    assert i_thema < i_load, "Thema muss vor dem Laden der Form stehen"


def test_ohne_thema_kein_preset_aufruf():
    zeilen = FormDoc(title="T").generate_runner("a.gbform")
    assert "GUI_THEME_PRESET" not in zeilen


@pytest.mark.parametrize("name", [t for t in FORM_THEMES if t])
def test_jedes_waehlbare_thema_hat_farben(name):
    """Die Auswahlliste und die Farbtabelle duerfen nicht auseinanderlaufen --
    sonst waehlt man ein Thema, das auf der Entwurfsflaeche wie die Vorgabe
    aussieht."""
    assert name in FORM_THEME_COLORS, f"{name} steht zur Wahl, hat aber keine Farben"
    for schluessel in ("win_bg", "win_border", "title_bg", "title_fg", "widget_bg",
                       "widget_border", "text_fg", "muted_fg", "accent",
                       "radius", "gradient", "gloss"):
        assert schluessel in FORM_THEME_COLORS[name], f"{name}: {schluessel} fehlt"


# --- Abgleich mit der Laufzeit -------------------------------------------

_FARB_SCHLUESSEL = ("win_bg", "win_border", "title_bg", "title_fg",
                    "widget_bg", "widget_border", "text_fg", "muted_fg", "accent")
_METRIK = {"radius": "corner_radius", "gradient": "gradient", "gloss": "gloss"}


@pytest.mark.parametrize("name", [t for t in FORM_THEMES if t])
def test_farben_stimmen_mit_der_laufzeit_ueberein(run_gb, name):
    src = ['IMPORT "gui"', f'GUI_THEME_PRESET("{name}")']
    src += [f'PRINT GUI_THEME_GET("{k}")' for k in _FARB_SCHLUESSEL]
    src += [f'PRINT GUI_METRIC_GET("{v}")' for v in _METRIK.values()]
    zeilen = [z.strip() for z in run_gb("\n".join(src) + "\n").split("\n") if z.strip()]
    assert len(zeilen) == len(_FARB_SCHLUESSEL) + len(_METRIK)

    echt = dict(zip(_FARB_SCHLUESSEL, (int(z) for z in zeilen)))
    echt.update(zip(_METRIK.keys(), (int(z) for z in zeilen[len(_FARB_SCHLUESSEL):])))
    nachbau = FORM_THEME_COLORS[name]
    abweichung = {k: (hex(nachbau[k]), hex(v)) for k, v in echt.items() if nachbau[k] != v}
    assert not abweichung, (
        f"Thema '{name}': Designer-Nachbau weicht von der Laufzeit ab "
        f"(Designer, Laufzeit): {abweichung}"
    )


def test_vorgabe_entspricht_dem_thema_ohne_preset(run_gb):
    """Der leere Eintrag ("(Vorgabe)") muss dem entsprechen, was die Laufzeit
    ohne jeden GUI_THEME_PRESET-Aufruf zeigt."""
    src = ['IMPORT "gui"']
    src += [f'PRINT GUI_THEME_GET("{k}")' for k in _FARB_SCHLUESSEL]
    zeilen = [z.strip() for z in run_gb("\n".join(src) + "\n").split("\n") if z.strip()]
    echt = dict(zip(_FARB_SCHLUESSEL, (int(z) for z in zeilen)))
    nachbau = FORM_THEME_COLORS[""]
    abweichung = {k: (hex(nachbau[k]), hex(v)) for k, v in echt.items() if nachbau[k] != v}
    assert not abweichung, f"Vorgabe weicht ab (Designer, Laufzeit): {abweichung}"
