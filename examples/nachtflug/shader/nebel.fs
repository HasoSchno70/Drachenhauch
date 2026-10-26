#version 330
// Sternennebel: Faeden aus Gas, die selbst leuchten -- Magenta, Tuerkis und
// Gold aus zwei Rauschfeldern gemischt --, davor dunkle Staubwolken, dahinter
// Sterne. Die Kamera treibt langsam hinein; im Aufstieg der Musik (zweite
// Haelfte) wird sie schneller, und ein heller Kern waechst.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
uniform float takt;
uniform float fort;      // 0..1 durch den Abschnitt
out vec4 finalColor;

float hash(vec3 p) { return fract(sin(dot(p, vec3(127.1, 311.7, 74.7))) * 43758.5453); }
float hash2(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float rausch(vec3 p) {
    vec3 i = floor(p), f = fract(p);
    vec3 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash(i), hash(i + vec3(1, 0, 0)), u.x), mix(hash(i + vec3(0, 1, 0)), hash(i + vec3(1, 1, 0)), u.x), u.y),
               mix(mix(hash(i + vec3(0, 0, 1)), hash(i + vec3(1, 0, 1)), u.x), mix(hash(i + vec3(0, 1, 1)), hash(i + vec3(1, 1, 1)), u.x), u.y), u.z);
}
// Faeden: 1 - |2n - 1| macht aus Rauschen Grate
float faeden(vec3 p) {
    float s = 0.0, a = 0.55;
    for (int i = 0; i < 4; i++) {
        float n = 1.0 - abs(rausch(p) * 2.0 - 1.0);
        s += a * n * n;
        p = p * 2.1 + vec3(3.1, 1.7, 5.3);
        a *= 0.5;
    }
    return s;
}
float staub(vec3 p) {
    return rausch(p * 0.5) * 0.6 + rausch(p * 1.1 + 7.0) * 0.4;
}

vec3 sterne(vec2 uv) {
    vec2 s = uv * 220.0;
    float h = hash2(floor(s));
    float st = step(0.996, h) * smoothstep(0.45, 0.0, length(fract(s) - 0.5));
    return vec3(0.8, 0.85, 1.0) * st * (0.6 + 0.4 * sin(time * 2.0 + h * 50.0));
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    float schub = pow(max(fort - 0.5, 0.0) * 2.0, 3.0);
    float z = time * 0.9 + schub * 22.0;
    vec3 ro = vec3(sin(time * 0.06) * 1.5, cos(time * 0.045) * 1.0, z);
    float dr = time * 0.025;
    vec3 d = normalize(vec3(uv, 1.25));
    d.xy = mat2(cos(dr), -sin(dr), sin(dr), cos(dr)) * d.xy;

    vec3 sammel = vec3(0.0);
    float durch = 1.0;
    float t = 0.4 + hash2(fragTexCoord * resolution) * 0.08;
    for (int i = 0; i < 56; i++) {
        vec3 p = ro + d * t;
        vec3 q = p * 0.28;
        float f = faeden(q + vec3(0.0, 0.0, time * 0.015));
        float leucht = smoothstep(0.72, 1.15, f);
        float dunkel = smoothstep(0.4, 0.75, staub(p * 0.35 + 11.0));
        // Farbe aus einem zweiten Feld: Magenta <-> Tuerkis, Gold in den hellsten Faeden
        float mischung = rausch(q * 0.7 + 21.0);
        vec3 farbe = mix(vec3(1.0, 0.15, 0.6), vec3(0.1, 0.8, 1.0), smoothstep(0.3, 0.7, mischung));
        farbe = mix(farbe, vec3(1.0, 0.75, 0.35), smoothstep(0.85, 1.1, f));
        float naehe = exp(-t * 0.05);
        sammel += farbe * leucht * durch * 0.7 * (0.6 + 0.4 * naehe) * (1.0 + 0.4 * kick);
        durch *= 1.0 - dunkel * 0.16 - leucht * 0.04;
        if (durch < 0.02) break;
        t += 0.22 + t * 0.035;
    }
    vec3 c = sterne(uv) * durch + sammel;
    // Kern: waechst im Aufstieg, pulsiert leise
    float kern = smoothstep(0.35, 1.0, fort);
    float r = length(uv);
    c += vec3(1.0, 0.85, 0.65) * exp(-r * (18.0 - 12.0 * kern)) * (0.15 + 1.6 * kern);
    float w = atan(uv.y, uv.x);
    float strahl = pow(0.5 + 0.5 * sin(w * 19.0 + time * 0.15) * sin(w * 11.0 - time * 0.1), 8.0);
    c += vec3(1.0, 0.8, 0.6) * strahl * exp(-r * 2.8) * kern * 0.5;
    c *= 1.0 - 0.3 * dot(uv, uv);
    c += (hash2(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(1.1));
    // ganz am Ende: hell werden, hinein in den weissen Blitz des Hoehepunkts
    c = mix(c, vec3(1.0, 0.97, 0.92), smoothstep(0.94, 1.0, fort));
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
