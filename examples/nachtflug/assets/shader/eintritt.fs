#version 330
// Wiedereintritt: wir stuerzen in die Atmosphaere. Am Rand des Bildes stroemt
// gluehendes Plasma nach aussen -- weiss an der Hitze, dann orange, magenta,
// violett -- und glueht mit jedem Schlag der Bassdrum auf; Funken fliegen
// vorbei, das Bild bebt. In der Mitte sieht man durch die flimmernde Luft
// hinab auf das Wolkenmeer im ersten Licht, das naeher kommt. Gegen Ende
// verlischt das Plasma, die Wolken liegen ruhig unter uns.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;      // Huellkurve der Bassdrum
uniform float fort;      // 0..1 durch den Abschnitt
uniform float ton;       // Melodie: 1 auf dem Anschlag, klingt ab
uniform float hoehe;     // wie hoch der Melodieton liegt, 0..1
uniform vec3 farbe;      // Farbe des Akkords
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
float fbm3(vec3 p) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 4; i++) { s += a * rausch3(p); p = p * 2.1 + vec3(1.7, 9.2, 3.3); a *= 0.5; }
    return s;
}

const vec3 SONNE = vec3(0.30, 0.10, 0.95);

// Der Blick nach unten: Himmel ueber einem Wolkenmeer im ersten Licht
vec3 tiefe(vec2 uv, float f) {
    float hoch = mix(40.0, 5.0, f);                 // Hoehe ueber den Wolken
    vec3 ro = vec3(0.0, hoch, time * mix(14.0, 6.0, f));
    float neig = mix(0.55, 0.12, f);                // Blick nach unten
    vec3 vorne = normalize(vec3(0.0, -sin(neig), cos(neig)));
    vec3 rechts = normalize(cross(vec3(0.0, 1.0, 0.0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 d = normalize(vorne + uv.x * rechts + uv.y * oben);
    vec3 s = normalize(SONNE);
    // Himmel: oben tiefes Blau, am Horizont ein gluehendes Band
    float h = d.y;
    vec3 himmel = mix(vec3(1.0, 0.55, 0.3), vec3(0.18, 0.12, 0.32), smoothstep(0.0, 0.25, h));
    himmel = mix(himmel, vec3(0.02, 0.03, 0.09), smoothstep(0.2, 0.7, h));
    himmel += vec3(1.0, 0.7, 0.4) * pow(max(dot(d, s), 0.0), 18.0) * 0.8;
    if (d.y >= -0.005) return himmel;
    float t = -hoch / d.y;
    vec2 p = (ro + d * t).xz * 0.035;
    float w = fbm(p + vec2(0.0, time * 0.05));
    float w2 = fbm(p * 2.7 + 5.0);
    float dicht = smoothstep(0.42, 0.72, w * 0.75 + w2 * 0.35);
    // Licht von der tiefen Sonne: Oberseiten rosa-gold, Taeler violett
    float kuppe = smoothstep(0.3, 0.9, w2);
    vec3 wolke = mix(vec3(0.32, 0.2, 0.46), vec3(1.0, 0.72, 0.56), kuppe) * 1.15;
    vec3 meer = vec3(0.01, 0.025, 0.07);
    vec3 c = mix(meer, wolke, dicht);
    // Dunst zum Horizont
    return mix(c, himmel, smoothstep(0.0, 1.0, 1.0 - exp(-t * 0.0025)));
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
    // wie stark das Plasma gerade ist: waechst, bleibt, verlischt am Ende
    float P = smoothstep(0.0, 0.18, fort) * (1.0 - smoothstep(0.68, 0.94, fort));
    // das Bild bebt mit der Bassdrum
    vec2 beben = vec2(sin(time * 83.0), cos(time * 71.0)) * kick * 0.006 * P;
    vec2 q = uv + beben;
    float r = length(q);
    vec2 dir = q / max(r, 1e-4);

    // flimmernde Luft vor dem Blick nach unten
    vec2 flimmer = vec2(rausch(q * 9.0 + time * 3.0), rausch(q * 9.0 - time * 2.7)) - 0.5;
    vec3 c = tiefe(q + flimmer * 0.018 * P, fort);

    // Plasma: Flammenzungen, die vom Rand her nach aussen stroemen (Rauschen
    // ueber Richtung und Abstand -- ohne Naht, weil die Richtung stetig ist).
    // Der Rand ist eine Ellipse wie das Bild, die Mitte bleibt frei.
    float re = length(vec2(q.x / 0.89, q.y / 0.5));
    float fl = fbm3(vec3(dir * 3.2, r * 1.8 - time * 3.6));
    float zunge = fbm3(vec3(dir * 9.0, r * 4.0 - time * 6.0));
    // der Rand franst aus: wo eine Zunge ist, reicht er weiter hinein
    float rand = smoothstep(0.45, 1.3, re + (fl - 0.5) * 0.7);
    float muster = smoothstep(0.42, 0.8, fl * 0.75 + zunge * 0.45);
    float glut = muster * rand * P;
    glut *= 1.0 + 0.8 * kick + 0.25 * ton;
    // Farbe nach Hitze: innen violett und magenta, dann orange, aussen weiss-gelb
    vec3 heiss = mix(vec3(0.55, 0.08, 0.6), vec3(1.0, 0.15, 0.35), smoothstep(0.05, 0.35, glut));
    heiss = mix(heiss, vec3(1.0, 0.5, 0.08), smoothstep(0.3, 0.65, glut));
    heiss = mix(heiss, vec3(1.0, 0.95, 0.75), smoothstep(0.65, 1.0, glut));
    c = c * (1.0 - 0.6 * glut) + heiss * glut * 1.9;
    // die Hitze ganz aussen, ein Schimmer zwischen den Zungen
    c += vec3(0.9, 0.3, 0.25) * smoothstep(0.9, 1.5, re) * P * (0.18 + 0.3 * kick);

    // Funken: je Winkelstreifen einer, der nach aussen schiesst
    float a = atan(q.y, q.x);
    float streifen = 150.0;
    float id = floor((a + 3.1416) / 6.2832 * streifen);
    float mitteA = (id + 0.5) / streifen * 6.2832 - 3.1416;
    float quer = abs(a - mitteA) * r;
    float tempo = 0.5 + hash1(id) * 1.3;
    float lauf = fract(hash1(id + 7.0) + time * tempo);
    float rs = mix(0.08, 1.3, lauf * lauf);
    float an = step(0.55, hash1(id + floor(time * tempo + hash1(id + 7.0)) * 13.0));
    float funke = exp(-quer * quer * 9.0e4) * exp(-max(rs - r, 0.0) * 30.0) * step(r, rs) * an;
    c += vec3(1.0, 0.75, 0.45) * funke * lauf * P * 1.6;

    c *= 1.0 - 0.25 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.9));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
