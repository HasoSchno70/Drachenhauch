#version 330
// Lichttunnel: Ringe aus achteckigen Rahmen, jeder ein wenig gedreht; der
// Flug geht im Takt voran. Leuchten wird aufsummiert, je naeher der Strahl
// an einem Rahmen vorbeigeht -- daher der weiche Schein. Auf jeder Viertel
// laeuft eine Lichtwelle durch den Tunnel, die Farbe folgt dem Akkord.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
uniform float takt;
uniform float akkord;    // 0..4, Akkord der Harmonie (weich)
out vec4 finalColor;

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
mat2 dreh(float a) { float c = cos(a), s = sin(a); return mat2(c, -s, s, c); }

vec3 farbeVon(float k) {
    // a-Moll violett, F tuerkis, C gold, G rosa -- weich ineinander
    vec3 f0 = vec3(0.55, 0.25, 1.0), f1 = vec3(0.1, 0.85, 1.0), f2 = vec3(1.0, 0.7, 0.25), f3 = vec3(1.0, 0.3, 0.6);
    float i = floor(k), f = smoothstep(0.0, 1.0, fract(k));
    vec3 a = i < 0.5 ? f0 : i < 1.5 ? f1 : i < 2.5 ? f2 : f3;
    vec3 b = i < 0.5 ? f1 : i < 1.5 ? f2 : i < 2.5 ? f3 : f0;
    return mix(a, b, f);
}

// Abstand zu einem Rahmen im Segment um z=0
float rahmen(vec3 p, float seg) {
    p.xy = dreh(seg * 0.18 + time * 0.08) * p.xy;
    // Achteck ueber gefaltete Ebenen
    vec2 q = abs(p.xy);
    float acht = max(max(q.x, q.y), (q.x + q.y) * 0.7071);
    float ring = abs(acht - 3.0) - 0.06;
    return max(ring, abs(p.z) - 0.05);
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    float fahrt = takt * 16.0 * 0.5;              // ein halber Abstand je Sechzehntel
    vec3 ro = vec3(0.4 * sin(time * 0.3), 0.3 * cos(time * 0.23), fahrt);
    vec3 d = normalize(vec3(uv * dreh(time * 0.05), 1.3));
    float t = 0.0;
    vec3 glut = vec3(0.0);
    float abstandSeg = 2.0;
    for (int i = 0; i < 90; i++) {
        vec3 p = ro + d * t;
        float seg = floor(p.z / abstandSeg + 0.5);
        vec3 q = p;
        q.z -= seg * abstandSeg;
        float h = rahmen(q, seg);
        // die Welle: welcher Rahmen leuchtet gerade auf? Auf jede Viertel einer
        float welle = fract(seg * 0.125 - takt * 4.0 * 0.25 * 0.5);
        float an = smoothstep(0.12, 0.0, abs(welle - 0.5)) * (0.4 + kick);
        vec3 farbe = farbeVon(mod(akkord + seg * 0.01, 4.0));
        glut += farbe * (0.012 + 0.08 * an) / (0.02 + h * h * 30.0);
        t += max(h * 0.6, 0.03);
        if (t > 60.0) break;
    }
    vec3 c = glut * 0.06;
    // Tiefe: in der Ferne ein warmer Kern
    c += farbeVon(akkord) * 0.25 * pow(max(d.z, 0.0), 60.0);
    c *= 1.0 - 0.35 * dot(uv, uv);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
