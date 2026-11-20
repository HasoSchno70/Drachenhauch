#version 330
// Loesung zu Kapitel 15, Erfinden: das Gluehen aus glow.fs, aber jeder
// Punkt bekommt die Farbe des Plasmas an seiner Stelle. Die Spuren sind
// weiss gemalt; ihre Helligkeit sagt nur noch, WIE STARK es leuchtet.
in vec2 fragTexCoord;
out vec4 finalColor;

uniform sampler2D texture0;
uniform vec2 groesse;
uniform float staerke;
uniform float zeit;

vec3 hsv(float h, float s, float v) {
    vec3 k = mod(vec3(5.0, 3.0, 1.0) + h / 60.0, 6.0);
    return v - v * s * clamp(min(k, 4.0 - k), 0.0, 1.0);
}

vec3 plasma(vec2 uv) {
    float x = uv.x * groesse.x;
    float y = (1.0 - uv.y) * groesse.y;
    float v = sin(x / 40.0 + zeit) + sin(y / 30.0 - zeit);
    v = v + sin((x + y) / 60.0 + zeit);
    return hsv(v * 60.0 + zeit * 40.0, 0.8, 1.0);
}

void main() {
    vec2 punkt = 1.0 / groesse;
    float hier = texture(texture0, fragTexCoord).r;
    float schimmer = 0.0;
    float summe = 0.0;
    for (int dy = -4; dy <= 4; dy++) {
        for (int dx = -4; dx <= 4; dx++) {
            float gewicht = exp(-float(dx * dx + dy * dy) / 8.0);
            vec2 ort = fragTexCoord + vec2(dx, dy) * 3.0 * punkt;
            ort = clamp(ort, punkt * 0.5, 1.0 - punkt * 0.5);
            schimmer += texture(texture0, ort).r * gewicht;
            summe += gewicht;
        }
    }
    float licht = hier + schimmer / summe * staerke;
    finalColor = vec4(plasma(fragTexCoord) * licht, 1.0);
}
