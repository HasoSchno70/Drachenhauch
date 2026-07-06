"""Farbpaletten und QSS fuer den GameBasic-Editor (PySide6).

Zwei Themes (Dark = VSCode Dark+, Light = VSCode Light+). Wechsel via
`set_active_theme(name)` -- emittiert `theme_signals.changed`, das
Widgets abonnieren, um ihre Style-abhaengigen Sachen neu zu setzen.

`COLORS` ist ein Live-Mapping: bestehender Code, der `COLORS["bg"]`
liest, sieht nach `set_active_theme()` automatisch die neuen Werte. Fuer
QSS muessen wir das App-Stylesheet aktiv neu anwenden -- das macht der
Listener im MainWindow ueber `theme_signals.changed`.
"""
from __future__ import annotations

from PySide6.QtCore import QObject, Signal
from PySide6.QtGui import QColor


COLORS_DARK: dict[str, str] = {
    # UI-Chrome -- inspiriert von den Logo-Farben (cyan/mint auf navy).
    "bg":            "#0B1421",     # tiefes Navy (Logo-BG-Ton)
    "bg_alt":        "#121E2C",
    "bg_panel":      "#1A2738",
    "bg_panel_hi":   "#22344A",     # oberer Gradient-Stop fuer Chrome
    "bg_hover":      "#243549",
    "fg":            "#D5EDF0",     # heller Cyan-Ton (Logo-Vordergrund)
    "fg_muted":      "#7A8E96",
    "line_no":       "#506570",
    "sel":           "#1A4F66",     # ruhiges Cyan fuer Selektion
    "sel_soft":      "#15394A",     # gedaempfte Selektion (Listen)
    "current_line":  "#152030",
    "indent_guide":  "#1F3346",
    "border":        "#243A4D",
    "border_soft":   "#1A2C3B",
    "link":          "#5EEAEA",
    "find_hit":      "#3A5566",
    "tooltip_bg":    "#1F2D40",
    "tooltip_fg":    "#D5EDF0",
    "accent":        "#2DE0E0",     # Logo-Hauptfarbe Cyan
    "accent_hover":  "#5EEAEA",
    "accent_text":   "#0B1421",
    "accent_soft":   "#16404C",     # gedaempfter Akzent-Hintergrund
    "success":       "#2DE89A",     # Logo-Mint
    "success_hover": "#5EEDB2",
    "danger":        "#E55A8C",     # warmes Magenta (Akzent gegen Cyan)
    "danger_hover":  "#F08AAB",
    # Code-Highlighting bleibt VSCode-Dark+ -- vom User explizit ausgenommen
    "kw_ctrl":       "#C586C0",
    "kw_decl":       "#569CD6",
    "type":          "#4EC9B0",
    "string":        "#CE9178",
    "number":        "#B5CEA8",
    "comment":       "#6A9955",
    "builtin":       "#DCDCAA",
    "identifier":    "#9CDCFE",
    "operator":      "#D4D4D4",
    "bool":          "#569CD6",
    "error":         "#F44747",
    "warning":       "#CCA700",     # Amber -- Live-Diagnostik-Warnungen
    "info":          "#4EC9B0",
}

COLORS_LIGHT: dict[str, str] = {
    # UI-Chrome -- helle Variante mit Logo-Cyan-Akzenten.
    "bg":            "#FAFCFD",
    "bg_alt":        "#F0F5F7",
    "bg_panel":      "#E5EEF2",
    "bg_panel_hi":   "#EEF6F9",     # oberer Gradient-Stop fuer Chrome
    "bg_hover":      "#D5E5EB",
    "fg":            "#0B1421",
    "fg_muted":      "#5A6E78",
    "line_no":       "#8FA3AC",
    "sel":           "#A8E0E8",
    "sel_soft":      "#CDEAF0",
    "current_line":  "#F3F8FA",
    "indent_guide":  "#E0EBF0",
    "border":        "#C8D5DC",
    "border_soft":   "#DCE7EC",
    "link":          "#0090A8",
    "find_hit":      "#7BD0DC",
    "tooltip_bg":    "#E8F4F7",
    "tooltip_fg":    "#0B1421",
    "accent":        "#0090A8",     # dunkleres Cyan fuer Kontrast auf hell
    "accent_hover":  "#00B0C8",
    "accent_text":   "#FFFFFF",
    "accent_soft":   "#CFEAF0",     # gedaempfter Akzent-Hintergrund
    "success":       "#0E9F6E",
    "success_hover": "#10B981",
    "danger":        "#C13A6A",
    "danger_hover":  "#D44E80",
    # Code-Highlighting bleibt VSCode-Light+
    "kw_ctrl":       "#AF00DB",
    "kw_decl":       "#0000FF",
    "type":          "#267F99",
    "string":        "#A31515",
    "number":        "#098658",
    "comment":       "#008000",
    "builtin":       "#795E26",
    "identifier":    "#001080",
    "operator":      "#1F1F1F",
    "bool":          "#0000FF",
    "error":         "#A1260D",
    "warning":       "#BF8803",     # Amber (dunkler fuer Light-Theme)
    "info":          "#1B6E1B",
}


# Aktive Palette (mutable). Code, der gegen `COLORS["bg"]` liest, sieht
# nach `set_active_theme(...)` automatisch die neuen Werte -- daher
# nicht mit lokalen Kopien arbeiten.
COLORS: dict[str, str] = dict(COLORS_DARK)
_ACTIVE_THEME = "dark"


class _ThemeSignals(QObject):
    changed = Signal(str)  # neuer Theme-Name


# Singleton -- einmal an `theme_signals.changed.connect(...)` haengen,
# dann werden alle `set_active_theme`-Calls automatisch propagiert.
theme_signals = _ThemeSignals()


def active_theme() -> str:
    return _ACTIVE_THEME


def set_active_theme(name: str) -> None:
    """Wechselt aktive Palette. Loest `theme_signals.changed` aus."""
    global _ACTIVE_THEME
    src = COLORS_LIGHT if name == "light" else COLORS_DARK
    _ACTIVE_THEME = "light" if name == "light" else "dark"
    COLORS.clear()
    COLORS.update(src)
    theme_signals.changed.emit(_ACTIVE_THEME)


def global_qss() -> str:
    """Globales Stylesheet basierend auf der aktuellen Palette.

    Bei Theme-Wechsel muss MainWindow das App-Stylesheet neu setzen --
    QSS liest seine Werte zur Build-Zeit, nicht live.

    Gestaltungsidee: ruhiges Navy als Editor-Flaeche, das Chrome (Toolbar,
    Tabs, Statusbar, Menubar) mit dezenten vertikalen Gradienten abgesetzt,
    und der Logo-Cyan-Akzent ueberall dort, wo der Nutzer interagiert
    (Fokus-Ringe, aktive Tabs, Scrollbar-Hover, Selektion, Statusbar-Kante).
    """
    c = COLORS
    chrome = (f"qlineargradient(x1:0, y1:0, x2:0, y2:1, "
              f"stop:0 {c['bg_panel_hi']}, stop:1 {c['bg_alt']})")
    # Ruhiger Akzent-Button: Verlauf dunkel -> Akzent statt greller Vollflaeche.
    # Aus dem Akzent abgeleitet, damit es in Dark UND Light funktioniert; helle
    # Schrift, weil beide Stops gedaempft/dunkel sind.
    _acc = QColor(c["accent"])
    acc_btn_top = _acc.darker(280).name()
    acc_btn_bot = _acc.darker(150).name()
    acc_btn_top_h = _acc.darker(230).name()
    acc_btn_bot_h = _acc.darker(120).name()
    return f"""
        QWidget {{
            background-color: {c['bg']};
            color: {c['fg']};
            font-family: "Segoe UI", sans-serif;
            font-size: 10pt;
        }}
        QMainWindow, QDialog {{
            background-color: {c['bg']};
        }}
        QDialog QLabel {{ background: transparent; }}

        /* --- Toolbar: Gradient + Akzent-Unterkante --- */
        QToolBar {{
            background: {chrome};
            border: 0;
            border-bottom: 1px solid {c['border']};
            padding: 5px 6px;
            spacing: 3px;
        }}
        QToolBar::separator {{
            background-color: {c['border']};
            width: 1px;
            margin: 5px 7px;
        }}
        QToolBar QToolButton {{
            background-color: transparent;
            color: {c['fg']};
            padding: 5px 11px;
            border-radius: 6px;
            border: 1px solid transparent;
        }}
        QToolBar QToolButton:hover {{
            background-color: {c['bg_hover']};
            border: 1px solid {c['border']};
        }}
        QToolBar QToolButton:pressed {{
            background-color: {c['accent_soft']};
        }}
        QToolBar QToolButton:checked {{
            background-color: {c['accent_soft']};
            border: 1px solid {c['accent']};
        }}
        QToolBar QToolButton:disabled {{
            color: {c['fg_muted']};
        }}
        /* Run/Stop bekommen farbige Hover-Akzente (Objektnamen im Code gesetzt). */
        QToolBar QToolButton#RunButton:hover {{
            background-color: {c['success']};
            border: 1px solid {c['success']};
            color: {c['accent_text']};
        }}
        QToolBar QToolButton#StopButton:hover {{
            background-color: {c['danger']};
            border: 1px solid {c['danger']};
            color: #FFFFFF;
        }}

        /* --- Statusbar: farbige Akzent-Oberkante --- */
        QStatusBar {{
            background: {chrome};
            color: {c['fg_muted']};
            border-top: 2px solid {c['accent']};
        }}
        QStatusBar::item {{ border: 0; }}
        QStatusBar QLabel {{
            border-left: 1px solid {c['border']};
            padding-left: 10px;
            padding-right: 10px;
        }}

        /* --- Menubar / Menus --- */
        QMenuBar {{
            background: {chrome};
            color: {c['fg']};
            padding: 2px 4px;
            border-bottom: 1px solid {c['border']};
        }}
        QMenuBar::item {{
            background: transparent;
            padding: 4px 10px;
            border-radius: 5px;
        }}
        QMenuBar::item:selected {{
            background-color: {c['bg_hover']};
        }}
        QMenu {{
            background-color: {c['bg_panel']};
            color: {c['fg']};
            border: 1px solid {c['border']};
            padding: 5px;
        }}
        QMenu::item {{
            padding: 5px 26px 5px 12px;
            border-radius: 5px;
        }}
        QMenu::item:selected {{
            background-color: {c['accent']};
            color: {c['accent_text']};
        }}
        QMenu::separator {{
            height: 1px;
            background-color: {c['border']};
            margin: 5px 10px;
        }}

        QSplitter::handle {{
            background-color: {c['bg_alt']};
        }}
        QSplitter::handle:hover {{
            background-color: {c['accent']};
        }}

        /* --- Tabs --- */
        QTabWidget::pane {{
            border: 0;
            background-color: {c['bg']};
        }}
        QTabBar {{
            background-color: {c['bg_alt']};
            qproperty-drawBase: 0;
        }}
        QTabBar::tab {{
            background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {c['bg_alt']}, stop:1 {c['bg']});
            color: {c['fg_muted']};
            padding: 7px 16px;
            border: 0;
            border-right: 1px solid {c['border']};
            border-top: 2px solid transparent;
            min-width: 80px;
        }}
        QTabBar::tab:hover {{
            background-color: {c['bg_hover']};
            color: {c['fg']};
        }}
        QTabBar::tab:selected {{
            background-color: {c['bg']};
            color: {c['fg']};
            border-top: 2px solid {c['accent']};
        }}
        QTabBar::close-button {{
            subcontrol-position: right;
            margin: 4px;
            border-radius: 3px;
        }}
        QTabBar::close-button:hover {{
            background-color: {c['danger']};
        }}

        /* --- Views & Inputs --- */
        QTreeView, QListView, QPlainTextEdit, QTextEdit, QLineEdit {{
            background-color: {c['bg']};
            color: {c['fg']};
            selection-background-color: {c['sel']};
            selection-color: {c['fg']};
            border: 0;
        }}
        QTreeView::item, QListView::item {{
            padding: 3px 6px;
            border-radius: 4px;
        }}
        QTreeView::item:hover, QListView::item:hover {{
            background-color: {c['bg_hover']};
        }}
        QTreeView::item:selected, QListView::item:selected {{
            background-color: {c['sel']};
            color: {c['fg']};
        }}
        QLineEdit {{
            background-color: {c['bg_alt']};
            border: 1px solid {c['border']};
            border-radius: 5px;
            padding: 5px 8px;
        }}
        QLineEdit:focus {{
            border: 1px solid {c['accent']};
        }}

        /* --- Buttons --- */
        QPushButton {{
            background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {c['bg_panel_hi']}, stop:1 {c['bg_panel']});
            color: {c['fg']};
            border: 1px solid {c['border']};
            border-radius: 6px;
            padding: 5px 14px;
        }}
        QPushButton:hover {{
            background: {c['bg_hover']};
            border: 1px solid {c['accent']};
        }}
        QPushButton:pressed {{
            background: {c['accent_soft']};
        }}
        QPushButton:disabled {{
            color: {c['fg_muted']};
            border-color: {c['border_soft']};
        }}
        QPushButton:default {{
            border: 1px solid {c['accent']};
        }}
        /* Primaer-Buttons: setProperty("accent", True) im Code. Ruhiger
           Verlauf dunkel -> Akzent (statt greller Vollflaeche), helle Schrift. */
        QPushButton[accent="true"] {{
            background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {acc_btn_top}, stop:1 {acc_btn_bot});
            color: #EAFBFB;
            border: 0;
            font-weight: 600;
        }}
        QPushButton[accent="true"]:hover {{
            background: qlineargradient(x1:0, y1:0, x2:0, y2:1,
                stop:0 {acc_btn_top_h}, stop:1 {acc_btn_bot_h});
        }}

        /* Bare QToolButtons ausserhalb einer echten QToolBar (z.B. die
           Eingabemodus-Leiste im Notenblatt-Editor, die eine QHBoxLayout statt
           QToolBar nutzt) -- ohne diese Regel greift nur "QToolBar QToolButton"
           und Hover/Pressed/Checked bleiben unsichtbar (Buttons wirken tot). */
        QToolButton {{
            background-color: transparent;
            color: {c['fg']};
            padding: 5px 11px;
            border-radius: 6px;
            border: 1px solid transparent;
        }}
        QToolButton:hover {{
            background-color: {c['bg_hover']};
            border: 1px solid {c['border']};
        }}
        QToolButton:pressed {{
            background-color: {c['accent_soft']};
        }}
        QToolButton:checked {{
            background-color: {c['accent_soft']};
            border: 1px solid {c['accent']};
        }}
        QToolButton:disabled {{
            color: {c['fg_muted']};
        }}

        /* --- Checkbox / Combo --- */
        QCheckBox {{ spacing: 6px; }}
        QCheckBox::indicator {{
            width: 15px;
            height: 15px;
            border: 1px solid {c['border']};
            background-color: {c['bg_alt']};
            border-radius: 4px;
        }}
        QCheckBox::indicator:hover {{
            border: 1px solid {c['accent']};
        }}
        QCheckBox::indicator:checked {{
            background-color: {c['accent']};
            border: 1px solid {c['accent']};
        }}
        QComboBox {{
            background-color: {c['bg_alt']};
            border: 1px solid {c['border']};
            border-radius: 5px;
            padding: 5px 8px;
        }}
        QComboBox:hover {{
            border: 1px solid {c['accent']};
        }}
        QComboBox QAbstractItemView {{
            background-color: {c['bg_panel']};
            border: 1px solid {c['border']};
            selection-background-color: {c['accent']};
            selection-color: {c['accent_text']};
        }}

        QToolTip {{
            background-color: {c['tooltip_bg']};
            color: {c['tooltip_fg']};
            border: 1px solid {c['accent']};
            padding: 5px 8px;
        }}

        /* --- Scrollbars: Akzent-Hover --- */
        QScrollBar:vertical {{
            background-color: {c['bg']};
            width: 13px;
            margin: 0;
        }}
        QScrollBar::handle:vertical {{
            background-color: {c['bg_hover']};
            min-height: 28px;
            border-radius: 4px;
            margin: 2px;
        }}
        QScrollBar::handle:vertical:hover {{
            background-color: {c['accent']};
        }}
        QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{
            height: 0;
        }}
        QScrollBar:horizontal {{
            background-color: {c['bg']};
            height: 13px;
            margin: 0;
        }}
        QScrollBar::handle:horizontal {{
            background-color: {c['bg_hover']};
            min-width: 28px;
            border-radius: 4px;
            margin: 2px;
        }}
        QScrollBar::handle:horizontal:hover {{
            background-color: {c['accent']};
        }}
        QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal {{
            width: 0;
        }}

        QHeaderView::section {{
            background: {chrome};
            color: {c['fg_muted']};
            padding: 5px;
            border: 0;
            border-bottom: 1px solid {c['border']};
        }}
        QProgressBar {{
            background-color: {c['bg_alt']};
            border: 1px solid {c['border']};
            border-radius: 5px;
            text-align: center;
            color: {c['fg']};
        }}
        QProgressBar::chunk {{
            background: qlineargradient(x1:0, y1:0, x2:1, y2:0,
                stop:0 {c['accent']}, stop:1 {c['success']});
            border-radius: 4px;
        }}

        /* --- SpinBox: getoente Flaeche, Akzent-Fokus, sichtbare Pfeile --- */
        QSpinBox, QDoubleSpinBox {{
            background-color: {c['bg_alt']};
            color: {c['fg']};
            border: 1px solid {c['border']};
            border-radius: 5px;
            padding: 4px 6px;
            selection-background-color: {c['sel']};
        }}
        QSpinBox:hover, QDoubleSpinBox:hover {{
            border: 1px solid {c['accent']};
        }}
        QSpinBox:focus, QDoubleSpinBox:focus {{
            border: 1px solid {c['accent']};
        }}
        QSpinBox:disabled, QDoubleSpinBox:disabled {{
            color: {c['fg_muted']};
            border-color: {c['border_soft']};
        }}
        QSpinBox::up-button, QDoubleSpinBox::up-button {{
            subcontrol-origin: border; subcontrol-position: top right;
            width: 16px; border-left: 1px solid {c['border']};
            border-top-right-radius: 5px; background-color: {c['bg_panel']};
        }}
        QSpinBox::down-button, QDoubleSpinBox::down-button {{
            subcontrol-origin: border; subcontrol-position: bottom right;
            width: 16px; border-left: 1px solid {c['border']};
            border-bottom-right-radius: 5px; background-color: {c['bg_panel']};
        }}
        QSpinBox::up-button:hover, QDoubleSpinBox::up-button:hover,
        QSpinBox::down-button:hover, QDoubleSpinBox::down-button:hover {{
            background-color: {c['accent_soft']};
        }}
        QSpinBox::up-arrow, QDoubleSpinBox::up-arrow {{
            width: 0; height: 0;
            border-left: 4px solid transparent; border-right: 4px solid transparent;
            border-bottom: 5px solid {c['accent']};
        }}
        QSpinBox::down-arrow, QDoubleSpinBox::down-arrow {{
            width: 0; height: 0;
            border-left: 4px solid transparent; border-right: 4px solid transparent;
            border-top: 5px solid {c['accent']};
        }}

        /* --- Slider: dicker Groove, gefuellte Spur, runder Akzent-Griff --- */
        QSlider::groove:horizontal {{
            height: 6px; border-radius: 3px;
            background-color: {c['bg_alt']};
        }}
        QSlider::sub-page:horizontal {{
            height: 6px; border-radius: 3px;
            background-color: {c['accent']};
        }}
        QSlider::add-page:horizontal {{
            height: 6px; border-radius: 3px;
            background-color: {c['bg_alt']};
        }}
        QSlider::handle:horizontal {{
            background-color: {c['accent']};
            border: 2px solid {c['bg']};
            width: 14px; height: 14px; margin: -6px 0; border-radius: 9px;
        }}
        QSlider::handle:horizontal:hover {{
            background-color: {c['accent_hover']};
        }}
        QSlider::groove:vertical {{
            width: 6px; border-radius: 3px; background-color: {c['bg_alt']};
        }}
        QSlider::add-page:vertical {{
            width: 6px; border-radius: 3px; background-color: {c['accent']};
        }}
        QSlider::sub-page:vertical {{
            width: 6px; border-radius: 3px; background-color: {c['bg_alt']};
        }}
        QSlider::handle:vertical {{
            background-color: {c['accent']};
            border: 2px solid {c['bg']};
            width: 14px; height: 14px; margin: 0 -6px; border-radius: 9px;
        }}

        /* --- GroupBox: Karte mit eingelassenem Titel --- */
        QGroupBox {{
            background-color: {c['bg_panel']};
            border: 1px solid {c['border']};
            border-radius: 8px;
            margin-top: 16px;
            padding: 10px 10px 8px 10px;
            font-weight: 600;
        }}
        QGroupBox::title {{
            subcontrol-origin: margin;
            subcontrol-position: top left;
            left: 10px; padding: 1px 8px;
            background-color: {c['accent_soft']};
            color: {c['accent']};
            border-radius: 5px;
        }}
    """


# Schriftart fuer Code-Bereich. Consolas ist auf Windows der Standard,
# fallback auf Courier New.
EDITOR_FONT_FAMILY = "Consolas"
EDITOR_FONT_SIZE = 11
