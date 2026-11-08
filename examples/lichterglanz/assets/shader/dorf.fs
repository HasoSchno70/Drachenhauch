#version 330
// Das Dorf im Schnee: ein Tal bei Nacht, ringsum verschneiter Tannenwald, in
// der Mitte die Kirche. Das Dorf baut sich auf (`fort`): erst waechst die
// Kirche aus dem Schnee, dann Haus um Haus -- von der Mitte nach aussen,
// jedes auf einem Schlag. Steht ein Haus, gehen seine Fenster an und aus dem
// Schornstein steigt Rauch; zuletzt zuenden sich die Laternen entlang der
// Gasse an. Die Kamera kreist erst hoch ueber dem Tal und senkt sich dann in
// die Gasse vor der Kirche. Im Abspann (`ende`) hebt sie den Blick zum Himmel,
// wo der Stern ueber dem Dorf steht (`sternPos`, in Bildpunkten -- main.dh
// laesst von dort den Lichterschwarm aufsteigen).
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float fort;
uniform float ende;
uniform vec2 sternPos;
uniform float kick;
uniform float ton;
uniform float hoehe;
uniform vec3 farbe;
uniform float aus;
uniform float ein;
uniform vec3 uebLicht;
uniform float guete;
out vec4 finalColor;

float G() { return guete > 0.0 ? guete : 1.0; }

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float rausch(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}

const float PI = 3.14159265;
const vec3 MOND = vec3(-0.4493, 0.4194, 0.7888);
const float ZELLE = 8.0;        // ein Haus je Zelle
const float DORF = 27.0;        // Radius des Dorfes, dahinter Wald
const float WZELLE = 4.5;       // eine Tanne je Waldzelle
const vec3 KIRCHE = vec3(0.0, 0.0, 4.5);
const vec3 BAUM = vec3(2.9, 0.0, -8.5);   // der Dorfbaum vor der Kirche

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}
float sdKegel(vec3 p, float r, float h) {
    // Kegel mit Spitze im Ursprung, Grundflaeche bei y = -h, Radius r
    vec2 q = vec2(r, -h);
    vec2 w = vec2(length(p.xz), p.y);
    vec2 a = w - q * clamp(dot(w, q) / dot(q, q), 0.0, 1.0);
    vec2 b = w - q * vec2(clamp(w.x / q.x, 0.0, 1.0), 1.0);
    float k = sign(q.y);
    float d = min(dot(a, a), dot(b, b));
    float s = max(k * (w.x * q.y - w.y * q.x), k * (w.y - q.y));
    return sqrt(d) * sign(s);
}

// Wie weit das Dorf steht: 0 = nur Schnee und Wald, 1 = alles gebaut
float aufbau() { return clamp((fort - 0.05) / 0.58, 0.0, 1.0); }

// Der Boden: im Dorf eben, dahinter steigen sanfte Huegel an
float gelaende(vec2 xz) {
    float r = length(xz);
    float h = smoothstep(DORF - 2.0, 75.0, r) * 10.0;
    h += (rausch(xz * 0.045) - 0.5) * 4.0 * smoothstep(DORF, DORF + 15.0, r);
    return h + 0.08 * rausch(xz * 0.6);
}

// --- Die Haeuser --------------------------------------------------------------
// Je Zelle ein Haus mit eigenem Mass, eigener Richtung und eigenem Beginn.
// Frei bleiben die Gasse (x-Zelle 0, suedlich) und der Platz der Kirche.
struct Haus {
    float da;       // 0 = kein Haus
    vec2 ort;
    float W;        // halbe Breite (entlang des Firsts)
    float D;        // halbe Tiefe
    float h;        // Wandhoehe
    float rh;       // Dachhoehe
    float dreh;     // 1 = First entlang z
    float u;        // wie weit es steht (0 .. 1)
    float vor;      // kurz davor: der Grundriss leuchtet auf (0 .. 1)
    float seed;
};
Haus hausIn(vec2 id) {
    Haus H;
    vec2 mitte = (id + 0.5) * ZELLE - vec2(ZELLE * 0.5, ZELLE * 0.5);
    H.seed = hash(id + 0.37) * 100.0;
    float r = length(mitte);
    bool gasse = id.x == 0.0 && id.y < 0.0;
    bool platz = id.x == 0.0 && (id.y == 0.0 || id.y == 1.0);
    H.da = (r < DORF && !gasse && !platz && hash(id + 5.1) > 0.1) ? 1.0 : 0.0;
    H.W = 1.6 + 0.8 * hash(id + 1.3);
    H.D = 1.5 + 0.7 * hash(id + 2.9);
    H.h = 2.4 + floor(hash(id + 4.4) * 3.0) * 0.55;
    H.rh = 1.6 + 0.8 * hash(id + 6.2);
    // an der Gasse stehen die Haeuser traufseitig zur Strasse
    H.dreh = abs(id.x) == 1.0 && id.y < 0.0 ? 1.0 : step(0.5, hash(id + 7.7));
    float spiel = ZELLE * 0.5 - max(H.W, H.D) - 0.7;
    H.ort = mitte + (vec2(hash(id + 8.1), hash(id + 9.3)) - 0.5) * 2.0 * max(spiel, 0.0);
    // Beginn: von der Mitte nach aussen, etwas gewuerfelt, auf einen Schlag
    // gerundet (64 Schlaege hat die Szene)
    float beginn = 0.12 + 0.62 * clamp(r / DORF, 0.0, 1.0) + 0.1 * hash(id + 3.3);
    beginn = floor(beginn * 40.0) / 40.0;
    H.u = clamp((aufbau() - beginn) / 0.11, 0.0, 1.0);
    H.vor = clamp((aufbau() - beginn + 0.05) / 0.05, 0.0, 1.0);
    return H;
}
// Wie tief das Haus noch im Schnee steckt: es waechst heraus und federt nach
float versenkt(Haus H) {
    float x = smoothstep(0.0, 0.7, H.u);
    float feder = 0.12 * sin(clamp((H.u - 0.55) / 0.45, 0.0, 1.0) * PI);
    return (1.0 - x) * (H.h + H.rh + 1.2) - feder;
}
// p in die Lage des Hauses: Mitte des Grundrisses auf dem Boden, First entlang x
vec3 hausLokal(vec3 p, Haus H) {
    vec3 l = p - vec3(H.ort.x, -versenkt(H), H.ort.y);
    return H.dreh > 0.5 ? vec3(l.z, l.y, -l.x) : l;
}
vec3 schornstein(Haus H) {
    return vec3(0.45 * H.W, H.h + H.rh * 0.75 + 0.45, 0.35 * H.D);
}
// Abstand zum Haus; m: 1 Wand, 2 Dach, 3 Schornstein
float hausD(vec3 l, Haus H, out float m) {
    float wand = sdBox(l - vec3(0.0, H.h * 0.5, 0.0), vec3(H.W, H.h * 0.5, H.D));
    m = 1.0;
    float y = l.y - H.h;
    vec2 n = normalize(vec2(H.rh, H.D + 0.35));
    float dach = max(max(dot(n, vec2(abs(l.z), y)) - n.x * (H.D + 0.35), -y - 0.15), abs(l.x) - (H.W + 0.25));
    dach -= 0.05;
    float d = wand;
    if (dach < d) { d = dach; m = 2.0; }
    vec3 sc = schornstein(H);
    float sch = sdBox(l - vec3(sc.x, (H.h + sc.y) * 0.5, sc.z), vec3(0.2, (sc.y - H.h) * 0.5, 0.2));
    if (sch < d) { d = sch; m = 3.0; }
    return d;
}

// --- Die Kirche ------------------------------------------------------------------
// Schiff mit Satteldach, davor der Turm mit spitzem Helm, Uhr und Glockenstube.
// Sie waechst als erstes aus dem Schnee.
float kircheU() { return clamp(aufbau() / 0.14, 0.0, 1.0); }
float kircheD(vec3 p, out float m) {
    vec3 q = p - KIRCHE;
    m = 4.0;
    float huelle = sdBox(q - vec3(0.0, 11.0, 1.5), vec3(3.2, 11.0, 7.4));
    if (huelle > 0.3) return huelle;
    q.y += (1.0 - smoothstep(0.0, 0.8, kircheU())) * 20.0;
    float schiff = sdBox(q - vec3(0.0, 3.2, 3.0), vec3(2.6, 3.2, 5.2));
    m = 4.0;
    float y = q.y - 6.4;
    vec2 n = normalize(vec2(3.0, 2.95));
    float dach = max(max(dot(n, vec2(abs(q.x), y)) - n.x * 2.95, -y - 0.15), abs(q.z - 3.0) - 5.4) - 0.05;
    float d = schiff;
    if (dach < d) { d = dach; m = 5.0; }
    float turm = sdBox(q - vec3(0.0, 6.6, -3.6), vec3(1.5, 6.6, 1.5));
    if (turm < d) { d = turm; m = 4.0; }
    vec3 h = q - vec3(0.0, 13.2, -3.6);
    vec2 hn = normalize(vec2(7.0, 1.65));
    float helm = max(max(dot(hn, vec2(abs(h.x), h.y)), dot(hn, vec2(abs(h.z), h.y))) - hn.x * 1.65, -h.y);
    if (helm < d) { d = helm; m = 5.0; }
    return d;
}

// --- Der Dorfbaum ---------------------------------------------------------------
float baumU() { return clamp((aufbau() - 0.82) / 0.12, 0.0, 1.0); }
float dorfbaumD(vec3 p) {
    vec3 q = p - BAUM;
    float huelle = max(length(q.xz) - 2.2, q.y - 8.0);
    if (huelle > 0.3) return huelle;
    float s = smoothstep(0.0, 0.6, baumU());
    if (s < 0.01) return 1e5;
    q /= s;
    float d = 1e5;
    for (int i = 0; i < 3; i++) {
        float fi = float(i);
        d = min(d, sdKegel(q - vec3(0.0, 3.0 + fi * 1.7, 0.0), 1.9 - fi * 0.45, 2.6));
    }
    d = min(d, max(length(q.xz) - 0.18, q.y - 0.6));
    return d * s;
}

// --- Laternen entlang der Gasse und am Platz ------------------------------------
const int LATERNEN = 10;
float laterneHoch() { return smoothstep(0.76, 0.84, aufbau()); }
vec3 laterne(int i) {
    float fi = float(i);
    float y = 2.7 - (1.0 - laterneHoch()) * 3.2;
    if (i < 8) return vec3(i % 2 == 0 ? -3.2 : 3.2, y, -7.0 - floor(fi * 0.5) * 7.5);
    return vec3(i == 8 ? -3.4 : 3.4, y, 0.2);
}
// Die Laternen zuenden sich zuletzt an, von der Kirche die Gasse hinab
float laterneAn(int i) {
    float reihe = i < 8 ? floor(float(i) * 0.5) + 1.0 : 0.0;
    return smoothstep(0.0, 0.03, aufbau() - 0.86 - reihe * 0.025);
}

// --- Der Wald ---------------------------------------------------------------------
// Eine Tanne je Zelle ausserhalb des Dorfes (manche fehlen), auf dem Gelaende
// (das Raster ist gedreht, damit keine Reihen auf die Kamera zulaufen)
const mat2 WDREH = mat2(0.8253, 0.5646, -0.5646, 0.8253);
float tanneD(vec3 p, out float schnee) {
    vec2 pr = WDREH * p.xz;
    vec2 id = floor(pr / WZELLE);
    vec2 mitteR = (id + 0.5) * WZELLE + (vec2(hash(id + 1.1), hash(id + 2.2)) - 0.5) * 1.0;
    vec2 mitte = mitteR * WDREH;
    float r = length(mitte);
    // Rand der Zelle: so weit darf der Strahl auch ohne Baum
    vec2 rel = pr - (id + 0.5) * WZELLE;
    float rand = WZELLE * 0.5 - max(abs(rel.x), abs(rel.y)) + 0.2;
    schnee = 0.0;
    // ueber den Wipfeln darf der Strahl ueber die Zellgrenzen hinweg
    float ueber = (p.y - gelaende(p.xz) - 7.0) * 0.8;
    if (ueber > 0.0) return max(rand, ueber);
    if (r < DORF + 3.0 || hash(id + 4.4) < 0.18) return rand;
    float gross = 0.7 + 0.45 * hash(id + 3.3);
    vec3 q = p - vec3(mitte.x, gelaende(mitte), mitte.y);
    q /= gross;
    float d = 1e5;
    for (int i = 0; i < 3; i++) {
        float fi = float(i);
        d = min(d, sdKegel(q - vec3(0.0, 2.6 + fi * 1.5, 0.0), 1.3 - fi * 0.32, 2.2));
    }
    d = min(d, max(length(q.xz) - 0.15, q.y - 0.6));
    return min(d * gross, rand);
}

// --- Die ganze Szene -------------------------------------------------------------
// Material: 0 Boden, 1 Wand, 2 Dach, 3 Schornstein, 4 Kirchenwand, 5 Kirchendach,
// 6 Tanne, 7 Laterne, 8 Dorfbaum
float mat;
vec2 zelle;
float szene(vec3 p) {
    float d = (p.y - gelaende(p.xz)) * 0.8;
    mat = 0.0;
    // das Haus der Zelle -- oder, wo keins steht, der Abstand bis zum Zellrand
    vec2 id = floor((p.xz + ZELLE * 0.5) / ZELLE);
    vec2 rel = p.xz + ZELLE * 0.5 - (id + 0.5) * ZELLE;
    float rand = ZELLE * 0.5 - max(abs(rel.x), abs(rel.y)) + 0.3;
    zelle = id;
    if (length(p.xz) < DORF + ZELLE) {
        Haus H = hausIn(id);
        rand = max(rand, p.y - 7.6);
        if (H.da > 0.5 && H.u > 0.0) {
            float hm;
            float hd = min(hausD(hausLokal(p, H), H, hm), rand);
            if (hd < d) { d = hd; mat = hm; }
        } else {
            d = min(d, rand);
        }
    }
    float km;
    float kd = kircheD(p, km);
    if (kd < d) { d = kd; mat = km; }
    float s;
    float td = tanneD(p, s);
    if (td < d) { d = td; mat = 6.0; }
    float bd = dorfbaumD(p);
    if (bd < d) { d = bd; mat = 8.0; }
    // die Laternen: Pfahl und Kopf
    float lh = sdBox(p - vec3(0.0, 1.6, -13.5), vec3(3.8, 1.6, 14.5));
    if (lh > 0.3) return min(d, lh);
    for (int i = 0; i < LATERNEN; i++) {
        vec3 lq = p - laterne(i);
        float pf = max(length(lq.xz) - 0.06, lq.y);
        pf = max(pf, -lq.y - 2.8);
        pf = min(pf, sdBox(lq - vec3(0.0, 0.18, 0.0), vec3(0.14, 0.2, 0.14)));
        if (pf < d) { d = pf; mat = 7.0; }
    }
    return d;
}

vec3 normale(vec3 p) {
    vec2 e = vec2(0.003, 0.0);
    return normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
}

// Die Fassade: Fachwerk aus dunklen Balken auf hellem Putz, Fenster in zwei
// Geschossen (so viele, wie die Wand breit ist), Andreaskreuze darunter. f ist
// die Lage auf der Wand (Mitte = 0), W ihre halbe Breite. In `fenster` das
// warme Licht, das herauskommt; `licht` sagt, wie weit das Haus erleuchtet ist.
float fenster;
vec3 fassade(vec2 f, float W, float h, float seed, float licht) {
    fenster = 0.0;
    vec3 putz = vec3(0.82, 0.76, 0.66);
    float b = 0.055;
    float balken = 0.0;
    balken = max(balken, step(abs(abs(f.x) - W + 0.04), b));
    balken = max(balken, step(abs(f.y - 0.06), b));
    balken = max(balken, step(abs(f.y - 1.45), b));
    balken = max(balken, step(abs(f.y - (h - 0.08)), b));
    if (h > 3.2) balken = max(balken, step(abs(f.y - 2.8), b));
    float n = max(1.0, floor(2.0 * W / 1.15));
    float spalte = clamp(floor((f.x + n * 0.575) / 1.15), 0.0, n - 1.0);
    float u = f.x - (-(n - 1.0) * 0.575 + spalte * 1.15);
    balken = max(balken, step(abs(abs(u) - 0.575), b * 0.8) * step(abs(f.x), W - 0.3));
    for (int g = 0; g < 3; g++) {
        float unten = 0.55 + float(g) * 1.2;
        float oben = unten + 0.72;
        if (oben > h - 0.2) break;
        float v = f.y - (unten + oben) * 0.5;
        if (abs(u) < 0.27 && abs(v) < 0.36) {
            float wurf = hash(vec2(seed + spalte * 3.1, float(g) * 7.0 + W));
            float an = step(0.2, wurf) * step(fract(wurf * 13.7), licht);
            float sprosse = step(abs(u), 0.022) + step(abs(v - 0.06), 0.022);
            float rahmen = step(0.24, abs(u)) + step(0.33, abs(v));
            if (sprosse + rahmen > 0.0) {
                balken = 1.0;
            } else {
                float vorhang = smoothstep(0.12, 0.2, abs(u));
                vec3 innen = mix(vec3(1.0, 0.62, 0.28), vec3(0.75, 0.25, 0.12), vorhang * 0.6);
                fenster = an * (0.9 + 0.1 * sin(time * 3.0 + spalte * 5.0 + seed));
                // in manchen ein Schwibbogen
                if (fract(wurf * 31.3) < 0.35 && an > 0.5) {
                    vec2 bm = vec2(u, v + 0.3);
                    float lichter = 0.0;
                    for (int k = 0; k < 7; k++) {
                        float a = 0.3 + float(k) * 0.42;
                        lichter += smoothstep(0.024, 0.0, length(bm - vec2(cos(a), sin(a)) * 0.2));
                    }
                    float bogen = smoothstep(0.012, 0.0, abs(length(bm) - 0.17)) * step(0.0, bm.y);
                    innen = innen * 0.3 + vec3(0.25, 0.12, 0.05) * bogen + vec3(1.0, 0.85, 0.5) * clamp(lichter * 2.0, 0.0, 1.0);
                }
                return innen * fenster + vec3(0.02, 0.025, 0.04) * (1.0 - an);
            }
        }
    }
    float vv = f.y - 0.3;
    if (abs(u) < 0.5 && abs(vv) < 0.22) {
        balken = max(balken, step(min(abs(vv - u * 0.44), abs(vv + u * 0.44)), 0.035));
    }
    putz *= 0.85 + 0.15 * rausch(f * 7.0);
    return mix(putz, vec3(0.12, 0.06, 0.035), balken);
}

vec3 sterne(vec3 d) {
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 160.0;
    vec2 z = floor(uv), f = fract(uv) - 0.5;
    float h = hash(z);
    float s = smoothstep(0.09, 0.0, length(f - (vec2(hash(z + 3.1), hash(z + 7.7)) - 0.5) * 0.6));
    float fun = 0.6 + 0.4 * sin(time * (1.0 + h * 3.0) + h * 40.0);
    return vec3(0.85, 0.9, 1.0) * s * step(0.982, h) * fun;
}

vec3 himmel(vec3 d) {
    float h = clamp(d.y, 0.0, 1.0);
    vec3 c = mix(vec3(0.06, 0.09, 0.19), vec3(0.008, 0.012, 0.04), pow(h, 0.6));
    c += sterne(d) * (1.0 + 1.0 * ton);
    float m = dot(d, MOND);
    c += vec3(0.95, 0.95, 0.88) * smoothstep(0.99935, 0.9996, m);
    c += vec3(0.25, 0.3, 0.45) * pow(max(m, 0.0), 120.0) * 1.2;
    c += vec3(0.12, 0.14, 0.22) * pow(max(m, 0.0), 8.0) * 0.4;
    // Berge in der Ferne
    float az = atan(d.x, d.z);
    float berg = 0.05 + 0.03 * sin(az * 5.0 + 1.0) + 0.015 * sin(az * 13.0) + 0.006 * sin(az * 41.0);
    c = mix(c, vec3(0.08, 0.1, 0.17), smoothstep(berg + 0.002, berg - 0.002, d.y));
    return c;
}

// Der Stern im Abspann: ein heller Kern, acht Strahlen, langsam pulsierend
vec3 stern(vec2 uv) {
    vec2 sp = vec2(sternPos.x - 0.5 * resolution.x, 0.5 * resolution.y - sternPos.y) / resolution.y;
    vec2 q = uv - sp;
    float r = length(q);
    float a = atan(q.y, q.x);
    float strahl = pow(abs(cos(a * 2.0)), 60.0) + pow(abs(cos(a * 2.0 + 0.785)), 80.0) * 0.5;
    float puls = 0.85 + 0.15 * sin(time * 2.0) + 0.3 * ton;
    vec3 c = vec3(1.0, 0.9, 0.7) * (exp(-r * 90.0) * 3.0 + exp(-r * 14.0) * 0.5 + strahl * exp(-r * 9.0) * 0.9) * puls;
    c += vec3(1.0, 0.75, 0.45) * exp(-r * 4.0) * 0.15;
    return c * ende;
}

// Das Licht der Laternen auf Schnee und Waenden
vec3 laternenLicht(vec3 p, vec3 n) {
    vec3 l = vec3(0.0);
    for (int i = 0; i < LATERNEN; i++) {
        vec3 v = laterne(i) + vec3(0.0, 0.15, 0.0) - p;
        float dd = dot(v, v);
        l += vec3(1.0, 0.68, 0.35) * max(dot(n, v * inversesqrt(dd)), 0.0) / (1.0 + dd * 0.35) * 3.0 * laterneAn(i) * (1.0 + 0.3 * kick);
    }
    return l;
}
vec3 laternenGlanz(vec3 ro, vec3 rd, float tmax) {
    vec3 c = vec3(0.0);
    for (int i = 0; i < LATERNEN; i++) {
        vec3 lp = laterne(i) + vec3(0.0, 0.15, 0.0);
        float tc = dot(lp - ro, rd);
        if (tc < 0.0 || tc > tmax + 0.3) continue;
        vec3 q = ro + rd * tc - lp;
        float d2 = dot(q, q) / max(tc * 0.02, 1.0);
        c += vec3(1.0, 0.72, 0.38) * (exp(-d2 * 30.0) * 2.4 + exp(-d2 * 1.5) * 0.2) * laterneAn(i) * (1.0 + 0.4 * kick);
    }
    return c;
}

// Rauch aus den Schornsteinen: eine Saeule, die breiter wird und im Wind
// treibt, mit Rauschen; Dichte an der Stelle p
float rauch(vec3 p) {
    vec2 id = floor((p.xz + ZELLE * 0.5) / ZELLE);
    if (length(p.xz) > DORF + ZELLE) return 0.0;
    Haus H = hausIn(id);
    if (H.da < 0.5 || H.u < 0.9) return 0.0;
    vec3 sc = schornstein(H);
    vec3 top = H.dreh > 0.5 ? vec3(-sc.z, sc.y, sc.x) : sc;
    top += vec3(H.ort.x, 0.0, H.ort.y);
    vec3 v = p - top;
    if (v.y < 0.0 || v.y > 7.0) return 0.0;
    vec2 trieb = vec2(0.35, 0.12) * v.y;
    float rr = 0.25 + v.y * 0.16;
    float dichte = exp(-dot(v.xz - trieb, v.xz - trieb) / (rr * rr)) * (1.0 - v.y / 7.0);
    dichte *= smoothstep(0.9, 1.0, H.u) * (0.4 + 0.9 * rausch(vec2(v.y * 1.3 - time * 0.9, (v.x + v.z) * 1.5 + H.seed)));
    return dichte;
}

float uebHash(float x) { return fract(sin(x * 91.7) * 43758.5453); }
vec3 uebergang(vec3 c, vec2 uv) {
    float r = length(uv);
    float maske;
    float k;
    if (aus > 0.0) {
        float R = aus * 1.7;
        maske = 1.0 - smoothstep(R - 0.6, R, r);
        k = aus;
    } else if (ein < 1.0) {
        float R = ein * 1.7;
        maske = smoothstep(R - 0.6, R, r);
        k = 1.0 - ein;
    } else {
        return c;
    }
    vec3 l = mix(uebLicht, vec3(1.0), exp(-r * 2.2)) * 1.05;
    c = mix(c, l, maske * 0.9);
    float a = atan(uv.y, uv.x);
    float id = floor(a * 60.0);
    float strahl = pow(uebHash(id), 5.0) * (0.6 + 0.4 * sin(time * 2.5 + id * 1.7));
    c += mix(uebLicht, vec3(1.0), 0.5) * strahl * k * smoothstep(0.08, 0.7, r) * 0.5;
    return c;
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    // Kamera: erst ein weiter Bogen hoch ueber dem Tal, waehrend das Dorf
    // waechst, dann hinab in die Gasse vor der Kirche; im Abspann hebt sie
    // den Blick
    float s = smoothstep(0.0, 0.62, fort);
    float w = -PI * 0.5 - 0.75 + 1.05 * s;
    float R = mix(62.0, 36.0, s);
    vec3 roA = vec3(cos(w) * R, mix(34.0, 15.0, s), sin(w) * R);
    vec3 taA = vec3(0.0, mix(9.0, 3.0, s), 0.0);
    float e = smoothstep(0.62, 0.97, fort);
    vec3 roB = vec3(0.8, 1.9, -24.0);
    vec3 taB = vec3(0.0, 5.0, 6.0);
    vec3 ro = mix(roA, roB, vec3(e, e * e, e));
    vec3 ta = mix(taA, taB, e);
    ta.y += ende * 6.0;
    vec3 vorne = normalize(ta - ro);
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.4 + uv.x * rechts + uv.y * oben);

    vec3 nebel = vec3(0.07, 0.09, 0.17);
    vec3 c;
    float t = 0.0;
    float hit = -1.0;
    float qualm = 0.0;
    int schritte = int(150.0 * G());
    for (int i = 0; i < 150; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float d = szene(p);
        if (d < 0.0012 * t + 0.0005) { hit = t; break; }
        float schritt = d * 0.85;
        // im Rauch kleinere Schritte, damit er sich sammelt
        float r = rauch(p);
        if (r > 0.002) schritt = min(schritt, 0.45);
        qualm += r * schritt;
        t += schritt;
        if (t > 160.0) break;
        // gehen die Schritte aus, nahe an etwas: dort zaehlt es als Treffer
        // (sonst malte der Himmel dunkle Vorhaenge ueber den Wald)
        if (i == schritte - 1 && d < 0.5) hit = t;
    }
    float tmax = 160.0;
    if (hit > 0.0) {
        tmax = hit;
        vec3 p = ro + rd * hit;
        szene(p);
        float m = mat;
        vec2 zi = zelle;
        vec3 n = normale(p);
        float mond = max(dot(n, MOND), 0.0);
        vec3 himmelLicht = vec3(0.08, 0.1, 0.18) * (0.6 + 0.4 * n.y);
        vec3 alb;
        vec3 eigen = vec3(0.0);
        bool schnee = m < 0.5 || (n.y > 0.45 && m < 5.5) || (m > 5.5 && m < 6.5 && n.y > 0.1 && rausch(p.xz * 6.0 + p.y * 3.0) > 0.35) || (m > 7.5 && n.y > 0.3 && rausch(p.xz * 5.0 + p.y * 4.0) > 0.55);
        if (m > 5.5 && m < 6.5 && !schnee) {
            alb = vec3(0.02, 0.07, 0.04) * (0.7 + 0.6 * rausch(p.xz * 9.0 + p.y * 7.0));
        } else if (m > 7.5 && !schnee) {
            // der Dorfbaum: Tannengruen mit bunten Lichtern
            vec3 q = p - BAUM;
            vec2 g = vec2(atan(q.z, q.x) * 3.0, q.y * 2.2);
            vec2 gi = floor(g);
            float lampe = smoothstep(0.22, 0.08, length(fract(g) - 0.5)) * step(0.35, hash(gi));
            float wahl = hash(gi + 3.0);
            vec3 lf = wahl < 0.33 ? vec3(1.0, 0.25, 0.15) : wahl < 0.66 ? vec3(1.0, 0.8, 0.3) : vec3(0.3, 0.55, 1.0);
            alb = vec3(0.02, 0.08, 0.04);
            eigen += lf * lampe * 3.0 * (0.7 + 0.3 * sin(time * 4.0 + wahl * 30.0)) * baumU();
        } else if (schnee) {
            alb = vec3(0.85, 0.9, 1.0);
            float funkel = step(0.995, hash(floor(p.xz * 60.0) + floor(time * 4.0)));
            eigen += vec3(1.0) * funkel * 0.6 * (1.0 + ton);
        } else if (m > 1.5 && m < 2.5 && abs(n.y) < 0.3) {
            // das Giebeldreieck: Putz mit Stiel und Riegel
            Haus H = hausIn(zi);
            vec3 l = hausLokal(p, H);
            float balken = max(step(abs(l.z), 0.06), step(abs(l.y - H.h - H.rh * 0.4), 0.05));
            alb = mix(vec3(0.8, 0.74, 0.64) * (0.85 + 0.15 * rausch(l.zy * 7.0)), vec3(0.12, 0.06, 0.035), balken);
        } else if (m < 1.5) {
            // Hauswand
            Haus H = hausIn(zi);
            vec3 l = hausLokal(p, H);
            vec3 ln = H.dreh > 0.5 ? vec3(n.z, n.y, -n.x) : n;
            float licht = smoothstep(0.75, 1.0, H.u);
            if (abs(ln.z) > 0.7) {
                alb = fassade(vec2(l.x, l.y), H.W, H.h, H.seed + sign(ln.z), licht);
            } else {
                alb = fassade(vec2(l.z, l.y), H.D, H.h, H.seed + 5.0 + sign(ln.x), licht);
            }
            eigen += alb * fenster * 1.7;
            alb *= 1.0 - fenster;
            // ein Haus, das gerade fertig wird, glueht kurz golden auf
            eigen += vec3(1.0, 0.7, 0.35) * 0.35 * smoothstep(0.6, 0.85, H.u) * (1.0 - smoothstep(0.85, 1.0, H.u));
        } else if (m < 2.5) {
            alb = vec3(0.2, 0.08, 0.05);   // Ziegel unter dem Schnee, an der Traufe
        } else if (m < 3.5) {
            alb = vec3(0.3, 0.12, 0.08);
        } else if (m < 4.5) {
            // die Kirche: heller Putz, hohe Fenster in warmem Buntglas, am Turm
            // die Uhr und die Glockenstube
            vec3 q = p - KIRCHE;
            q.y += (1.0 - smoothstep(0.0, 0.8, kircheU())) * 20.0;
            alb = vec3(0.78, 0.74, 0.68) * (0.85 + 0.15 * rausch(p.xy * 5.0 + p.z));
            float an = smoothstep(0.6, 1.0, kircheU());
            vec2 f = abs(n.x) > 0.7 ? vec2(q.z, q.y) : vec2(q.x, q.y);
            if (abs(n.x) > 0.7 && q.z > -2.0) {
                // Fenster im Schiff: Spitzbogen, alle 2.4 Einheiten
                float fz = mod(q.z - 0.2, 2.4) - 1.2;
                vec2 fp = vec2(fz, q.y - 3.0);
                float fe = max(abs(fp.x) - 0.42, abs(fp.y) - 1.3);
                fe = min(fe, length(vec2(abs(fp.x) + 0.25, fp.y - 1.3)) - 0.67);
                fe = max(fe, -fp.y - 1.3);
                if (fe < 0.0) {
                    vec2 bg = floor(fp * vec2(5.0, 4.0));
                    float wahl = hash(bg);
                    vec3 glas = wahl < 0.3 ? vec3(1.0, 0.35, 0.15) : wahl < 0.6 ? vec3(1.0, 0.75, 0.3) : wahl < 0.8 ? vec3(0.4, 0.5, 1.0) : vec3(0.4, 0.9, 0.5);
                    float blei = step(0.85, max(fract(fp.x * 5.0), fract(fp.y * 4.0)));
                    alb = vec3(0.05);
                    eigen += glas * (1.0 - blei) * 1.4 * an;
                }
            }
            if (abs(n.z) > 0.7 && q.z < -2.0) {
                // Turm: Uhr und Schallfenster der Glockenstube
                vec2 u2 = vec2(q.x, q.y - 10.0);
                float r = length(u2);
                if (r < 0.75) {
                    float wz = atan(u2.x, u2.y);
                    float zeiger = step(abs(u2.x), 0.035) * step(0.0, u2.y) * step(u2.y, 0.6);
                    zeiger = max(zeiger, step(abs(u2.x * cos(0.5) - u2.y * sin(0.5)), 0.05) * step(r, 0.42) * step(0.0, u2.x * sin(0.5) + u2.y * cos(0.5)));
                    float strich = step(0.6, r) * step(abs(mod(wz + PI / 12.0, PI / 6.0) - PI / 12.0) * r, 0.03);
                    alb = vec3(0.05);
                    eigen += vec3(1.0, 0.8, 0.5) * (1.0 - max(zeiger, strich) * 0.9) * 1.2 * an;
                }
                vec2 sf = vec2(abs(q.x) - 0.5, q.y - 11.8);
                if (abs(sf.x) < 0.25 && abs(sf.y) < 0.5) {
                    alb = vec3(0.02);
                    eigen += vec3(1.0, 0.65, 0.3) * 0.6 * (0.6 + 0.4 * kick) * an;
                }
                // die Tuer
                if (abs(q.x) < 0.6 && q.y < 2.1 && q.y > 0.0) {
                    alb = vec3(0.15, 0.07, 0.03);
                    eigen += vec3(1.0, 0.6, 0.25) * 0.4 * an * step(abs(q.x), 0.55) * step(q.y, 2.0) * step(0.03, abs(q.x));
                }
            }
        } else if (m < 5.5) {
            alb = vec3(0.18, 0.2, 0.24);
        } else if (m < 6.5) {
            alb = vec3(0.02, 0.07, 0.04);
        } else {
            alb = vec3(0.03);
            // der Kopf der Laterne leuchtet
            for (int i = 0; i < LATERNEN; i++) {
                vec3 lq = p - laterne(i);
                if (abs(lq.y - 0.18) < 0.21 && length(lq.xz) < 0.2) eigen += vec3(1.0, 0.75, 0.4) * 2.0 * laterneAn(i);
            }
        }
        vec3 licht = himmelLicht * 1.6 + vec3(0.5, 0.58, 0.8) * mond * 0.9 + laternenLicht(p, n);
        // Fensterlicht faellt auf den Schnee vor den Haeusern
        if (m < 0.5) {
            Haus H = hausIn(zi);
            if (H.da > 0.5) {
                vec3 l = hausLokal(p, H);
                vec2 aussen = max(abs(l.xz) - vec2(H.W, H.D), 0.0);
                float ab = length(aussen);
                licht += vec3(1.0, 0.6, 0.3) * 0.6 * exp(-ab * 1.3) * smoothstep(0.75, 1.0, H.u);
                // Schneestaub um ein Haus, das gerade herauswaechst
                eigen += vec3(0.6, 0.65, 0.75) * exp(-ab * 1.5) * sin(clamp(H.u / 0.75, 0.0, 1.0) * PI) * 0.5;
                // kurz bevor es kommt, leuchtet sein Grundriss golden im Schnee
                vec2 kante = abs(l.xz) - vec2(H.W, H.D);
                float rahmen = abs(length(max(kante, 0.0)) + min(max(kante.x, kante.y), 0.0));
                float glut = H.vor * (1.0 - smoothstep(0.2, 0.6, H.u));
                eigen += vec3(1.0, 0.7, 0.3) * (exp(-rahmen * 14.0) * 2.0 + exp(-rahmen * 2.5) * 0.25) * glut;
            }
            vec3 kq = p - KIRCHE;
            float ka = length(max(abs(kq.xz - vec2(0.0, 1.0)) - vec2(2.6, 6.6), 0.0));
            eigen += vec3(0.6, 0.65, 0.75) * exp(-ka * 1.0) * sin(clamp(kircheU() / 0.8, 0.0, 1.0) * PI) * 0.6;
            licht += vec3(1.0, 0.75, 0.4) * 0.5 * exp(-length(p.xz - BAUM.xz) * 0.5) * baumU();
        }
        c = alb * licht + eigen;
        c = mix(c, nebel, 1.0 - exp(-hit * 0.012));
    } else {
        c = himmel(rd);
    }
    // der Rauch, vom Mond und den Fenstern unten angeleuchtet
    c = mix(c, vec3(0.32, 0.34, 0.42), 1.0 - exp(-qualm * 0.9));
    c += laternenGlanz(ro, rd, tmax);
    if (ende > 0.0) c += stern(uv);
    c *= 1.0 - 0.32 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.92));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
