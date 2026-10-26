#version 330
// Tiefflug durch ein Tal bei Mondlicht: Grate aus Rauschen, Schnee auf den
// Kaemmen, Nebel in der Tiefe und unten ein Fluss aus Licht. Auf jeden
// Schlag der Bassdrum laeuft eine Lichtwelle den Fluss entlang voraus.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
uniform float takt;
uniform float fort;      // 0..1 durch den Abschnitt
out vec4 finalColor;

const vec3 MOND = vec3(-0.4924, 0.4924, 0.7176);

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float rausch(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}
const mat2 DREH = mat2(0.8, -0.6, 0.6, 0.8);
float talMitte(float z) { return 4.0 * sin(z * 0.045) + 2.0 * sin(z * 0.017 + 1.0); }
float gelaendeN(vec2 p, int stufen) {
    float h = 0.0, a = 1.0;
    vec2 q = p * 0.07;
    for (int i = 0; i < 6; i++) {
        if (i >= stufen) break;
        float n = 1.0 - abs(rausch(q) * 2.0 - 1.0);
        h += a * n * n;
        q = DREH * q * 2.03;
        a *= 0.47;
    }
    float tal = smoothstep(1.0, 11.0, abs(p.x - talMitte(p.y)));
    return h * 7.0 * (0.12 + 0.88 * tal) - 1.2;
}
float gelaende(vec2 p) { return gelaendeN(p, 6); }

vec3 normale(vec2 p, float t) {
    float e = 0.015 + t * 0.0015;
    return normalize(vec3(gelaende(p - vec2(e, 0)) - gelaende(p + vec2(e, 0)), 2.0 * e,
                          gelaende(p - vec2(0, e)) - gelaende(p + vec2(0, e))));
}

float fbm2(vec2 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 4; i++) { s += a * rausch(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; }
    return s;
}
// dasselbe Polarlicht wie ueber dem Ozean, nur mit weniger Schichten
vec3 polarlicht(vec3 ro, vec3 d) {
    vec3 c = vec3(0.0);
    if (d.y < 0.02) return c;
    for (int i = 0; i < 20; i++) {
        float k = float(i) / 20.0;
        float hoehe = 30.0 + k * 30.0;
        float t = (hoehe - ro.y) / d.y;
        vec2 p = (ro + d * t).xz * 0.008;
        float x = p.x + time * 0.004;
        float mitte = p.y * 0.0 + 0.6 * sin(x * 1.5 + time * 0.04) + 0.5 * (fbm2(vec2(x * 0.8, time * 0.02)) - 0.5) + (ro.z * 0.008 + 1.2);
        float abst = (p.y - mitte) * 7.0;
        float band = exp(-abst * abst);
        float strahl = smoothstep(0.35, 0.9, fbm2(vec2(x * 20.0, time * 0.15 + k * 0.6)));
        vec3 farbe = mix(vec3(0.15, 1.0, 0.55), vec3(0.6, 0.25, 1.0), smoothstep(0.25, 1.0, k));
        c += farbe * band * strahl * (1.0 - k) * (1.0 - k) * 0.3;
    }
    return c;
}

vec3 himmel(vec3 d) {
    vec3 c = mix(vec3(0.05, 0.07, 0.16), vec3(0.008, 0.012, 0.04), pow(max(d.y, 0.0), 0.45));
    c += vec3(0.65, 0.7, 0.85) * smoothstep(0.9990, 0.9994, dot(d, MOND));
    c += vec3(0.18, 0.22, 0.35) * pow(max(dot(d, MOND), 0.0), 10.0);
    // Sterne
    vec2 s = d.xy / max(d.z, 0.1) * 300.0;
    float h = hash(floor(s));
    c += vec3(0.7, 0.75, 0.9) * step(0.996, h) * smoothstep(0.5, 0.0, length(fract(s) - 0.5)) * smoothstep(0.05, 0.3, d.y);
    return c;
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    float z = time * 7.0;
    vec3 ro = vec3(talMitte(z), 0.0, z);
    ro.y = max(gelaendeN(ro.xz, 3), gelaendeN(ro.xz + vec2(0, 5), 3)) + 2.2;
    ro.y = max(ro.y, 1.6);
    vec3 ziel = vec3(talMitte(z + 16.0), ro.y - 0.5, z + 16.0);
    vec3 vorne = normalize(ziel - ro);
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    float neig = -0.2 * (talMitte(z + 8.0) - talMitte(z)) * 0.3;
    vec2 uvn = mat2(cos(neig), -sin(neig), sin(neig), cos(neig)) * uv;
    vec3 d = normalize(vorne * 1.4 + uvn.x * rechts + uvn.y * oben);

    // Raymarching: Hoehenfeld; nach dem Treffer das Stueck halbieren, bis die
    // Grenze genau sitzt (sonst Stufen an den Haengen)
    float t = 0.5, tAlt = 0.5;
    bool treffer = false;
    for (int i = 0; i < 160; i++) {
        vec3 p = ro + d * t;
        float h = p.y - gelaende(p.xz);
        if (h < 0.0) { treffer = true; break; }
        tAlt = t;
        t += max(h * 0.32, 0.015 + t * 0.003);
        if (t > 170.0) break;
    }
    if (treffer) {
        for (int i = 0; i < 6; i++) {
            float m = 0.5 * (tAlt + t);
            vec3 p = ro + d * m;
            if (p.y - gelaende(p.xz) < 0.0) t = m; else tAlt = m;
        }
    }
    vec3 c = himmel(d) + polarlicht(ro, d) * (0.6 + 0.6 * fort);
    if (treffer) {
        vec3 p = ro + d * t;
        vec3 n = normale(p.xz, t);
        float lam = max(dot(n, MOND), 0.0);
        float rand = pow(1.0 - max(dot(n, -d), 0.0), 3.0);
        vec3 fels = mix(vec3(0.07, 0.07, 0.1), vec3(0.2, 0.21, 0.27), smoothstep(0.55, 0.9, n.y));
        fels = mix(fels, vec3(0.75, 0.8, 0.95), smoothstep(3.2, 4.6, p.y) * smoothstep(0.5, 0.8, n.y));
        c = fels * (0.12 + 1.3 * lam) + vec3(0.15, 0.2, 0.35) * rand * 0.35;
        // der Fluss aus Licht im Talgrund
        float abst = abs(p.x - talMitte(p.z));
        float fluss = exp(-abst * abst * 2.5) * smoothstep(0.4, -1.2, p.y);
        float ufer = exp(-abst * abst * 0.12) * smoothstep(1.5, -1.2, p.y);
        // Wellen: auf jede Viertel eine, laeuft vom Betrachter weg
        float viertel = fract(takt * 4.0);
        float vor = p.z - ro.z;
        float welle = exp(-pow((vor - 6.0 - viertel * 70.0) * 0.35, 2.0));
        vec3 glut = mix(vec3(0.1, 0.85, 1.0), vec3(0.4, 1.0, 0.7), 0.5 + 0.5 * sin(p.z * 0.02));
        float staerke = 1.4 + 1.0 * fort + welle * (4.0 + 5.0 * kick);
        c += glut * fluss * staerke;
        // das Licht faellt auf die Ufer
        c += glut * ufer * 0.08 * staerke * max(n.y, 0.0);
        // Nebel: bläulich, dichter unten; vom Mond aufgehellt
        float nebel = 1.0 - exp(-t * 0.016);
        float tief = smoothstep(1.2, -1.0, p.y) * 0.3;
        vec3 nebelFarbe = vec3(0.07, 0.09, 0.18) + vec3(0.1, 0.12, 0.2) * pow(max(dot(d, MOND), 0.0), 4.0);
        c = mix(c, nebelFarbe, clamp(nebel + tief, 0.0, 1.0));
        // der Fluss scheint durch den Nebel
        c += glut * (fluss * 0.6 + ufer * 0.12) * staerke * 0.35 * exp(-t * 0.012);
    }
    c *= 1.0 - 0.38 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.9));
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
