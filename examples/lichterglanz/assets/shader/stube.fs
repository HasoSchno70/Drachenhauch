#version 330
// Heiligabend: erst von draussen vor dem Fenster (Schnee auf der Bank, Eis-
// blumen in den Ecken der Scheiben), dann hindurch in die Stube. Dort brennt
// der Kamin, unter dem Baum liegen Geschenke, an der Rueckwand tickt eine
// Uhr ueber einer Kommode mit Schwibbogen. Der Baum entfaltet seine Lagen
// von oben nach unten (`aufbau`), die Lichterkette geht von unten nach oben
// an (`kette`), dann schmueckt er sich zu "Stille Nacht": `kugeln` zaehlt
// die Kugeln (je Ton eine, ueber den Baum gestreut, die juengste geht gerade
// auf). Die Uhr geht auf Mitternacht zu. Gegen Mitternacht wird es dunkler
// (`nacht`), jede Glocke (`glocke`)
// schickt einen Lichtring von der Spitze aus, und mit `stern` geht der Stern
// auf der Spitze auf.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 resolution;
uniform float time;
uniform float takt;
uniform float hell;
uniform float fort;
uniform float kugeln;
uniform float aufbau;
uniform float kette;
uniform float glocke;
uniform float stern;
uniform float nacht;
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
const vec3 BAUM = vec3(1.0, 0.0, 5.2);
const float SPITZE = 3.05;
const float WAND_Z = -2.0;

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float sdKegel(vec3 p, vec2 c, float h) {
    vec2 q = h * vec2(c.x / c.y, -1.0);
    vec2 w = vec2(length(p.xz), p.y);
    vec2 a = w - q * clamp(dot(w, q) / dot(q, q), 0.0, 1.0);
    vec2 b = w - q * vec2(clamp(w.x / q.x, 0.0, 1.0), 1.0);
    float k = sign(q.y);
    float d = min(dot(a, a), dot(b, b));
    float s = max(k * (w.x * q.y - w.y * q.x), k * (w.y - q.y));
    return sqrt(d) * sign(s);
}

// Der Baum sind fuenf Lagen Tanne (Kegel), jede ueberdeckt die obere Haelfte
// der Lage darunter. Radius der Aussenkante in der Hoehe y (ueber dem Fuss):
// die breiteste Lage, die dort liegt -- ein Saegezahn, keine gerade Huelle.
float lageSpitze(float i) { return 1.05 + i * 0.5; }
// Wie weit die Lage i aufgegangen ist: anfangs liegen die Zweige wie ein
// zugeklappter Schirm am Stamm, dann oeffnen sie sich von der Spitze abwaerts
// und federn ein wenig nach.
float lageAuf(float i) {
    float x = clamp((aufbau - (4.0 - i) * 0.14) / 0.42, 0.0, 1.0);
    return 0.12 + 0.88 * (x * x * (3.0 - 2.0 * x) + 0.18 * sin(x * PI) * x);
}
float lageRadius(float i) { return (1.32 - i * 0.21) * lageAuf(i); }
float huelle(float y) {
    float r = 0.0;
    for (int i = 0; i < 5; i++) {
        float u = (lageSpitze(float(i)) - y) / 0.95;
        if (u >= 0.0 && u <= 1.0) r = max(r, lageRadius(float(i)) * u);
    }
    return r;
}

// Die Kugeln haengen in Reihen: je Lage eine am unteren Rand der Zweige und
// eine in der Mitte des sichtbaren Bandes, dazu drei an der Spitze -- 92,
// eine je Ton von "Stille Nacht". Sie sitzen auf der Seite, die die Kamera
// sieht (ein Bogen von 200 Grad), die Zahl je Reihe folgt dem Umfang. Die
// Nummer streut ueber den ganzen Baum (g * 37 mod 92), so fuellt er sich
// gleichmaessig statt von oben nach unten.
const float RY[11] = float[11](0.17, 0.39, 0.67, 0.89, 1.17, 1.39, 1.67, 1.89, 2.17, 2.39, 2.65);
const float RR[11] = float[11](1.222, 0.917, 1.028, 0.771, 0.834, 0.626, 0.639, 0.480, 0.444, 0.334, 0.202);
const float RN[11] = float[11](15.0, 11.0, 13.0, 9.0, 10.0, 8.0, 8.0, 6.0, 5.0, 4.0, 3.0);
const float RO[11] = float[11](0.0, 15.0, 26.0, 39.0, 48.0, 58.0, 66.0, 74.0, 80.0, 85.0, 89.0);
const float VORNE = -1.85;      // die Richtung vom Baum zur Kamera (Winkel in xz)
const float BOGEN = 1.75;       // halbe Breite des behaengten Bogens
const float KUGELN_ALLE = 92.0;

// Winkel a auf den Bogen gelegt: 0 .. 2 BOGEN, ausserhalb ans naechste Ende
float aufBogen(float a) {
    float rel = mod(a - (VORNE - BOGEN), 2.0 * PI);
    if (rel > 2.0 * BOGEN) rel = rel > BOGEN + PI ? 0.0 : 2.0 * BOGEN - 0.001;
    return rel;
}

float kugelNr;
float kugelReihe(vec3 q, float rel, int k, inout float nr, float best) {
    float n = RN[k];
    float w = 2.0 * BOGEN / n;
    float s = clamp(floor(rel / w), 0.0, n - 1.0);
    float g = RO[k] + s;
    float ac = VORNE - BOGEN + (s + 0.5 + 0.3 * (hash(vec2(g, 7.0)) - 0.5)) * w;
    // die Reihe gehoert zu einer Lage und geht mit ihr auf
    float e = lageAuf(float(min(k / 2, 4)));
    float r = RR[k] * e + 0.035;
    vec3 c = vec3(cos(ac) * r, RY[k] - 0.03 * hash(vec2(g, 9.0)), sin(ac) * r);
    float nummer = mod(g * 37.0, KUGELN_ALLE);
    float auf = clamp(kugeln - nummer, 0.0, 1.0);
    // die juengste waechst mit einem kleinen Ueberschwingen
    float gr = auf < 1.0 ? auf * (1.0 + 0.6 * sin(auf * PI)) : 1.0;
    gr *= smoothstep(0.55, 0.92, e);
    float rad = (0.062 + 0.02 * hash(vec2(g, 3.0))) * gr;
    if (rad <= 0.001) return best;
    float d = length(q - c) - rad;
    if (d < best) { nr = nummer; return d; }
    return best;
}

// Die naechste Kugel: aus den zwei Reihen, zwischen denen p liegt -- nicht
// nur aus der eigenen Zelle (sonst schneidet die Zellgrenze die Kugel ab)
float kugel(vec3 p) {
    vec3 q = p - BAUM;
    int k = 0;
    for (int i = 0; i < 10; i++) {
        if (q.y > (RY[i] + RY[i + 1]) * 0.5) k = i + 1;
    }
    int k2 = q.y > RY[k] ? min(k + 1, 10) : max(k - 1, 0);
    float rel = aufBogen(atan(q.z, q.x));
    float best = 1e5;
    best = kugelReihe(q, rel, k, kugelNr, best);
    if (k2 != k) best = kugelReihe(q, rel, k2, kugelNr, best);
    return best;
}

// Die Lichterkette: eine Schraube um den Baum, zehn Birnen je Umlauf, jede
// auf der Aussenkante der Zweige in ihrer Hoehe
float birneD;
float birneId;
float birneAn;
float lichterkette(vec3 p) {
    vec3 q = p - BAUM;
    float windungen = 5.0;
    float H = SPITZE - 0.45;
    float a = atan(q.z, q.x) + PI;
    // zu dieser Richtung gehoerende Hoehen der Schraube: y = SPITZE - (a/2pi + k) H/w
    float schritt = H / windungen;
    float k = floor(((SPITZE - 0.1 - q.y) / schritt) - a / (2.0 * PI) + 0.5);
    float best = 1e5;
    for (int j = -1; j <= 0; j++) {
        float kk = k + float(j);
        float aq = (floor(a / (2.0 * PI) * 10.0) + 0.5) / 10.0 * 2.0 * PI;
        float y = SPITZE - 0.1 - (aq / (2.0 * PI) + kk) * schritt;
        if (y < 0.2 || y > SPITZE - 0.15) continue;
        float r = huelle(y) + 0.03;
        vec3 c = vec3(cos(aq - PI) * r, y, sin(aq - PI) * r);
        float d = length(q - c) - 0.028;
        if (d < best) {
            best = d;
            birneId = kk * 10.0 + floor(a / (2.0 * PI) * 10.0);
            // das Licht laeuft die Kette von unten nach oben hinauf
            float u = (y - 0.2) / (SPITZE - 0.35);
            birneAn = smoothstep(u, u + 0.05, kette * 1.06);
        }
    }
    return best;
}

// Die Geschenke: fuenf auf einem Bogen vor dem Baum, zur Kamera hin, jedes
// etwas gedreht -- keins steckt in den Zweigen
const float GESCHENKE = 5.0;
vec3 geschenkOrt(float i) {
    float w = VORNE + (i - 2.0) * 0.42 + 0.08 * sin(i * 3.1);
    float r = 1.5 + 0.18 * hash(vec2(i, 5.0));
    return BAUM + vec3(cos(w) * r, 0.0, sin(w) * r);
}
vec3 geschenkMass(float i) {
    return vec3(0.15 + 0.08 * hash(vec2(i, 1.0)), 0.1 + 0.1 * hash(vec2(i, 2.0)), 0.15 + 0.06 * hash(vec2(i, 6.0)));
}
// p in die Lage des Geschenks i (Mitte des Bodens, gedreht)
vec3 geschenkLokal(vec3 p, float i) {
    vec3 l = p - geschenkOrt(i);
    float w = hash(vec2(i, 8.0)) * 1.2 - 0.6;
    float c = cos(w), s = sin(w);
    return vec3(c * l.x - s * l.z, l.y, s * l.x + c * l.z);
}

// Das Fenster oeffnet sich nach innen, bevor die Kamera hindurchfaehrt: zwei
// Fluegel, je an der Aussenkante angeschlagen (frueher fuhr sie mitten durch
// die Sprosse).
float oeffnen() { return smoothstep(0.075, 0.15, fort); }
float fluegel(vec3 p) {
    // beide Fluegel aus einem: x gespiegelt, Angel bei x = 0.97
    vec3 q = vec3(abs(p.x), p.y, p.z) - vec3(0.97, 1.55, WAND_Z - 0.05);
    float phi = oeffnen() * 1.45;
    float c = cos(phi), s = sin(phi);
    vec3 l = vec3(c * q.x - s * q.z, q.y, s * q.x + c * q.z);
    l.x += 0.485;
    float d = max(sdBox(l, vec3(0.485, 0.77, 0.03)), -sdBox(l, vec3(0.445, 0.73, 0.1)));
    d = min(d, sdBox(l - vec3(0.0, 0.12, 0.0), vec3(0.485, 0.03, 0.025)));
    return d;
}

// An der Rueckwand links vom Baum: eine Kommode, darauf ein Schwibbogen mit
// sieben Kerzen und einem ausgesaegten Dorf, darueber eine Wanduhr, deren
// Pendel auf jedem Schlag ausschlaegt.
const vec3 KOMMODE = vec3(-1.6, 0.0, 7.7);
const vec3 UHR = vec3(-1.6, 2.2, 7.92);
float sdDreieck(vec2 p, vec2 q) {
    // gleichschenklig, Spitze im Ursprung, Grundseite bei y = q.y (< 0)
    p.x = abs(p.x);
    vec2 a = p - q * clamp(dot(p, q) / dot(q, q), 0.0, 1.0);
    vec2 b = p - q * vec2(clamp(p.x / q.x, 0.0, 1.0), 1.0);
    float s = -sign(q.y);
    vec2 d = min(vec2(dot(a, a), s * (p.x * q.y - p.y * q.x)), vec2(dot(b, b), s * (p.y - q.y)));
    return -sqrt(d.x) * sign(d.y);
}
float pendelWinkel() { return 0.16 * sin(takt * 4.0 * PI); }
float flammeD;
float moebel(vec3 p, inout float m) {
    float d = 1e5;
    flammeD = 1e5;
    // Kommode mit Deckplatte
    vec3 k = p - KOMMODE;
    float kom = sdBox(k - vec3(0.0, 0.4, 0.0), vec3(0.72, 0.4, 0.28)) - 0.01;
    kom = min(kom, sdBox(k - vec3(0.0, 0.81, -0.01), vec3(0.76, 0.015, 0.3)));
    if (kom < d) { d = kom; m = 11.0; }
    // der Schwibbogen: ein halber Ring auf einer Leiste
    vec3 q = p - vec3(KOMMODE.x, 0.84, KOMMODE.z + 0.02);
    float bogen = max(length(vec2(length(q.xy) - 0.42, q.z)) - 0.02, -q.y);
    bogen = min(bogen, sdBox(q - vec3(0.0, 0.012, 0.0), vec3(0.46, 0.014, 0.04)));
    // darin ausgesaegt: Kirche und Tannen als Schattenriss
    float riss = sdBox(vec3(q.xy - vec2(0.0, 0.07), 0.0), vec3(0.07, 0.06, 1.0));
    riss = min(riss, sdDreieck(q.xy - vec2(0.0, 0.2), vec2(0.085, -0.07)));
    riss = min(riss, sdBox(vec3(q.xy - vec2(0.0, 0.2), 0.0), vec3(0.02, 0.07, 1.0)));
    riss = min(riss, sdDreieck(q.xy - vec2(0.0, 0.33), vec2(0.03, -0.07)));
    riss = min(riss, sdDreieck(vec2(abs(q.x) - 0.2, q.y - 0.24), vec2(0.07, -0.23)));
    riss = min(riss, sdDreieck(vec2(abs(q.x) - 0.31, q.y - 0.16), vec2(0.05, -0.15)));
    bogen = min(bogen, max(riss, abs(q.z) - 0.006));
    if (bogen < d) { d = bogen; m = 11.0; }
    // sieben Kerzen auf dem Bogen: die naechste aus dem Winkel
    float w = clamp(floor(atan(max(q.y, 0.0), q.x) / (PI / 8.0) + 0.5), 1.0, 7.0) * PI / 8.0;
    vec3 kq = q - vec3(cos(w) * 0.42, sin(w) * 0.42 + 0.02, 0.0);
    float kerze = max(length(kq.xz) - 0.011, abs(kq.y - 0.035) - 0.035);
    if (kerze < d) { d = kerze; m = 14.0; }
    flammeD = length((kq - vec3(0.0, 0.085, 0.0)) / vec3(1.0, 1.8, 1.0)) * 0.55 - 0.007;
    // die Wanduhr: Gehaeuse, Zifferblatt, Pendel
    vec3 u = p - UHR;
    float geh = sdBox(u - vec3(0.0, -0.25, 0.0), vec3(0.24, 0.55, 0.08)) - 0.01;
    geh = min(geh, max(sdDreieck(u.xy - vec2(0.0, 0.47), vec2(0.3, -0.16)), abs(u.z) - 0.07));
    if (geh < d) { d = geh; m = 11.0; }
    float blatt = max(length(u.xy) - 0.19, abs(u.z + 0.09) - 0.012);
    if (blatt < d) { d = blatt; m = 12.0; }
    vec2 ende = vec2(sin(pendelWinkel()), -cos(pendelWinkel())) * 0.42;
    vec2 pa = u.xy - vec2(0.0, -0.27);
    float h = clamp(dot(pa, ende) / dot(ende, ende), 0.0, 1.0);
    float pendel = max(length(pa - ende * h) - 0.008, abs(u.z + 0.1) - 0.005);
    pendel = min(pendel, max(length(pa - ende) - 0.055, abs(u.z + 0.1) - 0.01));
    if (pendel < d) { d = pendel; m = 13.0; }
    return d;
}

// Material: 0 Boden, 1 Wand, 2 Decke, 3 Baum, 4 Stamm, 5 Kugel, 6 Geschenk,
// 7 Kamin, 8 Fensterrahmen, 9 Schnee aussen, 10 Teppich, 11 dunkles Holz,
// 12 Zifferblatt, 13 Messing, 14 Kerzenwachs
float mat;
float geschenkId;
float szene(vec3 p) {
    // der Raum: eine hohle Kiste, die Fensterwand mit Oeffnung
    // Waende, Boden und Decke als Schale von 0.3 Dicke, die Fensterwand mit
    // ihrer Oeffnung -- von innen wie von aussen richtig
    vec3 rp = p - vec3(0.0, 1.6, 3.0);
    float raum = max(sdBox(rp, vec3(4.3, 1.9, 5.3)), -sdBox(rp, vec3(4.0, 1.6, 5.0)));
    float loch = sdBox(p - vec3(0.0, 1.55, WAND_Z - 0.15), vec3(1.05, 0.85, 0.3));
    raum = max(raum, -loch);
    float d = raum;
    mat = p.y < 0.01 && p.z > WAND_Z ? 0.0 : (p.y > 3.19 ? 2.0 : 1.0);
    // Fensterrahmen in der Scheibenebene, die Fluegel darin
    vec3 f = p - vec3(0.0, 1.55, WAND_Z - 0.05);
    float rahmen = max(sdBox(f, vec3(1.05, 0.85, 0.05)), -sdBox(f, vec3(0.97, 0.77, 0.1)));
    rahmen = min(rahmen, fluegel(p));
    if (rahmen < d) { d = rahmen; mat = 8.0; }
    // aussen: Schnee auf der Fensterbank und am Boden vor dem Haus
    float bank = sdBox(p - vec3(0.0, 0.66, WAND_Z - 0.42), vec3(1.2, 0.04, 0.14)) - 0.03 - 0.02 * rausch(p.xz * 9.0);
    if (bank < d) { d = bank; mat = 9.0; }
    float draussen = max(p.y + 0.4 - 0.1 * rausch(p.xz * 0.8), p.z - (WAND_Z - 0.3));
    if (draussen < d) { d = draussen; mat = 9.0; }
    // Teppich vor dem Baum
    float teppich = max(length((p.xz - vec2(0.3, 4.2)) / vec2(2.0, 1.4)) - 1.0, p.y - 0.012);
    if (teppich < d) { d = teppich; mat = 10.0; }
    // der Baum: fuenf Lagen Tanne
    // (ausserhalb der Huelle zaehlt nur der Abstand zu ihr: so tritt der Strahl
    // nie ueber Zweige hinweg und meldet doch keinen Treffer an der Huelle)
    vec3 q = p - BAUM;
    float baumHuelle = max(length(q.xz) - 1.6, q.y - SPITZE - 0.2);
    if (baumHuelle < 0.1) {
        float tanne = 1e5;
        for (int i = 0; i < 5; i++) {
            float fi = float(i);
            float k = sdKegel(q - vec3(0.0, lageSpitze(fi), 0.0), normalize(vec2(lageRadius(fi), 0.95)), 0.95);
            tanne = min(tanne, k);
        }
        // Zweige: zerzauster Rand
        float w = atan(q.z, q.x);
        tanne -= 0.04 * sin(w * 22.0 + q.y * 14.0) * sin(q.y * 31.0);
        float stamm = max(length(q.xz) - 0.08, q.y - 0.4);
        if (tanne < d) { d = tanne; mat = 3.0; }
        if (stamm < d) { d = stamm; mat = 4.0; }
        float kg = kugel(p);
        if (kg < d) { d = kg; mat = 5.0; }
        birneD = lichterkette(p);
    } else {
        birneD = 1e5;
        birneAn = 0.0;
        d = min(d, baumHuelle);
    }
    // Kommode, Schwibbogen und Uhr (nur in ihrer Naehe gerechnet)
    float mHuelle = max(abs(p.x - KOMMODE.x) - 0.95, max(KOMMODE.z - 0.45 - p.z, p.y - 2.85));
    if (mHuelle < 0.1) {
        float mm = mat;
        float md = moebel(p, mm);
        if (md < d) { d = md; mat = mm; }
        if (flammeD < birneD) { birneD = flammeD; birneId = 99.0; birneAn = 1.0; }
    } else {
        d = min(d, mHuelle);
    }
    d = min(d, birneD + 0.01);
    // Geschenke vor dem Baum, mit einer Schleife obenauf
    float gHuelle = max(length(p.xz - (BAUM.xz + vec2(cos(VORNE), sin(VORNE)) * 1.55)) - 1.7, p.y - 0.6);
    if (gHuelle < 0.1) {
        for (int i = 0; i < 5; i++) {
            float fi = float(i);
            vec3 l = geschenkLokal(p, fi);
            vec3 gg = geschenkMass(fi);
            float gs = sdBox(l - vec3(0.0, gg.y, 0.0), gg) - 0.01;
            vec3 sl = l - vec3(0.0, 2.0 * gg.y + 0.025, 0.0);
            sl.x = abs(sl.x) - 0.04;
            gs = min(gs, (length(sl / vec3(1.0, 0.55, 0.45)) - 0.045) * 0.45);
            if (gs < d) { d = gs; mat = 6.0; geschenkId = fi; }
        }
    } else {
        d = min(d, gHuelle);
    }
    // der Kamin an der linken Wand
    vec3 kp = p - vec3(-3.7, 0.0, 3.5);
    float kamin = sdBox(kp - vec3(0.0, 0.75, 0.0), vec3(0.35, 0.75, 0.95));
    kamin = max(kamin, -sdBox(kp - vec3(0.35, 0.45, 0.0), vec3(0.25, 0.4, 0.55)));
    kamin = min(kamin, sdBox(kp - vec3(0.05, 1.55, 0.0), vec3(0.45, 0.06, 1.1)));
    if (kamin < d) { d = kamin; mat = 7.0; }
    return d;
}

// Ein Stern mit fuenf Zacken (2D, Spitze nach oben), r aussen, rf innen/aussen
float sdStern5(vec2 p, float r, float rf) {
    const vec2 k1 = vec2(0.809016994375, -0.587785252292);
    const vec2 k2 = vec2(-0.809016994375, -0.587785252292);
    p.x = abs(p.x);
    p -= 2.0 * max(dot(k1, p), 0.0) * k1;
    p -= 2.0 * max(dot(k2, p), 0.0) * k2;
    p.x = abs(p.x);
    p.y -= r;
    vec2 ba = rf * vec2(-k1.y, k1.x) - vec2(0.0, 1.0);
    float h = clamp(dot(p, ba) / dot(ba, ba), 0.0, r);
    return length(p - ba * h) * sign(p.y * ba.x - p.x * ba.y);
}

vec3 normale(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(szene(p + e.xyy) - szene(p - e.xyy), szene(p + e.yxy) - szene(p - e.yxy), szene(p + e.yyx) - szene(p - e.yyx)));
}

// Das Feuer im Kamin: Flammen aus Rauschen, die nach oben ziehen
vec3 feuer(vec3 p) {
    vec3 kp = p - vec3(-3.7, 0.0, 3.5);
    if (kp.x < 0.05 || kp.x > 0.45 || abs(kp.z) > 0.55 || kp.y > 0.85) return vec3(0.0);
    vec2 q = vec2(kp.z, kp.y);
    float n = rausch(vec2(q.x * 6.0, q.y * 4.0 - time * 3.0)) * 0.6 + rausch(vec2(q.x * 13.0, q.y * 9.0 - time * 5.0)) * 0.4;
    float form = smoothstep(0.7, 0.05, q.y + abs(q.x) * 0.9) * n;
    vec3 c = mix(vec3(0.9, 0.2, 0.02), vec3(1.0, 0.85, 0.4), smoothstep(0.3, 0.8, form));
    return c * smoothstep(0.2, 0.55, form) * 3.0;
}

vec3 kaminLicht() {
    return vec3(1.0, 0.5, 0.18) * (0.85 + 0.15 * sin(time * 9.0) * sin(time * 4.3));
}

// draussen: Nachthimmel mit Sternen und dem Dorf im Hintergrund
vec3 draussenHimmel(vec3 d) {
    vec3 c = mix(vec3(0.05, 0.07, 0.15), vec3(0.01, 0.015, 0.04), clamp(d.y * 2.0, 0.0, 1.0));
    vec2 uv = vec2(atan(d.z, d.x), asin(clamp(d.y, -1.0, 1.0))) * 150.0;
    vec2 z = floor(uv);
    c += vec3(0.85, 0.9, 1.0) * step(0.985, hash(z)) * smoothstep(0.35, 0.0, length(fract(uv) - 0.5));
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
    // Kamera: vor dem Fenster, dann hindurch und auf den Baum zu; zum Stern
    // hebt sie den Blick
    float f1 = smoothstep(0.0, 0.3, fort);
    float f2 = smoothstep(0.25, 1.0, fort);
    vec3 ro = mix(vec3(0.0, 1.5, WAND_Z - 2.6), vec3(0.0, 1.55, WAND_Z + 0.6), f1);
    // (so weit vom Baum, dass Geschenke und Stern im Bild bleiben)
    ro = mix(ro, vec3(-0.7, 1.6, 0.5), f2);
    vec3 ta = mix(vec3(0.3, 1.45, 5.0), BAUM + vec3(0.0, 1.55, 0.0), f2);
    ta.y += stern * 0.35 + 0.1 * smoothstep(0.85, 0.95, fort);
    vec3 vorne = normalize(ta - ro);
    vec3 rechts = normalize(cross(vec3(0, 1, 0), vorne));
    vec3 oben = cross(vorne, rechts);
    vec3 rd = normalize(vorne * 1.3 + uv.x * rechts + uv.y * oben);

    // Wie hell die Stube ist: gegen Mitternacht nur noch Baum und Kamin
    float raumLicht = mix(1.0, 0.35, nacht);

    vec3 c;
    float t = 0.0;
    float hit = -1.0;
    vec3 glimmen = vec3(0.0);
    int schritte = int(120.0 * G());
    for (int i = 0; i < 120; i++) {
        if (i >= schritte) break;
        vec3 p = ro + rd * t;
        float d = szene(p);
        float bd = max(birneD, 0.0);
        glimmen += vec3(1.0, 0.78, 0.45) * 0.0016 / (0.0003 + bd * bd) * min(d, 0.2) * (0.7 + 0.3 * sin(time * 3.0 + birneId * 1.7)) * birneAn;
        if (d < 0.001 * t + 0.0005) { hit = t; break; }
        t += d * 0.8;
        if (t > 30.0) break;
    }
    if (hit > 0.0) {
        vec3 p = ro + rd * hit;
        szene(p);
        float m = mat;
        float kn = kugelNr;
        float gid = geschenkId;
        vec3 n = normale(p);
        vec3 alb;
        vec3 eigen = vec3(0.0);
        float glanz = 0.0;
        if (birneD < 0.004) {
            alb = vec3(0.05, 0.05, 0.04);
            eigen = vec3(1.0, 0.8, 0.5) * 3.0 * birneAn;
        } else if (m < 0.5) {
            // Dielen
            float diele = fract(p.x * 2.2);
            alb = mix(vec3(0.22, 0.11, 0.05), vec3(0.3, 0.16, 0.07), rausch(vec2(floor(p.x * 2.2), p.z * 0.6)));
            alb *= 0.75 + 0.25 * smoothstep(0.0, 0.05, diele);
            glanz = 0.25;
        } else if (m < 1.5) {
            // Tapete mit kleinem Muster; aussen weisser Putz
            if (p.z < WAND_Z - 0.2) {
                alb = vec3(0.7, 0.66, 0.6);
            } else if (p.y < 0.95) {
                // unten eine Holzvertaefelung mit Feldern und Abschlussleiste
                vec2 tp = vec2(p.x + p.z, p.y);
                float feld = smoothstep(0.0, 0.02, abs(fract(tp.x * 1.4) - 0.5) - 0.44) + smoothstep(0.0, 0.02, abs(tp.y - 0.47) - 0.38);
                alb = vec3(0.26, 0.13, 0.06) * (0.8 + 0.2 * rausch(vec2(tp.x * 3.0, tp.y * 40.0)));
                alb *= 1.0 - 0.3 * clamp(feld, 0.0, 1.0);
                alb = mix(alb, vec3(0.33, 0.18, 0.08), step(0.9, p.y));
            } else {
                vec2 tp = vec2(p.x + p.z, p.y) * 5.0;
                float muster = smoothstep(0.12, 0.08, length(fract(tp) - 0.5));
                alb = mix(vec3(0.42, 0.12, 0.1), vec3(0.6, 0.45, 0.25), muster * 0.5);
            }
        } else if (m < 2.5) {
            alb = vec3(0.4, 0.35, 0.3);
        } else if (m < 3.5) {
            alb = mix(vec3(0.01, 0.06, 0.025), vec3(0.06, 0.18, 0.07), rausch(p.xz * 40.0 + p.y * 20.0));
        } else if (m < 4.5) {
            alb = vec3(0.15, 0.08, 0.04);
        } else if (m < 5.5) {
            // Kugeln: rot, gold, silber, blau -- spiegelnd
            float wahl = hash(vec2(kn, 4.0));
            alb = wahl < 0.35 ? vec3(0.7, 0.03, 0.04) : wahl < 0.65 ? vec3(0.85, 0.6, 0.15) : wahl < 0.85 ? vec3(0.75, 0.78, 0.82) : vec3(0.1, 0.25, 0.75);
            glanz = 1.0;
            // die juengste leuchtet kurz auf
            float neu = clamp(1.0 - (kugeln - kn - 1.0), 0.0, 1.0);
            eigen += alb * neu * 1.5;
        } else if (m < 6.5) {
            float wahl = gid;
            vec3 papier = wahl < 0.5 ? vec3(0.7, 0.05, 0.05) : wahl < 1.5 ? vec3(0.1, 0.3, 0.6) : wahl < 2.5 ? vec3(0.8, 0.65, 0.2) : vec3(0.1, 0.45, 0.2);
            vec3 gp = geschenkLokal(p, gid);
            float band = step(abs(gp.x), 0.025) + step(abs(gp.z), 0.025) + step(2.0 * geschenkMass(gid).y + 0.005, gp.y);
            alb = mix(papier, vec3(0.9, 0.8, 0.5), clamp(band, 0.0, 1.0));
        } else if (m < 7.5) {
            alb = vec3(0.45, 0.2, 0.15) * (0.8 + 0.2 * step(0.5, fract(p.y * 6.0 + step(0.5, fract(p.z * 3.0)) * 0.5)));
        } else if (m < 8.5) {
            alb = vec3(0.12, 0.06, 0.03);
        } else if (m < 9.5) {
            alb = vec3(0.8, 0.85, 0.95);
        } else if (m < 10.5) {
            alb = vec3(0.45, 0.06, 0.06) * (0.8 + 0.2 * sin(length(p.xz - vec2(0.3, 4.2)) * 30.0));
        } else if (m < 11.5) {
            alb = vec3(0.2, 0.1, 0.05) * (0.85 + 0.15 * rausch(vec2(p.x * 30.0, p.y * 4.0)));
            glanz = 0.15;
        } else if (m < 12.5) {
            // das Zifferblatt: Striche fuer die Stunden, zwei Zeiger. Die Uhr
            // geht von zehn vor zwoelf auf Mitternacht, zu jedem Glockenschlag
            // leuchtet das Blatt auf.
            vec2 z = p.xy - UHR.xy;
            float r = length(z);
            float wz = atan(z.x, z.y);
            float stunde = abs(mod(wz + PI / 12.0, PI / 6.0) - PI / 12.0) * r;
            float striche = (1.0 - smoothstep(0.004, 0.007, stunde)) * step(0.14, r) * step(r, 0.175);
            float minuten = 50.0 + 10.0 * clamp((takt - 78.0) / 18.0, 0.0, 1.0);
            float wm = minuten / 60.0 * 2.0 * PI;
            float wh = (11.0 + minuten / 60.0) / 12.0 * 2.0 * PI;
            vec2 em = vec2(sin(wm), cos(wm));
            vec2 eh = vec2(sin(wh), cos(wh));
            float zm = abs(z.x * em.y - z.y * em.x) - 0.004;
            zm = max(zm, max(-dot(z, em) - 0.02, dot(z, em) - 0.155));
            float zh = abs(z.x * eh.y - z.y * eh.x) - 0.007;
            zh = max(zh, max(-dot(z, eh) - 0.02, dot(z, eh) - 0.1));
            float zeiger = 1.0 - smoothstep(0.0, 0.003, min(zm, zh));
            alb = vec3(0.85, 0.8, 0.68) * (1.0 - 0.9 * max(striche, zeiger));
            alb = mix(alb, vec3(0.6, 0.45, 0.2), smoothstep(0.17, 0.185, r));
            eigen += vec3(1.0, 0.85, 0.6) * glocke * 0.5 * (1.0 - max(striche, zeiger));
        } else if (m < 13.5) {
            alb = vec3(0.8, 0.6, 0.25);
            glanz = 0.8;
        } else {
            alb = vec3(0.85, 0.8, 0.7);
            eigen += vec3(1.0, 0.7, 0.4) * 0.15;
        }
        // Licht: Kamin, Baum (Lichterkette), Deckenlampe; draussen Mondlicht
        vec3 licht;
        if (p.z < WAND_Z - 0.1) {
            licht = vec3(0.2, 0.25, 0.4) * (0.5 + 0.5 * n.y) + vec3(0.9, 0.6, 0.3) * 0.9 * exp(-length(p.xy - vec2(0.0, 1.5)) * 1.0);
        } else {
            vec3 kl = vec3(-3.3, 0.6, 3.5) - p;
            float kd = length(kl);
            licht = kaminLicht() * max(dot(n, kl / kd), 0.0) * 4.0 / (1.0 + kd * kd * 0.5);
            vec3 bl = BAUM + vec3(0.0, 1.5, 0.0) - p;
            float bd = length(bl);
            licht += vec3(1.0, 0.75, 0.42) * (max(dot(n, bl / bd), 0.0) * 0.7 + 0.3) * 2.2 / (1.0 + bd * bd * 0.45) * (0.3 + 0.7 * kette);
            // die Kerzen im Schwibbogen
            vec3 swl = vec3(KOMMODE.x, 1.15, KOMMODE.z - 0.15) - p;
            float swd = length(swl);
            licht += vec3(1.0, 0.7, 0.35) * (max(dot(n, swl / swd), 0.0) * 0.8 + 0.2) * 0.9 / (1.0 + swd * swd * 2.0);
            licht += vec3(0.9, 0.75, 0.55) * (0.15 + 0.15 * n.y) * raumLicht;
            // der Stern auf der Spitze strahlt in den Raum
            vec3 sl = BAUM + vec3(0.0, SPITZE + 0.15, 0.0) - p;
            float sd = length(sl);
            licht += vec3(1.0, 0.9, 0.6) * stern * max(dot(n, sl / sd), 0.0) * 3.0 / (1.0 + sd * sd * 0.3);
            // die Glocken: ein Hauch Licht, der mit jedem Schlag durch den Raum geht
            licht += vec3(1.0, 0.8, 0.5) * glocke * 0.25;
        }
        c = alb * licht + eigen;
        if (glanz > 0.0) {
            vec3 r = reflect(rd, n);
            vec3 zumBaum = normalize(BAUM + vec3(0.0, 1.5, 0.0) - p);
            vec3 zumKamin = normalize(vec3(-3.3, 0.6, 3.5) - p);
            c += vec3(1.0, 0.8, 0.5) * pow(max(dot(r, zumKamin), 0.0), 30.0) * glanz * 1.5;
            c += vec3(1.0, 0.85, 0.6) * pow(max(dot(r, -zumBaum), 0.0), 8.0) * glanz * 0.4;
            if (m > 4.5 && m < 5.5) c += vec3(1.0, 0.95, 0.85) * pow(max(dot(r, normalize(vec3(0.2, 1.0, -0.3))), 0.0), 60.0);
        }
        c += feuer(p);
    } else {
        c = draussenHimmel(rd);
    }
    // die Scheibe: wer davor steht, sieht Spiegelung und Eisblumen in den Ecken
    float tg = (WAND_Z - 0.05 - ro.z) / rd.z;
    float zu = pow(1.0 - oeffnen(), 3.0);
    if (zu > 0.0 && tg > 0.0 && (hit < 0.0 || tg < hit)) {
        vec3 gp = ro + rd * tg;
        vec2 s = gp.xy - vec2(0.0, 1.55);
        if (abs(s.x) < 1.0 && abs(s.y) < 0.8) {
            vec2 scheibe = vec2(abs(s.x) - 0.5, s.y - (s.y > 0.12 ? 0.46 : -0.33));
            float ecke = length(abs(scheibe) - vec2(0.48, 0.33 + (s.y > 0.12 ? -0.0 : 0.12)));
            float eis = smoothstep(0.35, 0.05, ecke + 0.12 * rausch(gp.xy * 14.0)) * (0.6 + 0.4 * rausch(gp.xy * 40.0));
            c = mix(c, vec3(0.7, 0.8, 0.95), eis * 0.55 * zu);
            c += vec3(0.6, 0.7, 0.9) * 0.04 * zu;
        }
    }
    c += glimmen * (1.0 + 0.5 * ton);
    // der Stern auf der Spitze, im Bild gezeichnet: fuenf scharfe Zacken mit
    // je einer hellen und einer dunklen Facette, die sich beim Wiegen tauschen;
    // vor dem Aufgehen mattes Gold, danach hell mit Strahlen
    vec3 sp = BAUM + vec3(0.0, SPITZE + 0.15, 0.0) - ro;
    float sz = dot(sp, vorne);
    if (sz > 0.1) {
        vec2 su = vec2(dot(sp, rechts), dot(sp, oben)) / sz * 1.3;
        vec2 q = uv - su;
        float r = length(q);
        float a = atan(q.y, q.x) + PI * 0.5;
        float R = 0.16 / sz;
        float wiegen = sin(time * 0.9);
        vec2 qs = vec2(q.x / (0.82 + 0.18 * cos(time * 0.9)), q.y);
        float sd = sdStern5(qs, R, 0.42);
        float px = 1.5 / resolution.y;
        float form = smoothstep(px, -px, sd);
        float sek = mod(atan(qs.x, qs.y) + PI / 5.0, 2.0 * PI / 5.0) - PI / 5.0;
        float facette = 0.78 + 0.3 * sign(sek) * (0.6 + 0.4 * wiegen);
        facette *= 0.8 + 0.45 * (1.0 - clamp(length(qs) / R, 0.0, 1.0));
        vec3 gold = mix(vec3(0.5, 0.36, 0.12), vec3(1.0, 0.82, 0.42) * (1.0 + stern * 3.0), 0.25 + 0.75 * stern) * facette;
        gold *= 1.0 - 0.45 * smoothstep(-px * 3.0, 0.0, sd);
        c = mix(c, gold, form);
        float strahl = pow(abs(cos(a * 2.0)), 50.0) + pow(abs(cos(a * 2.0 + 0.785)), 70.0) * 0.5;
        c += vec3(1.0, 0.88, 0.6) * stern * (exp(-r * 30.0) * 2.0 + strahl * exp(-r * 4.0) * 1.0 + exp(-r * 2.5) * 0.3) * (1.0 + 0.3 * ton);
        // Mitternacht: ein Lichtring von der Spitze aus je Glockenschlag
        float ring = (1.0 - glocke) * 1.4;
        c += vec3(1.0, 0.85, 0.55) * glocke * exp(-pow((r - ring) * 14.0, 2.0)) * 0.6;
    }
    c *= 1.0 - 0.35 * dot(uv * 0.9, uv * 0.9);
    c += (hash(fragTexCoord * resolution + time) - 0.5) * 0.012;
    c = c / (1.0 + c);
    c = pow(c, vec3(0.92));
    c = uebergang(c, uv);
    vec3 oberlage = texture(texture0, fragTexCoord).rgb;
    finalColor = vec4(c * hell + oberlage, 1.0);
}
