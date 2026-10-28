#version 330
// Der Adventskranz: Tannengruen auf einem Holztisch, vier rote Kerzen, rotes
// Samtband, Beeren. `kerzen` (0..4) sagt, wie viele brennen (mit Bruchteil: die
// naechste entzuendet sich gerade), `fokus` (0..1) faehrt die Kamera in die
// Flamme der Kerze `ziel` hinein -- dort beginnt das naechste Kapitel. Im
// Hintergrund unscharfe Lichter einer Lichterkette. Text und Zeichnungen der
// Demo liegen in texture0 und werden obenauf gelegt.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;      // Blende: 0 schwarz .. 1 voll
uniform float kerzen;    // wie viele Kerzen brennen
uniform float fokus;     // Fahrt in die Flamme
uniform float ziel;      // ... welcher Kerze
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
float hash3(vec3 p) { return fract(sin(dot(p, vec3(127.1, 311.7, 74.7))) * 43758.5453); }
float rausch3(vec3 p) {
    vec3 i = floor(p), f = fract(p);
    vec3 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash3(i), hash3(i + vec3(1, 0, 0)), u.x),
                   mix(hash3(i + vec3(0, 1, 0)), hash3(i + vec3(1, 1, 0)), u.x), u.y),
               mix(mix(hash3(i + vec3(0, 0, 1)), hash3(i + vec3(1, 0, 1)), u.x),
                   mix(hash3(i + vec3(0, 1, 1)), hash3(i + vec3(1, 1, 1)), u.x), u.y), u.z);
}

const float R_KRANZ = 1.0;
const float KERZE_R = 0.088;

// Winkel der Kerze k (0..3): reihum, die erste vorne links
float kerzenWinkel(float k) { return 2.356 + k * 1.5708; }
// die erste brennt seit dem ersten Advent -- sie ist die kuerzeste
float kerzenHoehe(float k) { return 0.46 + k * 0.06; }
vec3 kerzenFuss(float k) {
    float w = kerzenWinkel(k);
    return vec3(cos(w) * R_KRANZ, 0.3, sin(w) * R_KRANZ);
}
// Brennt Kerze k? 0 = aus, 1 = voll, dazwischen entzuendet sie sich
float brennt(float k) { return clamp((kerzen - k) * 3.0, 0.0, 1.0); }
vec3 flamme(float k) {
    vec3 f = kerzenFuss(k);
    float flackern = 0.012 * sin(time * 9.0 + k * 3.0) + 0.008 * sin(time * 23.0 + k);
    return f + vec3(flackern, kerzenHoehe(k) + 0.085, flackern * 0.6);
}

// --- Abstandsfunktionen ------------------------------------------------------
float sdZylinder(vec3 p, float r, float h) {
    vec2 d = abs(vec2(length(p.xz), p.y - h * 0.5)) - vec2(r, h * 0.5);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

// Material: 0 Tisch, 1 Gruen, 2 Kerze, 3 Docht, 4 Beere, 5 Band
float mat;
float szene(vec3 p) {
    // Tisch
    float d = p.y;
    mat = 0.0;
    // Kranz: ein Ring aus Tannengruen, zerzaust
    vec2 q = vec2(length(p.xz) - R_KRANZ, p.y - 0.16);
    float k = length(q * vec2(1.0, 1.25)) - 0.24;
    if (k < 0.25) {
        float w = atan(p.z, p.x);
        // Zweige: schraeg stehende Buendel rund um den Ring
        float zweig = sin(w * 46.0 + q.y * 18.0) * sin(q.x * 30.0 + w * 13.0);
        // Nadeln: feine Grate schraeg ueber jedem Zweig
        float nadel = sin(w * 260.0 + q.y * 70.0 + q.x * 40.0) * sin(w * 90.0 - q.y * 120.0);
        k += -0.045 * zweig - 0.05 * rausch3(p * 14.0) - 0.012 * nadel + 0.03;
    }
    if (k < d) { d = k; mat = 1.0; }
    // Kerzen, Dochte
    for (int i = 0; i < 4; i++) {
        float fi = float(i);
        vec3 f = kerzenFuss(fi);
        float hk = kerzenHoehe(fi);
        vec3 lp = p - f;
        // ein wenig herabgelaufenes Wachs am oberen Rand
        float wachs = 0.008 * sin(atan(lp.z, lp.x) * 7.0 + fi) * smoothstep(hk - 0.12, hk, lp.y);
        float c = sdZylinder(lp, KERZE_R + wachs, hk) - 0.008;
        // die Oberseite leicht eingesunken
        c = max(c, -(length(lp.xz) - KERZE_R * 0.75) * 0.5 - (hk - lp.y) - 0.01 + 0.025);
        if (c < d) { d = c; mat = 2.0; }
        float dc = sdZylinder(lp - vec3(0.0, hk - 0.03, 0.0), 0.007, 0.05);
        if (dc < d) { d = dc; mat = 3.0; }
    }
    // Beeren: kleine rote Kugeln in Gruppen auf dem Gruen
    for (int i = 0; i < 8; i++) {
        float fi = float(i);
        float w = fi * 0.785 + 0.39;
        vec3 b = vec3(cos(w) * (R_KRANZ + 0.1), 0.34, sin(w) * (R_KRANZ + 0.1));
        vec3 bp = p - b;
        // je Gruppe drei Beeren
        float bb = length(bp - vec3(0.0, 0.0, 0.0)) - 0.045;
        bb = min(bb, length(bp - vec3(0.06, -0.02, 0.03)) - 0.04);
        bb = min(bb, length(bp - vec3(-0.03, -0.01, 0.06)) - 0.038);
        if (bb < d) { d = bb; mat = 4.0; }
    }
    // Schleifen aus rotem Samtband zwischen den Kerzen (vorne aussen)
    for (int i = 0; i < 4; i++) {
        float w = float(i) * 1.5708 + 1.5708;
        vec3 sp = p - vec3(cos(w) * (R_KRANZ + 0.24), 0.2, sin(w) * (R_KRANZ + 0.24));
        // zwei flache Ellipsoide schraeg
        vec3 a = vec3(-sin(w), 0.0, cos(w));
        float u = dot(sp, a);
        float s1 = length(vec3(abs(u) - 0.07, sp.y - abs(u) * 0.3, dot(sp, vec3(cos(w), 0, sin(w)))) * vec3(1.0, 1.6, 3.5)) - 0.07;
        if (s1 < d) { d = s1; mat = 5.0; }
    }
    return d;
}

vec3 normale(vec3 p) {
    vec2 e = vec2(0.0015, 0.0);
    return normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
}

// Unscharfe Lichter hinter dem Tisch (Bokeh): bunte Scheiben, die langsam
// pulsieren -- auf jeden Melodieton etwas heller
vec3 bokeh(vec2 uv) {
    vec3 c = vec3(0.0);
    for (int i = 0; i < 22; i++) {
        float fi = float(i);
        vec2 m = vec2(hash(vec2(fi, 1.3)) * 2.4 - 1.2, hash(vec2(fi, 7.1)) * 0.7 + 0.12);
        m.x += 0.03 * sin(time * 0.2 + fi);
        float r = 0.035 + hash(vec2(fi, 3.3)) * 0.05;
        float d = length(uv - m);
        float scheibe = smoothstep(r, r - 0.012, d) * (0.55 + 0.45 * smoothstep(r * 0.4, r, d));
        vec3 f = mix(vec3(1.0, 0.62, 0.25), vec3(1.0, 0.3, 0.2), hash(vec2(fi, 9.9)));
        if (hash(vec2(fi, 4.4)) < 0.3) f = vec3(0.4, 0.6, 1.0);
        float puls = 0.6 + 0.4 * sin(time * (0.8 + hash(vec2(fi, 5.5))) + fi * 2.0);
        c += f * scheibe * puls * 0.16 * (1.0 + 0.8 * ton);
    }
    return c;
}

// Die Flamme der Kerze k, wo der Strahl ihr am naechsten kommt: ein Tropfen,
// innen weiss-gelb, aussen orange, unten ein blauer Saum -- und ein weiter
// Schein darum.
vec3 flammenLicht(vec3 ro, vec3 rd, float tmax) {
    vec3 c = vec3(0.0);
    for (int i = 0; i < 4; i++) {
        float fi = float(i);
        float b = brennt(fi);
        if (b <= 0.0) continue;
        vec3 fz = flamme(fi);
        float tc = dot(fz - ro, rd);
        if (tc < 0.0 || tc > tmax + 0.2) continue;
        vec3 q = ro + rd * tc - fz;
        // Tropfenform: unten rund, oben spitz, leicht im Wind
        float y = q.y;
        q.x -= 0.01 * sin(time * 7.0 + fi * 2.0) * max(y, 0.0) * 10.0;
        float breite = mix(0.036, 0.007, smoothstep(-0.04, 0.12, y)) * mix(0.3, 1.0, b);
        float r = length(q.xz) / breite;
        float hoch = smoothstep(-0.06, -0.03, y) * smoothstep(0.14 * b, 0.02, y);
        float kern = exp(-r * r * 1.4) * hoch;
        vec3 fl = mix(vec3(1.0, 0.45, 0.08), vec3(1.0, 0.95, 0.75), smoothstep(0.2, 0.9, kern));
        fl = mix(vec3(0.25, 0.4, 1.0), fl, smoothstep(-0.05, -0.015, y));
        c += fl * kern * 3.5 * b;
        // Schein
        float d2 = dot(q, q);
        c += vec3(1.0, 0.6, 0.25) * exp(-d2 * 45.0) * 0.55 * b * (1.0 + 0.4 * ton);
        c += vec3(1.0, 0.5, 0.2) * exp(-d2 * 6.0) * 0.12 * b;
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
    // Kamera: kreist langsam schraeg ueber dem Kranz; mit `fokus` faehrt sie
    // auf die Flamme der Zielkerze zu, bis sie das Bild fuellt
    float w = 0.9 + time * 0.06;
    vec3 ro = vec3(cos(w) * 2.9, 1.75, sin(w) * 2.9);
    vec3 ta = vec3(0.0, 0.25, 0.0);
    vec3 fz = flamme(ziel);
    float f = fokus * fokus * (3.0 - 2.0 * fokus);
    ro = mix(ro, fz + normalize(ro - fz) * 0.12 + vec3(0.0, 0.02, 0.0), f);
    ta = mix(ta, fz + vec3(0.0, 0.03, 0.0), smoothstep(0.0, 0.6, fokus));
    vec3 vorne = normalize(ta - ro);
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.6 + uv.x * rechts + uv.y * oben);

    // Hintergrund: ein warmer, dunkler Raum mit unscharfen Lichtern
    vec3 c = mix(vec3(0.03, 0.015, 0.01), vec3(0.008, 0.006, 0.012), smoothstep(-0.3, 0.6, uv.y));
    c += bokeh(uv + vec2(w * 0.25, 0.0)) * smoothstep(-0.1, 0.25, rd.y + 0.15);

    // Raymarching
    float t = 0.0;
    float hit = -1.0;
    int schritte = int(110.0 * G());
    for (int i = 0; i < 110; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float d = szene(p);
        if (d < 0.0015 * t) { hit = t; break; }
        t += d * 0.8;
        if (t > 12.0) break;
    }
    float tmax = 12.0;
    if (hit > 0.0) {
        tmax = hit;
        vec3 p = ro + rd * hit;
        float m = mat;
        vec3 n = normale(p);
        vec3 alb;
        if (m < 0.5) {
            // dunkles Holz mit Maserung, der Kranz spiegelt sich ein wenig
            float maser = rausch3(vec3(p.x * 1.5, 0.0, p.z * 22.0)) * 0.6 + 0.4 * sin(p.z * 40.0 + rausch3(p * 3.0) * 6.0);
            alb = mix(vec3(0.08, 0.035, 0.015), vec3(0.16, 0.07, 0.03), maser);
        } else if (m < 1.5) {
            float spitze = rausch3(p * 30.0) * 0.6 + 0.4 * rausch3(p * 90.0);
            alb = mix(vec3(0.008, 0.04, 0.018), vec3(0.07, 0.19, 0.07), spitze);
            // hier und da ein Hauch Rauhreif auf den Spitzen
            alb += vec3(0.08, 0.1, 0.12) * smoothstep(0.78, 0.95, rausch3(p * 22.0 + 3.0)) * smoothstep(0.0, 0.25, n.y);
        } else if (m < 2.5) {
            alb = vec3(0.62, 0.04, 0.04);
        } else if (m < 3.5) {
            alb = vec3(0.02);
        } else if (m < 4.5) {
            alb = vec3(0.7, 0.02, 0.03);
        } else {
            alb = vec3(0.5, 0.02, 0.05) * (0.7 + 0.3 * sin(p.y * 80.0));
        }
        // Licht der Kerzen, dazu ein Hauch kaltes Licht vom Fenster
        vec3 licht = vec3(0.025, 0.03, 0.05) * (0.5 + 0.5 * n.y);
        for (int i = 0; i < 4; i++) {
            float fi = float(i);
            float b = brennt(fi);
            if (b <= 0.0) continue;
            vec3 l = flamme(fi) - p;
            float ld = length(l);
            l /= ld;
            float flackern = 0.85 + 0.15 * sin(time * 11.0 + fi * 2.7) * sin(time * 5.3 + fi);
            float diff = max(dot(n, l), 0.0);
            // Wachs laesst Licht durch: die Kerze leuchtet von innen
            if (m > 1.5 && m < 2.5) diff = diff * 0.6 + 0.4 * exp(-ld * 4.0) * 3.0;
            licht += vec3(1.0, 0.62, 0.3) * diff * b * flackern / (1.0 + ld * ld * 5.0) * 2.2;
        }
        c = alb * licht * 3.0;
        // Glanz auf Beeren und Band
        if (m > 3.5) {
            vec3 h = normalize(normalize(flamme(0.0) - p) - rd);
            c += vec3(1.0, 0.8, 0.6) * pow(max(dot(n, h), 0.0), 40.0) * 0.4 * brennt(0.0);
        }
        // in die Ferne dunkel
        c *= exp(-max(hit - 3.0, 0.0) * 0.4);
    }
    c += flammenLicht(ro, rd, tmax);
    // am Ende der Fahrt fuellt das Licht der Flamme das Bild
    c = mix(c, vec3(1.0, 0.8, 0.5) * 1.3, smoothstep(0.75, 1.0, fokus) * 0.6);
    c *= 1.0 + 0.1 * kick;
    c *= 1.0 - 0.4 * dot(uv * 0.85, uv * 0.85);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.9));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
