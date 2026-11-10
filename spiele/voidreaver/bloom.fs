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
            vec3 s = texture(texture0, fragTexCoord + vec2(x, y) * px * 1.5).rgb;
            bloom += max(s - 0.55, 0.0);
        }
    bloom /= 49.0;
    finalColor = vec4(base + bloom * 2.2, 1.0);
}
