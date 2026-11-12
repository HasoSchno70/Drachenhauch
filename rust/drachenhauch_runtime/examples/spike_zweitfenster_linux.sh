#!/bin/bash
# Spike zu Weg E unter Linux (WSLg): starten, zwei echte Klicks ins zweite
# Fenster (xdotool), Schliessen ueber den Fenstermanager (wmctrl).
E=${SPIKE_EXE:-/root/dht_spike/release/examples/spike_zweitfenster}
export SPIKE_BILDER=${SPIKE_BILDER:-480}
export SPIKE_AUSSEN=1
"$E" > /tmp/spike.log 2>&1 &
P=$!
W=$(timeout 20 xdotool search --sync --name "Spike: zweites Fenster" | head -1)
echo "fenster ${W:-nicht gefunden}"
if [ -n "$W" ]; then
    sleep 1.5
    xdotool mousemove --window "$W" 50 60 click 1
    sleep 1
    xdotool mousemove --window "$W" 60 70 click 1
    sleep 1
    wmctrl -c "Spike: zweites Fenster"
fi
wait $P
echo "exit $?"
grep -E '^ERGEBNIS|^  bild|GL-Fehler' /tmp/spike.log
