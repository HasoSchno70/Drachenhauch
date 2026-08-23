"""Git-Hilfen fuer den Editor (Blame).

Kapselt `git`-Subprozess-Aufrufe + Parsing. Der Porcelain-Parser ist von
der I/O getrennt (`parse_porcelain`), damit er ohne echtes Repo testbar ist.

Kein hartes `git`-Dependency: fehlt `git` oder ist die Datei nicht in einem
Repo/getrackt, liefert `blame()` ein Ergebnis mit `ok=False` + Meldung.
"""
from __future__ import annotations

import datetime as _dt
import subprocess
from dataclasses import dataclass
from pathlib import Path


@dataclass
class BlameLine:
    line: int            # finale Zeilennummer (1-basiert)
    sha: str             # Kurz-Hash (8) oder "" bei uncommitted
    author: str
    date: str            # "YYYY-MM-DD" in der Commit-Zeitzone
    summary: str
    uncommitted: bool = False


@dataclass
class BlameResult:
    ok: bool = False
    error: str = ""
    lines: list | None = None   # list[BlameLine]

    def __post_init__(self):
        if self.lines is None:
            self.lines = []


def _fmt_date(epoch: int, tz: str) -> str:
    """Epoch + tz ("+0200") -> Datum in der Commit-Zeitzone."""
    off = 0
    if tz and len(tz) == 5 and tz[0] in "+-":
        try:
            h, m = int(tz[1:3]), int(tz[3:5])
            off = (h * 60 + m) * 60 * (1 if tz[0] == "+" else -1)
        except ValueError:
            off = 0
    try:
        return _dt.datetime.fromtimestamp(
            epoch + off, _dt.timezone.utc).strftime("%Y-%m-%d")
    except (OverflowError, OSError, ValueError):
        return "?"


def parse_porcelain(text: str) -> list[BlameLine]:
    """Parst die Ausgabe von `git blame --porcelain` zu BlameLine-Liste.

    Commit-Header (author/summary/...) erscheinen nur beim ersten Auftreten
    eines Hashes -> Metadaten pro Hash cachen."""
    meta: dict[str, dict] = {}
    out: list[BlameLine] = []
    cur_sha = ""          # der zuletzt gelesene Hash; vor der ersten
                          # Inhaltszeile hat git ihn immer geliefert
    cur_final = 0
    expect_content = False
    for raw in text.split("\n"):
        if expect_content and raw.startswith("\t"):
            m = meta.setdefault(cur_sha, {})
            zero = set(cur_sha) <= {"0"}
            out.append(BlameLine(
                line=cur_final,
                sha="" if zero else cur_sha[:8],
                author=m.get("author", ""),
                date=_fmt_date(m.get("author-time", 0), m.get("author-tz", "")),
                summary=m.get("summary", ""),
                uncommitted=zero,
            ))
            expect_content = False
            continue
        parts = raw.split(" ")
        if len(parts) >= 3 and len(parts[0]) == 40 and _is_hex(parts[0]):
            # "<sha> <orig> <final> [<n>]"
            cur_sha = parts[0]
            try:
                cur_final = int(parts[2])
            except ValueError:
                cur_final = cur_final + 1
            meta.setdefault(cur_sha, {})
            expect_content = True
        elif cur_sha is not None:
            if raw.startswith("author "):
                meta.setdefault(cur_sha, {})["author"] = raw[len("author "):]
            elif raw.startswith("author-time "):
                try:
                    meta[cur_sha]["author-time"] = int(raw[len("author-time "):])
                except ValueError:
                    pass
            elif raw.startswith("author-tz "):
                meta[cur_sha]["author-tz"] = raw[len("author-tz "):]
            elif raw.startswith("summary "):
                meta[cur_sha]["summary"] = raw[len("summary "):]
    return out


def _is_hex(s: str) -> bool:
    try:
        int(s, 16)
        return True
    except ValueError:
        return False


def _run_git(args, cwd) -> tuple[int, str, str]:
    try:
        from .dhrt_locate import NO_WINDOW
        p = subprocess.run(
            ["git", *args], cwd=str(cwd), capture_output=True, text=True,
            encoding="utf-8", errors="replace", timeout=20,
            creationflags=NO_WINDOW,
        )
        return p.returncode, p.stdout, p.stderr
    except FileNotFoundError:
        return 127, "", "git nicht gefunden"
    except (OSError, subprocess.SubprocessError) as exc:
        # Nicht mehr blankes `except Exception` -- das haette auch echte
        # Programmierfehler (z.B. TypeError bei falschem `args`) stumm als
        # "Git-Fehler" maskiert statt sie als Bug sichtbar zu lassen.
        return 1, "", str(exc)


def is_git_repo(path) -> bool:
    """`path` muss bereits ein VERZEICHNIS sein (kein Datei-vs-Verzeichnis-
    Raten hier). Review-Fund: rief bisher zusaetzlich `_dir_of(path)` auf --
    der einzige Aufrufer (`blame()`) uebergibt aber schon `p.parent` (also
    bereits ein Verzeichnis). `_dir_of` wendete seine Datei-vs-Verzeichnis-
    Heuristik (".suffix vorhanden -> muss eine Datei sein -> eine Ebene
    hoch") dann ein ZWEITES Mal an -- hiess der Verzeichnisname selbst
    einen Punkt (z.B. "examples.bak/", "v1.2/"), landete die Pruefung
    faelschlich im GROSSELTERN-Verzeichnis statt im echten Parent."""
    rc, out, _ = _run_git(["rev-parse", "--is-inside-work-tree"], Path(path))
    return rc == 0 and out.strip() == "true"


def _dir_of(path) -> Path:
    p = Path(path)
    return p.parent if p.suffix else p


def blame(file_path) -> BlameResult:
    """Blame fuer eine Datei (Arbeitsbaum-Version, inkl. uncommitted Zeilen)."""
    p = Path(file_path)
    if not p.exists():
        return BlameResult(ok=False, error="Datei existiert nicht")
    cwd = p.parent
    if not is_git_repo(cwd):
        return BlameResult(ok=False, error="kein Git-Repository")
    rc, out, err = _run_git(
        ["blame", "--porcelain", "--", p.name], cwd)
    if rc != 0:
        msg = err.strip() or "git blame fehlgeschlagen"
        # Haeufigster Fall: Datei nicht getrackt.
        if "no such path" in msg.lower() or "not tracked" in msg.lower():
            msg = "Datei nicht im Repository (uncommitted)"
        return BlameResult(ok=False, error=msg)
    return BlameResult(ok=True, lines=parse_porcelain(out))
