#version 330
// Guenstiges Bloom: 7x7 Bright-Pass-Blur additiv ueber das Bild.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
out vec4 finalColor;
void main() {
    vec2 px = 1.0 / resolution;
    vec3 base = texture(texture0, fragTexCoord).rgb;
    vec3 bloom = vec3(0.0);
    for (int x = -3; x <= 3; x++)
        for (int y = -3; y <= 3; y++) {
            // Im Bild bleiben, und zwar ein HALBES TEXEL vom Rand entfernt:
            // genau auf der Kante mischt die bilineare Filterung die letzte
            // Zeile mit der ersten, und der Inhalt der Gegenseite blutet
            // herein. Klemmen auf 0..1 allein reicht dafuer nicht.
            vec2 su = clamp(fragTexCoord + vec2(x, y) * px * 1.5, px * 0.5, vec2(1.0) - px * 0.5);
            vec3 s = texture(texture0, su).rgb;
            bloom += max(s - 0.55, 0.0);
        }
    bloom /= 49.0;
    finalColor = vec4(base + bloom * 2.2, 1.0);
}
