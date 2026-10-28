#version 330
// Der Winterwald: ein zugefrorener See, ringsum verschneite Tannen, am
// anderen Ufer steht ein Reh im Mondlicht. Auf dem Eis spiegelt sich der
// Himmel, Diamantstaub glitzert in der Luft. Mit `kristall` (0..1) senkt sich
// die Kamera zum Eis, und Eisblumen wachsen ueber den See: sechsarmige
// Kristalle mit Seitenaesten, jeder mit eigenem Keim und eigener Zeit; jeder
// Melodieton laesst sie aufleuchten.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float takt;
uniform float hell;
uniform float fort;
uniform float kristall;
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
const vec3 SEE = vec3(0.0, 0.0, 26.0);
const float SEE_R = 17.0;
const vec3 MOND = vec3(0.1696, 0.2694, 0.9479);

float kapsel(vec3 p, vec3 a, vec3 b, float r) {
    vec3 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// Kegel mit der Spitze im Ursprung, nach unten offen (nach Inigo Quilez)
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

// Das Reh am anderen Ufer, seitlich stehend; den Kopf hebt und senkt es
float reh(vec3 p) {
    p -= vec3(-1.5, 0.0, 39.5);
    float kopfHub = 0.12 * sin(time * 0.6);
    float d = kapsel(p, vec3(-0.45, 1.0, 0.0), vec3(0.4, 1.05, 0.0), 0.22);
    d = min(d, kapsel(p, vec3(0.4, 1.1, 0.0), vec3(0.62, 1.55 + kopfHub, 0.0), 0.09));
    d = min(d, kapsel(p, vec3(0.62, 1.58 + kopfHub, 0.0), vec3(0.88, 1.46 + kopfHub, 0.0), 0.07));
    // Beine
    d = min(d, kapsel(vec3(p.x, p.y, abs(p.z)), vec3(0.32, 0.9, 0.1), vec3(0.36, 0.0, 0.1), 0.04));
    d = min(d, kapsel(vec3(p.x, p.y, abs(p.z)), vec3(-0.38, 0.9, 0.1), vec3(-0.44, 0.0, 0.1), 0.045));
    // Geweih: zwei Stangen mit Enden
    vec3 g = vec3(p.x, p.y - kopfHub, abs(p.z));
    d = min(d, kapsel(g, vec3(0.6, 1.66, 0.05), vec3(0.52, 2.05, 0.2), 0.022));
    d = min(d, kapsel(g, vec3(0.55, 1.88, 0.14), vec3(0.72, 2.0, 0.18), 0.018));
    d = min(d, kapsel(g, vec3(0.52, 2.05, 0.2), vec3(0.42, 2.2, 0.26), 0.016));
    return d;
}

// Material: 0 Schnee, 1 Eis, 2 Tanne, 3 Stamm, 4 Reh
float mat;
float szene(vec3 p) {
    float rs = length(p.xz - SEE.xz);
    // Gelaende: Schnee, zum See hin abfallend, ausserhalb sanfte Huegel
    float boden = p.y - 0.25 * smoothstep(SEE_R - 0.5, SEE_R + 3.0, rs) - 0.6 * rausch(p.xz * 0.15) * smoothstep(SEE_R, SEE_R + 8.0, rs);
    float d = boden;
    mat = rs < SEE_R ? 1.0 : 0.0;
    // Tannen auf einem Raster, nicht auf dem See und nicht vor der Kamera
    vec2 zelle = floor(p.xz / 3.2);
    vec2 lz = p.xz - (zelle + 0.5) * 3.2;
    vec2 versatz = (vec2(hash(zelle), hash(zelle + 4.7)) - 0.5) * 1.2;
    vec2 mitte = (zelle + 0.5) * 3.2 + versatz;
    float rm = length(mitte - SEE.xz);
    bool frei = rm > SEE_R + 1.2 && !(mitte.y < 15.0 && abs(mitte.x) < 7.0 && mitte.y > -20.0);
    if (frei) {
        float groesse = 0.8 + hash(zelle + 9.1) * 0.6;
        vec3 q = vec3(lz.x - versatz.x, p.y, lz.y - versatz.y) / groesse;
        float tanne = 1e5;
        for (int i = 0; i < 3; i++) {
            float fi = float(i);
            float spitze = 1.9 + fi * 0.95;
            float r = 0.95 - fi * 0.22;
            tanne = min(tanne, sdKegel(q - vec3(0.0, spitze, 0.0), normalize(vec2(r, 1.3)), 1.3));
        }
        tanne *= groesse;
        float stamm = max(length(q.xz) - 0.09, q.y - 1.0) * groesse;
        if (tanne < d) { d = tanne; mat = 2.0; }
        if (stamm < d) { d = stamm; mat = 3.0; }
    }
    float r = reh(p);
    if (r < d) { d = r; mat = 4.0; }
    return d;
}

vec3 normale(vec3 p) {
    vec2 e = vec2(0.003, 0.0);
    return normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
}

vec3 himmel(vec3 d) {
    float h = clamp(d.y, 0.0, 1.0);
    vec3 c = mix(vec3(0.05, 0.08, 0.16), vec3(0.006, 0.01, 0.035), pow(h, 0.55));
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 170.0;
    vec2 z = floor(uv);
    float hz = hash(z);
    c += vec3(0.85, 0.9, 1.0) * step(0.982, hz) * smoothstep(0.35, 0.0, length(fract(uv) - 0.5)) * (0.6 + 0.4 * sin(time * (1.0 + hz * 3.0) + hz * 30.0)) * (1.0 + ton);
    // ein leiser, gruener Schleier (Polarlicht), der mit der Melodie atmet
    float az = atan(d.x, d.z);
    float band = exp(-pow((d.y - 0.28 - 0.06 * sin(az * 3.0 + time * 0.05)) * 9.0, 2.0));
    float strahl = 0.5 + 0.5 * rausch(vec2(az * 30.0, time * 0.2));
    c += mix(vec3(0.1, 0.9, 0.5), vec3(0.4, 0.3, 1.0), smoothstep(0.2, 0.5, d.y)) * band * strahl * 0.12 * (1.0 + 0.6 * ton);
    float m = dot(d, MOND);
    float scheibe = smoothstep(0.99982, 0.9999, m);
    // Mare auf der Scheibe: dunklere Flecken aus Rauschen
    vec2 mq = vec2(d.x - MOND.x, d.y - MOND.y) * 300.0;
    c += vec3(0.98, 0.97, 0.9) * scheibe * (2.4 - 0.7 * rausch(mq * 0.8 + 3.0));
    c += vec3(0.3, 0.36, 0.5) * pow(max(m, 0.0), 90.0);
    c += vec3(0.12, 0.15, 0.24) * pow(max(m, 0.0), 6.0) * 0.5;
    return c;
}

// Eisblumen: um jeden Keim sechs Arme mit Seitenaesten. In den Sektor
// gefaltet genuegt ein Arm mit Aesten nach einer Seite.
float eisblume(vec2 q, float R) {
    float r = length(q);
    if (r > R + 0.05) return 0.0;
    float a = atan(q.y, q.x);
    a = abs(mod(a, PI / 3.0) - PI / 6.0);
    // zurueck aufs Mittel des Sektors: Arm entlang x
    a = PI / 6.0 - a;
    q = r * vec2(cos(a), sin(a));
    float breite = 0.008 + 0.006 * (1.0 - r / max(R, 0.01));
    float d = q.x < R ? abs(q.y) : length(q - vec2(R, 0.0));
    for (int k = 1; k <= 4; k++) {
        float s = R * float(k) / 5.0;
        float l = (R - s) * 0.55;
        vec2 dir = vec2(0.5, 0.866);
        vec2 pa = vec2(q.x - s, abs(q.y));
        float h = clamp(dot(pa, dir), 0.0, l);
        d = min(d, length(pa - dir * h));
    }
    return max(smoothstep(breite, 0.0, d), 0.3 * smoothstep(breite * 6.0, 0.0, d));
}

float eisblumen(vec2 p) {
    if (kristall <= 0.0) return 0.0;
    vec2 zelle = floor(p / 1.6);
    float s = 0.0;
    for (int j = -1; j <= 1; j++) {
        for (int i = -1; i <= 1; i++) {
            vec2 z = zelle + vec2(float(i), float(j));
            vec2 keim = (z + 0.5 + (vec2(hash(z), hash(z + 2.3)) - 0.5) * 0.6) * 1.6;
            float start = hash(z + 7.7) * 0.6;
            float R = clamp((kristall - start) / (1.0 - start), 0.0, 1.0) * (0.55 + hash(z + 1.1) * 0.4);
            vec2 q = p - keim;
            float w = hash(z + 5.5) * PI;
            q = vec2(cos(w) * q.x - sin(w) * q.y, sin(w) * q.x + cos(w) * q.y);
            s = max(s, eisblume(q, R));
        }
    }
    return s;
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

vec3 schattieren(vec3 p, vec3 rd, float m, vec3 n) {
    float mond = max(dot(n, MOND), 0.0);
    vec3 licht = vec3(0.09, 0.12, 0.2) * (0.55 + 0.45 * n.y) + vec3(0.55, 0.62, 0.85) * mond * 0.6;
    // Gegenlicht: Kanten leuchten im Mondlicht
    float kante = pow(1.0 - max(dot(n, -rd), 0.0), 3.0) * max(dot(rd, MOND), 0.0);
    vec3 alb;
    if (m < 0.5) {
        alb = vec3(0.85, 0.9, 1.0);
    } else if (m < 2.5 && m > 1.5) {
        float schnee = smoothstep(0.25, 0.55, n.y + 0.25 * rausch(p.xz * 6.0 + p.y * 3.0));
        alb = mix(vec3(0.02, 0.07, 0.04), vec3(0.8, 0.86, 0.95), schnee);
    } else if (m < 3.5) {
        alb = vec3(0.08, 0.05, 0.03);
    } else {
        alb = vec3(0.18, 0.1, 0.06);
    }
    return alb * licht + vec3(0.7, 0.75, 0.9) * kante * 0.6;
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    // Kamera: am Ufer entlang; zu den Kristallen senkt sie sich und schaut aufs Eis
    float k = kristall * kristall * (3.0 - 2.0 * kristall);
    float x = -3.0 + fort * 6.0;
    vec3 ro = vec3(x, mix(1.5, 0.55, k), mix(6.5, 10.5, k));
    vec3 ta = vec3(x * 0.3, mix(1.6, 0.0, k * 0.8), mix(30.0, 16.0, k));
    vec3 vorne = normalize(ta - ro);
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.3 + uv.x * rechts + uv.y * oben);

    vec3 nebel = vec3(0.07, 0.1, 0.18);
    vec3 c;
    float t = 0.0;
    float hit = -1.0;
    int schritte = int(130.0 * G());
    for (int i = 0; i < 130; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float d = szene(p);
        if (d < 0.001 * t + 0.0008) { hit = t; break; }
        t += d * 0.7;
        if (t > 80.0) break;
    }
    if (hit > 0.0) {
        vec3 p = ro + rd * hit;
        szene(p);
        float m = mat;
        vec3 n = normale(p);
        if (m > 0.5 && m < 1.5) {
            // Eis: spiegelt den Himmel, darunter dunkles Wasser, Risse und
            // duenne Schneeflecken; die Baeume am Ufer spiegeln sich als Saum
            vec2 w = p.xz;
            vec3 nn = normalize(vec3((rausch(w * 2.0) - 0.5) * 0.04, 1.0, (rausch(w * 2.0 + 5.0) - 0.5) * 0.04));
            vec3 rr = reflect(rd, nn);
            vec3 spiegel = himmel(rr);
            float saum = smoothstep(0.06 + 0.04 * rausch(vec2(atan(rr.x, rr.z) * 20.0, 1.0)), 0.02, rr.y);
            spiegel = mix(spiegel, vec3(0.01, 0.02, 0.03), saum);
            float fres = 0.15 + 0.85 * pow(1.0 - max(dot(nn, -rd), 0.0), 4.0);
            c = mix(vec3(0.01, 0.025, 0.045), spiegel, fres);
            float riss = smoothstep(0.03, 0.0, abs(rausch(w * 0.9) - 0.5)) * 0.5;
            c += vec3(0.25, 0.35, 0.5) * riss * 0.3;
            float flecken = smoothstep(0.62, 0.8, rausch(w * 0.6 + 3.0));
            c = mix(c, vec3(0.5, 0.56, 0.7), flecken * 0.4);
            // die Mondbahn: viele kleine Glanzpunkte auf dem Eis
            float bahn = pow(max(dot(normalize(vec3(rr.x, abs(rr.y), rr.z)), MOND), 0.0), 300.0);
            c += vec3(0.9, 0.92, 1.0) * bahn * step(0.6, hash(floor(w * 90.0))) * smoothstep(0.4, 0.0, length(fract(w * 90.0) - 0.5)) * 4.0;
            // Eisblumen, die leuchten, wenn ein Ton erklingt
            float eb = eisblumen(w);
            c += vec3(0.55, 0.85, 1.0) * eb * (0.7 + 1.2 * ton);
            c += vec3(1.0) * eb * step(0.97, hash(floor(w * 80.0) + floor(time * 6.0))) * 1.5;
        } else {
            c = schattieren(p, rd, m, n);
            if (m < 0.5) c += vec3(1.0) * step(0.996, hash(floor(p.xz * 50.0) + floor(time * 5.0))) * 0.5;
        }
        c = mix(c, nebel, 1.0 - exp(-hit * 0.028));
    } else {
        c = himmel(rd);
    }
    // Diamantstaub: Glitzer in der Luft, im Mondlicht dichter
    vec2 g = floor(fragTexCoord * resolution / 3.0);
    float glitz = step(0.9985, hash(g + floor(time * 8.0))) * (0.3 + max(dot(rd, MOND), 0.0));
    c += vec3(0.8, 0.9, 1.0) * glitz * (0.6 + ton);
    c *= 1.0 - 0.35 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.92));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
