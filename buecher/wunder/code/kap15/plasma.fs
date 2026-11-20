#version 330
// Kapitel 15 -- das Plasma aus Kapitel 2 als Shader. Dieses kleine
// Programm laeuft auf der Grafikkarte, einmal fuer JEDEN Bildpunkt.
in vec2 fragTexCoord;
out vec4 finalColor;

uniform float zeit;
uniform vec2 groesse;

// Farbton (0..360), Saettigung und Helligkeit (0..1) nach Rot, Gruen
// und Blau -- wie HSV im Programm.
vec3 hsv(float h, float s, float v) {
    vec3 k = mod(vec3(5.0, 3.0, 1.0) + h / 60.0, 6.0);
    return v - v * s * clamp(min(k, 4.0 - k), 0.0, 1.0);
}

void main() {
    // Wo bin ich? fragTexCoord geht von 0 bis 1; mal groesse gibt das
    // in Bildpunkten. Die Grafikkarte zaehlt y von UNTEN -- 1.0 - ...
    // dreht es um, wie im Programm: oben ist 0.
    float x = fragTexCoord.x * groesse.x;
    float y = (1.0 - fragTexCoord.y) * groesse.y;
    float v = sin(x / 40.0 + zeit) + sin(y / 30.0 - zeit);
    v = v + sin((x + y) / 60.0 + zeit);
    finalColor = vec4(hsv(v * 60.0 + zeit * 40.0, 0.8, 1.0), 1.0);
}
