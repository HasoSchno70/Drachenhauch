#version 330
// Sanftes Vignette + leichte Saettigung.
in vec2 fragTexCoord;
uniform sampler2D texture0;
out vec4 finalColor;
void main() {
    vec3 col = texture(texture0, fragTexCoord).rgb;
    float g = dot(col, vec3(0.299, 0.587, 0.114));
    col = mix(vec3(g), col, 1.25);                 // Saettigung
    vec2 c = fragTexCoord - 0.5;
    float v = smoothstep(0.75, 0.2, dot(c, c) * 2.0);
    finalColor = vec4(col * (0.3 + 0.7 * v), 1.0);
}
