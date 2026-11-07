#!/bin/sh
# Eine simulierte Kamera fuer tests/pruef/geraete/webcam.dhtest -- auf einem
# gewoehnlichen Linux-Rechner ohne Webcam (der Paketlauf ruft es mit
# sudo-Rechten auf).
#
#   vivid:  der Testtreiber des Kernels ("Virtual Video Test Driver"); er
#           legt mehrere /dev/video-Knoten an, einer davon nimmt Bilder auf
#           (Farbbalken, YUYV). Gelesen wird er ueber dieselben ioctls wie
#           eine echte Kamera.
#
# Auf stdout steht NUR eine Zeile NAME=WERT fuer $GITHUB_ENV:
#   DH_TEST_WEBCAM=vivid   (WEBCAM_OPEN findet die Karte ueber diesen Teiltext)
# Was nicht geht, wird auf stderr gemeldet -- die Faelle ueberspringen sich
# dann selbst.

meld() { echo "kamera_simulieren: $*" >&2; }

modul() {
    sudo modprobe "$@" 2>/dev/null && return 0
    # Auf den Azure-Kernen der GitHub-Laeufer liegen die Testtreiber in
    # linux-modules-extra.
    sudo apt-get install -y "linux-modules-extra-$(uname -r)" >&2 2>/dev/null || true
    sudo modprobe "$@"
}

if ! modul vivid n_devs=1 node_types=0x1; then
    meld "vivid laesst sich nicht laden -- die Kamera-Faelle ueberspringen sich"
    exit 0
fi
sleep 1
for dev in /dev/video*; do
    [ -e "$dev" ] || continue
    sudo chmod a+rw "$dev"
    name=$(cat "/sys/class/video4linux/$(basename "$dev")/name" 2>/dev/null)
    meld "$dev: $name"
    case "$name" in
        vivid*)
            echo "DH_TEST_WEBCAM=vivid"
            exit 0
            ;;
    esac
done
meld "vivid geladen, aber kein Aufnahme-Knoten gefunden"
exit 0
