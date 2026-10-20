"""Gegenpruefung des Gerber-Exports, unabhaengig von Funkenflug.

    python gerber_pruefen.py platine.zip platine.erwartet [bild.png]

Liest die Gerber-Lagen und die Excellon-Bohrdatei, rechnet aus der GEOMETRIE
(sich beruehrende Augen und Bahnen je Lage, durchkontaktierte Bohrungen,
Drahtbruecken aus der .erwartet-Datei) die Kupferinseln aus und vergleicht
sie mit den Netzen, die Funkenflug je Loch nennt:

  * jedes Loch eines Bauteils ist gebohrt und hat oben und unten ein Auge,
  * zwei Loecher im selben Netz liegen auf derselben Insel (nichts fehlt),
  * zwei Loecher verschiedener Netze nie (kein Kurzschluss),
  * Kupfer haelt Abstand zum Rand, alles liegt im Umriss.

Mit bild.png wird ausserdem eine Ansicht von oben gezeichnet (braucht Pillow).
"""
import math
import re
import sys
import zipfile


def gerber_lesen(text):
    """-> (blenden {code: durchmesser}, blitze [(x,y,d)], linien [(x1,y1,x2,y2,d)], attribute)"""
    blenden, blitze, linien, attr = {}, [], [], []
    fs = None
    aktiv = None
    x = y = 0.0
    for teil in re.findall(r'%[^%]*%|[^*%]+\*', text):
        teil = teil.strip()
        if teil.startswith('%'):
            inhalt = teil.strip('%').rstrip('*')
            m = re.match(r'FSLAX(\d)(\d)Y(\d)(\d)', inhalt)
            if m:
                fs = int(m.group(2))
            m = re.match(r'ADD(\d+)C,([\d.]+)', inhalt)
            if m:
                blenden[int(m.group(1))] = float(m.group(2))
            if inhalt.startswith('TF.'):
                attr.append(inhalt)
            if inhalt.startswith('MO') and inhalt != 'MOMM':
                raise ValueError('nicht mm: ' + inhalt)
            continue
        b = teil.rstrip('*').strip()
        if not b or b.startswith('G04'):
            continue
        m = re.fullmatch(r'D(\d+)', b)
        if m and int(m.group(1)) >= 10:
            aktiv = int(m.group(1))
            if aktiv not in blenden:
                raise ValueError('Blende D%d vor ihrer Definition benutzt' % aktiv)
            continue
        if b in ('G01', 'M02', 'G75', 'G74'):
            continue
        m = re.fullmatch(r'(?:X(-?\d+))?(?:Y(-?\d+))?D0([123])', b)
        if not m:
            raise ValueError('unbekannter Befehl: ' + b)
        if fs is None:
            raise ValueError('Koordinaten vor %FS')
        nx = int(m.group(1)) / 10 ** fs if m.group(1) is not None else x
        ny = int(m.group(2)) / 10 ** fs if m.group(2) is not None else y
        op = m.group(3)
        if op != '2' and aktiv is None:
            raise ValueError('Zeichnen ohne Blende')
        if op == '1':
            linien.append((x, y, nx, ny, blenden[aktiv]))
        elif op == '3':
            blitze.append((nx, ny, blenden[aktiv]))
        x, y = nx, ny
    if not text.rstrip().endswith('M02*'):
        raise ValueError('kein M02 am Ende')
    return blenden, blitze, linien, attr


def bohrungen_lesen(text):
    werkzeuge, loecher = {}, []
    aktiv = None
    kopf = True
    for z in text.splitlines():
        z = z.strip()
        if kopf:
            m = re.fullmatch(r'T(\d+)C([\d.]+)', z)
            if m:
                werkzeuge[int(m.group(1))] = float(m.group(2))
            if z == '%':
                kopf = False
            continue
        m = re.fullmatch(r'T(\d+)', z)
        if m:
            aktiv = int(m.group(1))
            continue
        m = re.fullmatch(r'X(-?[\d.]+)Y(-?[\d.]+)', z)
        if m:
            loecher.append((float(m.group(1)), float(m.group(2)), werkzeuge[aktiv]))
    if 'METRIC' not in text or not text.rstrip().endswith('M30'):
        raise ValueError('Bohrdatei: METRIC/M30 fehlt')
    return loecher


def abstand_punkt_strecke(px, py, x1, y1, x2, y2):
    dx, dy = x2 - x1, y2 - y1
    l2 = dx * dx + dy * dy
    t = 0.0 if l2 == 0 else max(0.0, min(1.0, ((px - x1) * dx + (py - y1) * dy) / l2))
    return math.hypot(px - (x1 + t * dx), py - (y1 + t * dy))


def strecken_abstand(a, b):
    # zwei Strecken (achsparallel oder beliebig): kleinster Endpunkt-Abstand
    # genuegt hier nicht bei Kreuzungen -> Schnitt pruefen
    (ax1, ay1, ax2, ay2), (bx1, by1, bx2, by2) = a, b

    def kreuz(ox, oy, px, py, qx, qy):
        return (px - ox) * (qy - oy) - (py - oy) * (qx - ox)
    d1 = kreuz(bx1, by1, bx2, by2, ax1, ay1)
    d2 = kreuz(bx1, by1, bx2, by2, ax2, ay2)
    d3 = kreuz(ax1, ay1, ax2, ay2, bx1, by1)
    d4 = kreuz(ax1, ay1, ax2, ay2, bx2, by2)
    if ((d1 > 0) != (d2 > 0)) and ((d3 > 0) != (d4 > 0)) and d1 and d2 and d3 and d4:
        return 0.0
    return min(abstand_punkt_strecke(ax1, ay1, *b), abstand_punkt_strecke(ax2, ay2, *b),
               abstand_punkt_strecke(bx1, by1, *a), abstand_punkt_strecke(bx2, by2, *a))


class UF:
    def __init__(self, n):
        self.e = list(range(n))

    def f(self, i):
        while self.e[i] != i:
            self.e[i] = self.e[self.e[i]]
            i = self.e[i]
        return i

    def u(self, a, b):
        self.e[self.f(a)] = self.f(b)


def pruefen(zip_pfad, erwartet_pfad, bild=None):
    z = zipfile.ZipFile(zip_pfad)
    dateien = {n: z.read(n).decode('utf-8') for n in z.namelist()}
    fehler = []
    lagen = {}
    for n, t in dateien.items():
        if n.endswith('.gbr'):
            lagen[n.rsplit('-', 1)[1][:-4]] = gerber_lesen(t)
    for muss in ('F_Cu', 'B_Cu', 'F_Mask', 'B_Mask', 'F_SilkS', 'Edge_Cuts'):
        if muss not in lagen:
            fehler.append('Lage fehlt: ' + muss)
    drl = [t for n, t in dateien.items() if n.endswith('.drl')]
    loecher = bohrungen_lesen(drl[0])
    # Umriss
    ex = [p for l in lagen['Edge_Cuts'][2] for p in ((l[0], l[1]), (l[2], l[3]))]
    xmin, xmax = min(p[0] for p in ex), max(p[0] for p in ex)
    ymin, ymax = min(p[1] for p in ex), max(p[1] for p in ex)
    # Kupfer-Elemente beider Lagen; Element = (lage, art, geometrie, halbe Breite)
    el = []
    for lage in ('F_Cu', 'B_Cu'):
        _, blitze, linien, _ = lagen[lage]
        for (x, y, d) in blitze:
            el.append((lage, 'p', (x, y), d / 2))
        for (x1, y1, x2, y2, d) in linien:
            el.append((lage, 'l', (x1, y1, x2, y2), d / 2))
    for (lage, art, g, r) in el:
        pts = [g] if art == 'p' else [(g[0], g[1]), (g[2], g[3])]
        for (x, y) in pts:
            if x - r < xmin + 0.2 or x + r > xmax - 0.2 or y - r < ymin + 0.2 or y + r > ymax - 0.2:
                fehler.append('Kupfer zu nah am Rand bei %.2f/%.2f (%s)' % (x, y, lage))
    n = len(el) + len(loecher)
    uf = UF(n)
    # Beruehrung je Lage
    for i in range(len(el)):
        for j in range(i + 1, len(el)):
            a, b = el[i], el[j]
            if a[0] != b[0]:
                continue
            if a[1] == 'p' and b[1] == 'p':
                d = math.hypot(a[2][0] - b[2][0], a[2][1] - b[2][1])
            elif a[1] == 'p':
                d = abstand_punkt_strecke(a[2][0], a[2][1], *b[2])
            elif b[1] == 'p':
                d = abstand_punkt_strecke(b[2][0], b[2][1], *a[2])
            else:
                d = strecken_abstand(a[2], b[2])
            if d < a[3] + b[3] - 1e-6:
                uf.u(i, j)
    # Bohrung verbindet die Augen oben und unten an ihrer Stelle
    for k, (x, y, dm) in enumerate(loecher):
        oben = unten = False
        for i, (lage, art, g, r) in enumerate(el):
            if art == 'p' and math.hypot(g[0] - x, g[1] - y) < 1e-3:
                uf.u(len(el) + k, i)
                if r * 2 < dm + 0.3:
                    fehler.append('Restring zu schmal bei %.2f/%.2f' % (x, y))
                oben |= lage == 'F_Cu'
                unten |= lage == 'B_Cu'
        if not (oben and unten):
            fehler.append('Bohrung ohne Auge oben/unten bei %.2f/%.2f' % (x, y))

    def loch_index(x, y):
        for k, (lx, ly, _) in enumerate(loecher):
            if abs(lx - x) < 1e-3 and abs(ly - y) < 1e-3:
                return len(el) + k
        return None
    # erwartete Netze, Bruecken
    netz_von = {}
    for z_ in open(erwartet_pfad, encoding='utf-8').read().splitlines():
        t = z_.split(' ', 3)
        if t[0] == 'B':
            a = loch_index(float(t[1]), float(t[2]))
            b = loch_index(float(t[3].split()[0]), float(t[3].split()[1]))
            if a is None or b is None:
                fehler.append('Bruecke ohne Bohrung: ' + z_)
            else:
                uf.u(a, b)
        elif t[0] == 'P':
            k = loch_index(float(t[1]), float(t[2]))
            if k is None:
                fehler.append('Bein nicht gebohrt: ' + z_)
            else:
                netz_von[k] = t[3]
    # Inseln gegen Netze
    insel_netze = {}
    netz_inseln = {}
    for k, netz in netz_von.items():
        i = uf.f(k)
        insel_netze.setdefault(i, set()).add(netz)
        netz_inseln.setdefault(netz, set()).add(i)
    for i, ns in insel_netze.items():
        echte = {x for x in ns}
        if len(echte) > 1:
            fehler.append('Kurzschluss: ' + ', '.join(sorted(echte)))
    for netz, ins in netz_inseln.items():
        if len(ins) > 1 and not netz.endswith('(offen)'):
            fehler.append('Netz %s zerfaellt in %d Inseln' % (netz, len(ins)))
    if bild:
        zeichnen(lagen, loecher, (xmin, ymin, xmax, ymax), bild)
    return fehler, len(loecher), len(netz_inseln)


def zeichnen(lagen, loecher, rahmen, pfad):
    from PIL import Image, ImageDraw
    xmin, ymin, xmax, ymax = rahmen
    s = 24
    w, h = int((xmax - xmin) * s) + 1, int((ymax - ymin) * s) + 1
    img = Image.new('RGB', (w, h), (18, 60, 30))
    d = ImageDraw.Draw(img)

    def p(x, y):
        return ((x - xmin) * s, (ymax - y) * s)
    farben = {'B_Cu': (60, 110, 200), 'F_Cu': (200, 120, 60), 'F_SilkS': (235, 235, 235)}
    for lage in ('B_Cu', 'F_Cu', 'F_SilkS'):
        _, blitze, linien, _ = lagen[lage]
        for (x1, y1, x2, y2, dm) in linien:
            d.line([p(x1, y1), p(x2, y2)], fill=farben[lage], width=max(1, int(dm * s)))
        for (x, y, dm) in blitze:
            r = dm / 2 * s
            cx, cy = p(x, y)
            d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=farben[lage])
    for (x, y, dm) in loecher:
        r = dm / 2 * s
        cx, cy = p(x, y)
        d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(10, 10, 10))
    img.save(pfad)


if __name__ == '__main__':
    f, nl, nn = pruefen(sys.argv[1], sys.argv[2], sys.argv[3] if len(sys.argv) > 3 else None)
    print('%s: %d Bohrungen, %d Netze, %s' % (sys.argv[1], nl, nn, 'OK' if not f else '%d Fehler' % len(f)))
    for x in f:
        print('  ' + x)
    sys.exit(1 if f else 0)
