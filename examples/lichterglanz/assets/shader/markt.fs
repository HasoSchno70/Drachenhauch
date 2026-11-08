#version 330
// Der Weihnachtsmarkt: ein Platz mit Kopfstein unter Schnee, ringsum
// Holzbuden mit Tannengirlande und Lichterkette an der Traufe; auf den
// Theken Gluehweintassen, Kugeln oder brennende Kerzen, an mancher haengen
// Lebkuchenherzen, aus den Gluehweinbuden steigt Dampf. Vom Karussell zu
// jeder Bude spannt sich eine Lichterkette. Leute in Mantel und Pudelmuetze
// schlendern auf zwei Bahnen um den Platz, andere stehen an den Buden. In
// der Mitte das Karussell mit rot-weissem Dach, Spiegelsaeule, goldener
// Krone und bunten Pferden -- es dreht sich auf der Audio-Uhr (`takt`), die
// Pferde heben und senken sich im Takt, die Lichter laufen mit der
// Bassdrum um. Die Kamera umkreist den Platz (`fort`).
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float takt;
uniform float hell;
uniform float fort;
uniform float kick;
uniform float ton;
uniform float hoehe;
uniform vec3 farbe;
uniform float aus;
uniform float ein;
uniform vec3 uebLicht;
uniform float guete;
out vec4 finalColor;

float G() { return guete > 0.0 ? guete : 1.0; }

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float rausch(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1, 0)), u.x), mix(hash(i + vec2(0, 1)), hash(i + vec2(1, 1)), u.x), u.y);
}

const float PI = 3.14159265;
const float BUDEN = 10.0;
const float RING = 11.0;

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Wiederholung im Kreis: p in den Sektor um die x-Achse drehen; id = Sektor
vec2 kreisWdh(vec2 p, float n, out float id) {
    float sek = 2.0 * PI / n;
    float a = atan(p.y, p.x) + sek * 0.5;
    id = floor(a / sek);
    a = mod(a, sek) - sek * 0.5;
    return length(p) * vec2(cos(a), sin(a));
}

float kapsel(vec3 p, vec3 a, vec3 b, float r) {
    vec3 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

vec2 drehe(vec2 p, float a) {
    float c = cos(a), s = sin(a);
    return vec2(c * p.x - s * p.y, s * p.x + c * p.y);
}

float drehung() { return takt * 0.55; }

// Ein Mensch, der geht (`schritt` = Phase des Schritts, 0 = steht): Mantel,
// Kopf, Pudelmuetze, zwei Beine. Blick in +z. teil: 0 Mantel, 1 Gesicht,
// 2 Muetze, 3 Hose
float menschTeil;
float mensch(vec3 p, float gross, float schritt) {
    p /= gross;
    p.y -= 0.03 * abs(sin(schritt));
    float mantel = min(kapsel(p, vec3(0.0, 0.55, 0.0), vec3(0.0, 0.92, 0.0), 0.21), kapsel(p, vec3(0.0, 0.92, 0.0), vec3(0.0, 1.3, 0.0), 0.17));
    float d = mantel;
    menschTeil = 0.0;
    float kopf = length(p - vec3(0.0, 1.53, 0.02)) - 0.105;
    if (kopf < d) { d = kopf; menschTeil = 1.0; }
    float muetze = max(length(p - vec3(0.0, 1.57, 0.0)) - 0.112, 1.56 - p.y);
    muetze = min(muetze, length(p - vec3(0.0, 1.72, -0.02)) - 0.045);
    if (muetze < d) { d = muetze; menschTeil = 2.0; }
    float sw = 0.16 * sin(schritt);
    float bein = min(kapsel(p, vec3(0.08, 0.55, 0.0), vec3(0.08, 0.05, sw), 0.06), kapsel(p, vec3(-0.08, 0.55, 0.0), vec3(-0.08, 0.05, -sw), 0.06));
    if (bein < d) { d = bein; menschTeil = 3.0; }
    // Arme, gegen die Beine schwingend
    float arm = min(kapsel(p, vec3(0.2, 1.25, 0.0), vec3(0.25, 0.82, -sw * 0.8), 0.055), kapsel(p, vec3(-0.2, 1.25, 0.0), vec3(-0.25, 0.82, sw * 0.8), 0.055));
    if (arm < d) { d = arm; menschTeil = 0.0; }
    return d * gross;
}

// Material: 0 Boden, 1 Bude, 2 Budendach, 3 Theke (leuchtet), 4 Karussell-
// Boden/Saeulen, 5 Karussell-Dach, 6 Pferd, 7 Mensch, 8 Thekenbrett,
// 9 Ware, 10 Girlande, 11 Lebkuchenherz
float mat;
float budeId;
float teilId;
float personId;
// die Gluehbirnen: Abstand zur naechsten und ihre Farbe
float birneD;
vec3 birneF;

float szene(vec3 p) {
    float d = p.y - 0.02 * rausch(p.xz * 3.0);
    mat = 0.0;
    birneD = 1e5;
    // --- Buden im Ring ---
    float id;
    vec2 k = kreisWdh(p.xz, BUDEN, id);
    vec3 q = vec3(k.x - RING, p.y, k.y);
    float bude = sdBox(q - vec3(0.0, 1.1, 0.0), vec3(0.9, 1.1, 1.35));
    // die Theke: eine Oeffnung in der Front
    float oeff = sdBox(q - vec3(-0.8, 1.35, 0.0), vec3(0.3, 0.45, 1.05));
    bude = max(bude, -oeff);
    if (bude < d) { d = bude; mat = 1.0; budeId = id; }
    float rueck = sdBox(q - vec3(-0.48, 1.35, 0.0), vec3(0.04, 0.46, 1.06));
    if (rueck < d) { d = rueck; mat = 3.0; budeId = id; }
    // das Thekenbrett, auf ihm die Waren -- je Bude eine Sorte
    float brett = sdBox(q - vec3(-1.0, 0.92, 0.0), vec3(0.2, 0.035, 1.12));
    if (brett < d) { d = brett; mat = 8.0; budeId = id; }
    float sorte = mod(id, 4.0);
    float wz = clamp(floor(q.z / 0.24 + 0.5), -4.0, 4.0);
    vec3 wq = q - vec3(-1.02, 0.955, wz * 0.24);
    float ware = 1e5;
    if (sorte < 0.5) {
        // Gluehweintassen
        ware = max(length(wq.xz) - 0.045, abs(wq.y - 0.05) - 0.05);
    } else if (sorte < 1.5) {
        // Christbaumkugeln
        ware = length(wq - vec3(0.0, 0.055, 0.0)) - 0.055;
    } else if (sorte < 2.5) {
        // Kerzen, verschieden hoch, jede brennt
        float kh = 0.07 + 0.06 * hash(vec2(id, wz));
        ware = max(length(wq.xz) - 0.028, abs(wq.y - kh) - kh);
        float fl = length((wq - vec3(0.0, 2.0 * kh + 0.035, 0.0)) / vec3(1.0, 1.9, 1.0)) * 0.55 - 0.009;
        if (fl < birneD) { birneD = fl; birneF = vec3(1.0, 0.65, 0.3) * (0.8 + 0.2 * sin(time * 13.0 + wz * 3.0)); }
    } else {
        // Lebkuchenherzen an der Traufe, sie schaukeln ein wenig
        float hz = clamp(floor(q.z / 0.42 + 0.5), -2.0, 2.0);
        vec3 hq = q - vec3(-1.08, 1.86, hz * 0.42);
        float sch = 0.12 * sin(time * 1.3 + hz * 2.0 + id);
        hq.zy = drehe(hq.zy, sch);
        // ein Herz (Spitze unten), knapp 0.2 hoch, an der Traufe
        vec2 h2 = vec2(abs(hq.z), (hq.y + 0.03) / 0.2 + 1.05);
        h2.x /= 0.2;
        float herz;
        if (h2.y + h2.x > 1.0) herz = length(h2 - vec2(0.25, 0.75)) - 0.3536;
        else herz = sqrt(min(dot(h2 - vec2(0.0, 1.0), h2 - vec2(0.0, 1.0)), dot(h2 - 0.5 * max(h2.x + h2.y, 0.0), h2 - 0.5 * max(h2.x + h2.y, 0.0)))) * sign(h2.x - h2.y);
        herz = max(herz * 0.2, abs(hq.x) - 0.015);
        if (herz < d) { d = herz; mat = 11.0; budeId = id; }
        ware = max(length(wq.xz) - 0.045, abs(wq.y - 0.03) - 0.03);
    }
    if (ware < d) { d = ware; mat = 9.0; budeId = id; teilId = wz; }
    // die Tannengirlande ueber der Theke
    float girl = length(q.xy - vec2(-1.2, 2.24)) - 0.075 - 0.03 * rausch(vec2(q.z * 14.0, id));
    girl = max(girl, abs(q.z) - 1.5);
    if (girl < d) { d = girl; mat = 10.0; budeId = id; }
    // Dach: Prisma, First tangential
    vec2 n = normalize(vec2(1.0 / 1.25, 1.0 / 0.75));
    float y = q.y - 2.2;
    float dach = max(max(dot(n, vec2(abs(q.x), y)) - dot(n, vec2(1.25, 0.0)), -y - 0.08), abs(q.z) - 1.55) - 0.04;
    if (dach < d) { d = dach; mat = 2.0; budeId = id; }
    // Lichterkette an der vorderen Traufe: eine Birne alle 0.3, leicht
    // durchhaengend
    float bz = clamp(floor(q.z / 0.3 + 0.5), -5.0, 5.0);
    vec3 bp = vec3(-1.25, 2.12 - 0.06 * (1.0 - pow(bz / 5.0, 2.0)), bz * 0.3);
    float b = length(q - bp) - 0.03;
    if (b < birneD) {
        birneD = b;
        float wahl = hash(vec2(id, bz));
        birneF = wahl < 0.25 ? vec3(1.0, 0.25, 0.15) : wahl < 0.5 ? vec3(0.3, 1.0, 0.35) : wahl < 0.75 ? vec3(0.35, 0.5, 1.0) : vec3(1.0, 0.75, 0.3);
        // jede zweite blinkt im Takt
        birneF *= 0.55 + 0.45 * step(0.5, fract(takt * 2.0 + mod(bz, 2.0) * 0.5));
    }

    // Lichterketten vom Karussell zu jeder Bude, durchhaengend
    float lx = clamp(floor((k.x - 3.6) / 0.45 + 0.5), 0.0, 14.0);
    float ls = lx / 14.0;
    vec3 lp = vec3(3.6 + lx * 0.45, 4.4 - 1.65 * ls - 0.4 * sin(ls * PI), 0.0);
    float kb = length(vec3(k.x, p.y, k.y) - lp) - 0.032;
    if (kb < birneD) {
        birneD = kb;
        float wahl = hash(vec2(id * 7.0, lx));
        birneF = wahl < 0.5 ? vec3(1.0, 0.82, 0.5) : wahl < 0.7 ? vec3(1.0, 0.3, 0.2) : wahl < 0.85 ? vec3(0.35, 0.6, 1.0) : vec3(0.4, 1.0, 0.45);
        birneF *= 0.6 + 0.4 * sin(time * 2.5 + lx * 1.3 + id * 2.0);
    }

    // --- Leute: zwei Bahnen um den Platz, dazu je Bude einer an der Theke ---
    // (ausserhalb ihrer Bahnen zaehlt nur der Abstand zu ihnen)
    float rp = length(p.xz);
    float bahnen = max(abs(rp - 5.4) - 1.6, p.y - 2.0);
    if (bahnen > 0.1) d = min(d, bahnen);
    else {
        for (int bahn = 0; bahn < 2; bahn++) {
            float fb = float(bahn);
            float R = fb < 0.5 ? 4.7 : 6.1;
            float anz = fb < 0.5 ? 11.0 : 12.0;
            float ri = fb < 0.5 ? 1.0 : -1.0;
            float lauf = time * (0.55 + 0.1 * fb) / R * ri;
            float mid;
            vec2 mk = kreisWdh(drehe(p.xz, -lauf), anz, mid);
            mk.y += 0.25 * sin(mid * 2.3 + fb);
            vec3 mq = vec3(mk.x - R - 0.3 * sin(mid * 1.7), p.y, mk.y * ri);
            if (hash(vec2(mid, fb + 2.0)) < 0.15) continue;
            float gross = hash(vec2(mid, fb)) < 0.12 ? 0.62 : 0.88 + 0.2 * hash(vec2(mid, fb + 5.0));
            float md = mensch(mq, gross, time * 5.5 + mid);
            if (md < d) { d = md; mat = 7.0; teilId = menschTeil; personId = mid * 3.1 + fb * 17.0; }
        }
    }
    float stehen = max(abs(rp - (RING - 1.75)) - 0.55, p.y - 2.0);
    if (stehen > 0.1) d = min(d, stehen);
    else {
        vec3 mq = vec3(q.x + 1.75, p.y, q.z - 0.3 * sin(id * 2.1));
        mq.xz = drehe(mq.xz, 1.57);
        float md = hash(vec2(id, 9.0)) < 0.25 ? 1e5 : mensch(mq, 0.92 + 0.15 * hash(vec2(id, 4.0)), 0.0);
        if (md < d) { d = md; mat = 7.0; teilId = menschTeil; personId = id * 5.3 + 40.0; }
    }

    // --- Karussell ---
    vec3 c = p;
    c.xz = drehe(c.xz, drehung());
    float platte = max(length(c.xz) - 2.7, abs(c.y - 0.2) - 0.2);
    float mast = max(length(c.xz) - 0.28, c.y - 4.0);
    float kar = min(platte, mast);
    if (kar < d) { d = kar; mat = 4.0; }
    // Dach: ein Kegel, rot-weiss gestreift
    vec2 r = vec2(length(c.xz), c.y);
    vec2 kn = normalize(vec2(1.5, 3.2));
    float kd = dot(r - vec2(0.0, 4.7), kn);
    float kegel = max(max(kd, -kd - 0.12), 3.05 - c.y);
    // obenauf eine goldene Krone mit Kugel
    float krone = length(c - vec3(0.0, 4.85, 0.0)) - 0.2;
    krone = min(krone, max(abs(length(c.xz) - 0.3) - 0.03, abs(c.y - 4.72) - 0.08 + 0.05 * sin(atan(c.z, c.x) * 8.0)));
    if (krone < d) { d = krone; mat = 12.0; }
    // die Kante mit Bogen-Behang
    float behang = max(abs(length(c.xz) - 3.15) - 0.04, max(c.y - 3.15, 2.85 - c.y + 0.08 * abs(sin(atan(c.z, c.x) * 16.0))));
    kegel = min(kegel, behang);
    if (kegel < d) { d = kegel; mat = 5.0; }
    // Pferde an Stangen, acht im Kreis; jedes hebt und senkt sich
    float pid;
    vec2 pk = kreisWdh(c.xz, 8.0, pid);
    vec3 pq = vec3(pk.x - 2.0, c.y, pk.y);
    float stange = max(length(pq.xz) - 0.035, c.y - 3.05);
    float hub = 0.22 * sin(takt * PI + pid * 1.7);
    vec3 hp = pq - vec3(0.0, 1.25 + hub, 0.0);
    // das Karussell dreht sich (von oben gesehen) gegen den Uhrzeigersinn,
    // die Pferde schauen in Laufrichtung
    hp.z = -hp.z;
    // Koerper laengs der Laufrichtung (tangential), Hals und Kopf vorn, vier
    // Beine im Galopp (vorn nach vorn, hinten nach hinten gestreckt)
    float koerper = kapsel(hp, vec3(0.0, 0.0, -0.22), vec3(0.0, 0.02, 0.22), 0.14);
    float hals = kapsel(hp, vec3(0.0, 0.05, 0.22), vec3(0.0, 0.36, 0.38), 0.075);
    float kopf = kapsel(hp, vec3(0.0, 0.38, 0.38), vec3(0.0, 0.3, 0.56), 0.06);
    float gal = 0.12 * sin(takt * PI * 2.0 + pid);
    float beine = min(kapsel(vec3(abs(hp.x), hp.yz), vec3(0.06, -0.05, 0.18), vec3(0.06, -0.38, 0.32 + gal), 0.03),
                      kapsel(vec3(abs(hp.x), hp.yz), vec3(0.06, -0.05, -0.2), vec3(0.06, -0.38, -0.36 - gal), 0.03));
    float schweif = kapsel(hp, vec3(0.0, 0.05, -0.34), vec3(0.0, -0.2, -0.46), 0.03);
    float pferd = min(min(min(koerper, hals), min(kopf, beine)), schweif);
    if (stange < d) { d = stange; mat = 4.0; }
    if (pferd < d) { d = pferd; mat = 6.0; budeId = pid; }
    // Birnen am Behang: 32 im Kreis, eine Lauflicht-Welle mit der Bassdrum
    float lid;
    vec2 lk = kreisWdh(c.xz, 32.0, lid);
    float lb = length(vec3(lk.x - 3.2, c.y - 2.95, lk.y)) - 0.045;
    if (lb < birneD) {
        birneD = lb;
        float welle = 0.5 + 0.5 * cos((lid / 32.0 - takt) * 2.0 * PI * 2.0);
        birneF = vec3(1.0, 0.8, 0.45) * (0.45 + 0.55 * welle + 0.6 * kick);
    }
    // ... und eine Reihe am Dach hinauf
    float lb2 = length(vec3(lk.x - 1.6, c.y - 3.95, lk.y)) - 0.035;
    if (lb2 < birneD) {
        birneD = lb2;
        birneF = vec3(1.0, 0.85, 0.6) * (0.7 + 0.3 * sin(time * 4.0 + lid));
    }
    return min(d, birneD + 0.02);
}

vec3 normale(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
}

// Dampf aus den Gluehweinbuden: eine Saeule vor jeder dritten Bude, aus
// Rauschen, das nach oben zieht; gerechnet, wo der Strahl der Achse der
// Saeule am naechsten kommt
vec3 dampf(vec3 ro, vec3 rd, float tmax) {
    vec3 c = vec3(0.0);
    for (int i = 0; i < 4; i++) {
        float a = (float(i) * 3.0 + 1.0) * 2.0 * PI / BUDEN;
        vec2 ort = vec2(cos(a), sin(a)) * (RING - 1.3);
        // naechster Punkt des Strahls an der senkrechten Achse
        vec2 o = ro.xz - ort;
        vec2 dxz = rd.xz;
        float t = -dot(o, dxz) / max(dot(dxz, dxz), 1e-4);
        if (t < 0.0 || t > tmax) continue;
        vec3 p = ro + rd * t;
        float r = length(p.xz - ort);
        float y = p.y - 1.3;
        if (y < 0.0 || y > 3.0) continue;
        float n = rausch(vec2(r * 3.0 + a, y * 2.0 - time * 0.9)) * rausch(vec2(y * 1.3 - time * 0.5, a));
        float dichte = n * exp(-r * r * 3.0 / (0.3 + y)) * smoothstep(3.0, 0.5, y) * smoothstep(0.0, 0.3, y);
        c += vec3(0.75, 0.7, 0.65) * dichte * 0.5;
    }
    return c;
}

vec3 himmel(vec3 d) {
    float h = clamp(d.y, 0.0, 1.0);
    vec3 c = mix(vec3(0.07, 0.07, 0.14), vec3(0.01, 0.012, 0.035), pow(h, 0.5));
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 150.0;
    vec2 z = floor(uv);
    c += vec3(0.85, 0.9, 1.0) * step(0.985, hash(z)) * smoothstep(0.35, 0.0, length(fract(uv) - 0.5)) * (0.6 + 0.4 * sin(time * 2.0 + hash(z) * 30.0));
    return c;
}

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
    float w = 0.4 + fort * 1.6 + time * 0.01;
    float abst = 8.8 - 0.8 * fort;
    vec3 ro = vec3(cos(w) * abst, 1.75 + 0.35 * sin(fort * 3.0), sin(w) * abst);
    vec3 ta = vec3(0.0, 2.0, 0.0);
    vec3 vorne = normalize(ta - ro);
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.25 + uv.x * rechts + uv.y * oben);

    vec3 c;
    float t = 0.0;
    float hit = -1.0;
    vec3 glimmen = vec3(0.0);
    int schritte = int(120.0 * G());
    for (int i = 0; i < 120; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float d = szene(p);
        // Schein der Birnen: sammelt sich, wo der Strahl an ihnen vorbeigeht
        glimmen += birneF * 0.0035 / (0.0004 + birneD * birneD) * min(d, 0.3);
        if (d < 0.001 * t + 0.0005) { hit = t; break; }
        t += d * 0.85;
        if (t > 40.0) break;
    }
    float tmax = 40.0;
    if (hit > 0.0) {
        tmax = hit;
        vec3 p = ro + rd * hit;
        szene(p);
        float m = mat;
        float bid = budeId;
        vec3 n = normale(p);
        vec3 alb;
        vec3 eigen = vec3(0.0);
        float glanz = 0.0;
        if (birneD < 0.005) {
            alb = vec3(0.0);
            eigen = birneF * 3.0;
        } else if (m < 0.5) {
            // Kopfstein unter duenner Schneedecke
            vec2 st = p.xz * 2.2;
            vec2 g = fract(st) - 0.5;
            float fuge = smoothstep(0.42, 0.5, max(abs(g.x), abs(g.y)));
            alb = mix(vec3(0.7, 0.72, 0.8), vec3(0.18, 0.17, 0.2), fuge * 0.6 * smoothstep(0.4, 0.8, rausch(p.xz * 1.3)));
        } else if (m < 1.5) {
            // Holzbretter, senkrecht; jede Bude in ihrer Farbe
            float brett = fract(atan(p.z, p.x) * RING * 2.2);
            vec3 holz = mix(vec3(0.3, 0.14, 0.06), vec3(0.22, 0.1, 0.05), hash(vec2(bid, 3.0)));
            alb = holz * (0.8 + 0.2 * smoothstep(0.0, 0.08, brett) * smoothstep(1.0, 0.92, brett));
        } else if (m < 2.5) {
            alb = n.y > 0.3 ? vec3(0.85, 0.88, 0.95) : vec3(0.25, 0.12, 0.06);
        } else if (m < 3.5) {
            // die Theke von innen: Lichter, Waren, warmes Leuchten
            alb = vec3(0.0);
            vec2 f = vec2(atan(p.z, p.x) * RING, p.y);
            float regal = step(0.5, fract(f.y * 4.0)) * 0.3 + 0.7;
            vec3 waren = mix(vec3(1.0, 0.55, 0.2), vec3(0.9, 0.2, 0.15), hash(floor(f * vec2(5.0, 4.0)) + bid));
            eigen = mix(vec3(1.0, 0.7, 0.35), waren, 0.5) * regal * 1.4;
        } else if (m < 4.5) {
            alb = vec3(0.75, 0.6, 0.3);
            vec3 c2 = p;
            c2.xz = drehe(c2.xz, drehung());
            float ra = length(c2.xz);
            if (ra < 0.32 && p.y > 0.45 && p.y < 3.9) {
                // die Spiegelsaeule: Felder mit kleinen Lichtern
                vec2 f = vec2(atan(c2.z, c2.x) / (2.0 * PI) * 12.0, p.y * 2.5);
                vec2 g = fract(f) - 0.5;
                float rahmen = step(0.42, max(abs(g.x), abs(g.y)));
                alb = mix(vec3(0.25, 0.3, 0.4), vec3(0.8, 0.6, 0.25), rahmen);
                float punkt = smoothstep(0.12, 0.05, length(g));
                eigen += vec3(1.0, 0.8, 0.5) * punkt * (0.6 + 0.6 * kick) * 2.0;
            }
            if (p.y > 0.38 && p.y < 0.42 && ra > 2.5) eigen += vec3(1.0, 0.75, 0.4) * 0.8;
        } else if (m < 5.5) {
            vec3 c2 = p;
            c2.xz = drehe(c2.xz, drehung());
            float streifen = step(0.5, fract(atan(c2.z, c2.x) / (2.0 * PI) * 16.0));
            alb = mix(vec3(0.75, 0.06, 0.06), vec3(0.9, 0.86, 0.8), streifen);
            if (n.y > 0.5 && p.y > 3.5) alb = mix(alb, vec3(0.9, 0.92, 1.0), 0.6);
        } else if (m < 6.5) {
            // Pferde: Schimmel, Rappe, Fuchs, Apfelschimmel; Sattel rot oder blau
            // mit Gold
            float wahl = hash(vec2(bid, 7.0));
            alb = wahl < 0.3 ? vec3(0.92, 0.9, 0.85) : wahl < 0.5 ? vec3(0.08, 0.07, 0.07) : wahl < 0.75 ? vec3(0.5, 0.24, 0.1) : vec3(0.75, 0.75, 0.78) * (0.75 + 0.25 * step(0.5, rausch(p.xz * 30.0 + p.y * 20.0)));
            vec3 c2 = p;
            c2.xz = drehe(c2.xz, drehung());
            float hub = 0.22 * sin(takt * PI + bid * 1.7);
            float sattel = step(abs(p.y - (1.37 + hub)), 0.025);
            vec3 sf = hash(vec2(bid, 2.0)) < 0.5 ? vec3(0.75, 0.05, 0.08) : vec3(0.08, 0.18, 0.7);
            alb = mix(alb, sf, sattel);
            alb += vec3(0.5, 0.35, 0.05) * step(0.88, fract(p.y * 7.0)) * sattel;
            glanz = 0.6;
        } else if (m < 7.5) {
            // Leute: Mantel in gedeckten Farben, Gesicht, bunte Muetze
            float wahl = hash(vec2(personId, 1.0));
            if (teilId < 0.5) {
                alb = wahl < 0.25 ? vec3(0.35, 0.06, 0.06) : wahl < 0.5 ? vec3(0.07, 0.1, 0.25) : wahl < 0.75 ? vec3(0.08, 0.2, 0.12) : vec3(0.25, 0.16, 0.1);
            } else if (teilId < 1.5) {
                alb = vec3(0.85, 0.6, 0.48);
            } else if (teilId < 2.5) {
                float w2 = hash(vec2(personId, 2.0));
                alb = w2 < 0.35 ? vec3(0.8, 0.08, 0.08) : w2 < 0.6 ? vec3(0.9, 0.9, 0.88) : w2 < 0.8 ? vec3(0.1, 0.35, 0.15) : vec3(0.85, 0.6, 0.1);
            } else {
                alb = vec3(0.05, 0.05, 0.06);
            }
        } else if (m < 8.5) {
            alb = vec3(0.4, 0.24, 0.12);
        } else if (m < 9.5) {
            float s2 = mod(bid, 4.0);
            float w = hash(vec2(bid, teilId));
            if (s2 < 0.5) {
                alb = w < 0.5 ? vec3(0.7, 0.08, 0.08) : vec3(0.1, 0.15, 0.55);
                alb = mix(alb, vec3(0.9), step(0.8, fract((p.y + atan(p.z, p.x) * 3.0) * 25.0)) * 0.6);
            } else if (s2 < 1.5) {
                alb = w < 0.3 ? vec3(0.8, 0.05, 0.05) : w < 0.6 ? vec3(0.85, 0.6, 0.15) : w < 0.8 ? vec3(0.75, 0.78, 0.85) : vec3(0.1, 0.4, 0.2);
                glanz = 1.0;
            } else if (s2 < 2.5) {
                alb = w < 0.5 ? vec3(0.9, 0.85, 0.7) : vec3(0.75, 0.1, 0.08);
                eigen += vec3(1.0, 0.6, 0.3) * 0.25;
            } else {
                alb = vec3(0.45, 0.22, 0.08);
            }
        } else if (m < 10.5) {
            alb = vec3(0.03, 0.12, 0.05) * (0.7 + 0.6 * rausch(p.xz * 20.0 + p.y * 15.0));
            if (n.y > 0.6) alb = vec3(0.8, 0.85, 0.95);
        } else if (m < 11.5) {
            // Lebkuchenherz mit Zuckerguss am Rand
            alb = vec3(0.4, 0.18, 0.06);
            float rr2 = rausch(p.yz * 40.0);
            alb = mix(alb, vec3(0.95, 0.9, 0.85), step(0.75, rr2) * 0.8);
        } else {
            alb = vec3(1.0, 0.75, 0.25);
            glanz = 1.0;
            eigen += vec3(1.0, 0.7, 0.3) * 0.25 * (1.0 + kick);
        }
        // Licht: Nachthimmel, das Karussell in der Mitte, die Buden ringsum
        vec3 licht = vec3(0.07, 0.08, 0.14) * (0.6 + 0.4 * n.y);
        vec3 zurMitte = vec3(0.0, 2.9, 0.0) - p;
        float dm = length(zurMitte);
        licht += vec3(1.0, 0.75, 0.42) * max(dot(n, zurMitte / dm), 0.0) * 6.0 / (1.0 + dm * dm * 0.35) * (1.0 + 0.5 * kick);
        float rr = length(p.xz);
        licht += vec3(1.0, 0.6, 0.3) * smoothstep(RING - 4.0, RING - 1.0, rr) * smoothstep(RING + 1.0, RING - 0.8, rr) * 0.7;
        // ein warmer Schimmer von den Buden hinter der Kamera
        licht += vec3(0.9, 0.6, 0.35) * max(dot(n, -rd), 0.0) * 0.22;
        c = alb * licht + eigen;
        if (glanz > 0.0) {
            vec3 r = reflect(rd, n);
            c += vec3(1.0, 0.8, 0.5) * pow(max(dot(r, normalize(vec3(0.0, 2.9, 0.0) - p)), 0.0), 20.0) * glanz * 1.2;
            c += vec3(1.0, 0.9, 0.7) * pow(max(r.y, 0.0), 30.0) * glanz * 0.4;
        }
        c = mix(c, vec3(0.06, 0.06, 0.12), 1.0 - exp(-hit * 0.03));
    } else {
        c = himmel(rd);
    }
    c += glimmen * (1.0 + 0.6 * ton);
    c += dampf(ro, rd, tmax);
    c *= 1.0 + 0.12 * kick;
    c *= 1.0 - 0.32 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.92));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
