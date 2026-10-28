#version 330
// Heiligabend: erst von draussen vor dem Fenster (Schnee auf der Bank, Eis-
// blumen in den Ecken der Scheiben), dann hindurch in die Stube. Dort brennt
// der Kamin, unter dem Baum liegen Geschenke, eine Lichterkette windet sich
// um die Tanne. Der Baum schmueckt sich zu "Stille Nacht": `kugeln` zaehlt
// die Kugeln (je Ton eine, von oben nach unten, die juengste geht gerade
// auf). Gegen Mitternacht wird es dunkler (`nacht`), jede Glocke (`glocke`)
// schickt einen Lichtring von der Spitze aus, und mit `stern` geht der Stern
// auf der Spitze auf.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float takt;
uniform float hell;
uniform float fort;
uniform float kugeln;
uniform float glocke;
uniform float stern;
uniform float nacht;
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
const vec3 BAUM = vec3(1.0, 0.0, 5.2);
const float SPITZE = 3.05;
const float WAND_Z = -2.0;
const float EBENEN = 12.0;   // Kugeln: zwoelf Ebenen zu je acht
const float JE_EBENE = 8.0;

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float sdKegel(vec3 p, vec2 c, float h) {
    vec2 q = h * vec2(c.x / c.y, -1.0);
    vec2 w = vec2(length(p.xz), p.y);
    vec2 a = w - q * clamp(dot(w, q) / dot(q, q), 0.0, 1.0);
    vec2 b = w - q * vec2(clamp(w.x / q.x, 0.0, 1.0), 1.0);
    float k = sign(q.y);
    float d = min(dot(a, a), dot(b, b));
    float s = max(k * (w.x * q.y - w.y * q.x), k * (w.y - q.y));
    return sqrt(d) * sign(s);
}

// Radius der Huelle des Baums in der Hoehe y
float baumRadius(float y) { return max(0.0, 1.32 * (SPITZE - y) / (SPITZE - 0.3)); }

// Die Kugeln: die Hoehe teilt der Baum in Ebenen, den Umfang in Sektoren;
// je Zelle eine Kugel auf der Huelle. Ihre Nummer zaehlt von oben nach unten,
// innerhalb der Ebene in einer festen, gewuerfelten Reihenfolge.
float kugelNr;
float kugel(vec3 p) {
    vec3 q = p - BAUM;
    float dy = (SPITZE - 0.55) / EBENEN;
    float ebene = clamp(floor((SPITZE - 0.35 - q.y) / dy), 0.0, EBENEN - 1.0);
    float yc = SPITZE - 0.35 - (ebene + 0.5) * dy;
    float versatz = ebene * 0.7;
    float a = atan(q.z, q.x) - versatz;
    float sek = 2.0 * PI / JE_EBENE;
    float s = floor(mod(a, 2.0 * PI) / sek);
    float ac = (s + 0.5) * sek + versatz;
    float r = baumRadius(yc) * 0.9 + 0.08;
    vec3 c = vec3(cos(ac) * r, yc - 0.05 * hash(vec2(ebene, s)), sin(ac) * r);
    // Reihenfolge in der Ebene: gewuerfelt, aber fest
    float rang = mod(s * 5.0 + ebene * 3.0, JE_EBENE);
    kugelNr = ebene * JE_EBENE + rang;
    float auf = clamp(kugeln - kugelNr, 0.0, 1.0);
    // die juengste waechst mit einem kleinen Ueberschwingen
    float gr = auf < 1.0 ? auf * (1.0 + 0.6 * sin(auf * PI)) : 1.0;
    float rad = (0.058 + 0.02 * hash(vec2(s, ebene + 3.0))) * gr;
    if (rad <= 0.001) return 1e5;
    return length(q - c) - rad;
}

// Die Lichterkette: eine Schraube um den Baum, eine Birne je Achtel Umlauf
float birneD;
float birneId;
float lichterkette(vec3 p) {
    vec3 q = p - BAUM;
    float windungen = 5.0;
    float H = SPITZE - 0.45;
    float a = atan(q.z, q.x) + PI;
    // zu dieser Richtung gehoerende Hoehen der Schraube: y = SPITZE - (a/2pi + k) H/w
    float schritt = H / windungen;
    float k = floor(((SPITZE - 0.1 - q.y) / schritt) - a / (2.0 * PI) + 0.5);
    float best = 1e5;
    for (int j = -1; j <= 0; j++) {
        float kk = k + float(j);
        float aq = (floor(a / (2.0 * PI) * 10.0) + 0.5) / 10.0 * 2.0 * PI;
        float y = SPITZE - 0.1 - (aq / (2.0 * PI) + kk) * schritt;
        if (y < 0.35 || y > SPITZE - 0.15) continue;
        float r = baumRadius(y) * 0.93 + 0.04;
        vec3 c = vec3(cos(aq - PI) * r, y, sin(aq - PI) * r);
        float d = length(q - c) - 0.028;
        if (d < best) { best = d; birneId = kk * 10.0 + floor(a / (2.0 * PI) * 10.0); }
    }
    return best;
}

// Material: 0 Boden, 1 Wand, 2 Decke, 3 Baum, 4 Stamm, 5 Kugel, 6 Geschenk,
// 7 Kamin, 8 Fensterrahmen, 9 Schnee aussen, 10 Teppich
float mat;
float geschenkId;
float szene(vec3 p) {
    // der Raum: eine hohle Kiste, die Fensterwand mit Oeffnung
    // Waende, Boden und Decke als Schale von 0.3 Dicke, die Fensterwand mit
    // ihrer Oeffnung -- von innen wie von aussen richtig
    vec3 rp = p - vec3(0.0, 1.6, 3.0);
    float raum = max(sdBox(rp, vec3(4.3, 1.9, 5.3)), -sdBox(rp, vec3(4.0, 1.6, 5.0)));
    float loch = sdBox(p - vec3(0.0, 1.55, WAND_Z - 0.15), vec3(1.05, 0.85, 0.3));
    raum = max(raum, -loch);
    float d = raum;
    mat = p.y < 0.01 && p.z > WAND_Z ? 0.0 : (p.y > 3.19 ? 2.0 : 1.0);
    // Fensterrahmen und Sprossen in der Scheibenebene
    vec3 f = p - vec3(0.0, 1.55, WAND_Z - 0.05);
    float rahmen = max(sdBox(f, vec3(1.05, 0.85, 0.05)), -sdBox(f, vec3(0.97, 0.77, 0.1)));
    rahmen = min(rahmen, sdBox(f, vec3(0.035, 0.85, 0.04)));
    rahmen = min(rahmen, sdBox(f - vec3(0.0, 0.12, 0.0), vec3(1.05, 0.035, 0.04)));
    if (rahmen < d) { d = rahmen; mat = 8.0; }
    // aussen: Schnee auf der Fensterbank und am Boden vor dem Haus
    float bank = sdBox(p - vec3(0.0, 0.66, WAND_Z - 0.42), vec3(1.2, 0.04, 0.14)) - 0.03 - 0.02 * rausch(p.xz * 9.0);
    if (bank < d) { d = bank; mat = 9.0; }
    float draussen = max(p.y + 0.4 - 0.1 * rausch(p.xz * 0.8), p.z - (WAND_Z - 0.3));
    if (draussen < d) { d = draussen; mat = 9.0; }
    // Teppich vor dem Baum
    float teppich = max(length((p.xz - vec2(0.3, 4.2)) / vec2(2.0, 1.4)) - 1.0, p.y - 0.012);
    if (teppich < d) { d = teppich; mat = 10.0; }
    // der Baum: fuenf Lagen Tanne
    vec3 q = p - BAUM;
    if (length(q.xz) < 1.6 && q.y < SPITZE + 0.2) {
        float tanne = 1e5;
        for (int i = 0; i < 5; i++) {
            float fi = float(i);
            float spitze = 1.05 + fi * 0.5;
            float r = 1.32 - fi * 0.21;
            float k = sdKegel(q - vec3(0.0, spitze + (i == 4 ? 0.0 : 0.0), 0.0), normalize(vec2(r, 0.95)), 0.95);
            tanne = min(tanne, k);
        }
        // Zweige: zerzauster Rand
        float w = atan(q.z, q.x);
        tanne -= 0.04 * sin(w * 22.0 + q.y * 14.0) * sin(q.y * 31.0);
        float stamm = max(length(q.xz) - 0.08, q.y - 0.4);
        if (tanne < d) { d = tanne; mat = 3.0; }
        if (stamm < d) { d = stamm; mat = 4.0; }
        float kg = kugel(p);
        if (kg < d) { d = kg; mat = 5.0; }
        birneD = lichterkette(p);
        d = min(d, birneD + 0.01);
    } else {
        birneD = 1e5;
    }
    // Geschenke unter dem Baum
    for (int i = 0; i < 4; i++) {
        float fi = float(i);
        vec3 gp = BAUM + vec3(-1.1 + fi * 0.7, 0.0, -1.0 + 0.35 * sin(fi * 2.0));
        vec3 gg = vec3(0.17 + 0.06 * hash(vec2(fi, 1.0)), 0.13 + 0.08 * hash(vec2(fi, 2.0)), 0.18);
        float gs = sdBox(p - gp - vec3(0.0, gg.y, 0.0), gg) - 0.01;
        if (gs < d) { d = gs; mat = 6.0; geschenkId = fi; }
    }
    // der Kamin an der linken Wand
    vec3 kp = p - vec3(-3.7, 0.0, 3.5);
    float kamin = sdBox(kp - vec3(0.0, 0.75, 0.0), vec3(0.35, 0.75, 0.95));
    kamin = max(kamin, -sdBox(kp - vec3(0.35, 0.45, 0.0), vec3(0.25, 0.4, 0.55)));
    kamin = min(kamin, sdBox(kp - vec3(0.05, 1.55, 0.0), vec3(0.45, 0.06, 1.1)));
    if (kamin < d) { d = kamin; mat = 7.0; }
    return d;
}

vec3 normale(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
}

// Das Feuer im Kamin: Flammen aus Rauschen, die nach oben ziehen
vec3 feuer(vec3 p) {
    vec3 kp = p - vec3(-3.7, 0.0, 3.5);
    if (kp.x < 0.05 || kp.x > 0.45 || abs(kp.z) > 0.55 || kp.y > 0.85) return vec3(0.0);
    vec2 q = vec2(kp.z, kp.y);
    float n = rausch(vec2(q.x * 6.0, q.y * 4.0 - time * 3.0)) * 0.6 + rausch(vec2(q.x * 13.0, q.y * 9.0 - time * 5.0)) * 0.4;
    float form = smoothstep(0.7, 0.05, q.y + abs(q.x) * 0.9) * n;
    vec3 c = mix(vec3(0.9, 0.2, 0.02), vec3(1.0, 0.85, 0.4), smoothstep(0.3, 0.8, form));
    return c * smoothstep(0.2, 0.55, form) * 3.0;
}

vec3 kaminLicht() {
    return vec3(1.0, 0.5, 0.18) * (0.85 + 0.15 * sin(time * 9.0) * sin(time * 4.3));
}

// draussen: Nachthimmel mit Sternen und dem Dorf im Hintergrund
vec3 draussenHimmel(vec3 d) {
    vec3 c = mix(vec3(0.05, 0.07, 0.15), vec3(0.01, 0.015, 0.04), clamp(d.y * 2.0, 0.0, 1.0));
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 150.0;
    vec2 z = floor(uv);
    c += vec3(0.85, 0.9, 1.0) * step(0.985, hash(z)) * smoothstep(0.35, 0.0, length(fract(uv) - 0.5));
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
    // Kamera: vor dem Fenster, dann hindurch und auf den Baum zu; zum Stern
    // hebt sie den Blick
    float f1 = smoothstep(0.0, 0.3, fort);
    float f2 = smoothstep(0.25, 1.0, fort);
    vec3 ro = mix(vec3(0.0, 1.5, WAND_Z - 2.6), vec3(0.0, 1.55, WAND_Z + 0.6), f1);
    ro = mix(ro, vec3(-0.6, 1.55, 1.2), f2);
    vec3 ta = mix(vec3(0.3, 1.45, 5.0), BAUM + vec3(0.0, 1.5, 0.0), f2);
    ta.y += stern * 0.7 + 0.2 * smoothstep(0.85, 0.95, fort);
    vec3 vorne = normalize(ta - ro);
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.3 + uv.x * rechts + uv.y * oben);

    // Wie hell die Stube ist: gegen Mitternacht nur noch Baum und Kamin
    float raumLicht = mix(1.0, 0.35, nacht);

    vec3 c;
    float t = 0.0;
    float hit = -1.0;
    vec3 glimmen = vec3(0.0);
    int schritte = int(120.0 * G());
    for (int i = 0; i < 120; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float d = szene(p);
        float bd = max(birneD, 0.0);
        glimmen += vec3(1.0, 0.78, 0.45) * 0.0016 / (0.0003 + bd * bd) * min(d, 0.2) * (0.7 + 0.3 * sin(time * 3.0 + birneId * 1.7));
        if (d < 0.001 * t + 0.0005) { hit = t; break; }
        t += d * 0.8;
        if (t > 30.0) break;
    }
    if (hit > 0.0) {
        vec3 p = ro + rd * hit;
        szene(p);
        float m = mat;
        float kn = kugelNr;
        float gid = geschenkId;
        vec3 n = normale(p);
        vec3 alb;
        vec3 eigen = vec3(0.0);
        float glanz = 0.0;
        if (birneD < 0.004) {
            alb = vec3(0.0);
            eigen = vec3(1.0, 0.8, 0.5) * 3.0;
        } else if (m < 0.5) {
            // Dielen
            float diele = fract(p.x * 2.2);
            alb = mix(vec3(0.22, 0.11, 0.05), vec3(0.3, 0.16, 0.07), rausch(vec2(floor(p.x * 2.2), p.z * 0.6)));
            alb *= 0.75 + 0.25 * smoothstep(0.0, 0.05, diele);
            glanz = 0.25;
        } else if (m < 1.5) {
            // Tapete mit kleinem Muster; aussen weisser Putz
            if (p.z < WAND_Z - 0.2) {
                alb = vec3(0.7, 0.66, 0.6);
            } else {
                vec2 tp = vec2(p.x + p.z, p.y) * 5.0;
                float muster = smoothstep(0.12, 0.08, length(fract(tp) - 0.5));
                alb = mix(vec3(0.42, 0.12, 0.1), vec3(0.6, 0.45, 0.25), muster * 0.5);
            }
        } else if (m < 2.5) {
            alb = vec3(0.4, 0.35, 0.3);
        } else if (m < 3.5) {
            alb = mix(vec3(0.01, 0.06, 0.025), vec3(0.06, 0.18, 0.07), rausch(p.xz * 40.0 + p.y * 20.0));
        } else if (m < 4.5) {
            alb = vec3(0.15, 0.08, 0.04);
        } else if (m < 5.5) {
            // Kugeln: rot, gold, silber, blau -- spiegelnd
            float wahl = hash(vec2(kn, 4.0));
            alb = wahl < 0.35 ? vec3(0.7, 0.03, 0.04) : wahl < 0.65 ? vec3(0.85, 0.6, 0.15) : wahl < 0.85 ? vec3(0.75, 0.78, 0.82) : vec3(0.1, 0.25, 0.75);
            glanz = 1.0;
            // die juengste leuchtet kurz auf
            float neu = clamp(1.0 - (kugeln - kn - 1.0), 0.0, 1.0);
            eigen += alb * neu * 1.5;
        } else if (m < 6.5) {
            float wahl = gid;
            vec3 papier = wahl < 0.5 ? vec3(0.7, 0.05, 0.05) : wahl < 1.5 ? vec3(0.1, 0.3, 0.6) : wahl < 2.5 ? vec3(0.8, 0.65, 0.2) : vec3(0.1, 0.45, 0.2);
            vec3 gp = p - (BAUM + vec3(-1.1 + gid * 0.7, 0.0, -1.0 + 0.35 * sin(gid * 2.0)));
            float band = step(abs(gp.x), 0.025) + step(abs(gp.z), 0.025);
            alb = mix(papier, vec3(0.9, 0.8, 0.5), clamp(band, 0.0, 1.0));
        } else if (m < 7.5) {
            alb = vec3(0.45, 0.2, 0.15) * (0.8 + 0.2 * step(0.5, fract(p.y * 6.0 + step(0.5, fract(p.z * 3.0)) * 0.5)));
        } else if (m < 8.5) {
            alb = vec3(0.12, 0.06, 0.03);
        } else if (m < 9.5) {
            alb = vec3(0.8, 0.85, 0.95);
        } else {
            alb = vec3(0.45, 0.06, 0.06) * (0.8 + 0.2 * sin(length(p.xz - vec2(0.3, 4.2)) * 30.0));
        }
        // Licht: Kamin, Baum (Lichterkette), Deckenlampe; draussen Mondlicht
        vec3 licht;
        if (p.z < WAND_Z - 0.1) {
            licht = vec3(0.2, 0.25, 0.4) * (0.5 + 0.5 * n.y) + vec3(0.9, 0.6, 0.3) * 0.9 * exp(-length(p.xy - vec2(0.0, 1.5)) * 1.0);
        } else {
            vec3 kl = vec3(-3.3, 0.6, 3.5) - p;
            float kd = length(kl);
            licht = kaminLicht() * max(dot(n, kl / kd), 0.0) * 4.0 / (1.0 + kd * kd * 0.5);
            vec3 bl = BAUM + vec3(0.0, 1.5, 0.0) - p;
            float bd = length(bl);
            licht += vec3(1.0, 0.75, 0.42) * (max(dot(n, bl / bd), 0.0) * 0.7 + 0.3) * 2.2 / (1.0 + bd * bd * 0.45);
            licht += vec3(0.9, 0.75, 0.55) * (0.15 + 0.15 * n.y) * raumLicht;
            // der Stern auf der Spitze strahlt in den Raum
            vec3 sl = BAUM + vec3(0.0, SPITZE + 0.15, 0.0) - p;
            float sd = length(sl);
            licht += vec3(1.0, 0.9, 0.6) * stern * max(dot(n, sl / sd), 0.0) * 3.0 / (1.0 + sd * sd * 0.3);
            // die Glocken: ein Hauch Licht, der mit jedem Schlag durch den Raum geht
            licht += vec3(1.0, 0.8, 0.5) * glocke * 0.25;
        }
        c = alb * licht + eigen;
        if (glanz > 0.0) {
            vec3 r = reflect(rd, n);
            vec3 zumBaum = normalize(BAUM + vec3(0.0, 1.5, 0.0) - p);
            vec3 zumKamin = normalize(vec3(-3.3, 0.6, 3.5) - p);
            c += vec3(1.0, 0.8, 0.5) * pow(max(dot(r, zumKamin), 0.0), 30.0) * glanz * 1.5;
            c += vec3(1.0, 0.85, 0.6) * pow(max(dot(r, -zumBaum), 0.0), 8.0) * glanz * 0.4;
            if (m > 4.5 && m < 5.5) c += vec3(1.0, 0.95, 0.85) * pow(max(dot(r, normalize(vec3(0.2, 1.0, -0.3))), 0.0), 60.0);
        }
        c += feuer(p);
    } else {
        c = draussenHimmel(rd);
    }
    // die Scheibe: wer davor steht, sieht Spiegelung und Eisblumen in den Ecken
    float tg = (WAND_Z - 0.05 - ro.z) / rd.z;
    if (tg > 0.0 && (hit < 0.0 || tg < hit)) {
        vec3 gp = ro + rd * tg;
        vec2 s = gp.xy - vec2(0.0, 1.55);
        if (abs(s.x) < 1.0 && abs(s.y) < 0.8) {
            vec2 scheibe = vec2(abs(s.x) - 0.5, s.y - (s.y > 0.12 ? 0.46 : -0.33));
            float ecke = length(abs(scheibe) - vec2(0.48, 0.33 + (s.y > 0.12 ? -0.0 : 0.12)));
            float eis = smoothstep(0.35, 0.05, ecke + 0.12 * rausch(gp.xy * 14.0)) * (0.6 + 0.4 * rausch(gp.xy * 40.0));
            c = mix(c, vec3(0.7, 0.8, 0.95), eis * 0.55);
            c += vec3(0.6, 0.7, 0.9) * 0.04;
        }
    }
    c += glimmen * (1.0 + 0.5 * ton);
    // der Stern auf der Spitze, im Bild gezeichnet: vor dem Aufgehen ein
    // matter goldener Umriss, danach hell mit Strahlen
    vec3 sp = BAUM + vec3(0.0, SPITZE + 0.15, 0.0) - ro;
    float sz = dot(sp, vorne);
    if (sz > 0.1) {
        vec2 su = vec2(dot(sp, rechts), dot(sp, oben)) / sz * 1.3;
        vec2 q = uv - su;
        float r = length(q);
        float a = atan(q.y, q.x) + PI * 0.5;
        float zacke = 0.5 + 0.5 * cos(a * 5.0);
        float groesse = 0.2 / sz;
        float form = smoothstep(groesse * (0.45 + 0.55 * zacke), groesse * (0.45 + 0.55 * zacke) - 0.004, r);
        c = mix(c, vec3(0.55, 0.42, 0.15) * (1.0 + stern * 4.0), form);
        float strahl = pow(abs(cos(a * 2.0)), 50.0) + pow(abs(cos(a * 2.0 + 0.785)), 70.0) * 0.5;
        c += vec3(1.0, 0.88, 0.6) * stern * (exp(-r * 30.0) * 2.0 + strahl * exp(-r * 4.0) * 1.0 + exp(-r * 2.5) * 0.3) * (1.0 + 0.3 * ton);
        // Mitternacht: ein Lichtring von der Spitze aus je Glockenschlag
        float ring = (1.0 - glocke) * 1.4;
        c += vec3(1.0, 0.85, 0.55) * glocke * exp(-pow((r - ring) * 14.0, 2.0)) * 0.6;
    }
    c *= 1.0 - 0.35 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.92));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
