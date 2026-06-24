"""Native Datei-/Ordner-Dialoge: nur Registrierung.

FILE_OPEN_DIALOG / FILE_SAVE_DIALOG / FOLDER_DIALOG sind blockierende native
OS-Dialoge (rfd, ans graphics-Feature gekoppelt) -- kein Funktionstest via
run_gb moeglich (modaler Dialog, braucht Nutzer-Interaktion). Live verifiziert
ueber examples/127_filedialog.gb. Hier wird geprueft, dass sie im eingefrorenen
gbrt-Index stehen -- sonst warnt der Editor live und der Drift-Test schlaegt an.
"""
from gamebasic.editor_qt.gbrt_meta import builtin_names_lower


def test_filedialog_builtins_registered():
    n = builtin_names_lower()
    for name in ("file_open_dialog", "file_save_dialog", "folder_dialog"):
        assert name in n, name
