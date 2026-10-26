#version 330
// Ozean bei Nacht: Sterne, Mond, Polarlicht ueber ruhigem Wasser, das den
// Himmel spiegelt. `morgen` (0..1) faerbt den Himmel zur Daemmerung (Abspann),
// `licht` (0..1) laesst das Polarlicht aufleben. Text und Zeichnungen der Demo
// liegen in texture0 und werden obenauf gelegt.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;      // Blende: 0 schwarz .. 1 voll
uniform float licht;     // Polarlicht
uniform float morgen;    // Daemmerung
uniform float kick;      // Huellkurve der Bassdrum
uniform float ton;       // Melodie: 1 auf dem Anschlag, klingt ab
uniform float hoehe;     // wie hoch der Melodieton liegt, 0..1
uniform vec3 farbe;      // Farbe des Akkords
uniform float aus;       // Szenenwechsel: Licht waechst aus der Mitte (0..1)
uniform float ein;       // ... und weicht nach aussen (0 = nur Licht, 1 = Szene)
uniform vec3 uebLicht;   // Farbe des Lichts beim Wechsel
out vec4 finalColor;

const vec3 MOND = vec3(-0.4067, 0.2892, 0.8667);   // normalize(-0.45, 0.32, 1.0)
const vec3 SONNE = vec3(0.4472, 0.0179, 0.8943);

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

vec3 sterne(vec3 d) {
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 180.0;
    vec2 z = floor(uv), f = fract(uv) - 0.5;
    float h = hash(z);
    float s = smoothstep(0.08, 0.0, length(f - (vec2(hash(z + 3.1), hash(z + 7.7)) - 0.5) * 0.6));
    float fun = 0.6 + 0.4 * sin(time * (1.0 + h * 3.0) + h * 40.0);
    return vec3(0.8, 0.85, 1.0) * s * step(0.985, h) * fun;
}

// Polarlicht als Vorhaenge: ein Band schlaengelt sich ueber den Himmel; jede
// Schicht in der Hoehe zeichnet es duenn, feine senkrechte Streifen geben die
// Strahlen. Unten gruen, nach oben violett und schwaecher.
vec3 polarlicht(vec3 ro, vec3 d) {
    vec3 c = vec3(0.0);
    if (d.y < 0.01) return c;
    for (int i = 0; i < 32; i++) {
        float k = float(i) / 32.0;
        float hoehe = 10.0 + k * 22.0;
        float t = (hoehe - ro.y) / d.y;
        if (t > 400.0) break;
        vec2 p = (ro + d * t).xz * 0.012;
        float x = p.x + time * 0.004;
        // wo das Band verlaeuft
        float mitte = 0.9 + 0.35 * sin(x * 1.3 + time * 0.03) + 0.4 * (fbm(vec2(x * 0.8, time * 0.02)) - 0.5);
        float abst = (p.y - mitte) * 9.0;
        float band = exp(-abst * abst);
        // Strahlen: Rauschen nur entlang des Bandes, sehr fein
        float strahl = fbm(vec2(x * 22.0, time * 0.15 + k * 0.6));
        strahl = smoothstep(0.35, 0.9, strahl);
        vec3 fl = mix(vec3(0.15, 1.0, 0.55), vec3(0.6, 0.25, 1.0), smoothstep(0.25, 1.0, k));
        fl = mix(fl, farbe, 0.3);
        c += fl * band * strahl * (1.0 - k) * (1.0 - k) * 0.22;
    }
    return c * (0.2 + 1.6 * licht) * (1.0 + 0.9 * ton);
}

vec3 himmel(vec3 ro, vec3 d, bool mitMond) {
    float h = clamp(d.y, 0.0, 1.0);
    vec3 nacht = mix(vec3(0.02, 0.035, 0.08), vec3(0.002, 0.004, 0.015), pow(h, 0.5));
    // Daemmerung: warm am Horizont, darueber Rosa, oben tiefes Blau
    vec3 daemm = mix(vec3(1.0, 0.55, 0.28), vec3(0.55, 0.32, 0.5), smoothstep(0.0, 0.12, h));
    daemm = mix(daemm, vec3(0.08, 0.12, 0.32), smoothstep(0.1, 0.6, h));
    vec3 c = mix(nacht, daemm, morgen);
    c += sterne(d) * (1.0 - morgen) * (1.0 - morgen) * (1.0 + 1.2 * ton);
    c += polarlicht(ro, d) * (1.0 - morgen);
    float m = dot(d, MOND);
    if (mitMond) c += vec3(0.95, 0.95, 1.0) * smoothstep(0.9993, 0.9996, m) * (1.0 - morgen);
    c += vec3(0.22, 0.27, 0.42) * pow(max(m, 0.0), 160.0) * (1.0 - morgen);
    c += vec3(1.0, 0.65, 0.35) * pow(max(dot(d, SONNE), 0.0), 30.0) * morgen * 1.8;
    c += vec3(1.0, 0.85, 0.6) * pow(max(dot(d, SONNE), 0.0), 600.0) * morgen * 3.0;
    return c;
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
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    // Kamera: knapp ueber dem Wasser, gleitet langsam voran, blickt leicht nach oben
    vec3 ro = vec3(time * 0.4, 1.6, time * 1.2);
    vec3 vorne = normalize(vec3(0.06 * sin(time * 0.05), 0.14, 1.0));
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 d = normalize(vorne + uv.x * rechts + uv.y * oben);
    vec3 c;
    if (d.y > 0.0) {
        c = himmel(ro, d, true);
    } else {
        // Wasser: Wellen aus wenigen Sinusbuendeln geben die Normale
        float t = -ro.y / d.y;
        vec3 p = ro + d * t;
        vec2 w = p.xz;
        vec2 g = vec2(0.0);
        for (int i = 0; i < 7; i++) {
            float fi = float(i);
            vec2 r = vec2(cos(fi * 1.7), sin(fi * 1.7));
            float f = 0.6 + fi * 0.6;
            float ph = dot(w, r) * f + time * (0.7 + fi * 0.21);
            g += r * cos(ph) * (0.07 / (1.0 + fi * 0.8));
        }
        // in der Ferne glaetten sich die Wellen (sonst flimmert es)
        g *= exp(-t * 0.02);
        vec3 n = normalize(vec3(-g.x, 1.0, -g.y));
        vec3 rd = reflect(d, n);
        rd.y = abs(rd.y);
        float fres = 0.04 + 0.96 * pow(1.0 - max(dot(n, -d), 0.0), 5.0);
        c = mix(vec3(0.004, 0.012, 0.02), himmel(ro, rd, false), fres);
        // Glitzerbahn: Mond und Sonne spiegeln sich in vielen kleinen Wellen
        float glitz = pow(max(dot(rd, MOND), 0.0), 900.0) * (1.0 - morgen);
        float sglitz = pow(max(dot(rd, SONNE), 0.0), 400.0) * morgen;
        c += vec3(0.85, 0.9, 1.0) * glitz * (3.0 + 4.0 * ton) + vec3(1.0, 0.7, 0.4) * sglitz * 3.0;
        // Nebel zum Horizont
        c = mix(c, himmel(ro, normalize(vec3(d.x, 0.0, d.z)), false), smoothstep(40.0, 160.0, t));
    }
    c *= 1.0 + 0.15 * kick;
    c *= 1.0 - 0.35 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.92));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
