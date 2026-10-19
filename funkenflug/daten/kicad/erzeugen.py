# Erzeugt die KiCad-7-Testdateien fuer pruef_kern.dh und den Selbsttest
# (python erzeugen.py). Handgeschrieben, weil hier kein KiCad installiert ist.
# Pin-Lagen werden mit KiCads eigener Transform-Rechnung (sch_symbol.cpp / sch_sexpr_parser.cpp) bestimmt -- also
# unabhaengig von der Herleitung in kicadimport.dh.
import os, uuid

ZIEL = os.path.dirname(os.path.abspath(__file__))
os.makedirs(ZIEL, exist_ok=True)

def u():
    return str(uuid.uuid4())

# KiCad: TRANSFORM(x1, y1, x2, y2); p' = (x1*x + y1*y, x2*x + y2*y)
ROT = {0: (1, 0, 0, -1), 90: (0, -1, -1, 0), 180: (-1, 0, 0, 1), 270: (0, 1, 1, 0)}
MIR = {'x': (1, 0, 0, -1), 'y': (-1, 0, 0, 1)}

def kombi(m, t):
    # SetOrientation: new = temp * m (inkrementell)
    x1, y1, x2, y2 = m
    tx1, ty1, tx2, ty2 = t
    return (x1 * tx1 + x2 * ty1, y1 * tx1 + y2 * ty1, x1 * tx2 + x2 * ty2, y1 * tx2 + y2 * ty2)

def transform(winkel, spiegel):
    m = ROT[winkel]
    if spiegel:
        m = kombi(m, MIR[spiegel])
    return m

def pinAbs(pos, winkel, spiegel, lp):
    x1, y1, x2, y2 = transform(winkel, spiegel)
    return (round(pos[0] + x1 * lp[0] + y1 * lp[1], 4), round(pos[1] + x2 * lp[0] + y2 * lp[1], 4))

def f(v):
    s = ('%.4f' % v).rstrip('0').rstrip('.')
    return '0' if s == '-0' else s

FX = '(effects (font (size 1.27 1.27)))'
FXH = '(effects (font (size 1.27 1.27)) hide)'

def pin(typ, x, y, a, laenge, name, nr, versteckt=False):
    return '      (pin %s line (at %s %s %d) (length %s)%s (name "%s" %s) (number "%s" %s))\n' % (
        typ, f(x), f(y), a, f(laenge), ' hide' if versteckt else '', name, FX, nr, FX)

LIB = {}
LIB['Device:R'] = dict(ref='R', wert='R', pins={'1': (0, 3.81), '2': (0, -3.81)}, text='''    (symbol "Device:R" (pin_numbers hide) (pin_names (offset 0)) (in_bom yes) (on_board yes)
      (property "Reference" "R" (at 2.032 0 90) %s)
      (property "Value" "R" (at 0 0 90) %s)
      (property "Footprint" "" (at -1.778 0 90) %s)
      (symbol "R_0_1"
        (rectangle (start -1.016 -2.54) (end 1.016 2.54) (stroke (width 0.254) (type default)) (fill (type none)))
      )
      (symbol "R_1_1"
%s%s      )
    )
''' % (FX, FX, FXH, pin('passive', 0, 3.81, 270, 1.27, '~', '1'), pin('passive', 0, -3.81, 90, 1.27, '~', '2')))

LIB['Test:Q_NPN'] = dict(ref='Q', wert='Q_NPN', fp='Package_TO_SOT_THT:TO-92_Inline', pins={'1': (-5.08, 0), '2': (2.54, 5.08), '3': (2.54, -5.08)}, text='''    (symbol "Test:Q_NPN" (pin_names (offset 0) hide) (in_bom yes) (on_board yes)
      (property "Reference" "Q" (at 5.08 1.27 0) %s)
      (property "Value" "Q_NPN" (at 5.08 -1.27 0) %s)
      (property "Footprint" "Package_TO_SOT_THT:TO-92_Inline" (at 5.08 -3.81 0) %s)
      (symbol "Q_NPN_0_1"
        (polyline (pts (xy 0.635 0.635) (xy 2.54 2.54)) (stroke (width 0) (type default)) (fill (type none)))
        (polyline (pts (xy 0.635 -0.635) (xy 2.54 -2.54) (xy 2.54 -2.54)) (stroke (width 0) (type default)) (fill (type none)))
        (polyline (pts (xy 0.635 1.905) (xy 0.635 -1.905) (xy 0.635 -1.905)) (stroke (width 0.508) (type default)) (fill (type none)))
        (polyline (pts (xy 1.27 -1.778) (xy 1.778 -1.27) (xy 2.286 -2.286) (xy 1.27 -1.778)) (stroke (width 0) (type default)) (fill (type outline)))
        (circle (center 1.27 0) (radius 2.8194) (stroke (width 0.254) (type default)) (fill (type none)))
      )
      (symbol "Q_NPN_1_1"
%s%s%s      )
    )
''' % (FX, FX, FXH, pin('input', -5.08, 0, 0, 5.715, 'B', '1'), pin('passive', 2.54, 5.08, 270, 2.54, 'C', '2'), pin('passive', 2.54, -5.08, 90, 2.54, 'E', '3')))

LIB['Test:NAND2'] = dict(ref='U', wert='74HC00', fp='Package_DIP:DIP-14_W7.62mm', pins={'1': (-7.62, 2.54), '2': (-7.62, -2.54), '3': (7.62, 0), '4': (-7.62, 2.54), '5': (-7.62, -2.54), '6': (7.62, 0)}, text='''    (symbol "Test:NAND2" (in_bom yes) (on_board yes)
      (property "Reference" "U" (at 0 5.08 0) %s)
      (property "Value" "74HC00" (at 0 -5.08 0) %s)
      (property "Footprint" "Package_DIP:DIP-14_W7.62mm" (at 0 0 0) %s)
      (property "ki_description" "zwei NAND mit 2 Eingaengen" (at 0 0 0) %s)
      (symbol "NAND2_0_0"
%s%s      )
      (symbol "NAND2_1_1"
        (arc (start 0 -3.81) (mid 3.81 0) (end 0 3.81) (stroke (width 0.254) (type default)) (fill (type background)))
        (polyline (pts (xy 0 3.81) (xy -3.81 3.81) (xy -3.81 -3.81) (xy 0 -3.81)) (stroke (width 0.254) (type default)) (fill (type background)))
%s%s%s      )
      (symbol "NAND2_2_1"
        (arc (start 0 -3.81) (mid 3.81 0) (end 0 3.81) (stroke (width 0.254) (type default)) (fill (type background)))
        (polyline (pts (xy 0 3.81) (xy -3.81 3.81) (xy -3.81 -3.81) (xy 0 -3.81)) (stroke (width 0.254) (type default)) (fill (type background)))
%s%s%s      )
    )
''' % (FX, FX, FXH, FXH,
       pin('power_in', 0, 7.62, 270, 0, 'VCC', '14', True), pin('power_in', 0, -7.62, 90, 0, 'GND', '7', True),
       pin('input', -7.62, 2.54, 0, 3.81, 'A', '1'), pin('input', -7.62, -2.54, 0, 3.81, 'B', '2'), pin('output', 7.62, 0, 180, 3.81, 'Y', '3'),
       pin('input', -7.62, 2.54, 0, 3.81, 'A', '4'), pin('input', -7.62, -2.54, 0, 3.81, 'B', '5'), pin('output', 7.62, 0, 180, 3.81, 'Y', '6')))

def power(name, oben):
    y = 1.27 if oben else -1.27
    return '''    (symbol "power:%s" (power) (pin_names (offset 0)) (in_bom yes) (on_board yes)
      (property "Reference" "#PWR" (at 0 -3.81 0) %s)
      (property "Value" "%s" (at 0 3.556 0) %s)
      (symbol "%s_0_1"
        (polyline (pts (xy 0 0) (xy 0 %s)) (stroke (width 0) (type default)) (fill (type none)))
        (polyline (pts (xy -1.27 %s) (xy 1.27 %s)) (stroke (width 0) (type default)) (fill (type none)))
      )
      (symbol "%s_1_1"
%s      )
    )
''' % (name, FXH, name, FX, name, f(y), f(y), f(y), name, pin('power_in', 0, 0, 90 if oben else 270, 0, name, '1', True))

LIB['power:VCC'] = dict(ref='#PWR', wert='VCC', pins={'1': (0, 0)}, text=power('VCC', True))
LIB['power:GND'] = dict(ref='#PWR', wert='GND', pins={'1': (0, 0)}, text=power('GND', False))

ROOT_UUID = u()

class Blatt:
    def __init__(self, name):
        self.name = name
        self.libs = set()
        self.teile = []
        self.rest = []

    def symbol(self, lib, ref, pos, winkel=0, spiegel=None, einheit=1, wert=None, stummel=True, extra=''):
        d = LIB[lib]
        self.libs.add(lib)
        sp = ' (mirror %s)' % spiegel if spiegel else ''
        ref_txt = ref
        self.teile.append('''  (symbol (lib_id "%s") (at %s %s %d)%s (unit %d) (in_bom yes) (on_board yes) (dnp no)
    (uuid %s)
    (property "Reference" "%s" (at %s %s 0) %s)
    (property "Value" "%s" (at %s %s 0) %s)
    (property "Footprint" "%s" (at 0 0 0) %s)%s
    (instances (project "beispiel" (path "/%s" (reference "%s") (unit %d))))
  )
''' % (lib, f(pos[0]), f(pos[1]), winkel, sp, einheit, u(), ref_txt, f(pos[0] + 3.81), f(pos[1] - 2.54), FX,
       wert or d['wert'], f(pos[0] + 3.81), f(pos[1] + 2.54), FX, d.get('fp', ''), FXH, extra, ROOT_UUID, ref_txt, einheit))
        punkte = {}
        for nr, lp in d['pins'].items():
            punkte[nr] = pinAbs(pos, winkel, spiegel, lp)
        return punkte

    def draht(self, a, b):
        self.rest.append('  (wire (pts (xy %s %s) (xy %s %s)) (stroke (width 0) (type default)) (uuid %s))\n' % (f(a[0]), f(a[1]), f(b[0]), f(b[1]), u()))

    def label(self, art, text, p, winkel=0, extra=''):
        self.rest.append('  (%s "%s"%s (at %s %s %d) (effects (font (size 1.27 1.27)) (justify left)) (uuid %s))\n' % (art, text, extra, f(p[0]), f(p[1]), winkel, u()))

    def stummel(self, mitte, p, name):
        # 2,54 mm vom Koerper weg, dort ein Label
        dx, dy = p[0] - mitte[0], p[1] - mitte[1]
        if abs(dx) >= abs(dy):
            q = (p[0] + (2.54 if dx > 0 else -2.54), p[1])
            w = 0 if dx > 0 else 180
        else:
            q = (p[0], p[1] + (2.54 if dy > 0 else -2.54))
            w = 270 if dy > 0 else 90
        self.draht(p, q)
        self.label('label', name, q, w)

    def text(self):
        libs = ''.join(LIB[k]['text'] for k in sorted(self.libs))
        return '''(kicad_sch (version 20230121) (generator eeschema)

  (uuid %s)

  (paper "A4")

  (title_block
    (title "KiCad-Beispiel")
    (rev "B")
    (company "Funkenflug-Pruefung")
  )

  (lib_symbols
%s  )

%s%s)
''' % (ROOT_UUID if self.name == 'haupt' else u(), libs, ''.join(self.rest), ''.join(self.teile))

haupt = Blatt('haupt')
# acht Lagen eines Transistors
lagen = [(0, None), (90, None), (180, None), (270, None), (0, 'y'), (90, 'x'), (180, 'y'), (270, 'x')]
for i, (w, s) in enumerate(lagen):
    pos = (25.4 * (i + 1), 50.8)
    ref = 'Q%d' % (i + 1)
    pk = haupt.symbol('Test:Q_NPN', ref, pos, w, s)
    for nr, p in pk.items():
        haupt.stummel(pos, p, '%s_%s' % (ref, nr))
# zwei Lagen eines Widerstands, R1 zwischen VCC und GND
pr1 = haupt.symbol('Device:R', 'R1', (25.4, 88.9), 0, None, wert='10k')
pv = haupt.symbol('power:VCC', '#PWR01', (25.4, 78.74))
pg = haupt.symbol('power:GND', '#PWR02', (25.4, 99.06))
haupt.draht(pv['1'], pr1['1'])
haupt.draht(pr1['2'], pg['1'])
pr2 = haupt.symbol('Device:R', 'R2', (50.8, 88.9), 90, None, wert='4k7')
for nr, p in pr2.items():
    haupt.stummel((50.8, 88.9), p, 'R2_%s' % nr)
# mehrteiliges IC, versteckte Versorgungs-Pins
for einheit in (1, 2):
    pos = (76.2 + 38.1 * (einheit - 1), 88.9)
    pk = haupt.symbol('Test:NAND2', 'U1', pos, 0, None, einheit=einheit)
    nrs = ('1', '2', '3') if einheit == 1 else ('4', '5', '6')
    for nr in nrs:
        haupt.stummel(pos, pk[nr], 'U1%s_%s' % ('AB'[einheit - 1], nr))
# Kreuzung mit Verbindungspunkt und ohne
haupt.draht((30.48, 139.7), (60.96, 139.7))
haupt.draht((45.72, 127), (45.72, 152.4))
haupt.rest.append('  (junction (at 45.72 139.7) (diameter 0) (color 0 0 0 0) (uuid %s))\n' % u())
haupt.label('label', 'KREUZ_A', (30.48, 139.7), 180)
haupt.label('label', 'KREUZ_B', (45.72, 127), 90)
haupt.draht((66.04, 139.7), (96.52, 139.7))
haupt.draht((81.28, 127), (81.28, 152.4))
haupt.label('label', 'FREI_C', (66.04, 139.7), 180)
haupt.label('label', 'FREI_D', (81.28, 127), 90)
# Unterblatt mit zwei Ports
haupt.rest.append('''  (sheet (at 152.4 88.9) (size 25.4 20.32) (fields_autoplaced)
    (stroke (width 0.1524) (type solid)) (fill (color 0 0 0 0.0000))
    (uuid %s)
    (property "Sheetname" "Filter" (at 152.4 88.1884 0) (effects (font (size 1.27 1.27)) (justify left bottom)))
    (property "Sheetfile" "unterblatt.kicad_sch" (at 152.4 109.8046 0) (effects (font (size 1.27 1.27)) (justify left top)))
    (pin "EIN" input (at 152.4 93.98 180) (effects (font (size 1.27 1.27)) (justify left)) (uuid %s))
    (pin "AUS" output (at 177.8 93.98 0) (effects (font (size 1.27 1.27)) (justify right)) (uuid %s))
    (instances (project "beispiel" (path "/%s" (page "2"))))
  )
''' % (u(), u(), u(), ROOT_UUID))
haupt.draht((142.24, 93.98), (152.4, 93.98))
haupt.label('label', 'SIG_EIN', (142.24, 93.98), 180)
haupt.draht((177.8, 93.98), (187.96, 93.98))
haupt.label('label', 'SIG_AUS', (187.96, 93.98), 0)
# was Funkenflug nicht kennt -> Hinweise
haupt.rest.append('  (text "Hallo KiCad" (at 120 160 0) (effects (font (size 1.27 1.27)) (justify left bottom)) (uuid %s))\n' % u())
# ein Bus D[0..1] mit zwei Eintraegen; die Drahtenden tragen D0 und D1
haupt.rest.append('  (bus (pts (xy 120.65 165.1) (xy 146.05 165.1)) (stroke (width 0) (type default)) (uuid %s))\n' % u())
haupt.label('label', 'D[0..1]', (125.73, 165.1), 0)
for i, x in enumerate((130.81, 138.43)):
    haupt.rest.append('  (bus_entry (at %s 165.1) (size 2.54 2.54) (stroke (width 0) (type default)) (uuid %s))\n' % (f(x), u()))
    haupt.draht((x + 2.54, 167.64), (x + 2.54, 172.72))
    haupt.label('label', 'D%d' % i, (x + 2.54, 172.72), 270)
haupt.rest.append('  (bus_alias "ADR" (members "A0" "A1"))\n')

unter = Blatt('unter')
pr3 = unter.symbol('Device:R', 'R3', (101.6, 76.2), 90, None, wert='1k')
unter.draht((88.9, 76.2), pr3['1'])
unter.draht(pr3['2'], (114.3, 76.2))
unter.label('hierarchical_label', 'EIN', (88.9, 76.2), 180, ' (shape input)')
unter.label('hierarchical_label', 'AUS', (114.3, 76.2), 0, ' (shape output)')

open(os.path.join(ZIEL, 'beispiel.kicad_sch'), 'w', encoding='utf-8', newline='\n').write(haupt.text())
open(os.path.join(ZIEL, 'unterblatt.kicad_sch'), 'w', encoding='utf-8', newline='\n').write(unter.text())
print('R1', pr1, 'R2', pr2, 'R3', pr3)
