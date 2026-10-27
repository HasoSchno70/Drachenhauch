#version 330
// Ueber den Wolken: Gleitflug ueber ein Wolkenmeer im ersten Morgenlicht.
// Die Wolkendecke ist ein Hoehenfeld aus Rauschen (wie die Berge im Tiefflug,
// nur weicher), beleuchtet von der Sonne, die knapp ueber dem Horizont
// steht: Oberseiten gold, Taeler violett, und wo die Sonne hinter einer
// Kuppe steht, ein silberner Rand. Der Himmel geht ueber den Abschnitt von
// Violett ueber Rosa nach Gold. Ein Schwarm Lichtstrahlen faellt durch die
// Luecken, die Wolken atmen mit der Bassdrum.
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
// Wolkenkuppen: Rauschen, dessen Spitzen gerundet und verstaerkt werden --
// grosse Ballen, darauf kleinere Buckel, kaum feine Kraeusel
float kuppen(vec2 p, int n) {
    float s = 0.0, a = 0.5;
    for (int i = 0; i < 6; i++) {
        if (i >= n) break;
        s += a * rausch(p);
        p = p * 2.1 + vec2(3.1, 1.7);
        a *= 0.46;
    }
    return s;
}

vec3 sonneRichtung() {
    float hebt = mix(0.035, 0.12, fort);
    return normalize(vec3(0.42, hebt, 1.0));
}

// Hoehe der Wolkendecke an (x, z); sie treibt langsam
float decke(vec2 xz, int n) {
    vec2 p = xz * 0.075 + vec2(time * 0.012, 0.0);
    float k = kuppen(p, n);
    return 4.2 * k * k - 0.4;
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
    // Kamera: gleitet knapp ueber der Decke, wiegt sich sanft
    float z = time * 4.5;
    vec3 ro = vec3(sin(time * 0.11) * 3.0, 4.6 + 0.4 * sin(time * 0.23), z);
    float roll = 0.05 * sin(time * 0.17);
    vec3 vorne = normalize(vec3(0.08 * sin(time * 0.09), -0.11, 1.0));
    vec3 rechts = normalize(cross(vec3(sin(roll), cos(roll), 0.0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 d = normalize(vorne + uv.x * rechts + uv.y * oben);

    int okt = int(mix(3.0, 6.0, G()));
    int schritte = int(110.0 * G());
    float t = 0.05;
    bool treffer = false;
    for (int i = 0; i < 110; i++) {
        if (i >= schritte) break;
        vec3 p = ro + d * t;
        float h = p.y - decke(p.xz, okt);
        if (h < 0.004 * t) { treffer = true; break; }
        if (t > 160.0) break;
        t += max(0.03, h * 0.55);
    }
    vec3 c;
    vec3 hm = himmel(d, s);
    if (treffer) {
        vec3 p = ro + d * t;
        vec2 e = vec2(0.06, 0.0);
        float h0 = decke(p.xz, okt);
        vec3 n = normalize(vec3(h0 - decke(p.xz + e.xy, okt), e.x, h0 - decke(p.xz + e.yx, okt)));
        float licht = clamp(dot(n, normalize(s + vec3(0.0, 0.12, 0.0))) * 0.95 + 0.22, 0.0, 1.0);
        // Selbstschatten: ein paar Schritte zur Sonne
        float schatten = 1.0;
        float ts = 0.25;
        for (int k = 0; k < 6; k++) {
            vec3 q = p + s * ts;
            float hh = q.y - decke(q.xz, 3);
            schatten = min(schatten, clamp(hh * 3.0 / ts + 0.4, 0.0, 1.0));
            ts *= 1.9;
        }
        vec3 tal = mix(vec3(0.18, 0.10, 0.34), vec3(0.24, 0.26, 0.52), fort);
        vec3 sonnig = mix(vec3(1.0, 0.55, 0.38), vec3(1.0, 0.78, 0.45), fort) * 1.5;
        c = mix(tal, sonnig, licht * (0.35 + 0.65 * schatten));
        // Silberrand, wo man gegen die Sonne auf eine Kuppe sieht
        float rand = pow(1.0 - max(dot(n, -d), 0.0), 3.0) * pow(max(dot(d, s), 0.0), 3.0);
        c += vec3(1.0, 0.9, 0.72) * rand * 1.8;
        // Licht vom Himmel auf die Oberseiten, oben weisser, wo es hoch hinaus geht
        c += himmel(vec3(0.0, 1.0, 0.0), s) * 0.18 * max(n.y, 0.0);
        c += vec3(0.1) * smoothstep(1.2, 2.6, h0);
        c *= 1.0 + 0.1 * kick;
        // Dunst zum Horizont
        c = mix(c, hm, 1.0 - exp(-t * 0.011));
    } else {
        c = hm;
    }
    // Lichtstrahlen von der Sonne her (im Bild, um ihren Ort), nur am Himmel
    float sz = dot(s, vorne);
    if (sz > 0.0 && !treffer) {
        vec2 sp = vec2(dot(s, rechts), dot(s, oben)) / sz;
        vec2 v = uv - sp;
        float w = atan(v.y, v.x);
        float strahlen = pow(rausch(vec2(w * 9.0, time * 0.15)), 3.0) * exp(-length(v) * 1.6);
        c += vec3(1.0, 0.82, 0.55) * strahlen * 0.3;
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
