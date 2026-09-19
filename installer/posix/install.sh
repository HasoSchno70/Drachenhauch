#!/bin/sh
# Installiert die Drachenhauch-IDE fuer den aktuellen Nutzer -- ohne root,
# nach den XDG-Regeln. Aus dem entpackten Ordner aufrufen:
#
#     ./install.sh              installieren (eine alte Fassung wird ersetzt)
#     ./install.sh --entfernen  wieder entfernen (die Beispiele bleiben)
#
# Danach: "Drachenhauch IDE" im Anwendungsmenue, `drachenhauch` und `dhrt`
# im Terminal (liegt ~/.local/bin nicht im PATH, sagt das Skript es),
# .dh-Dateien oeffnen in der IDE.
set -e
HIER="$(cd "$(dirname "$0")" && pwd)"
DATEN="${XDG_DATA_HOME:-$HOME/.local/share}"
ZIEL="$DATEN/drachenhauch-ide"
BIN="$HOME/.local/bin"
APPS="$DATEN/applications"
ICONS="$DATEN/icons/hicolor/256x256/apps"
MIME="$DATEN/mime/packages"

# Nur die EIGENEN Starter in ~/.local/bin anfassen: ein dhrt, das der
# Nutzer selbst dort hingelegt hat, bleibt stehen.
unserer() {
    [ -f "$1" ] && grep -q "drachenhauch-ide" "$1" 2>/dev/null
}

auffrischen() {
    update-mime-database "$DATEN/mime" >/dev/null 2>&1 || true
    update-desktop-database "$APPS" >/dev/null 2>&1 || true
    gtk-update-icon-cache -q "$DATEN/icons/hicolor" >/dev/null 2>&1 || true
}

if [ "$1" = "--entfernen" ]; then
    rm -rf "$ZIEL"
    for f in "$BIN/drachenhauch" "$BIN/dhrt"; do
        if unserer "$f"; then rm -f "$f"; fi
    done
    rm -f "$APPS/drachenhauch-ide.desktop" "$ICONS/drachenhauch-ide.png" "$MIME/drachenhauch-ide.xml"
    auffrischen
    echo "Drachenhauch IDE entfernt. Deine Beispiele (Dokumente/Drachenhauch oder ~/Drachenhauch) bleiben liegen."
    exit 0
fi

if [ "$HIER" = "$ZIEL" ]; then
    echo "Bitte aus dem entpackten Ordner aufrufen, nicht aus $ZIEL."
    exit 1
fi

echo "Installiere nach $ZIEL ..."
rm -rf "$ZIEL"
mkdir -p "$ZIEL" "$BIN" "$APPS" "$ICONS" "$MIME"
cp -R "$HIER/." "$ZIEL/"
chmod 755 "$ZIEL/dhrt" "$ZIEL/drachenhauch" "$ZIEL/install.sh"

# Kleine Skripte statt Verknuepfungen: der Starter sucht seine Dateien neben
# sich, und von einer Verknuepfung aus saehe er ~/.local/bin.
for f in drachenhauch dhrt; do
    if [ -e "$BIN/$f" ] && ! unserer "$BIN/$f"; then
        echo "Hinweis: $BIN/$f gibt es schon und bleibt, wie es ist."
        continue
    fi
    printf '#!/bin/sh\nexec "%s/%s" "$@"\n' "$ZIEL" "$f" > "$BIN/$f"
    chmod 755 "$BIN/$f"
done

if [ -f "$ZIEL/drachenhauch.png" ]; then
    cp "$ZIEL/drachenhauch.png" "$ICONS/drachenhauch-ide.png"
fi

cat > "$MIME/drachenhauch-ide.xml" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-drachenhauch">
    <comment>Drachenhauch-Quelltext</comment>
    <sub-class-of type="text/plain"/>
    <glob pattern="*.dh"/>
  </mime-type>
</mime-info>
EOF

cat > "$APPS/drachenhauch-ide.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Drachenhauch IDE
Comment=Programmieren in Drachenhauch
Exec="$ZIEL/drachenhauch" %f
Icon=drachenhauch-ide
Terminal=false
Categories=Development;IDE;
MimeType=text/x-drachenhauch;
EOF

auffrischen

echo "Fertig: \"Drachenhauch IDE\" im Anwendungsmenue, 'drachenhauch' im Terminal."
case ":$PATH:" in
    *":$BIN:"*) ;;
    *) echo "Hinweis: $BIN steht nicht im PATH -- in ~/.profile ergaenzen:"
       echo "    export PATH=\"\$PATH:$BIN\"" ;;
esac
