#version 330
// Kueste im Morgengrauen: Tiefflug ueber das Meer an Klippen entlang, auf
// der hoechsten steht ein Leuchtturm. Sein Strahl kreist im Takt -- eine
// Umdrehung in zwei Takten -- und streift ueber das Wasser und durch den
// Dunst. Unten bricht sich die Brandung an den Felsen. Der Himmel glueht
// schon, die Sonne ist noch nicht da: sie geht erst im Ausklang auf dem
// offenen Meer auf.
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

const vec3 SONNE = vec3(0.2611, 0.0081, 0.9653);   // knapp unter dem Horizont, vorn rechts

// Die Klippen: eine Felswand rechts, die der Kueste folgt, oben ein
// unruhiger Grat. Der Turm steht auf einem Vorsprung.
const vec2 TURM = vec2(7.2, 60.0);   // (x, z)
float kante(float z) { return 4.6 + 1.6 * sin(z * 0.07) + 1.2 * fbm(vec2(z * 0.12, 3.0)) - 2.0 * exp(-pow((z - TURM.y) / 9.0, 2.0)); }
float grat(float z) { return 3.4 + 2.2 * fbm(vec2(z * 0.05, 1.0)) + 1.0 * exp(-pow((z - TURM.y) / 7.0, 2.0)); }

float felsen(vec3 p) {
    float k = kante(p.z) + 0.6 * fbm(p.yz * vec2(0.9, 0.5)) - 0.25 * p.y * 0.2;
    float wand = k - p.x;                 // > 0 vor der Wand (ueber dem Meer)
    float oben = p.y - grat(p.z);         // > 0 ueber dem Grat
    return max(wand, oben) * 0.7;         // < 0 im Fels
}

float turm(vec3 p) {
    vec2 q = p.xz - TURM;
    float r = 0.42 - 0.06 * clamp((p.y - grat(TURM.y)) / 3.0, 0.0, 1.0);
    float zyl = length(q) - r;
    float unten = grat(TURM.y) - 0.3 - p.y;
    float oben = p.y - (grat(TURM.y) + 3.4);
    return max(zyl, max(unten, oben));
}

float szene(vec3 p) { return min(felsen(p), turm(p)); }

vec3 himmel(vec3 d) {
    float h = clamp(d.y, 0.0, 1.0);
    vec3 c = mix(vec3(1.0, 0.55, 0.32), vec3(0.62, 0.36, 0.52), smoothstep(0.0, 0.1, h));
    c = mix(c, vec3(0.10, 0.13, 0.32), smoothstep(0.08, 0.55, h));
    c += vec3(1.0, 0.6, 0.3) * pow(max(dot(d, SONNE), 0.0), 14.0) * 0.7;
    // die letzten Sterne
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 150.0;
    vec2 z = floor(uv), f = fract(uv) - 0.5;
    float st = smoothstep(0.08, 0.0, length(f)) * step(0.988, hash(z)) * smoothstep(0.25, 0.6, h);
    c += vec3(0.8, 0.85, 1.0) * st * 0.6;
    return c;
}

// Strahl des Leuchtturms: Abstand des Blickstrahls zur Achse des Lichtkegels
vec3 strahl(vec3 ro, vec3 d, float tMax, vec3 lampe, vec3 b) {
    // naechste Punkte zwischen Blickstrahl (ro + d t) und Achse (lampe + b u)
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
    return vec3(1.0, 0.93, 0.78) * k * 0.9;
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
    // Kamera: knapp ueber dem Wasser, faehrt an der Kueste entlang auf den
    // Turm zu und dreht am Ende zum offenen Meer
    float f = smoothstep(0.0, 1.0, fort);
    vec3 ro = vec3(mix(-1.0, 1.5, f), 1.4 + 0.15 * sin(time * 0.7), mix(14.0, 46.0, f));
    vec3 ziel = mix(vec3(TURM.x, grat(TURM.y) + 1.5, TURM.y), vec3(-6.0, 1.2, ro.z + 30.0), smoothstep(0.62, 1.0, fort));
    vec3 vorne = normalize(ziel - ro);
    vec3 rechts = normalize(cross(vec3(0.0, 1.0, 0.0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 d = normalize(vorne + uv.x * rechts + uv.y * oben);

    // Felsen und Turm
    float t = 0.1;
    float tFels = -1.0;
    int schritte = int(90.0 * G());
    for (int i = 0; i < 90; i++) {
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
        bool istTurm = turm(p) < felsen(p);
        vec3 grund;
        if (istTurm) {
            // weiss und rot geringelt, oben die Laterne
            float hh = p.y - grat(TURM.y);
            grund = mod(floor(hh * 1.6), 2.0) < 1.0 ? vec3(0.82, 0.8, 0.78) : vec3(0.6, 0.08, 0.06);
            if (hh > 2.75) grund = vec3(1.0, 0.92, 0.7) * 3.0;
        } else {
            // Gesteinsschichten, schraeg verworfen, dazu Flecken und Risse
            float schicht = fbm(vec2(p.y * 2.6 + p.z * 0.12, p.z * 0.25));
            grund = mix(vec3(0.15, 0.11, 0.11), vec3(0.38, 0.30, 0.25), schicht);
            grund *= 0.75 + 0.5 * fbm(p.yz * vec2(3.0, 1.2));
            // raue Oberflaeche: die Normale wackelt mit dem Rauschen
            n = normalize(n + 0.55 * vec3(0.0, rausch(p.yz * 5.0) - 0.5, rausch(p.zy * 5.0 + 3.0) - 0.5));
            // Gras oben
            grund = mix(grund, vec3(0.12, 0.16, 0.08), smoothstep(0.55, 0.85, n.y));
        }
        float licht = max(dot(n, normalize(SONNE + vec3(0.0, 0.25, 0.0))), 0.0);
        vec3 himmelLicht = vec3(0.32, 0.26, 0.42) * (0.5 + 0.5 * n.y);
        c = grund * (vec3(1.0, 0.6, 0.38) * licht * 0.9 + himmelLicht);
        // der Strahl streift die Felsen
        vec3 zuLampe = p - lampe;
        float imKegel = smoothstep(0.92, 0.99, dot(normalize(zuLampe), b));
        c += grund * vec3(1.0, 0.9, 0.7) * imKegel * 1.2;
        c = mix(c, himmel(normalize(vec3(d.x, 0.02, d.z))), 1.0 - exp(-tFels * 0.012));
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
        c = mix(vec3(0.02, 0.04, 0.07), himmel(rd), fres);
        // Gischt, wo das Wasser an die Felsen schlaegt
        float abst = kante(p.z) - p.x;
        float welle = fbm(vec2(p.z * 0.8 + abst * 0.6, time * 1.3 + abst * 2.0));
        float gischt = smoothstep(2.4, 0.0, abst + 0.4) * smoothstep(0.3, 0.7, welle);
        gischt *= 0.75 + 0.5 * kick;
        c = mix(c, vec3(0.95, 0.88, 0.9), clamp(gischt, 0.0, 1.0) * 0.9);
        // der Strahl auf dem Wasser
        vec3 zuLampe = p - lampe;
        float imKegel = smoothstep(0.985, 0.998, dot(normalize(zuLampe), b));
        c += vec3(1.0, 0.9, 0.7) * imKegel * 0.6;
        c = mix(c, himmel(normalize(vec3(d.x, 0.0, d.z))), smoothstep(30.0, 140.0, tWasser));
    } else {
        tTreffer = 1e9;
        c = himmel(d);
    }
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
