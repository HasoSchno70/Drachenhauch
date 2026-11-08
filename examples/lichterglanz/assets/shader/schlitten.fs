#version 330
// Der Schlittenflug: tief ueber verschneite Huegel und Waelder, in denen hier
// und da ein Dorf leuchtet, auf einen riesigen Mond am Horizont zu. Vor dem
// Mond zieht der Schlitten mit seinen Rentieren vorbei -- als Schattenriss,
// die Beine im Galopp auf dem Takt, das erste Rentier mit leuchtend roter
// Nase -- und hinter sich eine Spur aus Goldstaub. Mit der Rueckung (`rueck`)
// steigt die Kamera durch die Wolkendecke: darueber ein Wolkenmeer im
// Mondlicht, durch dessen Luecken die Doerfer leuchten, der Himmel hell vom
// Polarlicht. Dort schwingt das Gespann gross von unten heran, zieht ein
// langes Band aus Goldstaub hinter sich her und steigt zum Mond auf, dessen
// Scheibe es am Ende klein kreuzt.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float takt;
uniform float hell;
uniform float fort;
uniform float rueck;
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
float fbm(vec2 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 4; i++) { s += a * rausch(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; }
    return s;
}

const float PI = 3.14159265;
const vec3 MOND = vec3(0.0, 0.0995, 0.995);
const float MOND_R = 0.11;

// Das Gelaende: Huegel aus Rauschen, darauf Tannen als spitze Kegel in
// einem Raster (nur wo der Zufall es will)
float baeume(vec2 p) {
    vec2 z = floor(p / 2.5);
    vec2 m = (z + 0.5 + (vec2(hash(z), hash(z + 3.3)) - 0.5) * 0.6) * 2.5;
    float h = hash(z + 8.8);
    if (h < 0.45) return 0.0;
    float r = length(p - m);
    return max(0.0, 2.2 * (h - 0.3) - r * 2.6);
}
float gelaende(vec2 p) {
    return fbm(p * 0.04) * 14.0 - 4.0 + fbm(p * 0.25) * 1.2 + baeume(p);
}

// Lichter der Doerfer: warme Punkte in manchen Zellen
vec3 dorfLichter(vec2 p) {
    vec2 z = floor(p / 18.0);
    if (hash(z + 1.7) < 0.7) return vec3(0.0);
    vec2 m = (z + 0.5) * 18.0;
    vec2 q = p - m;
    vec2 zz = floor(q / 1.6);
    vec2 f = fract(q / 1.6) - 0.5;
    float an = step(0.5, hash(zz + z * 7.0)) * step(length(q), 6.5);
    vec3 c = vec3(1.0, 0.65, 0.3) * an * smoothstep(0.22, 0.0, length(f)) * (0.8 + 0.2 * sin(time * 2.0 + hash(zz) * 9.0));
    // ein warmer Schein ueber dem ganzen Dorf
    c += vec3(1.0, 0.55, 0.25) * 0.12 * smoothstep(9.0, 2.0, length(q));
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

float seg(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// Ein Rentier als Schattenriss, Blick nach rechts; `g` ist die Phase des
// Galopps. Masse in Koerperlaengen.
float rentier(vec2 p, float g) {
    float d = seg(p, vec2(-0.35, 0.0), vec2(0.3, 0.03)) - 0.13;
    d = min(d, seg(p, vec2(0.3, 0.05), vec2(0.5, 0.32)) - 0.065);
    d = min(d, seg(p, vec2(0.5, 0.33), vec2(0.68, 0.27)) - 0.055);
    // Geweih
    d = min(d, seg(p, vec2(0.48, 0.4), vec2(0.38, 0.62)) - 0.018);
    d = min(d, seg(p, vec2(0.42, 0.52), vec2(0.56, 0.62)) - 0.016);
    d = min(d, seg(p, vec2(0.38, 0.62), vec2(0.28, 0.7)) - 0.014);
    // Beine im Galopp: vorn und hinten gegenlaeufig gestreckt und gebeugt
    float s = sin(g);
    vec2 vk = vec2(0.24 + 0.12 * s, -0.22);
    d = min(d, seg(p, vec2(0.22, -0.05), vk) - 0.035);
    d = min(d, seg(p, vk, vk + vec2(0.1 + 0.08 * s, -0.16)) - 0.028);
    vec2 hk = vec2(-0.3 - 0.12 * s, -0.22);
    d = min(d, seg(p, vec2(-0.28, -0.05), hk) - 0.04);
    d = min(d, seg(p, hk, hk + vec2(-0.1 - 0.08 * s, -0.16)) - 0.03);
    // Stummelschwanz
    d = min(d, seg(p, vec2(-0.45, 0.04), vec2(-0.52, 0.1)) - 0.03);
    return d;
}

// Der Schlitten mit Kufe, Kasten, Fahrer und Sack; Blick nach rechts
float schlittenForm(vec2 p) {
    // Kufe: ein Bogen vorne hoch
    float kufe = seg(p, vec2(-0.6, -0.32), vec2(0.45, -0.32)) - 0.025;
    kufe = min(kufe, abs(length(p - vec2(0.45, -0.17)) - 0.15) - 0.025 + step(p.x, 0.45) * 1.0);
    float kasten = max(abs(p.x + 0.05) - 0.45, abs(p.y + 0.08) - 0.18);
    kasten = min(kasten, max(abs(p.x - 0.35) - 0.08, abs(p.y + 0.0) - 0.26));
    float fahrer = length(p - vec2(0.05, 0.22)) - 0.17;
    fahrer = min(fahrer, length(p - vec2(0.12, 0.45)) - 0.09);
    // die Muetze mit Bommel
    fahrer = min(fahrer, seg(p, vec2(0.1, 0.5), vec2(-0.05, 0.62)) - 0.04);
    fahrer = min(fahrer, length(p - vec2(-0.08, 0.63)) - 0.04);
    float sack = length((p - vec2(-0.33, 0.18)) / vec2(1.0, 1.15)) - 0.2;
    // Stuetzen zur Kufe
    float stuetze = min(seg(p, vec2(-0.4, -0.26), vec2(-0.4, -0.32)), seg(p, vec2(0.25, -0.26), vec2(0.25, -0.32))) - 0.02;
    return min(min(min(kufe, kasten), min(fahrer, sack)), stuetze);
}

vec2 drehe(vec2 p, float a) {
    float c = cos(a), s = sin(a);
    return vec2(c * p.x - s * p.y, s * p.x + c * p.y);
}
vec2 bezier(vec2 a, vec2 b, vec2 c, vec2 d, float t) {
    float u = 1.0 - t;
    return u * u * u * a + 3.0 * u * u * t * b + 3.0 * u * t * t * c + t * t * t * d;
}

// Das ganze Gespann im Bild: der Kopf des ersten Rentiers bei `kopf`, Groesse
// einer Koerperlaenge, Flugrichtung `winkel`. Liefert die Deckung (0..1), in
// `staub` den Goldstaub dicht hinter dem Schlitten und in `nase` die Stelle
// der roten Nase.
float staub;
vec2 nase;
float gespann(vec2 uv, vec2 kopf, float groesse, float winkel) {
    staub = 0.0;
    kopf.y += 0.15 * groesse * sin(takt * PI * 2.0);
    vec2 p = drehe(uv - kopf, -winkel) / groesse;
    nase = kopf + drehe(vec2(0.72, 0.26 + 0.03 * sin(takt * PI * 2.0)), winkel) * groesse;
    float d = 1e5;
    // vier Rentiere, paarweise hintereinander
    for (int i = 0; i < 4; i++) {
        float fi = float(i);
        vec2 o = vec2(-fi * 1.15, 0.03 * sin(takt * PI * 2.0 + fi * 1.3));
        d = min(d, rentier(p - o, takt * PI * 2.0 + fi * 0.9));
    }
    // Zuegel vom Schlitten zu den Tieren
    vec2 sl = vec2(-5.2, -0.05);
    d = min(d, seg(p, sl + vec2(0.4, 0.15), vec2(0.0, 0.05)) - 0.012);
    d = min(d, schlittenForm((p - sl) / 1.25) * 1.25);
    // Goldstaub: Funken hinter dem Schlitten, die zurueckbleiben und sinken
    vec2 sp = p - sl - vec2(-0.6, -0.3);
    for (int k = 0; k < 24; k++) {
        float fk = float(k);
        float alter = fract(fk / 24.0 + time * 0.6);
        vec2 ort = vec2(-alter * 4.5, -alter * alter * 1.4 + 0.25 * sin(fk * 2.3 + time));
        ort += (vec2(hash(vec2(fk, 1.0)), hash(vec2(fk, 2.0))) - 0.5) * alter * 1.2;
        float r = length(sp - ort);
        staub += exp(-r * r * 140.0) * (1.0 - alter) * (0.6 + 0.4 * sin(time * 9.0 + fk));
    }
    return smoothstep(0.02, -0.02, d * groesse * 30.0);
}

vec3 himmel(vec3 d) {
    float h = clamp(d.y, 0.0, 1.0);
    vec3 c = mix(vec3(0.09, 0.11, 0.22), vec3(0.01, 0.015, 0.05), pow(h, 0.45));
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 170.0;
    vec2 z = floor(uv);
    float hz = hash(z);
    c += vec3(0.85, 0.9, 1.0) * step(0.98, hz) * smoothstep(0.35, 0.0, length(fract(uv) - 0.5)) * (0.6 + 0.4 * sin(time * (1.0 + hz * 3.0) + hz * 30.0));
    // Polarlicht, mit der Rueckung hell
    float az = atan(d.x, d.z);
    for (int i = 0; i < 3; i++) {
        float fi = float(i);
        float band = exp(-pow((d.y - 0.22 - fi * 0.07 - 0.05 * sin(az * 2.5 + fi + time * 0.1)) * 14.0, 2.0));
        float strahl = 0.4 + 0.6 * rausch(vec2(az * 40.0 + fi * 7.0, time * 0.3));
        c += mix(vec3(0.15, 1.0, 0.55), vec3(0.6, 0.3, 1.0), fi * 0.5) * band * strahl * (0.05 + 0.3 * rueck) * (1.0 + 0.5 * ton);
    }
    // der Mond, riesig am Horizont, ein wenig golden
    float m = dot(d, MOND);
    float ra = acos(clamp(m, -1.0, 1.0));
    vec2 mq = vec2(d.x, d.y - MOND.y) * 18.0;
    float mare = 0.75 + 0.25 * fbm(mq * 1.3 + 4.0);
    c = mix(c, vec3(1.0, 0.93, 0.78) * 1.5 * mare, smoothstep(MOND_R, MOND_R - 0.002, ra));
    c += vec3(0.9, 0.75, 0.55) * exp(-max(ra - MOND_R, 0.0) * 9.0) * 0.25;
    return c;
}

// Die Wolkendecke (ab der Rueckung): eine Schicht in der Hoehe WOLKE, wie
// dicht sie an der Stelle xz ist
const float WOLKE = 15.0;
float wolkenDichte(vec2 xz) {
    return smoothstep(0.3, 0.55, fbm(xz * 0.015 + vec2(time * 0.015, 0.0)));
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    // Kamera: fliegt tief nach vorn, folgt den Huegeln, neigt sich in Kurven;
    // mit der Rueckung steigt sie durch die Wolken hinauf
    float z = fort * 260.0;
    float x = 30.0 * sin(z * 0.006);
    vec3 ro = vec3(x, 0.0, z);
    float steig = smoothstep(0.5, 0.64, fort);
    float wolke = smoothstep(0.47, 0.55, fort);
    ro.y = mix(max(gelaende(ro.xz), gelaende(ro.xz + vec2(0.0, 12.0))) + 4.5, 27.0 + 0.6 * sin(time * 0.7), steig);
    float neigen = -0.12 * cos(z * 0.006);
    vec3 vorne = normalize(vec3(0.18 * cos(z * 0.006), mix(-0.05, -0.13, steig) + 0.12 * sin(steig * PI), 1.0));
    vec3 rechts = normalize(cross(vec3(sin(neigen), cos(neigen), 0.0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.2 + uv.x * rechts + uv.y * oben);

    vec3 c;
    float t = 0.5;
    float hit = -1.0;
    int schritte = int(110.0 * G());
    for (int i = 0; i < 110; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float hh = gelaende(p.xz);
        float d = p.y - hh;
        if (d < 0.002 * t) { hit = t; break; }
        t += d * 0.45 + 0.01 * t;
        if (t > 220.0) break;
    }
    vec3 nebel = vec3(0.09, 0.1, 0.19);
    if (hit > 0.0) {
        vec3 p = ro + rd * hit;
        vec2 e = vec2(0.08, 0.0);
        vec3 n = normalize(vec3(gelaende(p.xz - e.xy) - gelaende(p.xz + e.xy), 2.0 * e.x, gelaende(p.xz - e.yx) - gelaende(p.xz + e.yx)));
        float baum = baeume(p.xz);
        vec3 alb = mix(vec3(0.82, 0.86, 0.96), vec3(0.03, 0.07, 0.05), smoothstep(0.15, 0.6, baum) * smoothstep(0.75, 0.35, n.y + 0.2 * rausch(p.xz * 3.0)));
        float mond = max(dot(n, normalize(MOND + vec3(0.0, 0.25, 0.0))), 0.0);
        vec3 licht = vec3(0.1, 0.12, 0.22) * (0.5 + 0.5 * n.y) + vec3(0.75, 0.68, 0.6) * mond * 0.55;
        c = alb * licht + dorfLichter(p.xz) * 2.5;
        c += vec3(1.0) * step(0.997, hash(floor(p.xz * 30.0) + floor(time * 6.0))) * 0.4 * (1.0 + ton);
        c = mix(c, nebel, 1.0 - exp(-hit * 0.012));
    } else {
        c = himmel(rd);
    }
    // die Wolkendecke: von oben im Mondlicht, durch die Luecken das Land
    if (wolke > 0.0) {
        float tc = (WOLKE - ro.y) / rd.y;
        if (tc > 0.0 && (hit < 0.0 || tc < hit)) {
            vec3 wp = ro + rd * tc;
            float dicht = wolkenDichte(wp.xz) * wolke;
            // Ballen: helle Kuppen, dunkle Taeler, zum Mond hin heller
            float relief = fbm(wp.xz * 0.06 + 3.0) * 0.65 + fbm(wp.xz * 0.2 + 7.0) * 0.35;
            float kuppe = smoothstep(0.35, 0.75, relief);
            vec3 wf = ro.y > WOLKE
                ? mix(vec3(0.16, 0.19, 0.3), vec3(0.85, 0.86, 0.92), kuppe) * (0.6 + 0.7 * pow(max(dot(normalize(vec3(rd.x, 0.0, rd.z)), normalize(vec3(MOND.x, 0.0, MOND.z))), 0.0), 6.0))
                : vec3(0.12, 0.13, 0.2) * (0.6 + 0.4 * relief);
            wf += vec3(1.0) * step(0.997, hash(floor(wp.xz * 4.0) + floor(time * 5.0))) * kuppe * 0.4;
            // das Polarlicht faerbt die Wolken
            wf += vec3(0.1, 0.5, 0.3) * rueck * 0.25 * relief;
            dicht *= 1.0 - smoothstep(120.0, 260.0, tc);
            c = mix(c, wf, dicht);
        }
        // mitten durch die Decke: alles im Nebel
        float drin = exp(-abs(ro.y - WOLKE) * 0.45) * wolke;
        c = mix(c, vec3(0.5, 0.54, 0.66), drin * 0.9);
    }
    // das Gespann: erst einmal klein vor dem Mond vorbei, nach der Rueckung
    // ueber den Wolken gross von unten heran und hinauf zum Mond
    vec3 mr = vec3(dot(MOND, rechts), dot(MOND, oben), dot(MOND, vorne));
    vec2 mitte = mr.xy / mr.z * 1.2;
    float laufA = clamp(fort / 0.46, 0.0, 1.0);
    float laufB = clamp((fort - 0.6) / 0.38, 0.0, 1.0);
    float sil = 0.0;
    float band = 0.0;
    float vorn = 0.0;
    if (fort < 0.47) {
        vec2 kopf = mitte + vec2((-2.2 + 4.4 * laufA) * MOND_R * 1.7, MOND_R * (0.15 + 0.35 * sin(laufA * PI)));
        sil = gespann(uv, kopf, 0.055, 0.0) * step(hit, 0.0);
    } else if (fort > 0.6) {
        vec2 a = vec2(-1.35, -0.3), b = vec2(-0.45, -0.46), cc = mitte + vec2(-0.75, -0.2), d = mitte + vec2(0.45, 0.08);
        vec2 kopf = bezier(a, b, cc, d, laufB);
        vec2 r = bezier(a, b, cc, d, min(laufB + 0.01, 1.0)) - bezier(a, b, cc, d, max(laufB - 0.01, 0.0));
        float gr = mix(0.26, 0.02, pow(laufB, 0.7));
        sil = gespann(uv, kopf, gr, atan(r.y, r.x));
        vorn = 1.0;
        // das Band aus Goldstaub: der Weg, den der Schlitten genommen hat
        for (int k = 1; k < 56; k++) {
            float fk = float(k);
            float sk = laufB - fk * 0.009;
            if (sk < 0.0) break;
            vec2 rk = bezier(a, b, cc, d, sk + 0.005) - bezier(a, b, cc, d, sk - 0.005);
            float gk = mix(0.26, 0.02, pow(sk, 0.7));
            vec2 hinten = bezier(a, b, cc, d, sk) + drehe(vec2(-5.9, -0.32), atan(rk.y, rk.x)) * gk;
            float alter = fk / 56.0;
            hinten += vec2(hash(vec2(fk, 3.0)) - 0.5, hash(vec2(fk, 4.0)) - 0.5) * gk * 1.5 * alter;
            hinten.y -= alter * alter * 0.05;
            float rr = length(uv - hinten) / (gk * 0.35 + 0.002);
            float funkeln = 0.55 + 0.45 * sin(time * 11.0 + fk * 2.7);
            band += (exp(-rr * rr * 3.0) * 1.2 + exp(-rr * 1.6) * 0.15) * (1.0 - alter) * funkeln;
        }
    }
    c = mix(c, vec3(0.01, 0.01, 0.02), sil);
    c += vec3(1.0, 0.78, 0.35) * (staub * step(hit, 0.0) * (1.0 - vorn) + staub * vorn + band) * (0.7 + 0.8 * ton);
    // die rote Nase des ersten Rentiers
    if (fort < 0.47 || fort > 0.6) {
        float gn = fort < 0.47 ? 0.055 : mix(0.26, 0.02, pow(laufB, 0.7));
        float rn = length(uv - nase) / gn;
        float sichtbar = fort < 0.47 ? step(hit, 0.0) : 1.0;
        c += vec3(1.0, 0.12, 0.06) * (exp(-rn * rn * 120.0) * 3.0 + exp(-rn * 5.0) * 0.35) * (0.8 + 0.4 * kick) * sichtbar;
    }
    c *= 1.0 + 0.1 * kick;
    c *= 1.0 - 0.3 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.92));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
