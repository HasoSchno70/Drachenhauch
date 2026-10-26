#version 330
// Nachbearbeitung fuer die 3D-Szene: Bloom (helle Stellen weich verbreitert),
// sanfte Farbtrennung am Rand, Vignette, weich begrenzt. `blitz` legt auf der
// Eins des Hoehepunkts ein kurzes Weiss darueber.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
uniform float blitz;
out vec4 finalColor;

float hash2(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
// Sterne und ein Hauch Nebel, wo das Bild schwarz ist (hinter der Galaxie)
vec3 grund(vec2 uv) {
    vec2 a = (uv - 0.5) * resolution / resolution.y;
    vec2 s = a * 240.0 + vec2(time * 0.6, 0.0);
    float h = hash2(floor(s));
    float st = step(0.995, h) * smoothstep(0.45, 0.0, length(fract(s) - 0.5)) * (0.6 + 0.4 * sin(time * 2.0 + h * 40.0));
    vec3 n = vec3(0.10, 0.04, 0.16) * exp(-pow(a.y + 0.1 + 0.15 * sin(a.x * 1.3), 2.0) * 6.0);
    return vec3(0.8, 0.85, 1.0) * st + n;
}

vec3 probe(vec2 uv) {
    vec2 px = 0.5 / resolution;
    return texture(texture0, clamp(uv, px, 1.0 - px)).rgb;
}

void main() {
    vec2 uv = fragTexCoord;
    vec2 m = uv - 0.5;
    // Farbtrennung: Rot und Blau ein wenig auseinander, zum Rand hin mehr
    float ab = 0.0015 + 0.002 * kick;
    vec3 c = vec3(probe(uv + m * ab).r, probe(uv).g, probe(uv - m * ab).b);
    // Bloom: zwei Ringe von Proben um den Punkt, nur das Helle zaehlt
    vec3 b = vec3(0.0);
    vec2 px = 1.0 / resolution;
    for (int r = 1; r <= 2; r++) {
        for (int i = 0; i < 12; i++) {
            float a = float(i) * 0.5236 + float(r);
            vec2 o = vec2(cos(a), sin(a)) * px * float(r) * 9.0;
            vec3 s = probe(uv + o);
            b += max(s - 0.35, 0.0);
        }
    }
    b /= 24.0;
    c += b * (2.4 + 2.0 * kick);
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    c += grund(uv) * (1.0 - smoothstep(0.01, 0.12, l));
    c *= 1.0 - 0.55 * dot(m, m) * 1.6;
    c = c / (1.0 + 0.35 * c);
    c = mix(c, vec3(1.0, 0.97, 0.92), clamp(blitz, 0.0, 1.0) * 0.85);
    finalColor = vec4(c * hell, 1.0);
}
