#version 330
// Das Dorf im Schnee: eine Gasse zwischen Fachwerkhaeusern bei Nacht. Auf den
// Daechern liegt Schnee (wo eine Flaeche nach oben zeigt), die Fenster
// leuchten warm, in manchen steht ein Schwibbogen, Laternen werfen Licht auf
// den Schnee, am Ende der Gasse der Kirchturm, darueber Mond und Sterne. Die
// Kamera geht langsam die Gasse hinauf (`fort`). Im Abspann (`ende`) hebt sie
// den Blick zum Himmel, wo der Stern ueber dem Dorf steht (`sternPos`, in
// Bildpunkten -- main.dh laesst von dort den Lichterschwarm aufsteigen).
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

const float ZELLE = 4.2;       // ein Haus je Zelle entlang der Gasse
const float FASSADE = 2.6;     // Abstand der Hauswand von der Mitte
const vec3 MOND = vec3(-0.4493, 0.4194, 0.7888);

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Hoehe der Wand des Hauses in Zelle id auf Seite s
float wandHoehe(float id, float s) { return 2.6 + floor(hash(vec2(id, s)) * 3.0) * 0.45; }

// Material: 0 Schnee am Boden, 1 Wand, 2 Dach, 3 Laternenpfahl
float mat;
vec3 zellInfo;   // id, Seite, Wandhoehe
float szene(vec3 p) {
    // Boden: Schnee mit flachen Wehen, an den Haeusern etwas angeweht, in der
    // Mitte zwei Spuren
    float boden = p.y - 0.06 * rausch(p.xz * 0.7) - 0.05 * smoothstep(1.6, 2.6, abs(p.x));
    boden += 0.02 * smoothstep(0.25, 0.0, abs(abs(p.x) - 0.6));
    float d = boden;
    mat = 0.0;
    float s = sign(p.x);
    float id = floor(p.z / ZELLE);
    float h = wandHoehe(id, s);
    vec3 q = vec3(abs(p.x) - (FASSADE + 1.7), p.y, p.z - (id + 0.5) * ZELLE);
    // Wand
    float w = sdBox(q - vec3(0.0, h * 0.5, 0.0), vec3(1.7, h * 0.5, 1.85));
    if (w < d) { d = w; mat = 1.0; }
    // Dach: ein Prisma, First entlang der Gasse, mit Ueberstand; die
    // Schneeschicht macht es oben runder
    float rh = 1.5 + hash(vec2(id, s + 4.0)) * 0.5;
    vec2 n = normalize(vec2(1.0 / 2.0, 1.0 / rh));
    float y = q.y - h;
    float dach = max(max(dot(n, vec2(abs(q.x), y)) - dot(n, vec2(2.0, 0.0)), -y - 0.12), abs(q.z) - 2.0);
    dach -= 0.06;
    if (dach < d) { d = dach; mat = 2.0; }
    // Laternen: alle zwei Zellen ein Pfahl am Gassenrand
    vec3 lp = vec3(abs(p.x) - 2.25, p.y, mod(p.z, ZELLE * 2.0) - ZELLE);
    float pfahl = length(lp.xz) - 0.05;
    pfahl = max(pfahl, p.y - 2.5);
    float kopf = sdBox(lp - vec3(0.0, 2.6, 0.0), vec3(0.12, 0.16, 0.12));
    pfahl = min(pfahl, kopf);
    if (pfahl < d) { d = pfahl; mat = 3.0; }
    zellInfo = vec3(id, s, h);
    return d;
}

vec3 normale(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
}

// Die Fassade: Fachwerk aus dunklen Balken auf hellem Putz, Fenster in zwei
// Geschossen, Andreaskreuze unter den Fenstern. Liefert die Farbe und in
// `fenster` das warme Licht, das aus dem Fenster kommt.
float fenster;
vec3 fassade(vec2 f, float id, float s, float h) {
    fenster = 0.0;
    vec3 putz = vec3(0.82, 0.76, 0.66);
    float balken = 0.0;
    float b = 0.055;
    // Pfosten und Riegel
    balken = max(balken, step(abs(abs(f.x) - 1.78), b));
    balken = max(balken, step(abs(abs(f.x) - 0.6), b * 0.8));
    balken = max(balken, step(abs(f.y - 0.06), b));
    balken = max(balken, step(abs(f.y - 1.45), b));
    balken = max(balken, step(abs(f.y - (h - 0.08)), b));
    if (h > 3.0) balken = max(balken, step(abs(f.y - 2.75), b));
    // Fenster: drei je Geschoss
    float spalte = clamp(floor((f.x + 1.8) / 1.2), 0.0, 2.0);
    float u = f.x - (-1.2 + spalte * 1.2);
    for (int g = 0; g < 2; g++) {
        float unten = g == 0 ? 0.55 : 1.75;
        float oben = unten + 0.75;
        if (oben > h - 0.2) break;
        float v = f.y - (unten + oben) * 0.5;
        if (abs(u) < 0.27 && abs(v) < 0.375) {
            float an = step(0.28, hash(vec2(id * 3.0 + spalte, s * 7.0 + float(g))));
            // Sprossenkreuz und Rahmen
            float sprosse = step(abs(u), 0.022) + step(abs(v - 0.06), 0.022);
            float rahmen = step(0.24, abs(u)) + step(0.345, abs(v));
            if (sprosse + rahmen > 0.0) {
                balken = 1.0;
            } else {
                // drinnen: warmes Licht, ein Vorhang an den Seiten
                float vorhang = smoothstep(0.12, 0.2, abs(u));
                vec3 innen = mix(vec3(1.0, 0.62, 0.28), vec3(0.75, 0.25, 0.12), vorhang * 0.6);
                fenster = an * (0.9 + 0.1 * sin(time * 3.0 + spalte * 5.0 + id));
                // ein Schwibbogen: ein Bogen aus sieben kleinen Lichtern
                if (hash(vec2(id + 13.0, s + spalte * 2.0 + float(g))) < 0.35 && an > 0.5) {
                    vec2 bm = vec2(u, v + 0.3);
                    float lichter = 0.0;
                    for (int k = 0; k < 7; k++) {
                        float a = 0.3 + float(k) * 0.42;
                        vec2 lpos = vec2(cos(a), sin(a)) * 0.2;
                        lichter += smoothstep(0.024, 0.0, length(bm - lpos));
                    }
                    float bogen = smoothstep(0.012, 0.0, abs(length(bm) - 0.17)) * step(0.0, bm.y);
                    innen = innen * 0.3 + vec3(0.25, 0.12, 0.05) * bogen + vec3(1.0, 0.85, 0.5) * clamp(lichter * 2.0, 0.0, 1.0);
                }
                return innen * fenster + vec3(0.02, 0.025, 0.04) * (1.0 - an);
            }
        }
    }
    // Andreaskreuze unter den Fenstern
    float vv = f.y - 0.3;
    if (abs(u) < 0.5 && abs(vv) < 0.22) {
        float x1 = abs(vv - u * 0.44);
        float x2 = abs(vv + u * 0.44);
        balken = max(balken, step(min(x1, x2), 0.035));
    }
    vec3 holz = vec3(0.12, 0.06, 0.035);
    putz *= 0.85 + 0.15 * rausch(f * 7.0);
    return mix(putz, holz, balken);
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
    vec3 c = mix(vec3(0.05, 0.08, 0.17), vec3(0.008, 0.012, 0.04), pow(h, 0.6));
    c += sterne(d) * (1.0 + 1.0 * ton);
    float m = dot(d, MOND);
    c += vec3(0.95, 0.95, 0.88) * smoothstep(0.99935, 0.9996, m);
    c += vec3(0.25, 0.3, 0.45) * pow(max(m, 0.0), 120.0) * 1.2;
    c += vec3(0.12, 0.14, 0.22) * pow(max(m, 0.0), 8.0) * 0.4;
    // der Kirchturm am Ende der Gasse, als Schattenriss vor dem Himmel
    float az = atan(d.x, d.z);
    float turm = step(abs(az - 0.02), 0.016) * step(d.y, 0.085);
    float spitze = step(abs(az - 0.02), 0.016 * (1.0 - (d.y - 0.085) / 0.075)) * step(d.y, 0.16);
    float zifferblatt = smoothstep(0.0075, 0.0055, length(vec2(az - 0.02, d.y - 0.066)));
    float sil = max(turm, spitze);
    c = mix(c, vec3(0.012, 0.014, 0.028), sil);
    c += vec3(1.0, 0.75, 0.4) * zifferblatt * 0.9;
    // Huegel in der Ferne
    float huegel = 0.035 + 0.02 * sin(az * 6.0 + 1.0) + 0.012 * sin(az * 17.0);
    c = mix(c, vec3(0.06, 0.08, 0.14), step(d.y, huegel) * (1.0 - sil));
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

// Das Licht der Laternen auf Schnee und Waenden: weiche Flecken, ohne Schatten
vec3 laternenLicht(vec3 p, vec3 n) {
    vec3 l = vec3(0.0);
    float zid = floor(p.z / (ZELLE * 2.0));
    for (int k = -1; k <= 1; k++) {
        float z = (zid + float(k)) * ZELLE * 2.0 + ZELLE;
        for (int s = 0; s < 2; s++) {
            vec3 lp = vec3(s == 0 ? -2.25 : 2.25, 2.6, z);
            vec3 v = lp - p;
            float dd = dot(v, v);
            l += vec3(1.0, 0.68, 0.35) * max(dot(n, v / sqrt(dd)), 0.0) / (1.0 + dd * 0.9) * 2.4 * (1.0 + 0.3 * kick);
        }
    }
    return l;
}

vec3 laternenGlanz(vec3 ro, vec3 rd, float tmax) {
    vec3 c = vec3(0.0);
    float zid = floor(ro.z / (ZELLE * 2.0));
    for (int k = 0; k < 6; k++) {
        float z = (zid + float(k)) * ZELLE * 2.0 + ZELLE;
        for (int s = 0; s < 2; s++) {
            vec3 lp = vec3(s == 0 ? -2.25 : 2.25, 2.6, z);
            float tc = dot(lp - ro, rd);
            if (tc < 0.0 || tc > tmax + 0.3) continue;
            vec3 q = ro + rd * tc - lp;
            float d2 = dot(q, q);
            c += vec3(1.0, 0.72, 0.38) * (exp(-d2 * 40.0) * 2.4 + exp(-d2 * 2.0) * 0.16) * (1.0 + 0.4 * kick);
        }
    }
    return c;
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
    // Kamera: geht die Gasse hinauf, schaukelt ein wenig im Schritt; im
    // Abspann bleibt sie stehen und hebt den Blick
    float z = 2.0 + fort * 34.0;
    vec3 ro = vec3(0.35 * sin(z * 0.25), 1.65 + 0.03 * sin(z * 3.1), z);
    float hebe = ende * 0.45;
    vec3 vorne = normalize(vec3(0.05 * sin(time * 0.13), 0.06 + hebe, 1.0));
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.3 + uv.x * rechts + uv.y * oben);

    vec3 nebel = vec3(0.06, 0.08, 0.15);
    vec3 c;
    float t = 0.0;
    float hit = -1.0;
    int schritte = int(120.0 * G());
    for (int i = 0; i < 120; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float d = szene(p);
        if (d < 0.001 * t + 0.0005) { hit = t; break; }
        t += d * 0.85;
        if (t > 70.0) break;
    }
    float tmax = 70.0;
    if (hit > 0.0) {
        tmax = hit;
        vec3 p = ro + rd * hit;
        szene(p);
        float m = mat;
        vec3 zi = zellInfo;
        vec3 n = normale(p);
        float mond = max(dot(n, MOND), 0.0);
        vec3 himmelLicht = vec3(0.07, 0.09, 0.16) * (0.6 + 0.4 * n.y);
        vec3 alb;
        vec3 eigen = vec3(0.0);
        if (m < 0.5 || (m > 1.5 && m < 2.5 && n.y > 0.35) || (m > 0.5 && m < 1.5 && n.y > 0.7)) {
            // Schnee: weiss mit kaltem Schimmer, funkelt im Licht
            alb = vec3(0.85, 0.9, 1.0);
            float funkel = step(0.995, hash(floor(p.xz * 60.0) + floor(time * 4.0)));
            eigen += vec3(1.0) * funkel * 0.6 * (1.0 + ton);
        } else if (m < 1.5) {
            // Fassade, nur an der Seite zur Gasse
            if (abs(n.x) > 0.7) {
                alb = fassade(vec2(p.z - (zi.x + 0.5) * ZELLE, p.y), zi.x, zi.y, zi.z);
                eigen += alb * fenster * 1.6;
                alb *= 1.0 - fenster;
            } else {
                alb = vec3(0.5, 0.46, 0.42);
            }
        } else if (m < 2.5) {
            alb = vec3(0.18, 0.07, 0.05);   // Ziegel unter dem Schnee, an der Traufe
        } else {
            alb = vec3(0.03);
        }
        vec3 licht = himmelLicht * 1.4 + vec3(0.5, 0.58, 0.8) * mond * 0.7 + laternenLicht(p, n);
        // das Licht der Fenster faellt auf den Schnee vor dem Haus
        if (m < 0.5) {
            float zz = p.z - (floor(p.z / ZELLE) + 0.5) * ZELLE;
            float vorDem = smoothstep(1.4, FASSADE, abs(p.x));
            licht += vec3(1.0, 0.6, 0.3) * vorDem * (0.5 + 0.5 * cos(zz * 2.6)) * 0.45;
        }
        c = alb * licht + eigen;
        c = mix(c, nebel, 1.0 - exp(-hit * 0.035));
    } else {
        c = himmel(rd);
    }
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
