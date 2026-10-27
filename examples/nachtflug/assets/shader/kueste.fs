#version 330
// Kueste im Morgengrauen: Tiefflug ueber das Meer an zerkluefteten Klippen
// entlang, vorbei an Brandungsfelsen im Wasser. Auf dem hoechsten Vorsprung
// steht ein Leuchtturm mit Galerie und Laterne, daneben das Haus des
// Waerters mit einem erleuchteten Fenster. Der Strahl kreist im Takt -- eine
// Umdrehung in zwei Takten -- und streift ueber Wasser, Felsen und durch den
// Dunst. Unten bricht sich die Brandung, Moewen kreisen um den Turm, am
// Horizont liegen ferne Landzungen, und rosa angestrahlte Wolkenstreifen
// ziehen ueber den Himmel. Die Sonne ist noch nicht da: sie geht erst im
// Ausklang auf dem offenen Meer auf.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
uniform float takt;
uniform float fort;      // 0..1 durch den Abschnitt
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
float hash1(float x) { return fract(sin(x * 91.7) * 43758.5453); }
float rausch(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}
float fbm(vec2 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 5; i++) { s += a * rausch(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; }
    return s;
}
float fbm3l(vec2 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 3; i++) { s += a * rausch(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; }
    return s;
}

const vec3 SONNE = vec3(0.2611, 0.0081, 0.9653);   // knapp unter dem Horizont, vorn rechts

// --- Die Kueste --------------------------------------------------------------
const vec2 TURM = vec2(7.2, 60.0);   // (x, z)
float kante(float z) { return 4.6 + 1.6 * sin(z * 0.07) + 1.2 * fbm3l(vec2(z * 0.12, 3.0)) - 2.0 * exp(-pow((z - TURM.y) / 9.0, 2.0)); }
float grat(float z) { return 3.4 + 2.2 * fbm3l(vec2(z * 0.05, 1.0)) + 1.0 * exp(-pow((z - TURM.y) / 7.0, 2.0)); }

float box(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Die Felswand: senkrechte Saeulen und Risse, waagerechte Baender, oben ein
// unruhiger Grat
float wand(vec3 p) {
    float k = kante(p.z);
    float saeule = abs(fract(p.z * 0.33 + 0.35 * fbm3l(vec2(p.y * 0.25, p.z * 0.1))) - 0.5) * 2.0;
    k += 0.45 * smoothstep(0.15, 0.9, saeule);
    k += 0.12 * sin(p.y * 3.1 + 4.0 * fbm3l(p.yz * vec2(0.6, 0.3)));
    k += 0.5 * fbm3l(p.yz * vec2(0.8, 0.45)) - 0.05 * p.y;
    float vorn = k - p.x;                 // > 0 vor der Wand (ueber dem Meer)
    float oben = p.y - grat(p.z) - 0.25 * fbm3l(p.xz * 1.3);
    return max(vorn, oben) * 0.65;
}

// Bloecke am Fuss der Wand: unregelmaessig verteilt, verschieden gross,
// halb im Wasser, kantig statt rund
float bloecke(vec3 p) {
    float zelle = floor(p.z / 1.3);
    if (hash1(zelle + 11.0) < 0.4) return 1.0;
    float cz = (zelle + 0.5) * 1.3 + (hash1(zelle) - 0.5) * 0.5;
    float r = 0.2 + 0.4 * hash1(zelle + 3.0) * hash1(zelle + 4.0);
    vec3 c = vec3(kante(cz) - 0.1 - hash1(zelle + 5.0) * 0.6, -0.12, cz);
    vec3 q = (p - c) * vec3(1.0, 1.4 + hash1(zelle + 7.0), 1.0 + 0.6 * hash1(zelle + 9.0));
    return (length(q) - r - 0.12 * (rausch(p.zy * 6.0 + zelle) - 0.5)) * 0.6;
}

// Brandungsfelsen im Wasser: zerklueftete Saeulen, oben gruen
float saeule(vec3 p, vec3 c, float r, float h) {
    vec2 q = p.xz - c.xz;
    float w = atan(q.y, q.x);
    float rr = r * (1.0 - 0.18 * p.y / h) + 0.18 * (rausch(vec2(w * 2.5, p.y * 1.4)) - 0.5);
    return max(length(q) - rr, p.y - h - 0.3 * rausch(q * 2.0)) * 0.8;
}
float stacks(vec3 p) {
    float d = saeule(p, vec3(3.3, 0.0, 34.0), 0.75, 3.2);
    d = min(d, saeule(p, vec3(-4.8, 0.0, 49.0), 1.2, 4.4));
    d = min(d, saeule(p, vec3(2.4, 0.0, 56.0), 0.6, 2.2));
    return d;
}

// Der Leuchtturm: Schaft, Galerie, Laterne, Kuppel
float turm(vec3 p) {
    float g = grat(TURM.y);
    vec2 q = p.xz - TURM;
    float hh = p.y - g;
    float r = 0.42 - 0.07 * clamp(hh / 3.0, 0.0, 1.0);
    float schaft = max(length(q) - r, max(-hh - 0.3, hh - 2.6));
    float galerie = max(length(q) - 0.55, abs(hh - 2.65) - 0.06);
    float laterne = max(length(q) - 0.3, abs(hh - 3.0) - 0.32);
    float kuppel = max(length(vec3(q.x, (hh - 3.32) * 1.4, q.y)) - 0.33, -(hh - 3.32));
    return min(min(schaft, galerie), min(laterne, kuppel));
}

// Das Haus des Waerters mit Satteldach
const vec3 HAUS = vec3(8.5, 0.0, 57.3);
float haus(vec3 p) {
    vec3 c = vec3(HAUS.x, grat(HAUS.z) + 0.45, HAUS.z);
    vec3 q = p - c;
    float koerper = box(q, vec3(0.75, 0.5, 0.95));
    vec3 d = q - vec3(0.0, 0.5, 0.0);
    float dach = max(abs(d.z) - 1.05, max(abs(d.x) * 0.85 + d.y - 0.62, -d.y));
    return min(koerper, dach);
}

float szene(vec3 p) {
    float d = min(wand(p), bloecke(p));
    d = min(d, stacks(p));
    d = min(d, turm(p));
    d = min(d, haus(p));
    return d;
}

// --- Himmel ------------------------------------------------------------------
vec3 himmel(vec3 d, float wolken) {
    float h = clamp(d.y, 0.0, 1.0);
    vec3 c = mix(vec3(1.0, 0.55, 0.32), vec3(0.62, 0.36, 0.52), smoothstep(0.0, 0.1, h));
    c = mix(c, vec3(0.10, 0.13, 0.32), smoothstep(0.08, 0.55, h));
    c += vec3(1.0, 0.6, 0.3) * pow(max(dot(d, SONNE), 0.0), 14.0) * 0.7;
    // Wolkenstreifen, von unten rosa angestrahlt
    vec2 w = vec2(atan(d.x, d.z) * 2.2 + time * 0.01, h * 16.0);
    float streif = smoothstep(0.5, 0.85, fbm(w * vec2(1.0, 0.6) + vec2(0.0, w.x * 0.3)));
    streif *= smoothstep(0.03, 0.08, h) * (1.0 - smoothstep(0.25, 0.5, h));
    vec3 angestrahlt = mix(vec3(1.0, 0.55, 0.45), vec3(0.55, 0.32, 0.5), smoothstep(0.05, 0.35, h));
    angestrahlt *= 0.6 + 0.6 * pow(max(dot(normalize(vec3(d.x, 0.0, d.z)), normalize(vec3(SONNE.x, 0.0, SONNE.z))), 0.0), 3.0);
    c = mix(c, angestrahlt, streif * 0.8 * wolken);
    // ferne Landzungen am Horizont, links bis geradeaus
    float az = atan(d.x, d.z);
    float land = 0.012 + 0.03 * fbm3l(vec2(az * 6.0, 2.0)) * smoothstep(0.35, -0.6, az);
    // (nicht in der Spiegelung -- dort machten sie harte Flecken ins Wasser)
    if (wolken > 0.5 && d.y < land && d.y > -0.01) c = mix(c, vec3(0.30, 0.22, 0.34), 0.85);
    // die letzten Sterne
    vec2 uv = vec2(az, asin(clamp(d.y, -1.0, 1.0))) * 150.0;
    vec2 z = floor(uv), f = fract(uv) - 0.5;
    float st = smoothstep(0.08, 0.0, length(f)) * step(0.988, hash(z)) * smoothstep(0.25, 0.6, h);
    c += vec3(0.8, 0.85, 1.0) * st * 0.6 * (1.0 - streif);
    return c;
}

// Strahl des Leuchtturms: Abstand des Blickstrahls zur Achse des Lichtkegels
vec3 strahl(vec3 ro, vec3 d, float tMax, vec3 lampe, vec3 b) {
    vec3 w = ro - lampe;
    float bb = dot(b, d);
    float nenner = 1.0 - bb * bb;
    if (nenner < 1e-4) return vec3(0.0);
    float t = (bb * dot(w, b) - dot(w, d)) / nenner;
    float u = (dot(w, b) - bb * dot(w, d)) / nenner;
    if (u < 0.0 || t < 0.0 || t > tMax) return vec3(0.0);
    vec3 a = ro + d * t;
    vec3 c = lampe + b * u;
    float dist = length(a - c);
    float breit = 0.12 + u * 0.05;
    float k = exp(-dist * dist / (breit * breit)) * exp(-u * 0.012);
    // Dunst im Strahl: nicht ganz gleichmaessig
    k *= 0.75 + 0.5 * rausch(vec2(u * 0.4 - time * 0.3, dist * 3.0));
    return vec3(1.0, 0.93, 0.78) * k * 0.9;
}

// Moewen: kreisen um den Turm, als kleine geschwungene Vs im Bild
float moewen(vec2 uv, vec3 ro, vec3 vorne, vec3 rechts, vec3 oben) {
    float m = 0.0;
    for (int i = 0; i < 5; i++) {
        float fi = float(i);
        float w = time * (0.25 + 0.05 * fi) + fi * 1.3;
        float r = 5.0 + fi * 1.4;
        vec3 v = vec3(TURM.x + cos(w) * r, grat(TURM.y) + 4.5 + fi * 0.7 + sin(time * 0.7 + fi) * 0.4, TURM.y + sin(w) * r);
        vec3 zu = v - ro;
        float z = dot(zu, vorne);
        if (z < 1.0) continue;
        vec2 sp = vec2(dot(zu, rechts), dot(zu, oben)) / z;
        float gr = 0.35 / z;                 // Spannweite im Bild
        vec2 q = (uv - sp) / gr;
        float schlag = sin(time * 7.0 + fi * 2.0) * 0.35;
        // zwei Fluegel: Boegen links und rechts der Mitte
        float x = abs(q.x);
        float y = q.y - (schlag * x - 0.6 * x * x + 0.25 * x);
        float fluegel = smoothstep(0.09, 0.0, abs(y)) * step(x, 1.0);
        m = max(m, fluegel);
    }
    return m;
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
    float strahlen = pow(uebHash(id), 5.0) * (0.6 + 0.4 * sin(time * 2.5 + id * 1.7));
    c += mix(uebLicht, vec3(1.0), 0.5) * strahlen * k * smoothstep(0.08, 0.7, r) * 0.5;
    return c;
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    // Kamera: knapp ueber dem Wasser, faehrt an der Kueste entlang auf den
    // Turm zu und dreht am Ende zum offenen Meer
    float f = smoothstep(0.0, 1.0, fort);
    vec3 ro = vec3(mix(-1.0, 0.5, f), 1.4 + 0.15 * sin(time * 0.7), mix(14.0, 46.0, f));
    vec3 ziel = mix(vec3(TURM.x, grat(TURM.y) + 1.5, TURM.y), vec3(-6.0, 1.2, ro.z + 30.0), smoothstep(0.62, 1.0, fort));
    vec3 vorne = normalize(ziel - ro);
    vec3 rechts = normalize(cross(vec3(0.0, 1.0, 0.0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 d = normalize(vorne + uv.x * rechts + uv.y * oben);

    float t = 0.1;
    float tFels = -1.0;
    int schritte = int(110.0 * G());
    for (int i = 0; i < 110; i++) {
        if (i >= schritte) break;
        vec3 p = ro + d * t;
        float h = szene(p);
        if (h < 0.002 * t) { tFels = t; break; }
        if (t > 120.0 || p.y < -0.5) break;
        t += h;
    }
    float tWasser = d.y < 0.0 ? -ro.y / d.y : 1e9;

    vec3 lampe = vec3(TURM.x, grat(TURM.y) + 3.0, TURM.y);
    float winkel = takt * 3.14159;        // eine Umdrehung in zwei Takten
    vec3 b = normalize(vec3(cos(winkel), -0.03, sin(winkel)));

    vec3 c;
    float tTreffer;
    if (tFels > 0.0 && tFels < tWasser) {
        tTreffer = tFels;
        vec3 p = ro + d * tFels;
        vec2 e = vec2(0.02, 0.0);
        vec3 n = normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
        float dT = turm(p), dH = haus(p), dS = stacks(p);
        float dmin = min(min(dT, dH), min(dS, min(wand(p), bloecke(p))));
        vec3 grund;
        vec3 leuchten = vec3(0.0);
        if (dT <= dmin + 0.001) {
            // weiss und rot geringelt, oben die Laterne
            float hh = p.y - grat(TURM.y);
            grund = mod(floor(hh * 1.6), 2.0) < 1.0 ? vec3(0.82, 0.8, 0.78) : vec3(0.6, 0.08, 0.06);
            if (hh > 2.6) grund = vec3(0.12);
            if (hh > 2.7 && hh < 3.3) { grund = vec3(0.2); leuchten = vec3(1.0, 0.9, 0.65) * 2.5; }
        } else if (dH <= dmin + 0.001) {
            vec3 q = p - vec3(HAUS.x, grat(HAUS.z) + 0.45, HAUS.z);
            grund = q.y > 0.5 ? vec3(0.32, 0.12, 0.1) : vec3(0.75, 0.72, 0.66);
            // ein erleuchtetes Fenster zur See hin
            vec2 fe = vec2(q.z, q.y) - vec2(-0.3, 0.05);
            if (n.x < -0.5 && abs(fe.x) < 0.2 && abs(fe.y) < 0.18) leuchten = vec3(1.0, 0.7, 0.35) * 1.6;
        } else {
            // Gesteinsschichten, schraeg verworfen, dazu Flecken und Risse
            float schicht = fbm(vec2(p.y * 2.6 + p.z * 0.12, p.z * 0.25));
            grund = mix(vec3(0.17, 0.12, 0.12), vec3(0.42, 0.33, 0.27), schicht);
            grund *= 0.7 + 0.6 * fbm(p.yz * vec2(3.0, 1.2));
            // nass und dunkel unten, Algen gruenlich
            grund = mix(grund * vec3(0.55, 0.65, 0.6), grund, smoothstep(0.1, 0.9, p.y));
            n = normalize(n + 0.5 * vec3(0.0, rausch(p.yz * 5.0) - 0.5, rausch(p.zy * 5.0 + 3.0) - 0.5));
            // Gras oben (nicht auf den Bloecken unten), Kraut an den Kanten
            grund = mix(grund, mix(vec3(0.10, 0.17, 0.07), vec3(0.22, 0.26, 0.1), rausch(p.xz * 4.0)), smoothstep(0.55, 0.85, n.y) * smoothstep(1.0, 1.6, p.y));
        }
        float licht = max(dot(n, normalize(SONNE + vec3(0.0, 0.25, 0.0))), 0.0);
        vec3 himmelLicht = vec3(0.34, 0.27, 0.44) * (0.5 + 0.5 * n.y);
        // Gegenlicht: ein warmer Saum an den Kanten zur Sonne
        float saum = pow(1.0 - max(dot(n, -d), 0.0), 3.0) * max(dot(d, SONNE), 0.0);
        c = grund * (vec3(1.0, 0.6, 0.38) * licht * 0.9 + himmelLicht) + vec3(1.0, 0.6, 0.35) * saum * 0.5 + leuchten;
        // der Strahl streift die Felsen
        float imKegel = smoothstep(0.92, 0.99, dot(normalize(p - lampe), b));
        c += grund * vec3(1.0, 0.9, 0.7) * imKegel * 1.2;
        c = mix(c, himmel(normalize(vec3(d.x, 0.02, d.z)), 1.0), 1.0 - exp(-tFels * 0.012));
    } else if (d.y < 0.0) {
        tTreffer = tWasser;
        vec3 p = ro + d * tWasser;
        vec2 w = p.xz;
        vec2 g = vec2(0.0);
        for (int i = 0; i < 7; i++) {
            float fi = float(i);
            vec2 r = vec2(cos(fi * 1.7), sin(fi * 1.7));
            float fr = 0.6 + fi * 0.6;
            float ph = dot(w, r) * fr + time * (0.9 + fi * 0.25);
            g += r * cos(ph) * (0.09 / (1.0 + fi * 0.8));
        }
        g *= exp(-tWasser * 0.02);
        vec3 n = normalize(vec3(-g.x, 1.0, -g.y));
        vec3 rd = reflect(d, n);
        rd.y = abs(rd.y);
        float fres = 0.04 + 0.96 * pow(1.0 - max(dot(n, -d), 0.0), 5.0);
        c = mix(vec3(0.02, 0.05, 0.08), himmel(rd, 0.25), fres);
        // Gischt: an der Wand, um die Brandungsfelsen, in Schueben mit der Bassdrum
        float abst = min(kante(p.z) - p.x, stacks(vec3(p.x, 0.2, p.z)) * 1.5);
        float welle = fbm(vec2(p.z * 0.8 + abst * 0.6 + p.x * 0.3, time * 1.3 + abst * 2.0));
        float gischt = smoothstep(2.0, 0.0, abst + 0.3) * smoothstep(0.3, 0.7, welle);
        gischt *= 0.75 + 0.6 * kick;
        c = mix(c, vec3(0.95, 0.88, 0.9), clamp(gischt, 0.0, 1.0) * 0.9);
        // Spiegelung des Fensters und der Laterne
        c += vec3(1.0, 0.85, 0.6) * pow(max(dot(rd, normalize(lampe - p)), 0.0), 200.0) * 0.8;
        // der Strahl auf dem Wasser
        float imKegel = smoothstep(0.985, 0.998, dot(normalize(p - lampe), b));
        c += vec3(1.0, 0.9, 0.7) * imKegel * 0.6;
        c = mix(c, himmel(normalize(vec3(d.x, 0.0, d.z)), 1.0), smoothstep(30.0, 140.0, tWasser));
    } else {
        tTreffer = 1e9;
        c = himmel(d, 1.0);
    }
    // die Moewen (vor dem Himmel, als dunkle Silhouetten)
    if (tTreffer > 15.0) c = mix(c, vec3(0.08, 0.07, 0.1), moewen(uv, ro, vorne, rechts, oben) * 0.85);
    // der Lichtkegel im Dunst, und die Lampe selbst
    c += strahl(ro, d, tTreffer, lampe, b) * (1.0 + 0.4 * ton);
    vec3 zl = lampe - ro;
    float zlL = length(zl);
    if (zlL < tTreffer + 0.5) {
        float zur = max(dot(d, zl / zlL), 0.0);
        float blick = max(dot(-b, zl / zlL), 0.0);
        c += vec3(1.0, 0.92, 0.75) * pow(zur, 4000.0) * 4.0;
        c += vec3(1.0, 0.85, 0.6) * pow(zur, 300.0) * (0.3 + 3.0 * pow(blick, 30.0));
    }
    c *= 1.0 + 0.08 * kick;
    c *= 1.0 - 0.3 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.9));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
