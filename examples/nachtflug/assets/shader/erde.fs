#version 330
// Erdaufgang im Orbit: die Erde kommt aus der Ferne naeher -- erst eine
// Kugel vor der Sonne, deren Atmosphaere als Ring von hinten leuchtet, dann
// fuellt ihre Nachtseite mit Stadtlichtern unter den Wolken das Bild. Am Ende
// geht die Sonne ueber dem Erdrand auf. `fort` (0..1) fuehrt durch die Szene.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float fort;      // 0..1 durch den Abschnitt
uniform float ton;       // Melodie: 1 auf dem Anschlag, klingt ab
uniform float hoehe;     // wie hoch der Melodieton liegt, 0..1
uniform vec3 farbe;      // Farbe des Akkords
uniform float aus;       // Szenenwechsel: Licht waechst aus der Mitte (0..1)
uniform float ein;       // ... und weicht nach aussen (0 = nur Licht, 1 = Szene)
uniform vec3 uebLicht;   // Farbe des Lichts beim Wechsel
uniform float guete;     // 1 = volle Qualitaet, kleiner = weniger Schritte (--leicht)
out vec4 finalColor;

float G() { return guete > 0.0 ? guete : 1.0; }

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
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
float fbm3(vec3 p, int n) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 6; i++) {
        if (i >= n) break;
        s += a * rausch3(p); p = p * 2.07 + vec3(1.7, 9.2, 3.3); a *= 0.5;
    }
    return s;
}

// Sterne und ein blasses Band der Milchstrasse
vec3 sterne(vec3 d) {
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 160.0;
    vec2 z = floor(uv), f = fract(uv) - 0.5;
    float h = hash(z);
    float s = smoothstep(0.09, 0.0, length(f - (vec2(hash(z + 3.1), hash(z + 7.7)) - 0.5) * 0.6));
    float fun = 0.6 + 0.4 * sin(time * (1.0 + h * 3.0) + h * 40.0);
    vec3 c = mix(vec3(0.75, 0.82, 1.0), vec3(1.0, 0.85, 0.7), hash(z + 1.3)) * s * step(0.975, h) * fun;
    float band = exp(-pow(dot(d, normalize(vec3(0.3, 0.8, -0.5))) * 6.0, 2.0));
    c += vec3(0.05, 0.045, 0.08) * band * smoothstep(0.45, 0.8, fbm3(d * 9.0, 4));
    return c;
}

// Strahl gegen Kugel (Mitte 0, Radius r): Abstand oder -1
float kugel(vec3 ro, vec3 d, float r) {
    float b = dot(ro, d);
    float c = dot(ro, ro) - r * r;
    float h = b * b - c;
    if (h < 0.0) return -1.0;
    float t = -b - sqrt(h);
    return t > 0.0 ? t : -1.0;
}

// Die Oberflaeche an der Stelle n (Normale), Sonne s
vec3 erde(vec3 n, vec3 d, vec3 s) {
    // die Erde dreht sich langsam
    float w = time * 0.012;
    vec3 q = vec3(n.x * cos(w) - n.z * sin(w), n.y, n.x * sin(w) + n.z * cos(w));
    int okt = int(mix(3.0, 6.0, G()));
    float land = smoothstep(0.5, 0.535, fbm3(q * 1.7 + 3.1, okt));
    float wolke = smoothstep(0.48, 0.78, fbm3(q * 3.4 + vec3(time * 0.004, 0.0, 0.0), okt));
    float ndl = dot(n, s);
    float tag = max(ndl, 0.0);
    // Tag: Meer, Land, Wolken, Glanz der Sonne auf dem Wasser
    vec3 meer = vec3(0.01, 0.05, 0.14);
    vec3 boden = mix(vec3(0.07, 0.12, 0.05), vec3(0.28, 0.22, 0.13), fbm3(q * 7.0, 3));
    vec3 c = mix(meer, boden, land) * tag * 1.7;
    vec3 r = reflect(d, n);
    c += vec3(1.0, 0.8, 0.55) * pow(max(dot(r, s), 0.0), 60.0) * (1.0 - land) * tag * 1.5;
    c = mix(c, vec3(1.0) * (tag * 1.25 + 0.015), wolke * 0.85);
    // Daemmerungsstreifen an der Grenze zur Nacht
    float daemm = exp(-ndl * ndl * 70.0) * smoothstep(-0.25, 0.05, ndl);
    c += vec3(1.0, 0.42, 0.18) * daemm * (0.25 + 0.4 * wolke);
    // Nacht: Lichter der Staedte -- Ballungen (grobes Rauschen) aus vielen
    // kleinen Punkten (feines Raster), die meisten nahe der Kueste
    float nacht = smoothstep(0.08, -0.12, ndl);
    float kueste = 1.0 - smoothstep(0.0, 0.35, abs(fbm3(q * 1.7 + 3.1, okt) - 0.52) * 6.0);
    float ballung = smoothstep(0.45, 0.75, rausch3(q * 9.0) * 0.7 + kueste * 0.5);
    vec3 zelle = floor(q * 220.0);
    float punkt = step(0.82, hash3(zelle)) * smoothstep(0.5, 0.0, length(fract(q * 220.0) - 0.5));
    float lichter = ballung * (punkt + 0.18 * ballung * ballung) * land * (1.0 - wolke * 0.8);
    float funkeln = 0.75 + 0.25 * sin(time * 3.0 + hash3(zelle) * 30.0);
    c += vec3(1.0, 0.7, 0.35) * lichter * nacht * funkeln * (1.3 + 0.8 * ton);
    // Mondlicht: die Wolken und ein Hauch auf dem Meer der Nachtseite
    c += (vec3(0.22, 0.28, 0.45) * wolke * 0.16 + vec3(0.01, 0.02, 0.05) * (1.0 - land)) * nacht;
    return c;
}

// Die Atmosphaere: ein duenner leuchtender Rand. Wo die Sonne dahinter steht,
// streut er sie nach vorn -- dort gluehen Orange und Gold, sonst Blau.
vec3 atmosphaere(vec3 ro, vec3 d, vec3 s, float tHit) {
    float tca = -dot(ro, d);
    vec3 pm = ro + d * max(tca, 0.0);
    float h = length(pm);
    if (tHit > 0.0) {
        // Strahl trifft die Erde: Dunst nur am Rand der Scheibe
        vec3 p = ro + d * tHit;
        vec3 n = normalize(p);
        float rand = pow(1.0 - max(dot(n, -d), 0.0), 4.0);
        float lit = smoothstep(-0.3, 0.3, dot(n, s));
        return mix(vec3(0.9, 0.35, 0.15), vec3(0.25, 0.5, 1.0), lit) * rand * (0.25 + 0.6 * lit);
    }
    if (h > 1.09) return vec3(0.0);
    float rand = exp(-max(h - 1.0, 0.0) / 0.016);
    vec3 nm = pm / max(h, 1e-4);
    float lit = dot(nm, s);
    float vorn = pow(max(dot(d, s), 0.0), 10.0);
    vec3 blau = vec3(0.28, 0.55, 1.0);
    vec3 glut = vec3(1.0, 0.5, 0.18);
    vec3 col = mix(glut, blau, smoothstep(-0.05, 0.45, lit));
    col = mix(col, farbe, 0.12);
    return col * rand * (0.12 + 1.2 * smoothstep(-0.4, 0.3, lit) + 5.0 * vorn);
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
    float f = smoothstep(0.0, 1.0, fort);
    // Kamera: kommt von weit her naeher und hebt den Blick, bis der Erdrand
    // im oberen Drittel liegt
    float dist = mix(4.6, 1.33, f);
    float rand = asin(1.0 / dist);          // halber Sehwinkel der Erde
    float neig = rand - 0.17;
    vec3 ro = vec3(0.0, 0.0, -dist);
    vec3 vorne = normalize(vec3(0.04 * sin(time * 0.07), sin(neig), cos(neig)));
    vec3 rechts = normalize(cross(vec3(0.0, 1.0, 0.0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 d = normalize(vorne + uv.x * rechts + uv.y * oben);
    // Die Sonne steht hinter der Erde und knapp unter ihrem Rand -- bis sie
    // am Ende darueber aufgeht
    float hebt = smoothstep(0.80, 0.97, fort);
    float sw = rand - mix(0.10, -0.07, hebt);
    vec3 s = normalize(vec3(0.12, sin(sw), cos(sw)));

    float t = kugel(ro, d, 1.0);
    vec3 c;
    if (t > 0.0) {
        vec3 n = normalize(ro + d * t);
        c = erde(n, d, s);
    } else {
        c = sterne(d) * (1.0 + 1.0 * ton) * (1.0 - 0.8 * hebt);
        // die Sonne, wenn die Erde sie nicht verdeckt
        float verdeckt = kugel(ro, s, 1.0) > 0.0 ? 0.0 : 1.0;
        float sd = max(dot(d, s), 0.0);
        c += vec3(1.0, 0.92, 0.8) * pow(sd, 3000.0) * 30.0 * verdeckt;
        c += vec3(1.0, 0.65, 0.35) * pow(sd, 40.0) * (0.15 + 0.85 * verdeckt) * (0.4 + 0.6 * hebt);
    }
    c += atmosphaere(ro, d, s, t);
    // Lichtstreif durch die Linse, wenn die Sonne hervorkommt
    float sz = dot(s, vorne);
    if (sz > 0.0) {
        vec2 sp = vec2(dot(s, rechts), dot(s, oben)) / sz;
        float frei = kugel(ro, s, 1.0) > 0.0 ? 0.0 : 1.0;
        float streif = exp(-abs(uv.y - sp.y) * 140.0) * exp(-abs(uv.x - sp.x) * 1.4);
        c += vec3(1.0, 0.72, 0.45) * streif * hebt * frei * 1.4;
    }
    c *= 1.0 - 0.3 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.01;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.9));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
