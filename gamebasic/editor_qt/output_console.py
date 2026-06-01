"""Output-Konsole + Run-Subprocess (`QProcess`).

Liest stdout des Run-Subprocesses asynchron, faerbt Info-/Error-/User-
Eingaben farbig und verlinkt `Datei.gb:N` in den Editor (via Signal
`jump_requested`). Die Eingabezeile leitet User-Input an stdin weiter --
fuer GB-Programme mit `INPUT`-Statement.
"""
from __future__ import annotations

import os
import re
import sys
from pathlib import Path

from PySide6.QtCore import QProcess, Qt, Signal
from PySide6.QtGui import (
    QColor, QFont, QKeySequence, QMouseEvent, QShortcut, QTextCharFormat,
    QTextCursor, QTextDocument,
)
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QLineEdit, QPlainTextEdit, QPushButton,
    QVBoxLayout, QWidget,
)

from .theme import COLORS, EDITOR_FONT_FAMILY, EDITOR_FONT_SIZE, theme_signals


# Regexes fuer Linkable-Output
#
# `_LINK_FILE_HEADER` matched den `Fehler in <datei>:`-Block, den
# `gamebasic.__main__` bei einem Top-Level-Fehler ausgibt. Datei-Pfade
# duerfen hier Spaces enthalten (path.name kann z.B. "mein sprite.gb"
# sein), darum `[^\n:]+?` lazy bis zum naechsten `:` statt `\S+`.
#
# `_LINK_FILE_LINE` matched freistehende `<datei>.gb:<zeile>`-Vorkommen
# im Output (z.B. Tracebacks). Hier bleiben wir strict bei `\S+`, weil
# Tracebacks selten Pfade mit Spaces enthalten -- ein zu offener Regex
# wuerde mehr falsch klassifizieren als richtig.
_LINK_LINE = re.compile(r"\[Zeile (\d+)\]")
_LINK_FILE_HEADER = re.compile(r"Fehler in ([^\n:]+?\.gb):")
_LINK_FILE_LINE = re.compile(r"(\S+\.gb):(\d+)")


def _find_gbrt(project_root: Path) -> Path | None:
    """Sucht das gebaute `gbrt`-Binary der nativen Runtime (Release bevorzugt).
    Spiegelt `gbrun._find_gbrt` -- bewusst dupliziert, damit der Editor nicht
    das Top-Level-Script `gbrun.py` importieren muss."""
    base = project_root / "rust" / "gb_runtime" / "target"
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


class _LinkableText(QPlainTextEdit):
    """`QPlainTextEdit`, das Anker-Klicks (AnchorHref) als Signal emittiert."""

    link_clicked = Signal(str)

    def mousePressEvent(self, event: QMouseEvent):  # noqa: N802
        if event.button() == Qt.MouseButton.LeftButton:
            cursor = self.cursorForPosition(event.pos())
            fmt = cursor.charFormat()
            href = fmt.anchorHref()
            if href:
                self.link_clicked.emit(href)
                return
        super().mousePressEvent(event)

    def mouseMoveEvent(self, event: QMouseEvent):  # noqa: N802
        cursor = self.cursorForPosition(event.pos())
        href = cursor.charFormat().anchorHref()
        if href:
            self.viewport().setCursor(Qt.CursorShape.PointingHandCursor)
        else:
            self.viewport().setCursor(Qt.CursorShape.IBeamCursor)
        super().mouseMoveEvent(event)


class OutputConsole(QWidget):
    """Konsole mit `Run`-Subprozess-Lifecycle.

    Signale:
        jump_requested(file_name_or_None, line)
            -- User klickt auf Link in der Konsole oder Output enthaelt
               eine erkannte Fehler-Adresse, die anklickbar gemacht wird.
        process_finished(returncode)
            -- Run-Subprozess hat sich beendet (oder wurde abgebrochen).
        process_started()
            -- Run-Subprozess steht.
    """

    jump_requested = Signal(object, int)
    process_finished = Signal(int)
    process_started = Signal()

    def __init__(self, project_root: Path, parent: QWidget | None = None):
        super().__init__(parent)
        self.project_root = project_root
        self._proc: QProcess | None = None
        self._user_stopped = False
        self._current_run_file: Path | None = None
        # Temporaere .gbc des aktuellen Native-Runs (nach Finish geloescht).
        self._native_gbc: Path | None = None

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Header
        self._header = QFrame()
        hl = QHBoxLayout(self._header)
        hl.setContentsMargins(10, 4, 6, 4)
        self._title = QLabel("Konsole")
        hl.addWidget(self._title)
        hl.addStretch(1)
        find_btn = QPushButton("Suchen")
        find_btn.clicked.connect(self._toggle_find_bar)
        hl.addWidget(find_btn)
        clear_btn = QPushButton("Leeren")
        clear_btn.clicked.connect(self.clear)
        hl.addWidget(clear_btn)
        layout.addWidget(self._header)

        # Suchen-Leiste (default versteckt). Wird ueber Strg+F oder Header-
        # Button eingeblendet; Esc oder Klick auf X versteckt sie wieder.
        self._find_bar = QFrame()
        fl = QHBoxLayout(self._find_bar)
        fl.setContentsMargins(10, 2, 6, 2)
        fl_label = QLabel("Suchen:")
        fl.addWidget(fl_label)
        self.find_entry = QLineEdit()
        self.find_entry.setPlaceholderText("im Output suchen ...")
        self.find_entry.returnPressed.connect(lambda: self._find_in_output(forward=True))
        fl.addWidget(self.find_entry, 1)
        prev_btn = QPushButton("Zurueck")
        prev_btn.clicked.connect(lambda: self._find_in_output(forward=False))
        fl.addWidget(prev_btn)
        next_btn = QPushButton("Weiter")
        next_btn.clicked.connect(lambda: self._find_in_output(forward=True))
        fl.addWidget(next_btn)
        close_find_btn = QPushButton("X")
        close_find_btn.setFixedWidth(24)
        close_find_btn.clicked.connect(self._hide_find_bar)
        fl.addWidget(close_find_btn)
        self._find_bar.setVisible(False)
        layout.addWidget(self._find_bar)

        # Text-Area
        self.text = _LinkableText()
        self.text.setReadOnly(True)
        self.text.setMouseTracking(True)
        font = QFont(EDITOR_FONT_FAMILY, EDITOR_FONT_SIZE)
        font.setStyleHint(QFont.StyleHint.Monospace)
        self.text.setFont(font)
        self.text.link_clicked.connect(self._on_link_clicked)
        layout.addWidget(self.text, 1)

        # Eingabe-Zeile fuer INPUT-Statements
        self._input_row = QFrame()
        il = QHBoxLayout(self._input_row)
        il.setContentsMargins(8, 2, 8, 4)
        self._prompt = QLabel(">")
        il.addWidget(self._prompt)
        self.input_entry = QLineEdit()
        self.input_entry.setFont(font)
        self.input_entry.setPlaceholderText("(kein Programm aktiv)")
        self.input_entry.setEnabled(False)
        self.input_entry.returnPressed.connect(self._send_user_input)
        il.addWidget(self.input_entry, 1)
        layout.addWidget(self._input_row)

        # Strg+F oeffnet die Suchen-Leiste, Esc schliesst sie. Beide nur
        # aktiv wenn der Konsolen-Bereich Fokus hat.
        sc_find = QShortcut(QKeySequence(QKeySequence.StandardKey.Find), self)
        sc_find.setContext(Qt.ShortcutContext.WidgetWithChildrenShortcut)
        sc_find.activated.connect(self._toggle_find_bar)
        sc_esc = QShortcut(QKeySequence("Esc"), self.find_entry)
        sc_esc.activated.connect(self._hide_find_bar)
        sc_esc_text = QShortcut(QKeySequence("Esc"), self.text)
        sc_esc_text.activated.connect(self._hide_find_bar)

        self._apply_style()
        theme_signals.changed.connect(self._on_theme_changed)

    # ------------------------------------------- Suchen-Leiste
    def _toggle_find_bar(self) -> None:
        if self._find_bar.isVisible():
            self._hide_find_bar()
        else:
            self._find_bar.setVisible(True)
            self.find_entry.setFocus()
            self.find_entry.selectAll()

    def _hide_find_bar(self) -> None:
        self._find_bar.setVisible(False)
        # Default-Highlights weg.
        cur = self.text.textCursor()
        cur.clearSelection()
        self.text.setTextCursor(cur)

    def _find_in_output(self, *, forward: bool = True) -> None:
        q = self.find_entry.text()
        if not q:
            return
        flags = QTextDocument.FindFlag(0)
        if not forward:
            flags |= QTextDocument.FindFlag.FindBackward
        found = self.text.find(q, flags)
        if not found:
            # Wrap-around: vom Anfang/Ende neu suchen.
            cur = self.text.textCursor()
            cur.movePosition(
                QTextCursor.MoveOperation.End if not forward else QTextCursor.MoveOperation.Start
            )
            self.text.setTextCursor(cur)
            self.text.find(q, flags)

    def _apply_style(self) -> None:
        c = COLORS
        self._header.setStyleSheet(f"background-color: {c['bg_panel']};")
        self._title.setStyleSheet(f"color: {c['fg']}; font-weight: bold;")
        self.text.setStyleSheet(
            f"background-color: {c['bg']}; color: {c['fg']}; "
            f"selection-background-color: {c['sel']};"
        )
        self._input_row.setStyleSheet(f"background-color: {c['bg_alt']};")
        self._prompt.setStyleSheet(f"color: {c['fg_muted']}; font-weight: bold;")
        self._find_bar.setStyleSheet(f"background-color: {c['bg_alt']};")

    def _on_theme_changed(self, _name: str) -> None:
        self._apply_style()

    # ----------------------------------------------- Output-Schreiben
    def append(self, msg: str, kind: str | None = None) -> None:
        """`kind` in {None, "info", "error", "muted", "user_input"}."""
        cursor = self.text.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        fmt = self._format_for(kind)
        cursor.setCharFormat(fmt)
        cursor.insertText(msg)
        # Links in der gerade eingefuegten Region anlegen.
        self._apply_links(cursor.position() - len(msg), msg)
        cursor.movePosition(QTextCursor.MoveOperation.End)
        self.text.setTextCursor(cursor)
        self.text.ensureCursorVisible()

    def clear(self) -> None:
        self.text.clear()

    @staticmethod
    def _format_for(kind: str | None) -> QTextCharFormat:
        f = QTextCharFormat()
        c = COLORS
        mapping = {
            "info":       c["info"],
            "error":      c["error"],
            "muted":      c["line_no"],
            "user_input": c["accent"],
        }
        f.setForeground(QColor(mapping.get(kind or "", c["fg"])))
        return f

    def _apply_links(self, base_pos: int, added: str) -> None:
        """Faerbt erkannte File:Line-Adressen blau+unterstrichen.

        Click-Handling: ueberschreiben wir global via `mousePressEvent` --
        dort lesen wir die anchorAt-Position. Anstatt echte HTML-Anker zu
        setzen (was QPlainTextEdit nicht hat), legen wir ein Mapping
        von (start, end) -> Ziel ab.
        """
        for m in _LINK_LINE.finditer(added):
            self._mark_link(
                base_pos + m.start(), base_pos + m.end(),
                self._current_run_file, int(m.group(1)),
            )
        for m in _LINK_FILE_HEADER.finditer(added):
            self._mark_link(
                base_pos + m.start(1), base_pos + m.end(1),
                m.group(1), 1,
            )
        for m in _LINK_FILE_LINE.finditer(added):
            ctx_start = max(0, m.start() - 12)
            if "Fehler in" in added[ctx_start:m.start()]:
                continue
            self._mark_link(
                base_pos + m.start(), base_pos + m.end(),
                m.group(1), int(m.group(2)),
            )

    def _mark_link(self, start: int, end: int, file, line: int) -> None:
        cursor = QTextCursor(self.text.document())
        cursor.setPosition(start)
        cursor.setPosition(end, QTextCursor.MoveMode.KeepAnchor)
        fmt = QTextCharFormat()
        fmt.setForeground(QColor(COLORS["link"]))
        fmt.setFontUnderline(True)
        # Ziel als Property speichern -- pro Link eindeutig.
        target = f"{file}|{line}" if file else f"|{line}"
        fmt.setProperty(QTextCharFormat.Property.AnchorHref, target)
        fmt.setAnchor(True)
        fmt.setAnchorHref(target)
        cursor.mergeCharFormat(fmt)

    def _on_link_clicked(self, href: str) -> None:
        # Format: "<file>|<line>"  oder  "|<line>" (gleiche Datei).
        if "|" not in href:
            return
        file_part, line_part = href.split("|", 1)
        try:
            line = int(line_part)
        except ValueError:
            return
        target = file_part if file_part else None
        self.jump_requested.emit(target, line)

    # ------------------------------------------------ Run-Lifecycle
    def is_running(self) -> bool:
        return self._proc is not None and self._proc.state() != QProcess.ProcessState.NotRunning

    def start_run(self, file_path: Path, extra_args: list[str] | None = None) -> bool:
        if self.is_running():
            return False
        self._user_stopped = False
        self._current_run_file = file_path
        self.clear()
        if extra_args and "--bench" in extra_args:
            self.append(f"⚡ Benchmark: {file_path.name}\n\n", "info")
        else:
            self.append(f"▶ Wird ausgefuehrt: {file_path.name}\n\n", "info")

        venv_python = (
            self.project_root / ".venv" / "Scripts" / "python.exe"
            if os.name == "nt"
            else self.project_root / ".venv" / "bin" / "python"
        )
        if not venv_python.exists():
            venv_python = Path(sys.executable)

        proc = QProcess(self)
        proc.setProcessChannelMode(QProcess.ProcessChannelMode.MergedChannels)
        proc.setWorkingDirectory(str(self.project_root))
        proc.readyReadStandardOutput.connect(self._on_stdout)
        proc.finished.connect(self._on_finished)
        proc.errorOccurred.connect(self._on_error)

        args = [str(self.project_root / "gbrun.py")]
        if extra_args:
            args.extend(extra_args)
        args.append(str(file_path))
        proc.start(str(venv_python), args)
        if not proc.waitForStarted(3000):
            self.append("Konnte Programm nicht starten.\n", "error")
            return False
        self._proc = proc
        self.input_entry.setEnabled(True)
        self.input_entry.setPlaceholderText("Eingabe (Enter sendet) ...")
        self.input_entry.setFocus()
        self.process_started.emit()
        return True

    def start_run_native(self, file_path: Path) -> bool:
        """Kompiliert die Datei nach `.gbc` (Python) und fuehrt sie mit der
        nativen Runtime `gbrt` aus -- direkt als QProcess, damit der Stop-Button
        gbrt selbst beendet (kein verwaister Prozess wie bei gbrun.py --native).

        Der Quell-Dateiname wird als 2. Arg an gbrt durchgereicht, sodass
        Laufzeitfehler `datei.gb:Zeile` zeigen. Arbeitsverzeichnis ist der
        Ordner der Quelldatei (relative Asset-Pfade)."""
        if self.is_running():
            return False

        gbrt = _find_gbrt(self.project_root)
        if gbrt is None:
            self.clear()
            self.append("Native Runtime 'gbrt' nicht gefunden.\n", "error")
            self.append("Einmalig bauen mit:\n", "muted")
            self.append("  .venv\\Scripts\\python.exe rust\\build_runtime.py\n",
                        "muted")
            return False

        # In temporaere .gbc kompilieren (Compile-Fehler sauber melden).
        import tempfile
        from gamebasic.errors import GameBasicError
        fd, tmp = tempfile.mkstemp(suffix=".gbc")
        os.close(fd)
        tmp_path = Path(tmp)
        self.clear()
        # Schon vor dem Kompilieren setzen, damit der `[Zeile N]`-Link einer
        # Compile-Fehlermeldung auf diese Datei springt (statt ins Leere).
        self._current_run_file = file_path
        self.append(f"▶ Nativ (gbrt): {file_path.name}\n\n", "info")
        try:
            from gamebasic.serialize import compile_file_to_gbc
            compile_file_to_gbc(file_path, tmp_path)
        except GameBasicError as exc:
            self.append(f"Compile-Fehler in {file_path.name}:\n  {exc}\n", "error")
            tmp_path.unlink(missing_ok=True)
            return False
        except Exception as exc:  # pragma: no cover - defensiv
            self.append(f"Compile-Fehler in {file_path.name}:\n  {exc}\n", "error")
            tmp_path.unlink(missing_ok=True)
            return False

        self._user_stopped = False
        self._native_gbc = tmp_path

        proc = QProcess(self)
        proc.setProcessChannelMode(QProcess.ProcessChannelMode.MergedChannels)
        # Arbeitsverzeichnis = Quell-Ordner, damit LOADIMAGE("assets/...") greift.
        proc.setWorkingDirectory(str(file_path.parent))
        proc.readyReadStandardOutput.connect(self._on_stdout)
        proc.finished.connect(self._on_finished)
        proc.errorOccurred.connect(self._on_error)
        proc.start(str(gbrt), [str(tmp_path), file_path.name])
        if not proc.waitForStarted(3000):
            self.append("Konnte gbrt nicht starten.\n", "error")
            tmp_path.unlink(missing_ok=True)
            self._native_gbc = None
            return False
        self._proc = proc
        self.input_entry.setEnabled(True)
        self.input_entry.setPlaceholderText("Eingabe (Enter sendet) ...")
        self.input_entry.setFocus()
        self.process_started.emit()
        return True

    def stop_run(self) -> None:
        if not self.is_running():
            return
        self._user_stopped = True
        assert self._proc is not None
        self._proc.terminate()
        if not self._proc.waitForFinished(2000):
            self._proc.kill()

    def _on_stdout(self) -> None:
        if self._proc is None:
            return
        data = bytes(self._proc.readAllStandardOutput())
        text = data.decode("utf-8", errors="replace")
        if text:
            self.append(text)

    def _on_finished(self, exit_code: int, _exit_status) -> None:
        # Restliches stdout abholen -- `finished` kann vor dem letzten
        # `readyReadStandardOutput` feuern.
        if self._proc is not None:
            data = bytes(self._proc.readAllStandardOutput())
            if data:
                self.append(data.decode("utf-8", errors="replace"))
        if self._user_stopped:
            self.append("\n■ Abgebrochen.\n", "muted")
        elif exit_code == 0:
            self.append("\n✓ Fertig.\n", "info")
        else:
            self.append(f"\n✗ Programm beendet mit Fehler-Code {exit_code}.\n", "error")
        self._proc = None
        # Temporaere .gbc eines Native-Runs aufraeumen.
        if self._native_gbc is not None:
            self._native_gbc.unlink(missing_ok=True)
            self._native_gbc = None
        self.input_entry.setEnabled(False)
        self.input_entry.setPlaceholderText("(kein Programm aktiv)")
        self.input_entry.clear()
        self.process_finished.emit(exit_code)

    def _on_error(self, _err) -> None:
        # Kommt z.B. bei "Datei nicht gefunden". Abschluss kommt parallel
        # ueber `finished` -- hier nichts zu tun.
        return

    def _send_user_input(self) -> None:
        if not self.is_running():
            return
        line = self.input_entry.text()
        self.input_entry.clear()
        self.append(line + "\n", "user_input")
        assert self._proc is not None
        self._proc.write((line + "\n").encode("utf-8"))
