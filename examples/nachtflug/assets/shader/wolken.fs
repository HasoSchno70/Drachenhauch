#version 330
// Ueber den Wolken: Gleitflug ueber ein aufgerissenes Wolkenmeer im ersten
// Morgenlicht. Die Wolken sind ein Volumen: eine Schicht aus Dichte (3D-
// Rauschen, darunter grobe Felder, die bestimmen, wo ueberhaupt Wolken
// sind), die der Strahl Schritt fuer Schritt durchquert. Je Schritt fragt er
// ein Stueck zur Sonne hin, wie viel Wolke dazwischen liegt -- so leuchten
// die Oberseiten gold, die Unterseiten bleiben violett, und duenne Raender
// scheinen durch. Durch die Luecken sieht man tief unten das Meer
// glitzern; einzelne Wolkentuerme ragen heraus. Der Himmel geht von Violett
// ueber Rosa nach Gold, die Wolken atmen mit der Bassdrum.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
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
float rausch(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}
float fbm2(vec2 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 4; i++) { s += a * rausch(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; }
    return s;
}
float hash3(vec3 p) { return fract(sin(dot(p, vec3(127.1, 311.7, 74.7))) * 43758.5453); }
float rausch3(vec3 p) {
    vec3 i = floor(p), f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = mix(hash3(i), hash3(i + vec3(1, 0, 0)), f.x);
    float b = mix(hash3(i + vec3(0, 1, 0)), hash3(i + vec3(1, 1, 0)), f.x);
    float c = mix(hash3(i + vec3(0, 0, 1)), hash3(i + vec3(1, 0, 1)), f.x);
    float d = mix(hash3(i + vec3(0, 1, 1)), hash3(i + vec3(1, 1, 1)), f.x);
    return mix(mix(a, b, f.y), mix(c, d, f.y), f.z);
}

const float UNTEN = 0.0;     // Unterseite der Wolkenschicht
const float OBEN = 9.0;      // hoechste Tuerme
const float MEER = -40.0;

vec3 sonneRichtung() {
    float hebt = mix(0.06, 0.16, fort);
    return normalize(vec3(0.42, hebt, 1.0));
}

// Wo Wolken sind (0 = Luecke, 1 = dicht) und wie hoch sie dort reichen
float feld(vec2 xz) {
    return smoothstep(0.44, 0.6, fbm2(xz * 0.026 + vec2(3.0, time * 0.004)));
}

// Dichte der Wolken am Punkt p; `fein` = Zahl der Rausch-Lagen
float dichte(vec3 p, int fein) {
    if (p.y < UNTEN || p.y > OBEN) return 0.0;
    float f = feld(p.xz);
    if (f < 0.01) return 0.0;
    // die Oberseite: je dichter das Feld, desto hoeher, dazu einzelne Tuerme
    float turm = smoothstep(0.62, 0.8, fbm2(p.xz * 0.05 + 7.0)) * 4.5;
    float dach = 1.5 + 2.6 * f + turm * f;
    float profil = smoothstep(UNTEN, UNTEN + 1.2, p.y) * smoothstep(dach, dach - 1.8, p.y);
    vec3 q = p * 0.32 + vec3(time * 0.05, 0.0, 0.0);
    float n = 0.0, a = 0.5;
    for (int i = 0; i < 4; i++) {
        if (i >= fein) break;
        n += a * rausch3(q);
        q = q * 2.2 + vec3(1.7, 9.2, 3.3);
        a *= 0.5;
    }
    return clamp((n - 0.46 + f * 0.25) * 3.5 * profil, 0.0, 1.0);
}

vec3 himmel(vec3 d, vec3 s) {
    float h = clamp(d.y, -0.2, 1.0);
    float f = fort;
    // von Violett ueber Rosa nach Gold
    vec3 horiz = mix(mix(vec3(0.85, 0.42, 0.55), vec3(1.0, 0.58, 0.42), smoothstep(0.0, 0.5, f)),
                     vec3(1.0, 0.78, 0.45), smoothstep(0.5, 1.0, f));
    vec3 zenit = mix(vec3(0.10, 0.08, 0.28), vec3(0.16, 0.30, 0.66), f);
    vec3 c = mix(horiz, zenit, smoothstep(0.0, 0.45, h));
    float sd = max(dot(d, s), 0.0);
    c += vec3(1.0, 0.75, 0.45) * pow(sd, 12.0) * 0.55;
    c += vec3(1.0, 0.9, 0.7) * pow(sd, 400.0) * 2.5;
    c += vec3(1.0, 0.97, 0.9) * smoothstep(0.9993, 0.9997, sd) * 6.0 * (1.0 + 0.4 * ton);
    return c;
}

// --- Wildgaense ------------------------------------------------------------------
// Ein Schwarm in V-Form ueberholt die Kamera und zieht voraus, der Sonne
// entgegen, in Hoehe der Wolkentuerme -- man sieht ihn schraeg von oben und
// hinten, das V mit der Spitze voran, die Fluegel weit ausgebreitet. Jede
// Gans ist aus wenigen Strecken in der Welt gebaut (Rumpf, Hals, zwei
// Fluegel mit Knick) und ins Bild projiziert; die Fluegel schlagen auf und
// ab, jede ein wenig spaeter als die vor ihr. Zurueck kommt die Maske der
// Gans am Bildpunkt und ihr Abstand -- damit Wolken davor sie verdecken und
// sie die Wolken dahinter.
float strecke(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// Weltpunkt -> Bild (xy) und Abstand vor der Kamera (z)
vec3 insBild(vec3 w, vec3 ro, vec3 vorne, vec3 rechts, vec3 oben) {
    vec3 zu = w - ro;
    float z = dot(zu, vorne);
    return vec3(vec2(dot(zu, rechts), dot(zu, oben)) / max(z, 0.01), z);
}

float gaense(vec2 uv, vec3 ro, vec3 vorne, vec3 rechts, vec3 oben, out float tiefe) {
    tiefe = 1e9;
    float u = clamp((fort - 0.06) / 0.9, 0.0, 1.0);
    if (u <= 0.0 || u >= 1.0) return 0.0;
    vec3 F = normalize(vec3(0.42, 0.03, 1.0));      // Flugrichtung: zur Sonne
    vec3 S = normalize(vec3(F.z, 0.0, -F.x));       // quer dazu
    vec3 H = cross(F, S) * -1.0;                    // nach oben
    float vor = mix(24.0, 90.0, u * u * (3.0 - 2.0 * u));
    vec3 spitze = vec3(ro.x, 0.0, ro.z) + F * vor + vec3(0.0, mix(5.6, 7.6, u), 0.0);
    float maske = 0.0;
    float px = 1.0 / resolution.y;
    for (int i = 0; i < 13; i++) {
        float fi = float(i);
        // 0 = die Spitze, dann abwechselnd linker und rechter Arm
        float k = floor((fi + 1.0) * 0.5);
        float seite = mod(fi, 2.0) < 0.5 ? 1.0 : -1.0;
        // der rechte Arm ist kuerzer -- wie meist bei einem echten Zug
        if (seite < 0.0 && k > 4.0) continue;
        vec3 g = spitze - F * k * 2.7 + S * seite * k * 1.5;
        g += H * 0.25 * sin(time * 0.9 + fi * 1.3) + S * 0.15 * sin(time * 0.7 + fi * 2.1);
        vec3 m = insBild(g, ro, vorne, rechts, oben);
        if (m.z < 1.5) continue;
        float gr = 1.3 / m.z;                       // ungefaehre Groesse im Bild
        if (length(uv - m.xy) > gr * 1.6) continue;
        float schlag = sin(time * 5.0 - k * 0.55 + seite * 0.25);
        float dick = 1.0 / m.z;                     // 1 Welteinheit im Bild
        // Rumpf und Hals, vorn der Kopf
        vec3 a = insBild(g - F * 0.55, ro, vorne, rechts, oben);
        vec3 b = insBild(g + F * 0.25, ro, vorne, rechts, oben);
        vec3 c = insBild(g + F * 0.8 + H * 0.05, ro, vorne, rechts, oben);
        float form = strecke(uv, a.xy, b.xy) - 0.16 * dick;
        form = min(form, strecke(uv, b.xy, c.xy) - 0.06 * dick);
        form = min(form, length(uv - c.xy) - 0.09 * dick);
        // zwei Fluegel mit Knick: der innere Teil hebt sich weniger als die Spitze
        for (int w = 0; w < 2; w++) {
            float sw = w == 0 ? 1.0 : -1.0;
            vec3 knick = insBild(g + S * sw * 0.6 + H * schlag * 0.25 - F * 0.05, ro, vorne, rechts, oben);
            vec3 tip = insBild(g + S * sw * 1.15 + H * schlag * 0.65 - F * 0.3, ro, vorne, rechts, oben);
            form = min(form, strecke(uv, b.xy * 0.6 + a.xy * 0.4, knick.xy) - 0.11 * dick);
            form = min(form, strecke(uv, knick.xy, tip.xy) - 0.06 * dick);
        }
        float mk = smoothstep(px * 1.2, 0.0, form);
        if (mk > maske) { maske = mk; tiefe = length(g - ro); }
    }
    return maske;
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
    vec3 s = sonneRichtung();
    // Kamera: gleitet ueber der Schicht, wiegt sich sanft
    vec3 ro = vec3(sin(time * 0.11) * 4.0, 11.5 + 0.6 * sin(time * 0.23), time * 5.0);
    float roll = 0.05 * sin(time * 0.17);
    vec3 vorne = normalize(vec3(0.08 * sin(time * 0.09), -0.2, 1.0));
    vec3 rechts = normalize(cross(vec3(sin(roll), cos(roll), 0.0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 d = normalize(vorne + uv.x * rechts + uv.y * oben);

    vec3 hm = himmel(d, s);
    // Hintergrund: unter der Schicht das Meer, sonst der Himmel
    vec3 grund = hm;
    if (d.y < 0.0) {
        float tm = (MEER - ro.y) / d.y;
        vec3 p = ro + d * tm;
        vec2 w = p.xz * 0.3;
        vec3 n = normalize(vec3(rausch(w + time * 0.3) - 0.5, 5.0, rausch(w.yx - time * 0.25) - 0.5));
        vec3 r = reflect(d, n);
        vec3 meer = vec3(0.03, 0.10, 0.24) + vec3(1.0, 0.8, 0.55) * pow(max(dot(r, s), 0.0), 50.0) * 1.8;
        // die Wolken werfen ihre Schatten hinunter
        float schatten = feld(p.xz + s.xz / s.y * (UNTEN - MEER) * 0.15);
        meer *= 1.0 - 0.55 * schatten;
        vec3 schleier = mix(vec3(0.40, 0.36, 0.62), vec3(0.52, 0.60, 0.80), fort);
        grund = mix(meer, schleier, 0.22);
        grund = mix(grund, hm, 1.0 - exp(-tm * 0.0025));
    }

    // die Gaense: dunkle Silhouetten, im Gegenlicht mit warmem Saum, in der
    // Ferne vom Dunst aufgehellt
    float gTiefe;
    float gMaske = gaense(uv, ro, vorne, rechts, oben, gTiefe);
    vec3 gFarbe = mix(vec3(0.09, 0.07, 0.1), vec3(0.55, 0.35, 0.3), 0.25 * pow(max(dot(d, s), 0.0), 3.0));
    gFarbe = mix(gFarbe, hm, 1.0 - exp(-gTiefe * 0.006));
    bool gDa = gMaske <= 0.0;

    // durch die Schicht: von oben bis unten (oder bis zur Sichtweite)
    vec3 licht = vec3(0.0);
    float durch = 1.0;          // wie viel vom Hintergrund noch durchkommt
    if (d.y < 0.0) {
        float t0 = max((OBEN - ro.y) / d.y, 0.0);
        float t1 = min((UNTEN - ro.y) / d.y, 160.0);
        float t = t0 + 0.25 * hash(fragTexCoord * resolution + fract(time));
        int schritte = int(80.0 * G());
        vec3 tal = mix(vec3(0.20, 0.13, 0.34), vec3(0.26, 0.28, 0.50), fort);
        vec3 sonnig = mix(vec3(1.0, 0.58, 0.40), vec3(1.0, 0.82, 0.55), fort) * 2.2;
        vec3 himmelOben = himmel(vec3(0.0, 1.0, 0.0), s);
        for (int i = 0; i < 80; i++) {
            if (i >= schritte || t > t1 || durch < 0.03) break;
            float dt = 0.2 + t * 0.018;
            // eine Gans vor dieser Stelle legt sich ueber das, was dahinter kommt
            if (!gDa && t > gTiefe) {
                licht += durch * gMaske * gFarbe;
                durch *= 1.0 - gMaske;
                gDa = true;
            }
            vec3 p = ro + d * t;
            float dd = dichte(p, 4);
            if (dd > 0.002) {
                // wie viel Wolke zwischen hier und der Sonne liegt
                float zurSonne = dichte(p + s * 0.8, 2) + dichte(p + s * 2.2, 2) * 0.7;
                float sonnenLicht = exp(-zurSonne * 3.2);
                // oben mehr Himmelslicht als unten
                float hoch = clamp((p.y - UNTEN) / 6.0, 0.0, 1.0);
                vec3 farbeHier = tal * (0.55 + 0.45 * hoch) + himmelOben * 0.12 * hoch
                    + sonnig * sonnenLicht * (0.45 + 0.55 * pow(max(dot(d, s), 0.0), 4.0));
                farbeHier *= 1.0 + 0.08 * kick;
                // Dunst: was fern ist, verschwimmt mit dem Himmel
                farbeHier = mix(farbeHier, hm, 1.0 - exp(-t * 0.006));
                float a = 1.0 - exp(-dd * dt * 1.6);
                licht += durch * a * farbeHier;
                durch *= 1.0 - a;
            }
            t += dt;
        }
    }
    if (!gDa) {
        licht += durch * gMaske * gFarbe;
        durch *= 1.0 - gMaske;
    }
    vec3 c = licht + grund * durch;

    // Lichtstrahlen von der Sonne her (im Bild, um ihren Ort), nur am Himmel
    float sz = dot(s, vorne);
    if (sz > 0.0) {
        vec2 sp = vec2(dot(s, rechts), dot(s, oben)) / sz;
        vec2 v = uv - sp;
        float w = atan(v.y, v.x);
        float strahlen = pow(rausch(vec2(w * 9.0, time * 0.15)), 3.0) * exp(-length(v) * 1.6);
        c += vec3(1.0, 0.82, 0.55) * strahlen * 0.3 * durch;
        c += vec3(1.0, 0.75, 0.5) * exp(-length(v) * 3.5) * 0.25 * (1.0 + ton);
    }
    c = mix(c, c * mix(vec3(1.0), farbe * 1.3, 0.12), 0.6);
    c *= 1.0 - 0.28 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.01;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.95));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
