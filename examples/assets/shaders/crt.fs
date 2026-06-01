#version 330
// CRT-Roehre: Barrel-Verzerrung, Scanlines, Aperture-Maske, Vignette, Flacker.
// Uniforms: resolution (vec2, Pixel), time (float, Sekunden).
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
out vec4 finalColor;

void main() {
    vec2 uv = fragTexCoord;
    vec2 cc = uv - 0.5;
    float dist = dot(cc, cc);
    uv += cc * dist * 0.16;                       // Barrel
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        finalColor = vec4(0.0, 0.0, 0.0, 1.0); return;
    }
    vec3 col = texture(texture0, uv).rgb;
    col *= 0.82 + 0.18 * sin(uv.y * resolution.y * 3.14159);   // Scanlines
    float m = mod(gl_FragCoord.x, 3.0);                        // RGB-Maske
    vec3 mask = (m < 1.0) ? vec3(1.06,0.96,0.96)
              : (m < 2.0) ? vec3(0.96,1.06,0.96) : vec3(0.96,0.96,1.06);
    col *= mask;
    col *= 0.97 + 0.03 * sin(time * 8.0);                      // Flacker
    col *= 0.45 + 0.55 * smoothstep(0.8, 0.1, dist * 2.0);     // Vignette
    finalColor = vec4(col, 1.0);
}
