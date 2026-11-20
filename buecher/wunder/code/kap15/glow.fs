#version 330
// Kapitel 15 -- Gluehen: zu jedem Bildpunkt kommt ein weicher Schimmer
// aus seiner Umgebung dazu. Wo es hell ist, strahlt es in die
// Nachbarschaft; wo alles dunkel ist, bleibt es dunkel.
in vec2 fragTexCoord;
out vec4 finalColor;

uniform sampler2D texture0;
uniform vec2 groesse;
uniform float staerke;

void main() {
    // Ein Bildpunkt, ausgedrueckt in fragTexCoord-Einheiten.
    vec2 punkt = 1.0 / groesse;
    vec3 hier = texture(texture0, fragTexCoord).rgb;
    // 9 x 9 Nachbarn im Abstand von je 3 Bildpunkten, die nahen zaehlen
    // mehr als die fernen.
    vec3 schimmer = vec3(0.0);
    float summe = 0.0;
    for (int dy = -4; dy <= 4; dy++) {
        for (int dx = -4; dx <= 4; dx++) {
            float gewicht = exp(-float(dx * dx + dy * dy) / 8.0);
            vec2 ort = fragTexCoord + vec2(dx, dy) * 3.0 * punkt;
            // Am Rand nicht ueber das Bild hinaus lesen.
            ort = clamp(ort, punkt * 0.5, 1.0 - punkt * 0.5);
            schimmer += texture(texture0, ort).rgb * gewicht;
            summe += gewicht;
        }
    }
    finalColor = vec4(hier + schimmer / summe * staerke, 1.0);
}
