#version 330
// Die Galaxie: hinter den Lichtspuren der 3D-Szene (texture0) rechnet dieser
// Shader die leuchtende Scheibe selbst -- Spiralarme, die mit derselben
// Drehung umlaufen wie die Spuren, dunkle Staubbahnen an ihrer Innenkante,
// rosa Knoten, wo Sterne entstehen, einen hellen Kern mit Strahlen und auf
// jeden Schlag einen Lichtring, der nach aussen laeuft. Dazu fuer die
// Spuren ein Bloom, eine sanfte Farbtrennung und die Vignette. `blitz`
// legt auf der Eins des Hoehepunkts ein kurzes Weiss darueber. Dahinter der
// Weltraum: farbige Nebel mit Faeden und dunklem Staub, das Band einer fernen
// Milchstrasse und drei Schichten Sterne, fest im Raum.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
uniform float blitz;
uniform float takt;
uniform float fort;      // 0..1 durch den Abschnitt
uniform vec3 kamera;     // dieselbe Kamera wie CAMERA3D in main.dh
uniform float ton;       // Melodie: 1 auf dem Anschlag, klingt ab
uniform float hoehe;     // wie hoch der Melodieton liegt, 0..1
uniform vec3 farbe;      // Farbe des Akkords
uniform float aus;       // Szenenwechsel: Licht waechst aus der Mitte (0..1)
uniform float ein;       // ... und weicht nach aussen (0 = nur Licht, 1 = Szene)
uniform vec3 uebLicht;   // Farbe des Lichts beim Wechsel
uniform float guete;     // 1 = volle Qualitaet, kleiner = weniger Schritte (--leicht)
out vec4 finalColor;

// Guete: ohne gesetzten Wert (0) gilt die volle
float G() { return guete > 0.0 ? guete : 1.0; }

const float FOV = 52.0;

float hash2(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float rausch(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash2(i), hash2(i + vec2(1, 0)), u.x), mix(hash2(i + vec2(0, 1)), hash2(i + vec2(1, 1)), u.x), u.y);
}
float fbm(vec2 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 4; i++) { s += a * rausch(p); p = p * 2.07 + vec2(5.3, 1.9); a *= 0.5; }
    return s;
}

// Wo die Arme liegen: wie in main.dh, Winkel = r * 0.22 - Zeit * Umlauf(r)
float armPhase(float r, float phi) {
    return 2.0 * (phi - r * 0.22 + time * 0.9 / (r + 2.0));
}

// Leuchten und Staub an einem Punkt der Scheibe
void scheibe(vec3 p, out vec3 licht, out float staub) {
    float r = length(p.xz);
    float phi = atan(p.z, p.x);
    float ph = armPhase(r, phi);
    float arm = pow(0.5 + 0.5 * cos(ph), 4.0);
    float dicke = 0.3 + 1.1 * exp(-r * 0.22);
    float senk = exp(-(p.y * p.y) / (dicke * dicke));
    // Klumpen entlang der Arme: Rauschen in einem Feld, das mitdreht
    vec2 q = vec2(r * 0.9, ph * 2.2);
    float klumpen = fbm(q);
    float scheibeHell = exp(-r / 10.0) * (0.05 + 1.5 * arm * (0.3 + 1.1 * klumpen)) * smoothstep(40.0, 26.0, r);
    vec3 farbeArm = mix(vec3(0.45, 0.6, 1.0), vec3(0.4, 0.9, 1.0), smoothstep(0.4, 0.8, rausch(q * 0.7 + 3.0)));
    farbeArm = mix(farbeArm, farbe, 0.3);
    vec3 fScheibe = mix(vec3(1.0, 0.78, 0.5), farbeArm, smoothstep(1.5, 9.0, r));
    licht = fScheibe * scheibeHell * senk;
    // rosa Knoten, wo Sterne entstehen: nur auf den Armen, nur wo das Rauschen hoch ist
    licht += vec3(1.0, 0.35, 0.7) * pow(max(klumpen - 0.5, 0.0) * 2.5, 2.0) * arm * senk * smoothstep(5.0, 12.0, r) * smoothstep(32.0, 20.0, r) * 1.6;
    // Kern
    licht += vec3(1.0, 0.85, 0.62) * exp(-r * r / 2.5) * exp(-p.y * p.y / 0.8) * (1.3 + 1.0 * kick + 0.8 * ton) * (1.0 - 0.45 * fort);
    // der Ring des Schlags
    float welle = fract(takt * 4.0) * 36.0;
    licht += vec3(0.55, 0.8, 1.0) * exp(-(r - welle) * (r - welle) * 0.12) * senk * (0.08 + 0.35 * kick) * smoothstep(36.0, 20.0, welle);
    // Staub an der Innenkante der Arme
    float bahn = pow(0.5 + 0.5 * cos(ph + 0.6), 10.0);
    staub = bahn * smoothstep(3.0, 7.0, r) * smoothstep(34.0, 18.0, r) * exp(-(p.y * p.y) / 0.08) * (0.6 + 0.8 * rausch(q * 1.7 + 9.0));
}

// --- Der Weltraum dahinter -------------------------------------------------
// Alles haengt an der RICHTUNG des Blicks, fest im Raum -- dreht die Kamera,
// ziehen Nebel und Sterne mit, wie in echtem All.
float hash3(vec3 p) { return fract(sin(dot(p, vec3(127.1, 311.7, 74.7))) * 43758.5453); }
float rausch3(vec3 p) {
    vec3 i = floor(p), f = fract(p);
    vec3 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash3(i), hash3(i + vec3(1, 0, 0)), u.x), mix(hash3(i + vec3(0, 1, 0)), hash3(i + vec3(1, 1, 0)), u.x), u.y),
               mix(mix(hash3(i + vec3(0, 0, 1)), hash3(i + vec3(1, 0, 1)), u.x), mix(hash3(i + vec3(0, 1, 1)), hash3(i + vec3(1, 1, 1)), u.x), u.y), u.z);
}
float fbm3(vec3 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 5; i++) { s += a * rausch3(p); p = p * 2.03 + vec3(1.7, 9.2, 4.1); a *= 0.5; }
    return s;
}
// Faeden: aus Rauschen werden Grate
float faeden3(vec3 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 4; i++) {
        float n = 1.0 - abs(rausch3(p) * 2.0 - 1.0);
        s += a * n * n;
        p = p * 2.1 + vec3(3.1, 1.7, 5.3);
        a *= 0.5;
    }
    return s;
}

// eine Schicht Sterne: Zellen auf der Himmelskugel, je Zelle hoechstens einer
vec3 sternSchicht(vec3 d, float dichte, float schwelle, float groesse, float hell) {
    vec3 q = d * dichte;
    vec3 z = floor(q);
    float h = hash3(z);
    if (h < schwelle) return vec3(0.0);
    vec3 pos = z + 0.5 + (vec3(hash3(z + 1.3), hash3(z + 2.7), hash3(z + 4.1)) - 0.5) * 0.7;
    float r = length(q - pos);
    float glanz = smoothstep(groesse, 0.0, r) + 0.25 * exp(-r * 14.0 / groesse);
    float fun = 0.75 + 0.25 * sin(time * (1.0 + h * 2.0) + h * 40.0);
    vec3 farbe = mix(vec3(0.65, 0.78, 1.0), vec3(1.0, 0.84, 0.66), fract(h * 17.0));
    return farbe * glanz * fun * hell;
}

vec3 weltraum(vec3 d) {
    // drei Schichten Sterne, fein bis hell
    vec3 c = sternSchicht(d, 260.0, 0.93, 0.18, 0.55);
    c += sternSchicht(d, 120.0, 0.96, 0.22, 0.8);
    c += sternSchicht(d, 45.0, 0.985, 0.18, 1.4);
    // das Band einer fernen Milchstrasse: ein schraeger Grosskreis
    vec3 achse = normalize(vec3(0.3, 0.85, -0.42));
    float band = exp(-pow(dot(d, achse) * 4.5, 2.0));
    c += sternSchicht(d, 420.0, 0.8, 0.2, 0.35) * band;
    c += vec3(0.32, 0.3, 0.42) * band * (0.04 + 0.08 * fbm3(d * 6.0));
    // Nebel: zwei Felder fuer Dichte und Farbe, dazu Faeden und dunkler Staub
    vec3 q = d * 2.3 + vec3(0.0, 0.0, time * 0.004);
    float dicht = smoothstep(0.36, 0.78, fbm3(q));
    float fad = faeden3(q * 1.8 + 5.0);
    float mischung = fbm3(q * 0.7 + 11.0);
    vec3 f = mix(vec3(0.75, 0.18, 0.55), vec3(0.18, 0.3, 0.95), smoothstep(0.35, 0.65, mischung));
    f = mix(f, vec3(0.1, 0.75, 0.8), smoothstep(0.62, 0.8, mischung) * 0.7);
    f = mix(f, farbe, 0.2);
    float leucht = dicht * (0.25 + 1.4 * pow(fad, 3.0));
    vec3 nebel = f * leucht * 0.75;
    // helle Kerne, wo der Nebel am dichtesten ist
    nebel += mix(f, vec3(1.0), 0.5) * pow(dicht * fad, 4.0) * 0.5;
    // Staub: dunkle Schwaden, die Sterne und Nebel verschlucken
    float staub = smoothstep(0.5, 0.75, fbm3(q * 1.3 + 23.0)) * 0.75;
    return (c + nebel) * (1.0 - staub);
}

// Ein Komet: zieht auf einer weiten Bahn ueber der Scheibe; sein Schweif
// zeigt von der Mitte der Galaxie weg, ein zweiter, blauer (Gas) etwas
// versetzt. Kopf und Schweifende werden in die Bildebene projiziert (mit
// derselben Kamera wie der Strahl), der Schweif ist eine Strecke dazwischen.
vec2 projiziere(vec3 x, vec3 vorne, vec3 rechts, vec3 oben, float tf) {
    vec3 v = x - kamera;
    float z = max(dot(v, vorne), 0.01);
    return vec2(dot(v, rechts), dot(v, oben)) / (z * 2.0 * tf);
}
float zurStrecke(vec2 q, vec2 a, vec2 b, out float h) {
    vec2 pa = q - a, ba = b - a;
    h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-6), 0.0, 1.0);
    return length(pa - ba * h);
}
vec3 komet(vec2 q, vec3 vorne, vec3 rechts, vec3 oben, float tf) {
    float a = time * 0.11 + 2.2;
    vec3 kopf = vec3(cos(a) * 30.0, 9.0 + 4.0 * sin(a * 0.7), sin(a) * 20.0);
    if (dot(kopf - kamera, vorne) < 1.0) return vec3(0.0);
    vec3 weg = normalize(kopf + vec3(0.0, 4.0, 0.0));
    vec2 k = projiziere(kopf, vorne, rechts, oben, tf);
    vec2 s1 = projiziere(kopf + weg * 11.0, vorne, rechts, oben, tf);
    vec3 seit = normalize(cross(weg, vec3(0.0, 1.0, 0.0)));
    vec2 s2 = projiziere(kopf + weg * 14.0 + seit * 2.5, vorne, rechts, oben, tf);
    float h1, h2;
    float d1 = zurStrecke(q, k, s1, h1);
    float d2 = zurStrecke(q, k, s2, h2);
    float breite1 = 0.002 + 0.016 * h1;
    float breite2 = 0.001 + 0.008 * h2;
    vec3 c = vec3(1.0, 0.95, 0.85) * exp(-d1 * d1 / (breite1 * breite1)) * pow(1.0 - h1, 1.6) * 0.9;
    c += vec3(0.4, 0.7, 1.0) * exp(-d2 * d2 / (breite2 * breite2)) * pow(1.0 - h2, 1.2) * 0.7;
    float kd = length(q - k);
    c += vec3(0.85, 0.95, 1.0) * (exp(-kd * kd * 9.0e4) * 2.5 + exp(-kd * 60.0) * 0.25);
    return c;
}

vec3 probe(vec2 uv) {
    vec2 px = 0.5 / resolution;
    return texture(texture0, clamp(uv, px, 1.0 - px)).rgb;
}

// Szenenwechsel durch Licht (uv: zentriert, durch die Hoehe geteilt): ein
// Kreis aus Licht -- innen weiss, am Rand in der Farbe des Wechsels -- waechst
// aus der Mitte bzw. weicht nach aussen, dazu Strahlen wie bei einem Sprung.
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
    vec2 uv = fragTexCoord;
    vec2 m = uv - 0.5;

    // der Strahl der Kamera -- wie raylib ihn fuer CAMERA3D rechnet
    vec3 ziel = vec3(0.0, -0.5 * fort, 0.0);
    vec3 vorne = normalize(ziel - kamera);
    vec3 rechts = normalize(cross(vorne, vec3(0.0, 1.0, 0.0)));
    vec3 oben = cross(rechts, vorne);
    float tf = tan(radians(FOV) * 0.5);
    vec2 p = m * vec2(resolution.x / resolution.y, 1.0) * 2.0 * tf;
    vec3 d = normalize(vorne + p.x * rechts + p.y * oben);

    // die Scheibe: durch die Schicht |y| < 3 marschieren
    vec3 gal = vec3(0.0);
    float durch = 1.0;
    float t0 = 0.0, t1 = 0.0;
    if (abs(d.y) > 1e-4) {
        float ta = (3.0 - kamera.y) / d.y, tb = (-3.0 - kamera.y) / d.y;
        t0 = max(min(ta, tb), 0.0);
        t1 = max(ta, tb);
    } else if (abs(kamera.y) < 3.0) {
        t1 = 120.0;
    }
    t1 = min(t1, t0 + 110.0);
    if (t1 > t0) {
        const int N = 48;
        float nEff = floor(float(N) * G());
        float dt = (t1 - t0) / nEff;
        float t = t0 + dt * hash2(fragTexCoord * resolution + fract(time));
        for (int i = 0; i < N; i++) {
            vec3 q = kamera + d * t;
            vec3 l; float s;
            scheibe(q, l, s);
            gal += l * durch * dt * 1.3;
            durch *= exp(-s * dt * 1.8);
            t += dt;
            if (float(i) + 1.0 >= nEff) break;
        }
    }
    // Kern als Stern mit Strahlen, dort, wo die Mitte im Bild liegt
    vec3 zurMitte = normalize(-kamera);
    float wk = max(dot(d, zurMitte), 0.0);
    float abw = acos(min(wk, 1.0));
    vec3 ax = normalize(cross(zurMitte, oben));
    vec3 ay = cross(ax, zurMitte);
    vec2 kp = vec2(dot(d, ax), dot(d, ay));
    float strahl = exp(-abs(kp.x) * 900.0) * exp(-abs(kp.y) * 22.0) + exp(-abs(kp.y) * 900.0) * exp(-abs(kp.x) * 22.0);
    gal += vec3(1.0, 0.9, 0.75) * (exp(-abw * 70.0) * 0.8 + strahl * 0.5) * (0.8 + 0.6 * kick) * durch;

    // die Lichtspuren: Farbtrennung und Bloom
    float ab = 0.0012 + 0.002 * kick;
    vec3 c = vec3(probe(uv + m * ab).r, probe(uv).g, probe(uv - m * ab).b);
    vec3 b = vec3(0.0);
    vec2 px = 1.0 / resolution;
    for (int r = 1; r <= 3; r++) {
        for (int i = 0; i < 10; i++) {
            float a = float(i) * 0.6283 + float(r) * 0.9;
            vec2 o = vec2(cos(a), sin(a)) * px * float(r * r) * 4.0;
            vec3 s = probe(uv + o);
            b += max(s - 0.3, 0.0) / float(r);
        }
    }
    b /= 18.0;
    c += b * (2.2 + 1.8 * kick);
    // der Weltraum liegt hinter allem: verdeckt nur von den hellen Spuren,
    // gedaempft von Staub und Leuchten der Scheibe
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    vec3 raum = weltraum(d) * durch * (1.0 - smoothstep(0.05, 0.35, l));
    c += gal + raum / (1.0 + 2.0 * dot(gal, vec3(0.333)));
    c += komet(m * vec2(resolution.x / resolution.y, 1.0), vorne, rechts, oben, tf) * (1.0 - smoothstep(0.1, 0.5, l));
    c *= 1.0 - 0.5 * dot(m, m) * 1.6;
    c = c / (1.0 + 0.4 * c);
    c = pow(c, vec3(0.95));
    c = mix(c, vec3(1.0, 0.97, 0.92), clamp(blitz, 0.0, 1.0) * 0.85);
    c = uebergang(c, m * vec2(resolution.x / resolution.y, 1.0));
    finalColor = vec4(c * hell, 1.0);
}
