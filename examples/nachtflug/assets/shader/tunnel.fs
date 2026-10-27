#version 330
// Lichttunnel: Neonringe aus achteckigen Rahmen, jeder ein wenig gedreht,
// mit weissem Kern und farbigem Schein; dahinter die Wand des Schachts aus
// Panelen, ueber deren Fugen Lichtimpulse auf den Betrachter zulaufen und in
// denen sich die Ringe spiegeln; vorbeirasende Lichtfunken; am Ende des
// Tunnels ein helles Ziel mit Strahlen. Der Flug geht im Takt voran. Auf
// jeder Viertel laeuft eine Lichtwelle durch die Ringe, die Farbe folgt dem
// Akkord, jeder Melodieton laesst einen Ring aufleuchten.
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

const float SEG = 2.0;       // Abstand der Ringe
const float WAND = 3.7;      // Halbmesser des Schachts

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float hash1(float x) { return fract(sin(x * 78.233) * 43758.5453); }
mat2 dreh(float a) { float c = cos(a), s = sin(a); return mat2(c, -s, s, c); }

// Abstand zu einem Rahmen im Segment um z=0
float rahmen(vec3 p, float seg) {
    p.xy = dreh(seg * 0.18 + time * 0.08) * p.xy;
    // Achteck ueber gefaltete Ebenen
    vec2 q = abs(p.xy);
    float acht = max(max(q.x, q.y), (q.x + q.y) * 0.7071);
    float ring = abs(acht - 3.0) - 0.045;
    return max(ring, abs(p.z) - 0.04);
}

// Die Farbe eines Rings: die Akkordfarbe im Wechsel mit zwei verschobenen
// Toenen, weich ineinander und kraeftig gesaettigt -- so wird der Tunnel
// nicht einfarbig.
vec3 ringFarbe(float seg) {
    float w = 0.5 + 0.5 * sin(seg * 1.3);
    float v = 0.5 + 0.5 * sin(seg * 0.47 + 1.7);
    vec3 f = mix(farbe, farbe.brg, w * 0.6);
    f = mix(f, farbe.gbr, v * 0.3);
    f = mix(vec3(dot(f, vec3(0.3333))), f, 1.7);
    return clamp(f, 0.0, 1.2);
}

// Wie hell ein Ring gerade ist: die Welle je Viertel und die Melodie
float ringHell(float seg, float melSeg) {
    float welle = fract(seg * 0.125 - takt * 0.5);
    float an = smoothstep(0.12, 0.0, abs(welle - 0.5)) * (0.4 + kick);
    an += smoothstep(1.2, 0.0, abs(seg - melSeg)) * ton * 1.6;
    return an;
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

vec3 tonAbbilden(vec3 c) {
    // ACES-Naeherung: kraeftige Farben, weiche Lichter
    return clamp((c * (2.51 * c + 0.03)) / (c * (2.43 * c + 0.59) + 0.14), 0.0, 1.0);
}

void main() {
    vec2 uv = (fragTexCoord * resolution - 0.5 * resolution) / resolution.y;
    float fahrt = takt * 16.0 * 0.5;              // ein halber Abstand je Sechzehntel
    vec3 ro = vec3(0.4 * sin(time * 0.3), 0.3 * cos(time * 0.23), fahrt);
    vec3 d = normalize(vec3(uv * dreh(time * 0.05), 1.3));
    float melSeg = floor((ro.z + 5.0 + hoehe * 22.0) / SEG + 0.5);

    // wo der Strahl die Wand des Schachts trifft (Zylinder um die z-Achse)
    float a2 = dot(d.xy, d.xy);
    float b2 = dot(ro.xy, d.xy);
    float c2 = dot(ro.xy, ro.xy) - WAND * WAND;
    float tWand = a2 > 1e-5 ? (-b2 + sqrt(max(b2 * b2 - a2 * c2, 0.0))) / a2 : 200.0;

    // die Ringe: Kern und Schein getrennt aufsummiert
    float t = 0.0;
    vec3 kern = vec3(0.0);
    vec3 schein = vec3(0.0);
    float tEnde = min(tWand, 70.0);
    for (int i = 0; i < 100; i++) {
        vec3 p = ro + d * t;
        float seg = floor(p.z / SEG + 0.5);
        vec3 q = p;
        q.z -= seg * SEG;
        float h = rahmen(q, seg);
        float an = ringHell(seg, melSeg);
        vec3 f = ringFarbe(seg);
        float nebel = exp(-t * 0.085);
        kern += mix(vec3(1.0), f, 0.8) * (0.35 + 2.0 * an) * 0.00018 / (0.0002 + h * h) * nebel;
        schein += f * (0.006 + 0.07 * an) / (0.02 + h * h * 20.0) * nebel;
        t += max(h * 0.6, 0.03);
        if (t > tEnde) break;
    }
    vec3 c = kern * 0.06 + schein * 0.05;

    // die Wand: Panele mit Fugen, Lichtimpulse, Spiegelung der Ringe
    if (tWand < 70.0) {
        vec3 pw = ro + d * tWand;
        float w = atan(pw.y, pw.x) / 6.2832 + 0.5;
        float su = fract(w * 24.0);
        float sv = fract(pw.z / SEG + 0.5);
        float fugeU = smoothstep(0.06, 0.0, min(su, 1.0 - su));
        float fugeV = smoothstep(0.04, 0.0, min(sv, 1.0 - sv));
        float spalte = floor(w * 24.0);
        // Lichtimpulse laufen die Laengsfugen entlang auf den Betrachter zu
        float puls = fract(pw.z * 0.04 + time * 0.55 + hash1(spalte) * 7.0);
        puls = pow(smoothstep(0.75, 1.0, puls), 3.0) * step(0.55, hash1(spalte + 3.1));
        float segW = floor(pw.z / SEG + 0.5);
        vec3 fw = ringFarbe(segW);
        // der naechste Ring spiegelt sich in der Wand
        float zuRing = abs(pw.z - segW * SEG);
        float spiegel = exp(-zuRing * zuRing * 2.5) * (0.3 + 1.2 * ringHell(segW, melSeg));
        vec3 wand = vec3(0.012, 0.014, 0.024) + fw * 0.05;
        wand += fw * (fugeU * 0.25 + fugeV * 0.35) * (0.4 + 0.6 * spiegel);
        wand += mix(fw, vec3(1.0), 0.4) * fugeU * puls * (0.8 + 1.5 * kick);
        wand += fw * spiegel * 0.12;
        c += wand * exp(-tWand * 0.09);
    }

    // vorbeirasende Funken: Striche in Polarkoordinaten des Bildes
    float r = length(uv);
    float ang = atan(uv.y, uv.x);
    float id = floor(ang * 48.0);
    float tiefe = 1.0 / max(r, 0.02);
    float lauf = fract(tiefe * 0.35 - takt * 2.0 + hash1(id) * 11.0);
    float strich = smoothstep(0.9, 1.0, lauf) * step(0.7, hash1(id + 0.37)) * smoothstep(0.15, 0.6, r);
    c += mix(farbe, vec3(1.0), 0.6) * strich * 0.35;

    // das Ziel am Ende des Tunnels: ein heller Kern mit weichen Strahlen
    float mitte = max(d.z, 0.0);
    float strahlen = 0.5 + 0.5 * sin(ang * 9.0 + time * 0.4) * sin(ang * 5.0 - time * 0.27);
    c += mix(farbe, vec3(1.0), 0.65) * (pow(mitte, 500.0) * (1.2 + 0.8 * ton) + pow(mitte, 60.0) * 0.25 + pow(mitte, 18.0) * strahlen * 0.08);

    c *= 1.0 - 0.3 * dot(uv, uv);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.01;
    c = tonAbbilden(c * 1.15);
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
