"""
Pyramid Pusher -- Pixel-Art-Generator.

Erzeugt alle 32x32-Tiles + den Helden-Sheet + die Truhe als PNGs in diesem
Ordner. Die Dateien sind ganz normale PNG-Sheets -- du kannst sie jederzeit im
Sprite-Editor (`gbsprites pyramid_pusher/assets/hero.png`) oeffnen und nachmalen.

Aufruf:  .venv\\Scripts\\python.exe pyramid_pusher\\assets\\gen_art.py

Stil: Sandstein-Grabkammer, passend zum Titelbild (Indiana-Jones-Forscher mit
Hut + Fackel, goldene Truhe, Tuerkis-Ankh auf den Kisten).
"""
import os
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
T = 32  # Tile-/Sprite-Kantenlaenge

# --- Palette (Sandstein / Gold / Tuerkis) -----------------------------------
SAND_HI  = (232, 200, 138)
SAND_MID = (210, 168,  96)
SAND_LO  = (176, 130,  74)
STONE_HI = (201, 165, 107)
STONE_MID= (138, 102,  56)
STONE_LO = ( 94,  68,  31)
MORTAR   = ( 70,  48,  20)
GOLD_HI  = (245, 197,  66)
GOLD     = (210, 158,  30)
GOLD_LO  = (150, 104,  16)
TEAL     = ( 43, 176, 160)
TEAL_HI  = (120, 224, 208)
WARM     = (255, 208,  96)
SKIN     = (224, 168, 120)
SKIN_LO  = (180, 124,  82)
HAT      = (107,  68,  35)
HAT_LO   = ( 74,  46,  22)
SHIRT    = (200, 160, 106)
SHIRT_LO = (150, 116,  72)
SATCHEL  = ( 74,  48,  24)
BOOTS    = ( 48,  32,  18)
FLAME    = (255, 150,  40)
FLAME_HI = (255, 232,  120)
DARK     = ( 40,  28,  16)


def rng(seed):
    return np.random.default_rng(seed)


def canvas(w=T, h=T):
    return np.zeros((h, w, 4), dtype=np.uint8)


def put(img, x, y, col, a=255):
    if 0 <= x < img.shape[1] and 0 <= y < img.shape[0]:
        img[y, x, 0], img[y, x, 1], img[y, x, 2], img[y, x, 3] = col[0], col[1], col[2], a


def fill(img, x0, y0, x1, y1, col, a=255):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(img, x, y, col, a)


def shade(col, f):
    return (int(max(0, min(255, col[0] * f))),
            int(max(0, min(255, col[1] * f))),
            int(max(0, min(255, col[2] * f))))


def save(img, name):
    Image.fromarray(img, "RGBA").save(os.path.join(HERE, name))
    print("  ->", name, img.shape[1], "x", img.shape[0])


# ---------------------------------------------------------------------------
def make_floor():
    img = canvas()
    r = rng(7)
    for y in range(T):
        for x in range(T):
            n = r.integers(-10, 11)
            base = SAND_MID
            put(img, x, y, shade(base, 1 + n / 255.0))
    # eingelassene Fugen (Steinplatten 16er-Raster)
    for y in range(T):
        for x in range(T):
            if x % 16 == 0 or y % 16 == 0:
                put(img, x, y, shade(SAND_LO, 0.9))
            if x % 16 == 1 or y % 16 == 1:
                put(img, x, y, shade(SAND_HI, 1.05))
    # ein paar Risse/Koernung
    for _ in range(14):
        x = int(r.integers(2, T - 2)); y = int(r.integers(2, T - 2))
        put(img, x, y, shade(SAND_LO, 0.8))
    save(img, "floor.png")


def make_wall():
    img = canvas()
    r = rng(13)
    # Ziegel: 2 Reihen, versetzt
    fill(img, 0, 0, T - 1, T - 1, STONE_MID)
    for y in range(T):
        for x in range(T):
            n = r.integers(-8, 9)
            put(img, x, y, shade(STONE_MID, 1 + n / 255.0))
    # Moertelfugen
    rows = [0, 11, 22]
    for ry in rows:
        for x in range(T):
            put(img, x, ry, MORTAR)
            put(img, x, ry + 1, shade(MORTAR, 1.2))
    # vertikale Fugen, je Reihe versetzt
    def vjoint(y0, y1, xs):
        for x in xs:
            for y in range(y0, y1):
                put(img, x % T, y, MORTAR)
    vjoint(2, 11, [0, 16])
    vjoint(13, 22, [8, 24])
    vjoint(24, 32, [0, 16])
    # obere Lichtkante je Ziegel
    for ry in rows:
        for x in range(T):
            put(img, x, ry + 2, shade(STONE_HI, 1.0))
    # dezente Gravur (Auge-Andeutung) mittig
    cx, cy = 16, 16
    for dx in range(-3, 4):
        put(img, cx + dx, cy, shade(STONE_LO, 0.8))
    put(img, cx, cy - 1, shade(STONE_LO, 0.8))
    put(img, cx, cy + 1, shade(STONE_LO, 0.8))
    save(img, "wall.png")


def make_goal():
    """Transparentes Overlay: leuchtendes Ankh-Zielfeld auf dem Boden."""
    img = canvas()
    cx, cy = 16, 16
    # weicher Lichthof
    for y in range(T):
        for x in range(T):
            d = ((x - cx + 0.5) ** 2 + (y - cy + 0.5) ** 2) ** 0.5
            if d < 13:
                a = int(max(0, 120 * (1 - d / 13)))
                put(img, x, y, WARM, a)
    # Ankh-Symbol (Tuerkis, hell)
    def ank(col):
        for y in range(cy - 1, cy + 9):           # Stamm
            put(img, cx, y, col); put(img, cx - 1, y, col)
        for x in range(cx - 5, cx + 5):           # Querbalken
            put(img, x, cy + 2, col); put(img, x, cy + 3, col)
        for t in range(7):                         # Schleife oben
            ang = t / 6 * 6.28318
            import math
            px = int(round(cx - 0.5 + 3.2 * math.sin(ang)))
            py = int(round(cy - 5 + 3.2 * (1 - math.cos(ang)) - 1))
            put(img, px, py, col); put(img, px + 1, py, col)
    ank(TEAL_HI)
    save(img, "goal.png")


def _lerp(a, b, t):
    return (int(a[0] + (b[0] - a[0]) * t),
            int(a[1] + (b[1] - a[1]) * t),
            int(a[2] + (b[2] - a[2]) * t))


def _ankh(img, cx, cy, col):
    import math
    for y in range(cy, cy + 9):                 # Stamm
        put(img, cx, y, col); put(img, cx - 1, y, col)
    for x in range(cx - 5, cx + 5):             # Querbalken
        put(img, x, cy + 2, col); put(img, x, cy + 3, col)
    for a in range(0, 360, 12):                 # Schleife (Ring) oben
        rad = a * math.pi / 180.0
        rx = cx - 0.5 + 3.2 * math.sin(rad)
        ry = cy - 4 - 3.2 * math.cos(rad)
        put(img, int(round(rx)), int(round(ry)), col)
        put(img, int(round(rx)) + 1, int(round(ry)), col)


def make_crate(on_goal=False):
    """Sandstein-Schatzkiste mit Goldbeschlag + Tuerkis-Ankh-Kartusche.
    on_goal = goldglaenzende, leuchtende 'platziert'-Variante."""
    img = canvas()
    r = rng(99 if on_goal else 42)
    if on_goal:
        body0, body1 = (236, 196, 96), (176, 128, 40)
        frame_hi, frame, frame_lo = (255, 232, 130), (228, 178, 60), (150, 104, 16)
        panel, panel_lo, panel_hi = (158, 112, 28), (112, 76, 14), (212, 160, 54)
        ankh, ankh_hi = (150, 240, 224), (224, 255, 250)
        rivet, rivet_hi = (255, 238, 150), (255, 255, 220)
    else:
        body0, body1 = (228, 194, 132), (150, 112, 66)
        frame_hi, frame, frame_lo = (232, 196, 120), (192, 150, 84), (116, 88, 46)
        panel, panel_lo, panel_hi = (122, 92, 54), (90, 66, 36), (164, 128, 78)
        ankh, ankh_hi = (54, 192, 174), (150, 236, 222)
        rivet, rivet_hi = (208, 184, 120), (245, 230, 180)

    # Korpus mit senkrechtem Verlauf (oben heller -> 3D) + Koernung
    for y in range(2, T - 2):
        col = _lerp(body0, body1, (y - 2) / float(T - 5))
        for x in range(2, T - 2):
            n = r.integers(-7, 8)
            put(img, x, y, shade(col, 1 + n / 255.0))

    # Abgeschraegter Goldrahmen (3 px): oben/links hell, unten/rechts dunkel
    for i in range(3):
        top = frame_hi if i < 2 else frame
        bot = frame_lo if i < 2 else frame
        for x in range(i, T - i):
            put(img, x, i, top); put(img, x, T - 1 - i, bot)
        for y in range(i, T - i):
            put(img, i, y, top); put(img, T - 1 - i, y, bot)

    # Ecknieten (Goldstollen mit Glanzpunkt)
    for (cx, cy) in [(5, 5), (T - 6, 5), (5, T - 6), (T - 6, T - 6)]:
        fill(img, cx - 1, cy - 1, cx + 1, cy + 1, frame_lo)
        fill(img, cx - 1, cy - 1, cx, cy, rivet)
        put(img, cx - 1, cy - 1, rivet_hi)

    # Eingelassene Kartusche (Panel) in der Mitte
    x0, y0, x1, y1 = 8, 8, T - 9, T - 9
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if (x == x0 or x == x1) and (y == y0 or y == y1):
                continue                        # Ecken abrunden
            put(img, x, y, panel)
    for x in range(x0, x1 + 1):                  # graviert: oben Schatten, unten Glanz
        put(img, x, y0, panel_lo); put(img, x, y1, panel_hi)
    for y in range(y0, y1 + 1):
        put(img, x0, y, panel_lo); put(img, x1, y, panel_hi)

    # on_goal: weicher tuerkiser Schein HINTER dem Ankh (Ankh bleibt scharf)
    if on_goal:
        for y in range(y0 + 1, y1):
            for x in range(x0 + 1, x1):
                d = ((x - 16) ** 2 + (y - 14.5) ** 2) ** 0.5
                if d < 7.0:
                    put(img, x, y, ankh_hi, int(75 * (1 - d / 7.0)))

    # Tuerkis-Ankh + Glanzkante (auf dem Goal heller)
    ank_col = ankh_hi if on_goal else ankh
    _ankh(img, 16, 13, ank_col)
    glint = (255, 255, 255) if on_goal else ankh_hi
    put(img, 15, 13, glint); put(img, 15, 14, glint)

    if on_goal:
        # leuchtender Saum aussen + Eck-Funken
        for x in range(T):
            put(img, x, 0, rivet_hi, 235); put(img, x, T - 1, rivet_hi, 205)
        for y in range(T):
            put(img, 0, y, rivet_hi, 235); put(img, T - 1, y, rivet_hi, 205)
        for (sx, sy) in [(4, 3), (T - 5, 4), (3, T - 5), (T - 4, T - 4)]:
            put(img, sx, sy, (255, 255, 255))

    save(img, "crate_on.png" if on_goal else "crate.png")


# --- Helden-Palette (Forscher: Lederjacke, Fedora, Satchel, Fackel) ---
H_OUT  = (26, 16, 8)       # Outline
H_HAT  = (120, 78, 40); H_HATLO = (84, 52, 26); H_HATHI = (152, 106, 58)
H_BAND = (66, 42, 22)
H_SKIN = (228, 174, 124); H_SKINLO = (186, 130, 86)
H_JKT  = (122, 80, 42); H_JKTHI = (156, 108, 60); H_JKTLO = (86, 54, 28)
H_SHIRT = (208, 180, 130)
H_BELT = (62, 42, 24); H_BUCKLE = (224, 180, 76)
H_PANT = (96, 76, 50); H_PANTLO = (66, 50, 32)
H_BOOT = (56, 38, 22)
H_BAG  = (104, 68, 34); H_BAGHI = (140, 96, 50)
H_TW   = (104, 68, 36); H_TWLO = (74, 46, 24)
H_FOUT = (214, 78, 22); H_FL = (255, 150, 44); H_FHI = (255, 226, 132); H_HAIR = (58, 38, 22)


def _outline(im, col=H_OUT):
    """4-Nachbar-Outline: transparente Pixel neben opaken werden dunkel."""
    src = im.copy()
    h, w = im.shape[0], im.shape[1]
    for y in range(h):
        for x in range(w):
            if im[y, x, 3] != 0:
                continue
            hit = False
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and src[ny, nx, 3] > 0:
                    hit = True
                    break
            if hit:
                put(im, x, y, col, 255)


def _draw_hero(facing, step):
    im = canvas(T, T)
    cx = 16
    fwd = (step == 0)        # welches Bein/Arm vorn

    # ===== BEINE =====
    ll = 29 + (1 if fwd else 0)
    rl = 29 + (0 if fwd else 1)
    fill(im, cx - 4, 24, cx - 1, ll, H_PANT)
    fill(im, cx + 1, 24, cx + 4, rl, H_PANT)
    fill(im, cx - 4, ll - 1, cx - 1, ll, H_BOOT)
    fill(im, cx + 1, rl - 1, cx + 4, rl, H_BOOT)
    put(im, cx - 4, 24, H_PANTLO); put(im, cx + 4, 24, H_PANTLO)

    # ===== TORSO: Hemd + offene Lederjacke =====
    fill(im, cx - 3, 14, cx + 2, 22, H_SHIRT)
    fill(im, cx - 5, 14, cx - 3, 23, H_JKT)
    fill(im, cx + 2, 14, cx + 4, 23, H_JKT)
    fill(im, cx - 5, 14, cx + 4, 15, H_JKT)          # Schultern
    fill(im, cx - 5, 14, cx - 5, 23, H_JKTHI)         # linke Kante hell
    fill(im, cx + 4, 14, cx + 4, 23, H_JKTLO)         # rechte Kante dunkel
    # Guertel + Schnalle
    fill(im, cx - 5, 22, cx + 4, 23, H_BELT)
    put(im, cx - 1, 22, H_BUCKLE); put(im, cx, 22, H_BUCKLE); put(im, cx, 23, H_BUCKLE)
    # Satchel-Riemen diagonal + Tasche an der Huefte
    for i in range(10):
        put(im, cx - 5 + i, 14 + i, H_BAND)
    if facing != 1:
        fill(im, cx + 3, 19, cx + 5, 23, H_BAG)
        put(im, cx + 3, 19, H_BAGHI)
    else:
        fill(im, cx - 4, 17, cx + 3, 22, H_BAG)        # Rucksack von hinten
        fill(im, cx - 4, 17, cx + 3, 17, H_BAGHI)
    # Arme
    fill(im, cx - 6, 15, cx - 5, 21, H_JKT)           # linker Arm am Koerper
    put(im, cx - 6, 15, H_JKTHI)

    # ===== FACKEL (rechte Hand, vorn/seitlich) =====
    if facing != 1:
        tx = 22
        fill(im, cx + 4, 16, cx + 6, 19, H_JKT)       # rechter Arm raus
        fill(im, tx, 15, tx + 1, 23, H_TW)            # Stab
        put(im, tx, 15, H_TWLO); put(im, tx + 1, 23, H_TWLO)
        # Flamme (Tropfenform, dreifarbig)
        fill(im, tx - 1, 9, tx + 2, 14, H_FL)
        put(im, tx, 7, H_FL); put(im, tx, 6, H_FHI)
        fill(im, tx, 8, tx + 1, 13, H_FHI)
        put(im, tx - 1, 13, H_FOUT); put(im, tx + 2, 13, H_FOUT); put(im, tx + 2, 11, H_FOUT)
    else:
        # Rueckenansicht: Fackel hinterm Kopf, nur Schein
        fill(im, cx + 5, 7, cx + 6, 12, H_TW)
        put(im, cx + 5, 7, H_FHI); put(im, cx + 6, 6, H_FL)

    # ===== KOPF =====
    fill(im, cx - 4, 8, cx + 3, 13, H_SKIN)
    fill(im, cx - 4, 8, cx + 3, 8, H_SKINLO)          # Schatten unter Krempe
    fill(im, cx - 3, 13, cx + 2, 13, H_SKINLO)        # Kinnschatten
    if facing == 0:
        put(im, cx - 2, 10, H_OUT); put(im, cx + 1, 10, H_OUT)   # Augen
        put(im, cx - 1, 12, H_SKINLO); put(im, cx, 12, H_SKINLO) # Mund/Nase
    elif facing == 2:
        # Profil rechts
        fill(im, cx - 4, 8, cx + 3, 13, H_SKIN)
        put(im, cx + 1, 10, H_OUT)                    # Auge
        put(im, cx + 4, 11, H_SKIN); put(im, cx + 4, 12, H_SKINLO)  # Nase
        fill(im, cx - 4, 9, cx - 3, 13, H_HAIR)       # Hinterkopf/Haar
    elif facing == 1:
        fill(im, cx - 4, 9, cx + 3, 13, H_HAIR)       # Hinterkopf

    # ===== HUT (Fedora) =====
    fill(im, cx - 7, 6, cx + 6, 7, H_HAT)             # breite Krempe
    fill(im, cx - 7, 7, cx + 6, 7, H_HATLO)           # Krempenschatten
    fill(im, cx - 4, 2, cx + 3, 6, H_HAT)             # Kopfteil
    fill(im, cx - 3, 1, cx + 2, 1, H_HAT)             # gerundete Spitze
    fill(im, cx - 4, 2, cx + 3, 2, H_HATHI)           # Lichtkante oben
    fill(im, cx - 4, 5, cx + 3, 5, H_BAND)            # Hutband
    put(im, cx - 7, 6, H_HATLO); put(im, cx + 6, 6, H_HATLO)
    if facing == 0:
        put(im, cx - 1, 8, H_HAT); put(im, cx, 8, H_HAT)   # Krempe vorn gesenkt
    if facing == 2:
        fill(im, cx + 4, 6, cx + 7, 7, H_HAT)         # Krempe zeigt nach rechts
        fill(im, cx - 7, 6, cx - 5, 7, H_HATLO)

    _outline(im)
    return im


def make_hero():
    """Sheet: 4 Reihen (down, up, right, left) x 2 Frames -> 64x128."""
    sheet = canvas(2 * T, 4 * T)
    rows = [0, 1, 2]      # down, up, right
    for frame in range(2):
        for fi, facing in enumerate(rows):
            tile = _draw_hero(facing, frame)
            sheet[fi * T:(fi + 1) * T, frame * T:(frame + 1) * T, :] = tile
    # left = right gespiegelt
    right = sheet[2 * T:3 * T, :, :].copy()
    sheet[3 * T:4 * T, :, :] = right[:, ::-1, :]
    save(sheet, "hero.png")


def make_chest(open_=False):
    S = 48
    img = canvas(S, S)
    r = rng(3)
    woods = [(150, 100, 50), (120, 78, 38), (96, 60, 28)]
    # Korpus
    fill(img, 6, 22, S - 7, S - 6, woods[1])
    for y in range(22, S - 5):
        for x in range(6, S - 6):
            n = r.integers(-7, 8)
            put(img, x, y, shade(woods[1], 1 + n / 255.0))
    # vertikale Planken
    for x in range(6, S - 6, 7):
        for y in range(22, S - 5):
            put(img, x, y, woods[2])
    # Goldbeschlaege
    fill(img, 4, 22, 5, S - 6, GOLD); fill(img, S - 6, 22, S - 5, S - 6, GOLD)
    fill(img, 4, S - 6, S - 5, S - 5, GOLD_LO)
    for bx in (12, 24, 36):
        fill(img, bx, 22, bx + 1, S - 6, GOLD_HI)
    # Deckel
    if not open_:
        fill(img, 5, 12, S - 6, 23, woods[0])
        for x in range(5, S - 5):
            yb = 12 - int(round(3 * np.sin((x - 5) / (S - 10) * np.pi)))
            for y in range(yb, 13):
                put(img, x, y, woods[0])
                put(img, x, yb, GOLD_HI)
        fill(img, 5, 18, S - 6, 19, GOLD)        # Mittelband
        # Schloss
        fill(img, 21, 17, 27, 24, GOLD_HI)
        fill(img, 23, 20, 25, 23, DARK)
    else:
        # offener Deckel nach hinten geklappt -> flacher Streifen oben
        fill(img, 5, 6, S - 6, 12, woods[0])
        fill(img, 5, 6, S - 6, 6, GOLD_HI)
        # Goldglanz aus dem Inneren
        fill(img, 8, 16, S - 9, 24, GOLD_HI)
        for y in range(16, 26):
            for x in range(8, S - 8):
                d = abs(x - 24) + abs(y - 20)
                if d < 14 and r.integers(0, 100) < 70:
                    put(img, x, y, FLAME_HI if d < 7 else GOLD)
        # ein paar Lichtstrahlen
        for x in range(10, S - 8, 3):
            put(img, x, 14, FLAME_HI, 200)
    save(img, "chest_open.png" if open_ else "chest_closed.png")


ICE_HI = (206, 230, 245)
ICE_MID = (150, 192, 224)
ICE_LO = (104, 150, 192)


def make_pit():
    """Offenes Sand-Loch: dunkler Schlund mit Steinrand."""
    img = canvas()
    r = rng(21)
    cx, cy = 16.0, 16.0
    for y in range(T):
        for x in range(T):
            d = ((x - cx + 0.5) ** 2 + (y - cy + 0.5) ** 2) ** 0.5
            if d < 12.5:
                t = d / 12.5
                col = (int(8 + 40 * t), int(6 + 30 * t), int(4 + 18 * t))
                put(img, x, y, col)
            else:
                n = r.integers(-8, 9)
                put(img, x, y, shade(SAND_MID, 1 + n / 255.0))
    # Steinrand (Lichtkante oben, Schatten innen)
    for a in range(0, 360, 6):
        import math
        rad = a * math.pi / 180.0
        rx = int(round(cx - 0.5 + 12.5 * math.sin(rad)))
        ry = int(round(cy - 0.5 - 12.5 * math.cos(rad)))
        put(img, rx, ry, SAND_HI if ry < cy else STONE_LO)
    save(img, "pit.png")


def make_pit_filled():
    """Gefuelltes Loch: Boden mit Schutt/Geroell."""
    img = canvas()
    r = rng(22)
    for y in range(T):
        for x in range(T):
            n = r.integers(-8, 9)
            put(img, x, y, shade(SAND_MID, 1 + n / 255.0))
    # Geroell-Brocken in der Mitte
    for _ in range(26):
        x = int(r.integers(7, 25)); y = int(r.integers(7, 25))
        s = int(r.integers(1, 3))
        fill(img, x, y, x + s, y + s, shade(STONE_MID, 0.8 + r.integers(0, 40) / 100.0))
    save(img, "pit_filled.png")


def make_button():
    """Boden mit rundem Stein-Schalter + Tuerkis-Edelstein."""
    img = canvas()
    r = rng(23)
    for y in range(T):
        for x in range(T):
            n = r.integers(-8, 9)
            put(img, x, y, shade(SAND_MID, 1 + n / 255.0))
    cx, cy = 16.0, 16.0
    for y in range(T):
        for x in range(T):
            d = ((x - cx + 0.5) ** 2 + (y - cy + 0.5) ** 2) ** 0.5
            if d < 8:
                put(img, x, y, shade(STONE_MID, 1.1 if (x + y) < 30 else 0.85))
            if d < 4:
                put(img, x, y, TEAL if d > 1.5 else TEAL_HI)
    # Rand-Glanz
    for a in range(0, 360, 8):
        import math
        rad = a * math.pi / 180.0
        rx = int(round(cx - 0.5 + 8 * math.sin(rad)))
        ry = int(round(cy - 0.5 - 8 * math.cos(rad)))
        put(img, rx, ry, STONE_HI if ry < cy else STONE_LO)
    save(img, "button.png")


def make_door(open_=False):
    img = canvas()
    if open_:
        # offener Durchgang: dunkle Oeffnung + Steinrahmen mit Gold
        fill(img, 0, 0, T - 1, T - 1, (16, 12, 8))
        fill(img, 0, 0, 3, T - 1, STONE_MID); fill(img, T - 4, 0, T - 1, T - 1, STONE_MID)
        fill(img, 0, 0, T - 1, 3, STONE_MID)
        fill(img, 1, 1, 2, T - 1, STONE_HI); fill(img, 1, 1, T - 1, 2, STONE_HI)
        fill(img, 4, 3, 5, T - 1, GOLD); fill(img, T - 6, 3, T - 5, T - 1, GOLD)
        fill(img, 4, 3, T - 5, 4, GOLD)
        # Lichtschein aus dem Durchgang
        for y in range(5, T):
            for x in range(7, T - 6):
                if r_in(x, T - 6) and (x + y) % 3 == 0:
                    put(img, x, y, (40, 30, 18))
    else:
        # geschlossene Steintuer mit Riegel
        fill(img, 0, 0, T - 1, T - 1, shade(STONE_MID, 0.92))
        for y in range(T):
            for x in range(T):
                if x % 8 == 0:
                    put(img, x, y, STONE_LO)
        fill(img, 0, 0, T - 1, 1, STONE_HI)
        fill(img, 0, 0, 1, T - 1, STONE_HI)
        fill(img, 0, T - 2, T - 1, T - 1, STONE_LO)
        # horizontale Goldriegel
        fill(img, 2, 9, T - 3, 11, GOLD); fill(img, 2, 9, T - 3, 9, GOLD_HI)
        fill(img, 2, 21, T - 3, 23, GOLD); fill(img, 2, 21, T - 3, 21, GOLD_HI)
        # Schloss
        fill(img, 13, 13, 18, 19, GOLD_HI); fill(img, 15, 15, 16, 17, (40, 28, 16))
    save(img, "door_open.png" if open_ else "door_closed.png")


def r_in(x, hi):
    return 0 <= x < hi


def make_ice():
    """Eisplatte: blasse blaue, glaenzende Flaeche mit Rissen."""
    img = canvas()
    r = rng(24)
    for y in range(T):
        for x in range(T):
            n = r.integers(-6, 7)
            put(img, x, y, shade(ICE_MID, 1 + n / 255.0))
    # glaenzende Highlights (diagonale Streifen)
    for d in range(-T, T, 7):
        for y in range(T):
            x = y + d
            if 0 <= x < T:
                put(img, x, y, ICE_HI)
    # Risse
    for _ in range(3):
        x = int(r.integers(6, 26)); y = int(r.integers(4, 12))
        for k in range(int(r.integers(6, 14))):
            put(img, x, y, ICE_LO)
            x += int(r.integers(-1, 2)); y += 1
            if not (0 <= x < T and 0 <= y < T):
                break
    # Rahmen leicht heller (Plattenkante)
    for i in range(T):
        put(img, i, 0, ICE_HI); put(img, 0, i, ICE_HI)
        put(img, i, T - 1, ICE_LO); put(img, T - 1, i, ICE_LO)
    save(img, "ice.png")


def make_star():
    """Weisser 5-Punkt-Stern mit dunklem Rand -> im Spiel gold/dunkel getoent."""
    import math
    from PIL import ImageDraw
    S = 24
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    cx = cy = S / 2.0
    R = S * 0.46
    r = R * 0.42
    pts = []
    for k in range(10):
        ang = -math.pi / 2 + k * math.pi / 5
        rad = R if k % 2 == 0 else r
        pts.append((cx + rad * math.cos(ang), cy + rad * math.sin(ang)))
    d.polygon(pts, fill=(255, 255, 255, 255), outline=(34, 22, 12, 255))
    im.save(os.path.join(HERE, "star.png"))
    print("  -> star.png", S, "x", S)


def make_lightmask():
    """Fackel-Licht-Maske fuer den Dunkelmodus. Wird multiplikativ ueber die
    Szene gezeichnet: heller (=weisser) Kern laesst die Kacheln durch, der
    grosse dunkle Rand dimmt alles ringsum fast schwarz. Gross genug, damit er
    bei jeder Spielerposition den ganzen Bildschirm ueberdeckt (in GB skaliert).
    """
    import math
    S = 512
    img = canvas(S, S)
    cx = cy = S / 2.0
    r0 = S * 0.066     # voll beleuchtet
    r1 = S * 0.30      # weicher Abfall bis hier
    dark = (20, 15, 10)
    lite = (255, 248, 235)
    for y in range(S):
        for x in range(S):
            d = math.hypot(x - cx + 0.5, y - cy + 0.5)
            if d <= r0:
                v = 1.0
            elif d >= r1:
                v = 0.0
            else:
                tt = (d - r0) / (r1 - r0)
                v = 1.0 - (tt * tt * (3 - 2 * tt))   # smoothstep
            col = (int(dark[0] + (lite[0] - dark[0]) * v),
                   int(dark[1] + (lite[1] - dark[1]) * v),
                   int(dark[2] + (lite[2] - dark[2]) * v))
            put(img, x, y, col, 255)
    save(img, "lightmask.png")


if __name__ == "__main__":
    print("Pyramid Pusher -- erzeuge Pixel-Art ...")
    make_floor()
    make_wall()
    make_goal()
    make_crate(False)
    make_crate(True)
    make_hero()
    make_chest(False)
    make_chest(True)
    make_star()
    make_pit()
    make_pit_filled()
    make_button()
    make_door(False)
    make_door(True)
    make_ice()
    make_lightmask()
    print("fertig.")
