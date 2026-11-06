#!/bin/sh
# Simulierte Pins fuer tests/pruef/pins.dhtest -- auf einem gewoehnlichen
# Linux-Rechner ohne Raspberry Pi (die CI ruft es mit sudo-Rechten auf).
#
#   gpio-sim:  ein GPIO-Chip mit 32 Leitungen, Beschriftung "dhsim", Leitung
#              17 heisst "GPIO17". Eingaenge stellt man ueber die Dateien
#              sim_gpioN/pull (pull-up/pull-down), Ausgaenge liest man in
#              sim_gpioN/value.
#   i2c-stub:  ein I2C-Bus mit einem Baustein an 0x50 (256 Register); kann
#              nur SMBus, kein rohes I2C.
#
# Auf stdout stehen NUR Zeilen NAME=WERT fuer $GITHUB_ENV:
#   DH_TEST_GPIO=dhsim, DH_TEST_GPIO_SIM=<Ordner der sim_gpio-Dateien>,
#   DH_TEST_I2C=<Busnummer>
# Was nicht geht, wird auf stderr gemeldet und ausgelassen -- die Faelle
# ueberspringen sich dann selbst.

meld() { echo "pins_simulieren: $*" >&2; }

modul() {
    sudo modprobe "$@" 2>/dev/null && return 0
    # Auf den Azure-Kernen der GitHub-Laeufer liegen die Simulatoren in
    # linux-modules-extra.
    sudo apt-get install -y "linux-modules-extra-$(uname -r)" >&2 2>/dev/null || true
    sudo modprobe "$@"
}

# ---------------------------------------------------------------- GPIO
if modul gpio-sim; then
    mountpoint -q /sys/kernel/config || sudo mount -t configfs none /sys/kernel/config
    K=/sys/kernel/config/gpio-sim/dh
    if sudo mkdir -p "$K/bank0" \
        && echo 32 | sudo tee "$K/bank0/num_lines" >/dev/null \
        && echo dhsim | sudo tee "$K/bank0/label" >/dev/null \
        && sudo mkdir -p "$K/bank0/line17" \
        && echo GPIO17 | sudo tee "$K/bank0/line17/name" >/dev/null \
        && echo 1 | sudo tee "$K/live" >/dev/null; then
        dev=$(cat "$K/dev_name")
        chip=$(cat "$K/bank0/chip_name")
        sim="/sys/devices/platform/$dev/$chip"
        sudo chmod a+rw "/dev/$chip"
        sudo chmod a+w "$sim"/sim_gpio*/pull
        meld "gpio-sim: /dev/$chip, $sim"
        echo "DH_TEST_GPIO=dhsim"
        echo "DH_TEST_GPIO_SIM=$sim"
    else
        meld "gpio-sim liess sich nicht einrichten"
    fi
else
    meld "gpio-sim gibt es nicht"
fi

# ---------------------------------------------------------------- I2C
if modul i2c-dev && modul i2c-stub chip_addr=0x50; then
    for d in /sys/class/i2c-dev/i2c-*; do
        [ -e "$d/name" ] || continue
        if grep -q "SMBus stub" "$d/name"; then
            n=${d##*/i2c-}
            sudo chmod a+rw "/dev/i2c-$n"
            meld "i2c-stub: /dev/i2c-$n"
            echo "DH_TEST_I2C=$n"
            break
        fi
    done
else
    meld "i2c-stub gibt es nicht"
fi
exit 0
