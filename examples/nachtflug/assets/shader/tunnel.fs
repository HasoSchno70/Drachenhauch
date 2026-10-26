#version 330
// Lichttunnel: Ringe aus achteckigen Rahmen, jeder ein wenig gedreht; der
// Flug geht im Takt voran. Leuchten wird aufsummiert, je naeher der Strahl
// an einem Rahmen vorbeigeht -- daher der weiche Schein. Auf jeder Viertel
// laeuft eine Lichtwelle durch den Tunnel, die Farbe folgt dem Akkord, und
// jeder Melodieton laesst einen Rahmen aufleuchten.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float hell;
uniform float kick;
uniform float takt;
uniform float ton;       // Melodie: 1 auf dem Anschlag, klingt ab
uniform float hoehe;     // wie hoch der Melodieton liegt, 0..1
uniform vec3 farbe;      // Farbe des Akkords
uniform float aus;       // Szenenwechsel: Licht waechst aus der Mitte (0..1)
uniform float ein;       // ... und weicht nach aussen (0 = nur Licht, 1 = Szene)
uniform vec3 uebLicht;   // Farbe des Lichts beim Wechsel
out vec4 finalColor;

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
mat2 dreh(float a) { float c = cos(a), s = sin(a); return mat2(c, -s, s, c); }

// Abstand zu einem Rahmen im Segment um z=0
float rahmen(vec3 p, float seg) {
    p.xy = dreh(seg * 0.18 + time * 0.08) * p.xy;
    // Achteck ueber gefaltete Ebenen
    vec2 q = abs(p.xy);
    float acht = max(max(q.x, q.y), (q.x + q.y) * 0.7071);
    float ring = abs(acht - 3.0) - 0.06;
    return max(ring, abs(p.z) - 0.05);
}

// Szenenwechsel durch Licht (uv: zentriert, durch die Hoehe geteilt): ein
// Kreis aus Licht -- innen weiss, am Rand in der Farbe des Wechsels -- waechst
// aus der Mitte bzw. weicht nach aussen, dazu Strahlen wie bei einem Sprung.
float uebHash(float x) { return fract(sin(x * 91.7) * 43758.5453); }
vec3 uebergang(vec3 c, vec2 uv) {
    float r = length(uv);
    float maske;
    float k;
    if (aus > 0.0) {
        float R = aus * 1.7;
        maske = 1.0 - smoothstep(R - 0.6, R, r);
        k = aus;
    } else if (ein < 1.0) {
        float R = ein * 1.7;
        maske = smoothstep(R - 0.6, R, r);
        k = 1.0 - ein;
    } else {
        return c;
    }
    vec3 l = mix(uebLicht, vec3(1.0), exp(-r * 2.2)) * 1.05;
    c = mix(c, l, maske * 0.9);
    float a = atan(uv.y, uv.x);
    float id = floor(a * 60.0);
    float strahl = pow(uebHash(id), 5.0) * (0.6 + 0.4 * sin(time * 2.5 + id * 1.7));
    c += mix(uebLicht, vec3(1.0), 0.5) * strahl * k * smoothstep(0.08, 0.7, r) * 0.5;
    return c;
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    float fahrt = takt * 16.0 * 0.5;              // ein halber Abstand je Sechzehntel
    vec3 ro = vec3(0.4 * sin(time * 0.3), 0.3 * cos(time * 0.23), fahrt);
    vec3 d = normalize(vec3(uv * dreh(time * 0.05), 1.3));
    float t = 0.0;
    vec3 glut = vec3(0.0);
    float abstandSeg = 2.0;
    float melSeg = floor((ro.z + 5.0 + hoehe * 22.0) / abstandSeg + 0.5);
    for (int i = 0; i < 90; i++) {
        vec3 p = ro + d * t;
        float seg = floor(p.z / abstandSeg + 0.5);
        vec3 q = p;
        q.z -= seg * abstandSeg;
        float h = rahmen(q, seg);
        // die Welle: welcher Rahmen leuchtet gerade auf? Auf jede Viertel einer
        float welle = fract(seg * 0.125 - takt * 4.0 * 0.25 * 0.5);
        float an = smoothstep(0.12, 0.0, abs(welle - 0.5)) * (0.4 + kick);
        // die Melodie: ein Rahmen leuchtet auf, je hoeher der Ton, desto weiter vorn
        an += smoothstep(1.2, 0.0, abs(seg - melSeg)) * ton * 1.6;
        vec3 fRahmen = mix(farbe, farbe.gbr, 0.18 + 0.18 * sin(seg * 0.7));
        glut += fRahmen * (0.012 + 0.08 * an) / (0.02 + h * h * 30.0);
        t += max(h * 0.6, 0.03);
        if (t > 60.0) break;
    }
    vec3 c = glut * 0.06;
    // Tiefe: in der Ferne ein warmer Kern
    c += farbe * (0.25 + 0.4 * ton) * pow(max(d.z, 0.0), 60.0);
    c *= 1.0 - 0.35 * dot(uv, uv);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
