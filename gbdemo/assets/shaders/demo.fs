#version 330
// Der Post-Effekt der GameBasic-Demo. EIN Shader fuer alle Szenen: `mode`
// schaltet um, `bass`/`hoehen` kommen aus dem echten AUDIO_FFT, `fade`
// blendet zwischen den Szenen ab.
//
// Wichtig fuers Verstaendnis: POSTFX bekommt das FERTIGE Bild. Ein Hintergrund
// (Plasma, Tunnel) laesst sich hier also nur einblenden, wo das Bild dunkel
// ist -- die Demo zeichnet ihre Szenen deshalb auf Schwarz, und der Shader
// fuellt die dunklen Stellen.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;      // Sekunden seit Demo-Start
uniform float bass;      // 0..1, geglaettete Bassenergie
uniform float hoehen;    // 0..1, geglaettete Hoehen
uniform float fade;      // 1 = voll sichtbar, 0 = schwarz
uniform float mode;      // 0 = nur Glanz, 1 = Plasma, 2 = Tunnel, 3 = CRT
out vec4 finalColor;

float luma(vec3 c) { return dot(c, vec3(0.299, 0.587, 0.114)); }

vec3 plasma(vec2 uv, float t) {
    float v = sin(uv.x * 9.0 + t * 1.3)
            + sin(uv.y * 11.0 - t * 0.9)
            + sin((uv.x + uv.y) * 7.0 + t * 1.7)
            + sin(length(uv - 0.5) * 18.0 - t * 2.4);
    v *= 0.25;
    // Bewusst dunkel und blaustichig: das Plasma ist HINTERGRUND. Volle
    // Saettigung frisst hier sonst jeden Text auf.
    vec3 c = vec3(0.5 + 0.5 * sin(v * 3.14159 + vec3(0.0, 1.6, 3.1)));
    c *= vec3(0.16, 0.30, 0.62);
    c += vec3(0.10, 0.02, 0.22) * (0.5 + 0.5 * sin(v * 6.0 - time));
    return c * (0.55 + 0.85 * bass);
}

vec3 tunnel(vec2 uv, float t) {
    vec2 p = uv - 0.5;
    p.x *= resolution.x / resolution.y;
    float r = length(p);
    float a = atan(p.y, p.x);
    // 1/r = klassischer Demo-Tunnel: gleiche Ringe wirken perspektivisch
    float u = 0.35 / max(r, 0.02) + t * 0.9;
    float v = a / 3.14159 * 4.0 + sin(t * 0.4) * 2.0;
    float karo = step(0.5, fract(u)) * step(0.5, fract(v))
               + step(0.5, 1.0 - fract(u)) * step(0.5, 1.0 - fract(v));
    // Wie beim Plasma: das hier ist HINTERGRUND. Ein voll gesaettigtes Karo
    // frisst den Ring davor auf.
    vec3 c = mix(vec3(0.015, 0.035, 0.08), vec3(0.04, 0.17, 0.20), karo);
    c *= smoothstep(0.0, 0.45, r);                    // Mitte dunkel = Tiefe
    return c * (0.6 + 0.8 * bass);
}

void main() {
    vec2 uv = fragTexCoord;
    vec2 px = 1.0 / resolution;

    // CRT-Modus verzerrt schon beim Abtasten (Roehren-Woelbung)
    if (mode > 2.5) {
        vec2 cc = uv - 0.5;
        uv += cc * dot(cc, cc) * 0.14;
        if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
            finalColor = vec4(0.0, 0.0, 0.0, 1.0); return;
        }
    }

    vec3 col = texture(texture0, uv).rgb;

    // Hintergrund dort einblenden, wo das Bild dunkel ist
    if (mode > 0.5 && mode < 2.5) {
        vec3 bg = (mode < 1.5) ? plasma(uv, time) : tunnel(uv, time);
        col = mix(bg, col, smoothstep(0.02, 0.30, luma(col)));
    }

    // Bright-Pass-Bloom -- laesst Neon und Emissives glimmen.
    //
    // Die Abtastpunkte muessen im Bild BLEIBEN -- und zwar ein halbes Texel
    // vom Rand entfernt. Auf 0..1 zu klemmen reicht NICHT: genau auf der
    // Texturkante mischt die bilineare Filterung die letzte Zeile mit der
    // ersten, und der helle Inhalt vom unteren Rand blutet oben wieder
    // herein (war als Reihe Farbfetzen in der obersten Pixelzeile zu sehen).
    vec2 rand = px * 0.5;
    vec3 bloom = vec3(0.0);
    for (int x = -2; x <= 2; x++)
        for (int y = -2; y <= 2; y++) {
            vec2 su = clamp(uv + vec2(x, y) * px * 2.5, rand, vec2(1.0) - rand);
            bloom += max(texture(texture0, su).rgb - 0.55, 0.0);
        }
    col += bloom / 25.0 * (1.6 + 1.8 * hoehen);

    if (mode > 2.5) {
        col *= 0.80 + 0.20 * sin(uv.y * resolution.y * 3.14159);   // Scanlines
        float m = mod(gl_FragCoord.x, 3.0);
        col *= (m < 1.0) ? vec3(1.08, 0.95, 0.95)
             : (m < 2.0) ? vec3(0.95, 1.08, 0.95) : vec3(0.95, 0.95, 1.08);
    }

    // Vignette, auf den Bass leicht atmend
    vec2 d = uv - 0.5;
    col *= 1.0 - dot(d, d) * (1.15 - 0.35 * bass);

    finalColor = vec4(col * fade, 1.0);
}
