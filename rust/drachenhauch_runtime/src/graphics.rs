//! raylib-Grafik-Backend (Schritt 4). Nur mit `--features graphics`.
//!
//! Modell: Draw-Builtins zeichnen NICHT sofort, sondern haengen ein `Cmd`
//! an eine Liste. `CLS` leert die Liste + merkt die Clear-Farbe. `FLIP`
//! rendert alle Cmds in einem `begin_drawing`/`end_drawing`-Block und
//! praesentiert. Das vermeidet, raylibs Draw-Handle ueber Builtin-Aufrufe
//! hinweg zu halten (Borrow-Checker) und braucht keine Render-Texture.
//!
//! Pixel-Output ist renderer-abhaengig (raylib/GPU) und nicht golden-testbar
//! -- verifiziert wird per Headless-Screenshot (DHRT_FRAMES/DHRT_SCREENSHOT);
//! deterministisch testbar ist nur `PRINT`/stdout.

use std::collections::HashMap;
use std::rc::Rc;

use raylib::prelude::*;

// Web (emscripten): yieldet ans Browser-Event-Loop. Mit `-s ASYNCIFY` (vom
// build_wasm.py gesetzt) wickelt das den kompletten Rust-Stack ab und setzt ihn
// beim naechsten Tick fort -- so kooperiert der blockierende GB-Render-Loop
// (`WHILE ... FLIP() ... WEND`) mit dem Browser, statt den Tab einzufrieren.
#[cfg(target_os = "emscripten")]
extern "C" {
    fn emscripten_sleep(ms: std::os::raw::c_uint);
}
use raylib::core::shaders::RaylibShader;   // get_shader_location auf Shader
use raylib::core::texture::RaylibRenderTexture2D;   // .texture() auf RenderTexture2D

#[derive(Clone)]
enum Cmd {
    Clear(Color),
    Pixel(i32, i32, Color),
    Line(i32, i32, i32, i32, Color),
    BoxFill(i32, i32, i32, i32, Color),
    RectLines(i32, i32, i32, i32, Color),
    Circle(i32, i32, f32, Color),
    Triangle(i32, i32, i32, i32, i32, i32, Color),
    TriLines(i32, i32, i32, i32, i32, i32, Color),
    Ellipse(i32, i32, i32, i32, Color, bool), // cx, cy, rh, rv, color, filled
    Poly(Vec<(i32, i32)>, f32, Color, bool),  // points, thick, color, closed
    FillPoly(Vec<(i32, i32)>, Color),
    // x, y, text, size, color, font_idx (-1 = Default), spacing
    Text(i32, i32, String, i32, Color, i64, f32),
    // TEXTROT: (cx, cy, text, groesse, farbe, font, spacing, winkel_grad, skala)
    TextRot(i32, i32, String, i32, Color, i64, f32, f32, f32),
    // Review-Fund: Texture/TexturePart/TextureFlipped/AtlasDraw hatten keine
    // vom cam_zoom unabhaengige Ziel-Groesse -- die Ziel-Rechtecke wurden beim
    // Replay entweder aus der Quell-Groesse (sw,sh) uebernommen oder aus der
    // rohen Textur-Breite/-Hoehe gelesen, in beiden Faellen NUR mit dem
    // SCREEN()-Skalierungsfaktor `s` multipliziert, nie mit `cam_zoom`
    // (CAMERA_SET-Zoom). TextureRect/SpriteDraw machten es schon richtig
    // (ssize()/cam_zoom-Multiplikation beim EMIT, bevor der Cmd gebaut wird)
    // -- jetzt tragen auch diese Varianten eine explizite, bereits
    // zoom-skalierte Ziel-Groesse.
    Texture(usize, i32, i32, i32, i32),               // tex, x, y, dw, dh (zoom-skaliert)
    TexturePart(usize, i32, i32, i32, i32, i32, i32, i32, i32), // tex, sx,sy,sw,sh, dx,dy, dw,dh
    TexturePartEx(usize, i32, i32, i32, i32, i32, i32, i32, i32), // +dw,dh (skaliert)
    TextureRect(usize, i32, i32, i32, i32),           // tex skaliert in dx,dy,dw,dh (bounds-safe)
    TextureFlipped(usize, i32, i32, i32, i32, bool, bool), // tex, x, y, dw, dh, flip_h, flip_v
    TextureRot(usize, i32, i32, f32, f32, Color),      // tex, cx, cy, winkel_grad, skala (inkl. cam_zoom), tint (um Zentrum)
    AtlasDraw(usize, i32, i32, i32, i32, i32, i32, i32, i32, bool, bool, Color), // tex, sx,sy,sw,sh, dx,dy, dw,dh, flip_h, flip_v, tint
    // tex, src(sx,sy,sw,sh), dst(dx,dy,dw,dh), flip_x, flip_y, tint
    SpriteDraw(usize, i32, i32, i32, i32, i32, i32, i32, i32, bool, bool, Color),
    // Clip-Stack (Scissor): Push schneidet mit dem aktuellen Clip, Pop stellt
    // den vorigen wieder her. Koordinaten logisch (pre-scale), beim Replay * s.
    ScissorPush(i32, i32, i32, i32),
    ScissorPop,
    // 2D-Extras (Batch 1): dicke Linien, runde Rechtecke, Gradienten, Splines.
    LineEx(i32, i32, i32, i32, f32, Color),            // x1,y1,x2,y2, thick, color
    RoundRect(i32, i32, i32, i32, i32, Color, bool),   // x1,y1,x2,y2, radius, color, filled
    GradientRect(i32, i32, i32, i32, Color, Color, bool), // x1,y1,x2,y2, c1, c2, vertical
    Spline(Vec<(i32, i32)>, f32, Color),               // points, thick, color
    /// Rundes Rechteck mit SENKRECHTEM Verlauf: x1,y1,x2,y2, Eckenradius,
    /// Farbe oben, Farbe unten.
    ///
    /// Der Baustein fuer alles, was plastisch aussehen soll (Knoepfe, Leisten,
    /// Felder). `RoundRect` + `GradientRect` uebereinander geht NICHT sauber:
    /// der Verlauf ist rechteckig und muesste um den Eckenradius eingerueckt
    /// werden, was bei kleinen Widgets als Rand sichtbar bleibt. Hier wird
    /// stattdessen zeilenweise gefuellt und der Eckeneinzug aus dem Radius
    /// gerechnet -- eine Zeichenanweisung, die Schleife laeuft beim Abspielen.
    /// Alpha wird mitinterpoliert, damit auch Glanzkanten (weiss -> unsichtbar)
    /// damit gehen.
    RoundGradient(i32, i32, i32, i32, i32, Color, Color),
    /// Kreisring-Ausschnitt: cx, cy, r_innen, r_aussen, winkel_von, winkel_bis,
    /// Farbe, gefuellt. Deckt Kuchenstueck (r_innen = 0), Donut-Segment und
    /// Tacho-Bogen mit EINER Variante ab -- raylibs `draw_ring` kann alle drei.
    /// Winkel in Grad, 0 = rechts, wachsend im Uhrzeigersinn (Bildschirm-y).
    Ring(i32, i32, f32, f32, f32, f32, Color, bool),
    BlendMode(i32),                                    // 0=alpha,1=additive,2=multiplied,4=subtract
    RtDraw(usize, i32, i32, f32, Color, bool),         // render-target idx, x, y, scale, tint, flip_v
}

/// 3D-Zeichenbefehle (Modul `g3d`). Werden beim FLIP in einem
/// `begin_mode3D`-Block VOR den 2D-Layern gerendert -- das 2D-HUD liegt also
/// immer obenauf. Koordinaten sind Welt-Einheiten (kein Screen-Scale `s`).
#[derive(Clone)]
enum Cmd3D {
    Cube(f32, f32, f32, f32, f32, f32, Color),       // x,y,z, w,h,d
    CubeWires(f32, f32, f32, f32, f32, f32, Color),
    Sphere(f32, f32, f32, f32, Color),               // cx,cy,cz, r
    SphereWires(f32, f32, f32, f32, Color),
    Cylinder(f32, f32, f32, f32, f32, f32, Color),   // x,y,z, r_top,r_bot, h
    Plane(f32, f32, f32, f32, f32, Color),           // cx,cy,cz, size_x,size_z
    Line(f32, f32, f32, f32, f32, f32, Color),       // x1,y1,z1, x2,y2,z2
    Point(f32, f32, f32, Color),
    Grid(i32, f32),                                  // slices, spacing
    // Geladene/prozedurale Modelle (Index in Graphics.models).
    Model(usize, f32, f32, f32, f32, Color),         // idx, x,y,z, scale, tint
    // idx, x,y,z, achse_x,achse_y,achse_z, winkel, scale, tint
    ModelEx(usize, f32, f32, f32, f32, f32, f32, f32, f32, Color),
    ModelWires(usize, f32, f32, f32, f32, Color),    // idx, x,y,z, scale, tint
    // Modul m3d: Modell mit beliebiger Welt-Matrix (column-major, OpenGL-Order).
    // idx, mat, tint. Gerendert via rl-Matrix-Stack (rlMultMatrixf) -> kein
    // mutabler Model-Borrow noetig; DrawMesh honoriert rlGetMatrixTransform().
    ModelMatrix(usize, Rc<[f32; 16]>, Color),
    // Modul m3d: GPU-Instancing -- dasselbe Modell mit N Welt-Matrizen in EINEM
    // Draw-Call (raylib DrawMeshInstanced). idx, Matrizen (column-major), tint.
    ModelInstanced(usize, Rc<Vec<[f32; 16]>>, Color),
    // Billboard: Textur (Index), die immer zur Kamera zeigt. idx, x,y,z, size, tint
    Billboard(usize, f32, f32, f32, f32, Color),
}

struct Layer {
    z: i32,
    cmds: Vec<Cmd>,
}

/// Render-Target (RENDERTARGET_*): eine RenderTexture2D mit eigenem Command-
/// Buffer. RENDERTARGET_BEGIN lenkt `emit` hierher um; beim FLIP wird der Buffer
/// (falls nicht leer) auf die Textur gerendert, bevor die Hauptszene laeuft.
struct RenderTarget {
    rt: RenderTexture2D,
    cmds: Vec<Cmd>,
    /// Bleibt der Inhalt ueber das Bild hinaus stehen?
    ///
    /// Normalerweise wird ein Target vor jedem Bild transparent geleert -- das
    /// ist die richtige Voreinstellung fuer "Szene zwischenspeichern".
    /// Stehenbleiben ist die Voraussetzung fuer RUECKKOPPLUNG: das Bild von
    /// eben leicht verschoben wieder hineinzeichnen, was Schweife, Nachzieher
    /// und den klassischen Demo-Feedback-Effekt ergibt.
    behalten: bool,
}

struct Atlas {
    tex_idx: usize,
    frames: HashMap<String, (i32, i32, i32, i32)>,
}

/// GPU-Textur + CPU-Image (Image noetig fuer imgfx-Transformationen).
struct Tex {
    tex: Texture2D,
    img: Image,
}

/// Beleuchtung (Blinn-Phong via Standard-rlights-Shader). Bis zu 4 Lichter.
const MAX_LIGHTS: usize = 4;

/// Ein Licht + die gecachten Uniform-Locations im Lighting-Shader.
struct LightData {
    enabled: bool,
    kind: i32,        // 0 = directional, 1 = point
    pos: [f32; 3],
    target: [f32; 3],
    color: [f32; 4],
    loc_enabled: i32,
    loc_type: i32,
    loc_pos: i32,
    loc_target: i32,
    loc_color: i32,
}

/// GLSL fuer die laufende Grafik-Schnittstelle zurechtlegen.
///
/// Unsere Shader sind in Desktop-GLSL 330 geschrieben. Der Browser faehrt
/// WebGL 2, und dessen Sprache (GLSL ES 3.00) ist bis auf ZWEI Dinge dieselbe:
/// die Versionszeile, und dass Genauigkeiten ausdruecklich angegeben werden
/// muessen. Statt jeden Shader doppelt zu pflegen -- zwei Fassungen laufen
/// unweigerlich auseinander -- wird hier nur der Kopf getauscht. Der Rumpf
/// (`in`/`out`/`texture()`/`textureLod()`) ist in beiden Sprachen gueltig.
///
/// Das gilt auch fuer SHADER_LOAD: ein Shader, den jemand fuer den Desktop
/// geschrieben hat, laeuft dadurch unveraendert im Browser mit.
/// Kopf fuer WebGL 2. Die Genauigkeits-Angaben sind in GLSL ES Pflicht -- und
/// `sampler2D` steht dort ohne Angabe auf `lowp`, was fuer eine Tiefenkarte
/// (Schatten) zu grob waere.
///
/// Auf dem Desktop nur vom Test benutzt -- die Logik soll dort trotzdem
/// nachweisbar sein, statt nur im Browser zu existieren.
#[cfg_attr(not(target_os = "emscripten"), allow(dead_code))]
const GLSL_KOPF_WEB: &str = "#version 300 es\n\
                             precision highp float;\n\
                             precision highp int;\n\
                             precision highp sampler2D;\n\
                             precision highp samplerCube;\n";

/// Die Versionszeile durch `kopf` ersetzen (bzw. ihn voranstellen).
///
/// Absichtlich plattform-unabhaengig, damit die Logik auf dem Desktop
/// **getestet** werden kann -- ein `#[cfg(emscripten)]`-Rumpf waere hier nie
/// unter einem Test gelaufen. Gesucht wird ZEILENWEISE: ein "#version" mitten
/// in einem Kommentar ist keine Versionsangabe.
#[cfg_attr(not(target_os = "emscripten"), allow(dead_code))]
fn kopf_ersetzen(src: &str, kopf: &str) -> String {
    let mut ausgabe = String::with_capacity(src.len() + kopf.len());
    let mut ersetzt = false;
    for zeile in src.split_inclusive('\n') {
        if !ersetzt && zeile.trim_start().starts_with("#version") {
            ausgabe.push_str(kopf);
            ersetzt = true;
        } else {
            ausgabe.push_str(zeile);
        }
    }
    // Ohne Versionszeile gilt GLSL 1.10 bzw. ES 1.00 -- der Kopf muss trotzdem
    // nach vorne, sonst uebersetzt gar nichts davon.
    if !ersetzt { return format!("{kopf}{src}"); }
    ausgabe
}

#[cfg(target_os = "emscripten")]
pub fn fuer_ziel_uebersetzen(src: &str) -> std::borrow::Cow<'_, str> {
    std::borrow::Cow::Owned(kopf_ersetzen(src, GLSL_KOPF_WEB))
}

#[cfg(not(target_os = "emscripten"))]
pub fn fuer_ziel_uebersetzen(src: &str) -> std::borrow::Cow<'_, str> {
    std::borrow::Cow::Borrowed(src)
}

#[cfg(test)]
mod glsl_kopf_tests {
    use super::{kopf_ersetzen, GLSL_KOPF_WEB};

    #[test]
    fn versionszeile_wird_getauscht_und_rumpf_bleibt() {
        let quelle = "#version 330\nin vec2 fragTexCoord;\nvoid main() {}\n";
        let neu = kopf_ersetzen(quelle, GLSL_KOPF_WEB);
        assert!(neu.starts_with("#version 300 es\n"));
        assert!(!neu.contains("#version 330"), "alte Version blieb stehen: {neu}");
        assert!(neu.contains("in vec2 fragTexCoord;"), "Rumpf ging verloren");
        assert!(neu.contains("void main() {}"));
        // Genau EINE Versionszeile -- eine zweite waere ein Uebersetzungsfehler.
        assert_eq!(neu.matches("#version").count(), 1);
    }

    #[test]
    fn ohne_versionszeile_kommt_der_kopf_nach_vorne() {
        let neu = kopf_ersetzen("void main() {}\n", GLSL_KOPF_WEB);
        assert!(neu.starts_with("#version 300 es\n"));
        assert!(neu.ends_with("void main() {}\n"));
    }

    #[test]
    fn version_im_kommentar_ist_keine_versionszeile() {
        // Sonst landete der Kopf mitten im Shader -- und die echte Angabe
        // bliebe stehen.
        let quelle = "#version 330\n// nutzt #version 330 Merkmale\nvoid main() {}\n";
        let neu = kopf_ersetzen(quelle, GLSL_KOPF_WEB);
        assert!(neu.starts_with("#version 300 es\n"));
        assert!(neu.contains("// nutzt #version 330 Merkmale"),
                "der Kommentar wurde angetastet: {neu}");
    }
}

/// Eingebetteter Lighting-Vertex-Shader (raylib rlights, GLSL 330).
const LIGHT_VS: &str = r#"#version 330
in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec3 vertexNormal;
in vec4 vertexColor;
in vec4 vertexTangent;
uniform mat4 mvp;
uniform mat4 matModel;
uniform mat4 matNormal;
out vec3 fragPosition;
out vec2 fragTexCoord;
out vec4 fragColor;
out vec3 fragNormal;
out vec3 fragTangent;
void main()
{
    fragPosition = vec3(matModel*vec4(vertexPosition, 1.0));
    fragTexCoord = vertexTexCoord;
    fragColor = vertexColor;
    fragNormal = normalize(vec3(matNormal*vec4(vertexNormal, 1.0)));
    fragTangent = normalize(vec3(matModel*vec4(vertexTangent.xyz, 0.0)));
    gl_Position = mvp*vec4(vertexPosition, 1.0);
}
"#;

/// Eingebetteter Lighting-Fragment-Shader (raylib rlights, GLSL 330).
const LIGHT_FS: &str = r#"#version 330
in vec3 fragPosition;
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragNormal;
in vec3 fragTangent;
uniform sampler2D texture0;
uniform sampler2D texture2;   // MATERIAL_MAP_NORMAL (nur wenn useNormalMap == 1)
uniform int useNormalMap;
uniform vec4 colDiffuse;
out vec4 finalColor;
#define MAX_LIGHTS 4
#define LIGHT_DIRECTIONAL 0
#define LIGHT_POINT 1
const float PI = 3.14159265359;
struct Light {
    int enabled;
    int type;
    vec3 position;
    vec3 target;
    vec4 color;
};
uniform Light lights[MAX_LIGHTS];
uniform vec4 ambient;
uniform vec3 viewPos;
uniform float metalness;   // 0 = Dielektrikum, 1 = Metall
uniform float roughness;   // 0 = spiegelnd, 1 = matt
uniform vec4 emissive;     // rgb = Eigenleucht-Farbe, a = Staerke (0 = aus)
uniform vec4 fogColor;
uniform float fogDensity;
// Analytisches Environment-Lighting (IBL-Approximation). envIntensity 0 = aus.
uniform vec3 envSky;
uniform vec3 envGround;
uniform float envIntensity;
// Echtes HDR-Cubemap-IBL (LIGHT_ENV_HDR). useIBLMaps 1 => die drei Maps statt
// des analytischen envSample-Pfades. Default 0 -> LIGHT_ENV-Naeherung.
uniform int useIBLMaps;
uniform samplerCube irradianceMap;   // diffuse Hemisphaeren-Irradiance
uniform samplerCube prefilterMap;    // spekulare GGX-Prefilter (Roughness-Mips)
uniform sampler2D brdfLUT;           // Environment-BRDF (NoV x roughness)
// Shadow-Mapping (lights[0] = schattenwerfendes directional light).
uniform mat4 lightVP;
uniform sampler2D shadowMap;
uniform int shadowMapResolution;
uniform int shadowsEnabled;

// --- Cook-Torrance BRDF ---
float DistributionGGX(vec3 N, vec3 H, float r) {
    float a = r*r; float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0); float d = NdotH*NdotH*(a2 - 1.0) + 1.0;
    return a2/(PI*d*d + 1e-7);
}
float GeometrySchlickGGX(float NdotV, float r) {
    float k = (r + 1.0); k = (k*k)/8.0;
    return NdotV/(NdotV*(1.0 - k) + k);
}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float r) {
    return GeometrySchlickGGX(max(dot(N, V), 0.0), r)*GeometrySchlickGGX(max(dot(N, L), 0.0), r);
}
vec3 fresnelSchlick(float cosT, vec3 F0) {
    return F0 + (1.0 - F0)*pow(clamp(1.0 - cosT, 0.0, 1.0), 5.0);
}
vec3 fresnelSchlickRough(float cosT, vec3 F0, float r) {
    vec3 fr = max(vec3(1.0 - r), F0);
    return F0 + (fr - F0)*pow(clamp(1.0 - cosT, 0.0, 1.0), 5.0);
}
// Prozedurale Umgebung: vertikaler Gradient Boden<->Himmel.
vec3 envSample(vec3 dir) { return mix(envGround, envSky, dir.y*0.5 + 0.5); }
// Analytische Environment-BRDF (Karis-Mobile-Approximation, ersetzt die LUT).
vec2 envBRDFApprox(float NoV, float r) {
    vec4 c0 = vec4(-1.0, -0.0275, -0.572, 0.022);
    vec4 c1 = vec4(1.0, 0.0425, 1.04, -0.04);
    vec4 rr = r*c0 + c1;
    float a004 = min(rr.x*rr.x, exp2(-9.28*NoV))*rr.x + rr.y;
    return vec2(-1.04, 1.04)*a004 + rr.zw;
}
float shadowFactor(vec3 N, vec3 l0) {
    if (shadowsEnabled != 1) return 0.0;
    vec4 p = lightVP*vec4(fragPosition, 1.0);
    vec3 proj = p.xyz/p.w; proj = proj*0.5 + 0.5;
    if (proj.z > 1.0 || proj.x < 0.0 || proj.x > 1.0 || proj.y < 0.0 || proj.y > 1.0) return 0.0;
    float bias = max(0.0015*(1.0 - dot(N, l0)), 0.00015);
    vec2 texel = vec2(1.0/float(shadowMapResolution));
    int cnt = 0;
    for (int x = -1; x <= 1; x++)
        for (int y = -1; y <= 1; y++) {
            float d = texture(shadowMap, proj.xy + vec2(x, y)*texel).r;
            if (proj.z - bias > d) cnt++;
        }
    return float(cnt)/9.0;
}
void main()
{
    vec3 albedo = colDiffuse.rgb*texture(texture0, fragTexCoord).rgb;
    float metal = clamp(metalness, 0.0, 1.0);
    float rough = clamp(roughness, 0.04, 1.0);
    // Normale aus Geometrie + (optionaler) Normal-Map ueber TBN.
    vec3 geomN = normalize(fragNormal);
    vec3 T = normalize(fragTangent - dot(fragTangent, geomN)*geomN);
    vec3 B = cross(geomN, T);
    vec3 nmap = texture(texture2, fragTexCoord).rgb*2.0 - 1.0;
    vec3 N = (useNormalMap == 1) ? normalize(mat3(T, B, geomN)*nmap) : geomN;
    vec3 V = normalize(viewPos - fragPosition);
    vec3 F0 = mix(vec3(0.04), albedo, metal);
    vec3 l0dir = -normalize(lights[0].target - lights[0].position);
    float shadow = shadowFactor(N, l0dir);
    vec3 Lo = vec3(0.0);
    for (int i = 0; i < MAX_LIGHTS; i++)
    {
        if (lights[i].enabled != 1) continue;
        vec3 L; float atten = 1.0;
        if (lights[i].type == LIGHT_DIRECTIONAL) {
            L = -normalize(lights[i].target - lights[i].position);
        } else {
            vec3 d = lights[i].position - fragPosition;
            L = normalize(d); float dist = length(d); atten = 1.0/(dist*dist);
        }
        vec3 H = normalize(V + L);
        vec3 radiance = lights[i].color.rgb*atten;
        float NDF = DistributionGGX(N, H, rough);
        float G = GeometrySmith(N, V, L, rough);
        vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);
        vec3 spec = (NDF*G*F)/(4.0*max(dot(N, V), 0.0)*max(dot(N, L), 0.0) + 1e-4);
        vec3 kD = (vec3(1.0) - F)*(1.0 - metal);
        float NdotL = max(dot(N, L), 0.0);
        vec3 contrib = (kD*albedo/PI + spec)*radiance*NdotL;
        if (i == 0) contrib *= (1.0 - shadow*0.9);   // nur Schattenlicht dimmen
        Lo += contrib;
    }
    vec3 ambientTerm = ambient.rgb*albedo;
    // Image-Based-Lighting (analytische Umgebung): diffuse Hemisphaeren-Irradiance
    // + spiegelnde Sky-Reflexion (roughness-abhaengig verschwommen) + Env-BRDF.
    if (envIntensity > 0.0) {
        float NoV = max(dot(N, V), 0.0);
        vec3 F = fresnelSchlickRough(NoV, F0, rough);
        vec3 kD = (vec3(1.0) - F)*(1.0 - metal);
        vec3 R = reflect(-V, N);
        vec3 diffuseIBL;
        vec3 specularIBL;
        if (useIBLMaps == 1) {
            // Echtes HDR-Cubemap-IBL: vorberechnete Maps abtasten.
            vec3 irradiance = texture(irradianceMap, N).rgb;
            diffuseIBL = irradiance*albedo;
            const float MAX_REFLECTION_LOD = 4.0;   // prefilterMap hat 5 Mips (0..4)
            vec3 prefiltered = textureLod(prefilterMap, R, rough*MAX_REFLECTION_LOD).rgb;
            vec2 brdf = texture(brdfLUT, vec2(NoV, rough)).rg;
            specularIBL = prefiltered*(F*brdf.x + brdf.y);
        } else {
            // Analytische Naeherung (LIGHT_ENV): vertikaler Gradient + Karis-BRDF.
            vec3 irradiance = envSample(N);
            diffuseIBL = irradiance*albedo;
            vec3 avg = (envSky + envGround)*0.5;
            vec3 prefiltered = mix(envSample(R), avg, rough);
            vec2 brdf = envBRDFApprox(NoV, rough);
            specularIBL = prefiltered*(F0*brdf.x + brdf.y);
        }
        ambientTerm += (kD*diffuseIBL + specularIBL)*envIntensity;
    }
    vec3 color = ambientTerm + Lo;
    color = color/(color + vec3(1.0));        // Reinhard-Tonemapping
    color = pow(color, vec3(1.0/2.2));         // Gamma
    finalColor = vec4(color, 1.0);
    // Exponentieller Tiefen-Fog (fogDensity 0 => kein Effekt).
    float fd = length(viewPos - fragPosition)*fogDensity;
    float fog = clamp(1.0/exp(fd*fd), 0.0, 1.0);
    finalColor = mix(fogColor, finalColor, fog);
    // Eigenleuchten (durchschlaegt den Fog -> Neon/Glow, mit Bloom-POSTFX).
    finalColor.rgb += emissive.rgb * emissive.a;
}
"#;

// === GPU-Instancing (MODEL_INSTANCED) ===
// Eigener Shader-Pfad: derselbe Mesh wird via raylib `DrawMeshInstanced` mit N
// Per-Instance-Welt-Matrizen in EINEM Draw-Call gerendert. Der Lighting-Shader
// (LIGHT_VS/FS) taugt dafuer NICHT -- er liest die Welt-Transform aus dem
// `matModel`-Uniform; Instancing liefert sie stattdessen als Vertex-Attribut
// `instanceTransform` (4 vec4-Spalten, location = SHADER_LOC_MATRIX_MODEL).
// raylib laedt `mvp` = view*projection ins MVP-Uniform und identitaet in
// matModel/matNormal -> die Modell-Transform MUSS aus instanceTransform kommen,
// inkl. der Normalen (matNormal ist bei Instancing unbrauchbar).
const INST_VS: &str = r#"#version 330
in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec3 vertexNormal;
in vec4 vertexColor;
in mat4 instanceTransform;       // per-Instanz (location = SHADER_LOC_MATRIX_MODEL)
uniform mat4 mvp;                 // = view*projection (raylib setzt es bei Instancing)
out vec3 fragPosition;
out vec2 fragTexCoord;
out vec4 fragColor;
out vec3 fragNormal;
void main()
{
    fragPosition = vec3(instanceTransform*vec4(vertexPosition, 1.0));
    fragTexCoord = vertexTexCoord;
    fragColor = vertexColor;
    // Normale aus der Instanz-Matrix ableiten (matNormal ist hier Identitaet);
    // korrekt fuer Rotation + (uniforme) Skalierung.
    fragNormal = normalize(mat3(instanceTransform)*vertexNormal);
    gl_Position = mvp*instanceTransform*vec4(vertexPosition, 1.0);
}
"#;

// Eigenstaendiger Instancing-Fragment-Shader: Ambient + bis MAX_LIGHTS
// Blinn-Phong-Lichter (directional/point), Fallback auf flaches Albedo wenn kein
// Licht aktiv (lightCount==0). Bewusst schlanker als LIGHT_FS -- ohne PBR/IBL/
// Schatten/Normal-Maps (Grenze des Instancing-Pfades, siehe docs/module-m3d.md).
const INST_FS: &str = r#"#version 330
in vec3 fragPosition;
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragNormal;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
out vec4 finalColor;
#define MAX_LIGHTS 4
struct Light { int enabled; int type; vec3 position; vec3 target; vec4 color; };
uniform Light lights[MAX_LIGHTS];
uniform vec4 ambient;
uniform vec3 viewPos;
uniform int lightCount;
void main()
{
    vec3 albedo = colDiffuse.rgb*texture(texture0, fragTexCoord).rgb*fragColor.rgb;
    if (lightCount == 0) { finalColor = vec4(albedo, 1.0); return; }
    vec3 N = normalize(fragNormal);
    vec3 V = normalize(viewPos - fragPosition);
    vec3 lit = ambient.rgb*albedo;
    for (int i = 0; i < MAX_LIGHTS; i++)
    {
        if (lights[i].enabled != 1) continue;
        vec3 L; float atten = 1.0;
        if (lights[i].type == 0) {
            L = -normalize(lights[i].target - lights[i].position);
        } else {
            vec3 dd = lights[i].position - fragPosition;
            L = normalize(dd); float dist = length(dd);
            atten = 1.0/(1.0 + 0.09*dist + 0.032*dist*dist);
        }
        float NdotL = max(dot(N, L), 0.0);
        vec3 H = normalize(V + L);
        float spec = pow(max(dot(N, H), 0.0), 32.0)*0.3;
        lit += (albedo*NdotL + vec3(spec))*lights[i].color.rgb*atten;
    }
    finalColor = vec4(lit, 1.0);
}
"#;

// === HDR-Cubemap-IBL (LIGHT_ENV_HDR) ===
// Vorberechnungs-Shader, Port von raylibs `shaders_basic_pbr`-Beispiel /
// learnopengl IBL. Gerendert in FBOs ueber `rlLoadDrawCube`/`rlLoadDrawQuad`
// (immediate glDrawArrays mit dem via rlEnableShader gebundenen Shader; die
// matProjection/matView-Uniforms werden pro Face per rlSetUniformMatrix gesetzt).

/// Gemeinsamer Cubemap-Vertex-Shader (equirect / irradiance / prefilter).
/// vertexPosition liegt auf Attrib-Location 0 (rlLoadDrawCube-Layout).
const CUBEMAP_VS: &str = r#"#version 330
in vec3 vertexPosition;
uniform mat4 matProjection;
uniform mat4 matView;
out vec3 fragPosition;
void main()
{
    fragPosition = vertexPosition;
    gl_Position = matProjection*matView*vec4(vertexPosition, 1.0);
}
"#;

/// equirectangular Panorama -> Cubemap (sphaerische Projektion).
const EQUIRECT_FS: &str = r#"#version 330
in vec3 fragPosition;
uniform sampler2D equirectangularMap;   // Texture-Unit 0
out vec4 finalColor;
vec2 SampleSphericalMap(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= vec2(0.1591, 0.3183);   // 1/(2*PI), 1/PI
    uv += 0.5;
    return uv;
}
void main()
{
    vec2 uv = SampleSphericalMap(normalize(fragPosition));
    finalColor = vec4(texture(equirectangularMap, uv).rgb, 1.0);
}
"#;

/// Diffuse Irradiance-Convolution ueber die Environment-Cubemap (Hemisphaere).
const IRRADIANCE_FS: &str = r#"#version 330
in vec3 fragPosition;
uniform samplerCube environmentMap;   // Texture-Unit 0
out vec4 finalColor;
const float PI = 3.14159265359;
void main()
{
    vec3 N = normalize(fragPosition);
    vec3 irradiance = vec3(0.0);
    vec3 up = vec3(0.0, 1.0, 0.0);
    vec3 right = normalize(cross(up, N));
    up = normalize(cross(N, right));
    float sampleDelta = 0.025;
    float nrSamples = 0.0;
    for (float phi = 0.0; phi < 2.0*PI; phi += sampleDelta)
    {
        for (float theta = 0.0; theta < 0.5*PI; theta += sampleDelta)
        {
            vec3 tangent = vec3(sin(theta)*cos(phi), sin(theta)*sin(phi), cos(theta));
            vec3 sampleVec = tangent.x*right + tangent.y*up + tangent.z*N;
            irradiance += texture(environmentMap, sampleVec).rgb*cos(theta)*sin(theta);
            nrSamples++;
        }
    }
    irradiance = PI*irradiance*(1.0/nrSamples);
    finalColor = vec4(irradiance, 1.0);
}
"#;

/// Prefilter: GGX-Importance-Sampling der Environment-Cubemap pro Roughness-Mip.
const PREFILTER_FS: &str = r#"#version 330
in vec3 fragPosition;
uniform samplerCube environmentMap;   // Texture-Unit 0
uniform float roughness;
out vec4 finalColor;
const float PI = 3.14159265359;
float DistributionGGX(vec3 N, vec3 H, float r)
{
    float a = r*r; float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float d = NdotH*NdotH*(a2 - 1.0) + 1.0;
    return a2/(PI*d*d);
}
float RadicalInverse_VdC(uint bits)
{
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits)*2.3283064365386963e-10;
}
vec2 Hammersley(uint i, uint n) { return vec2(float(i)/float(n), RadicalInverse_VdC(i)); }
vec3 ImportanceSampleGGX(vec2 Xi, vec3 N, float r)
{
    float a = r*r;
    float phi = 2.0*PI*Xi.x;
    float cosTheta = sqrt((1.0 - Xi.y)/(1.0 + (a*a - 1.0)*Xi.y));
    float sinTheta = sqrt(1.0 - cosTheta*cosTheta);
    vec3 H = vec3(cos(phi)*sinTheta, sin(phi)*sinTheta, cosTheta);
    vec3 up = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    vec3 tangent = normalize(cross(up, N));
    vec3 bitangent = cross(N, tangent);
    return normalize(tangent*H.x + bitangent*H.y + N*H.z);
}
void main()
{
    vec3 N = normalize(fragPosition);
    vec3 R = N; vec3 V = R;
    const uint SAMPLE_COUNT = 1024u;
    vec3 prefiltered = vec3(0.0);
    float totalWeight = 0.0;
    for (uint i = 0u; i < SAMPLE_COUNT; i++)
    {
        vec2 Xi = Hammersley(i, SAMPLE_COUNT);
        vec3 H = ImportanceSampleGGX(Xi, N, roughness);
        vec3 L = normalize(2.0*dot(V, H)*H - V);
        float NdotL = max(dot(N, L), 0.0);
        if (NdotL > 0.0)
        {
            float D = DistributionGGX(N, H, roughness);
            float NdotH = max(dot(N, H), 0.0);
            float HdotV = max(dot(H, V), 0.0);
            float pdf = D*NdotH/(4.0*HdotV) + 0.0001;
            float resolution = 512.0;   // Quell-Cubemap-Face-Aufloesung
            float saTexel = 4.0*PI/(6.0*resolution*resolution);
            float saSample = 1.0/(float(SAMPLE_COUNT)*pdf + 0.0001);
            float mipLevel = roughness == 0.0 ? 0.0 : 0.5*log2(saSample/saTexel);
            prefiltered += textureLod(environmentMap, L, mipLevel).rgb*NdotL;
            totalWeight += NdotL;
        }
    }
    prefiltered = prefiltered/totalWeight;
    finalColor = vec4(prefiltered, 1.0);
}
"#;

/// BRDF-LUT-Vertex-Shader (Fullscreen-Quad via rlLoadDrawQuad: pos@0, uv@2).
const BRDF_VS: &str = r#"#version 330
in vec3 vertexPosition;
in vec2 vertexTexCoord;
out vec2 fragTexCoord;
void main()
{
    fragTexCoord = vertexTexCoord;
    gl_Position = vec4(vertexPosition, 1.0);
}
"#;

/// BRDF-Integration (Environment-BRDF-LUT: x=NdotV, y=roughness).
const BRDF_FS: &str = r#"#version 330
in vec2 fragTexCoord;
out vec4 finalColor;
const float PI = 3.14159265359;
float RadicalInverse_VdC(uint bits)
{
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits)*2.3283064365386963e-10;
}
vec2 Hammersley(uint i, uint n) { return vec2(float(i)/float(n), RadicalInverse_VdC(i)); }
vec3 ImportanceSampleGGX(vec2 Xi, vec3 N, float r)
{
    float a = r*r;
    float phi = 2.0*PI*Xi.x;
    float cosTheta = sqrt((1.0 - Xi.y)/(1.0 + (a*a - 1.0)*Xi.y));
    float sinTheta = sqrt(1.0 - cosTheta*cosTheta);
    vec3 H = vec3(cos(phi)*sinTheta, sin(phi)*sinTheta, cosTheta);
    vec3 up = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    vec3 tangent = normalize(cross(up, N));
    vec3 bitangent = cross(N, tangent);
    return normalize(tangent*H.x + bitangent*H.y + N*H.z);
}
float GeometrySchlickGGX(float NdotV, float r)
{
    float k = (r*r)/2.0;   // IBL-Variante
    return NdotV/(NdotV*(1.0 - k) + k);
}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float r)
{
    return GeometrySchlickGGX(max(dot(N, L), 0.0), r)*GeometrySchlickGGX(max(dot(N, V), 0.0), r);
}
vec2 IntegrateBRDF(float NdotV, float roughness)
{
    vec3 V = vec3(sqrt(1.0 - NdotV*NdotV), 0.0, NdotV);
    float A = 0.0; float B = 0.0;
    vec3 N = vec3(0.0, 0.0, 1.0);
    const uint SAMPLE_COUNT = 1024u;
    for (uint i = 0u; i < SAMPLE_COUNT; i++)
    {
        vec2 Xi = Hammersley(i, SAMPLE_COUNT);
        vec3 H = ImportanceSampleGGX(Xi, N, roughness);
        vec3 L = normalize(2.0*dot(V, H)*H - V);
        float NdotL = max(L.z, 0.0);
        float NdotH = max(H.z, 0.0);
        float VdotH = max(dot(V, H), 0.0);
        if (NdotL > 0.0)
        {
            float G = GeometrySmith(N, V, L, roughness);
            float G_Vis = (G*VdotH)/(NdotH*NdotV);
            float Fc = pow(1.0 - VdotH, 5.0);
            A += (1.0 - Fc)*G_Vis;
            B += Fc*G_Vis;
        }
    }
    return vec2(A, B)/float(SAMPLE_COUNT);
}
void main()
{
    finalColor = vec4(IntegrateBRDF(fragTexCoord.x, fragTexCoord.y), 0.0, 1.0);
}
"#;

/// Skybox-Vertex-Shader: Translation aus der View-Matrix entfernen (mat3),
/// damit die Cubemap immer um die Kamera zentriert (unendlich weit) erscheint.
const SKYBOX_VS: &str = r#"#version 330
in vec3 vertexPosition;
uniform mat4 matProjection;
uniform mat4 matView;
out vec3 fragPosition;
void main()
{
    fragPosition = vertexPosition;
    mat4 rotView = mat4(mat3(matView));   // Translation raus -> folgt der Kamera
    gl_Position = matProjection*rotView*vec4(vertexPosition, 1.0);
}
"#;

/// Skybox-Fragment-Shader: Cubemap in Blickrichtung sampeln (+ Gamma, da die
/// env-Cubemap linear-geclampte HDR-Werte haelt).
const SKYBOX_FS: &str = r#"#version 330
in vec3 fragPosition;
uniform samplerCube environmentMap;   // Texture-Unit 0
out vec4 finalColor;
void main()
{
    vec3 color = texture(environmentMap, fragPosition).rgb;
    color = pow(color, vec3(1.0/2.2));   // linear -> sRGB
    finalColor = vec4(color, 1.0);
}
"#;

/// Schnappschuss des ZEICHENZUSTANDS fuer `GFX_PUSH`/`GFX_POP`.
///
/// Warum es das gibt: Licht, Nebel, Himmel, Schatten, Kamera, Schrift und der
/// Post-Effekt sind GLOBAL. Wer sie in einer Szene umstellt, muss sie beim
/// Verlassen von Hand zurueckstellen -- und vergisst man eine Zeile, sieht man
/// den Fehler erst zwei Szenen spaeter (der HDR-Himmel hinter dem naechsten
/// Bild, der Nebel in der falschen Szene). Mit PUSH/POP wird daraus eine
/// Eigenschaft der Sprache statt einer Disziplinfrage.
///
/// **Enthalten** ist ausschliesslich EINSTELLUNG, keine RESSOURCE:
/// 2D-Kamera samt Ruetteln, aktive Layer, Hintergrundfarbe, Licht (Ambient,
/// Nebel, alle Lichtquellen), Umgebung (analytisch + IBL-Schalter + Himmel),
/// Schatten (an/aus, Bereich, Ziel), 3D-Kamera samt View-/Projektions-
/// Ueberschreibung, Schrift (Groesse, Font, Abstaende) und der Post-Effekt.
///
/// **Nicht enthalten** (und das ist Absicht):
/// * geladene Ressourcen -- Modelle, Texturen, Shader, Schriften,
///   Render-Targets, IBL-Maps. Die bleiben geladen; POP schaltet nur ihre
///   Benutzung zurueck.
/// * die Schatten-AUFLOESUNG (`SHADOW_ENABLE(res)`) -- sie haengt an einem
///   allozierten Tiefenpuffer, den ein POP nicht neu bauen soll.
/// * der Blend-Modus -- der ist ohnehin nur ein Bild lang gueltig (er steht im
///   Zeichenpuffer, nicht im Zustand) und braucht kein Zuruecksetzen.
/// * Fenster/Vollbild, Automation, Kontaktbogen -- das ist Programm-Rahmen,
///   nicht Szenen-Zustand.
#[derive(Clone)]
struct GfxState {
    cam: (f64, f64, f64, f64),
    shake: (f64, f64, Option<std::time::Instant>, f64, f64),
    active_layer: usize,
    clear_color: Color,
    light_ambient: [f32; 4],
    light_fog: [f32; 4],
    light_fog_density: f32,
    /// Nur die WERTE der Lichter -- die Uniform-Locations bleiben, wo sie sind.
    lights: Vec<(bool, i32, [f32; 3], [f32; 3], [f32; 4])>,
    env: ([f32; 3], [f32; 3], f32),
    use_ibl_maps: bool,
    skybox_enabled: bool,
    shadow: (bool, f32, f32, [f32; 3]),
    cam3d: Camera3D,
    cam3d_view: Option<[f32; 16]>,
    cam3d_proj: Option<[f32; 16]>,
    text: (i32, i64, f32),
    post_shader_idx: Option<usize>,
}

/// Web: den PUFFER der HTML-Leinwand auf die Fenstergroesse bringen.
///
/// Nachgemessen im Browser: nach `SCREEN(480,320)` meldet die Laufzeit brav
/// 480x320 -- der Puffer der `<canvas>` blieb aber 1x1, und per CSS auf volle
/// Breite gestreckt wurde daraus eine einzige Farbflaeche. raylibs Web-Pfad
/// setzt die Groesse offenbar spaeter selbst noch einmal, deshalb wird sie in
/// den ersten Bildern erneut gesetzt (siehe `web_canvas_ticks`) statt nur
/// einmal beim Anlegen.
#[cfg(target_os = "emscripten")]
fn web_leinwand_groesse(w: i32, h: i32) {
    extern "C" {
        fn emscripten_set_canvas_element_size(
            target: *const std::os::raw::c_char,
            width: std::os::raw::c_int,
            height: std::os::raw::c_int,
        ) -> std::os::raw::c_int;
    }
    if let Ok(ziel) = std::ffi::CString::new("#canvas") {
        unsafe { emscripten_set_canvas_element_size(ziel.as_ptr(), w, h); }
    }
}

pub struct Graphics {
    rl: RaylibHandle,
    thread: RaylibThread,
    width: i32,
    height: i32,
    scale: i32,
    // SET_FULLSCREEN(TRUE): merkt sich, ob wir gerade im (pixel-skalierten,
    // borderless) Vollbild-Modus sind + die windowed (width,height,scale) zum
    // Zurueckschalten. Siehe set_fullscreen() fuer die Technik.
    fullscreen: bool,
    pre_fullscreen: Option<(i32, i32, i32)>,
    // Z-Layer: Index 0 ist der Default-/Main-Layer (z=0). LAYER(name) schaltet
    // `active` um. FLIP komponiert alle Layer aufsteigend nach z und leert sie.
    layers: Vec<Layer>,
    layer_names: HashMap<String, usize>,
    active: usize,
    // Render-Targets (RENDERTARGET_*): leben ueber Frames; active_rt lenkt `emit`
    // um, solange ein Target via RENDERTARGET_BEGIN aktiv ist.
    render_targets: Vec<RenderTarget>,
    active_rt: Option<usize>,
    clear_color: Color,
    // Transparenter Fenster-Framebuffer (SCREEN_TRANSPARENT): der Desktop scheint
    // dort durch, wo mit Alpha < 255 gecleart/gezeichnet wird. Steuert, ob CLS das
    // Alpha-Byte woertlich nimmt (statt es auf 255 zu zwingen).
    transparent: bool,
    // Kamera (Modul `camera`): World->Screen-Transform fuer alle Draws.
    cam_x: f64,
    cam_y: f64,
    cam_zoom: f64,
    // Rotation in Grad, um den Bildschirm-Mittelpunkt (logische width/height/2),
    // NACH Translation+Zoom angewandt -- bei 0.0 bit-identisch zum alten
    // (translate+scale-only) Verhalten. Positiv = Kamera dreht sich im
    // Uhrzeigersinn -> die Welt erscheint gegen den Uhrzeigersinn gedreht
    // (Standard-View-Matrix-Konvention). Wirkt ausschliesslich auf POSITIONEN
    // (w2s, damit auf jeden Draw automatisch) -- KEINE automatische Kontur-
    // Rotation von Formen/Bildern/Sprites (die behalten ihre eigene Achse;
    // wer ein Objekt auch optisch mitdrehen will, addiert CAMERA_ROTATION()
    // selbst zum eigenen Rotationswinkel, z.B. bei DRAWIMAGEROT). Siehe
    // docs/module-camera.md.
    cam_rotation: f64,
    // CAMERA_SHAKE: zufaelliger Kamera-Offset, klingt linear ueber die Dauer
    // ab. Pro Frame EIN Offset (in flip() fuer den naechsten Frame gewuerfelt).
    shake_amp: f64,
    shake_dur_ms: f64,
    shake_start: Option<std::time::Instant>,
    shake_x: f64,
    shake_y: f64,
    // 3D (Modul `g3d`): Befehlsliste + Perspektiv-Kamera. cmds3d wird pro
    // FLIP geleert; cam3d wird von CAMERA3D gesetzt (sonst Default-Blick).
    cmds3d: Vec<Cmd3D>,
    cam3d: Camera3D,
    // Modul m3d: optionale View-/Projektions-Matrix-Overrides (CAMERA3D_VIEW/
    // _PROJECTION). Wenn gesetzt, ueberschreiben sie nach begin_mode3D die von
    // raylib aus cam3d gebauten Matrizen (Ortho, Custom-Frustum, Shadow-Tricks).
    // CAMERA3D(...) loescht beide -> zurueck zur Standard-Perspektive.
    cam3d_view: Option<[f32; 16]>,
    cam3d_proj: Option<[f32; 16]>,
    // 3D-Modelle (LOADMODEL / MESH_*): bleiben ueber Frames erhalten.
    models: Vec<Model>,
    // Skelett-Animationen (MODEL_LOAD_ANIMS): je Set eine RAII-`ModelAnimations`-
    // Collection (raylib-rs 6.0 -- ersetzt den fruehreren rohen FFI-Workaround,
    // der noetig war weil der 5.x-Wrapper die Structs flach kopierte und dann
    // UnloadModelAnimations rief -> Use-after-free).
    model_anims: Vec<ModelAnimations>,
    // Beleuchtung (Blinn-Phong via rlights-Shader). light_shader wird beim
    // ersten LIGHT_ENABLE geladen; lights[] sind bis zu MAX_LIGHTS Lichter.
    light_shader: Option<Shader>,
    // Modell-Indizes mit Normal-Map (useNormalMap=1 nur fuer diese).
    normal_mapped: std::collections::HashSet<usize>,
    loc_use_normal: i32,
    // Per-Modell PBR-Parameter (metalness, roughness); Default (0, 0.6).
    pbr_params: std::collections::HashMap<usize, (f32, f32)>,
    loc_metalness: i32,
    loc_roughness: i32,
    // Per-Modell Eigenleuchten (r, g, b, staerke); Default (0,0,0,0) = aus.
    emissive: std::collections::HashMap<usize, (f32, f32, f32, f32)>,
    loc_emissive: i32,
    lights: Vec<LightData>,
    light_ambient: [f32; 4],
    light_fog: [f32; 4],
    light_fog_density: f32,
    // Analytisches Environment-Lighting (IBL): Sky/Ground-Farbe + Intensitaet.
    env_sky: [f32; 3],
    env_ground: [f32; 3],
    env_intensity: f32,
    loc_env_sky: i32,
    loc_env_ground: i32,
    loc_env_intensity: i32,
    // Echtes HDR-Cubemap-IBL (LIGHT_ENV_HDR): vorberechnete GL-Texturen +
    // Locs. use_ibl_maps gatet den Cubemap-Pfad im Shader (sonst analytisch).
    ibl_irradiance: u32,
    ibl_prefilter: u32,
    ibl_brdf: u32,
    // env-Cubemap (LDR) -- fuer die Skybox aufbewahrt (sonst nach IBL-Gen frei).
    ibl_env: u32,
    skybox_enabled: bool,
    skybox_shader: Option<Shader>,
    skybox_loc_proj: i32,
    skybox_loc_view: i32,
    use_ibl_maps: bool,
    loc_use_ibl: i32,
    loc_irradiance: i32,
    loc_prefilter: i32,
    loc_brdf: i32,
    loc_view: i32,
    loc_ambient: i32,
    loc_fog_color: i32,
    loc_fog_density: i32,
    // Shadow-Mapping: eigenes Depth-FBO (sampleable) + Light-Space-Uniforms.
    shadow_enabled: bool,
    shadow_fbo: u32,
    shadow_depth: u32,
    shadow_res: i32,
    shadow_area: f32,
    shadow_dist: f32,
    shadow_target: [f32; 3],
    loc_light_vp: i32,
    loc_shadow_map: i32,
    loc_shadow_res: i32,
    loc_shadows_on: i32,
    // GPU-Instancing (MODEL_INSTANCED): eigener Shader (INST_VS/FS), lazy geladen.
    // inst_light_locs sind die pro-Licht-Uniform-Locations IN DIESEM Programm
    // (parallel zu `lights`); werden bei Bedarf in update_inst_light_uniforms
    // (re)aufgeloest, wenn sich die Lichter-Anzahl aendert.
    inst_shader: Option<Shader>,
    inst_loc_view: i32,
    inst_loc_ambient: i32,
    inst_loc_count: i32,
    inst_light_locs: Vec<[i32; 5]>,
    text_size: i32,
    // TTF-Fonts (LOADFONT): via raylib load_font_ex geladen. active_font = -1
    // -> raylib-Default-Font; text_spacing = Buchstabenabstand fuer DrawTextEx.
    fonts: Vec<Font>,
    // Render-Groesse je geladenem Font (parallel zu `fonts`). Von LOADFONT
    // gesetzt -> SETFONT uebernimmt sie als text_size (ergonomisch: die in
    // LOADFONT gewaehlte Groesse "wirkt" direkt). 0 = Sentinel (DHRT_FONT-
    // Default): NICHT auto-anwenden, damit der hoch gebackene Default bei der
    // programmweiten text_size bleibt.
    font_sizes: Vec<i32>,
    active_font: i64,
    /// Ausweich-Font fuer Text mit Umlauten, solange kein eigener Font gesetzt
    /// ist: die eingebaute raylib-Schrift kennt nur ASCII und zeichnet sonst
    /// `K?ln`. `None` = kein System-Font gefunden (dann bleibt es beim `?`).
    ///
    /// Bewusst NEBEN `fonts` und nicht darin: sonst waere er Handle 0 und
    /// jedes `LOADFONT` bekaeme eine um eins verschobene Nummer.
    /// Reiner ASCII-Text geht weiterhin durch die eingebaute Schrift -- so
    /// sieht jedes bestehende Programm aus wie zuvor.
    fallback: Option<Font>,
    text_spacing: f32,
    textures: Vec<Tex>,
    image_cache: HashMap<String, i64>,
    /// Platz je Textur: wurde sie mit IMAGE_FREE freigegeben? Der Platz
    /// bleibt stehen und wird NICHT neu vergeben -- sonst zeigte ein
    /// stehengebliebenes Handle spaeter still auf ein fremdes Bild.
    tex_frei: Vec<bool>,
    atlases: Vec<Atlas>,
    pub frame_count: u64,
    max_frames: Option<u64>,
    screenshot: Option<String>,
    // --- Kontaktbogen (DHRT_CONTACT) ---------------------------------------
    // Ein einzelner Screenshot zeigt einen AUGENBLICK. Vieles geht aber erst
    // ueber die Zeit schief: etwas kippt zu frueh um, ein Rand bleibt stehen,
    // eine Bewegung ruckelt. Der Kontaktbogen nimmt in festen Abstaenden Bilder
    // auf und setzt sie als beschriftetes Raster in EINE PNG -- damit wird ein
    // Ablauf pruefbar, nicht nur ein Standbild.
    contact_path: Option<String>,
    contact_every: u64,
    contact_cols: usize,
    contact_max: usize,
    contact_shots: Vec<(u64, Image)>,
    contact_written: bool,
    shot_taken: bool,
    // --- Eingabe aufzeichnen/abspielen (AUTOMATION_*) --------------------
    // Die Liste liegt in einer Box, weil `SetAutomationEventList` sich einen
    // ROHEN ZEIGER auf sie merkt: laege sie direkt im Graphics-Struct, wuerde
    // jedes Verschieben von Graphics den Zeiger ins Leere zeigen lassen.
    auto_list: Option<Box<AutomationEventList>>,
    auto_recording: bool,
    /// Zieldatei der laufenden Aufnahme (beim Stoppen geschrieben).
    auto_path: Option<String>,
    /// Kopien der abzuspielenden Ereignisse (die Liste selbst besitzt raylib).
    auto_events: Vec<AutomationEvent>,
    auto_playing: bool,
    auto_play_idx: usize,
    /// Frame-Zaehler der Wiedergabe (0 = erster Frame nach AUTOMATION_PLAY).
    auto_play_frame: u32,
    /// Frame-Nummer des ersten Ereignisses -- die Aufnahme beginnt nicht
    /// zwingend bei 0 (raylibs Zaehler laeuft seit Programmstart durch).
    auto_play_base: u32,
    /// raylib-Tastencodes, die die laufende Wiedergabe zuletzt selbst
    /// eingespeist hat. Noetig, weil raylib eingespeiste Tasten auch in seine
    /// "zuletzt gedrueckt"-Warteschlange legt -- ohne diese Liste meldet
    /// `KEY_ANY_HIT` die Demo-Tasten als Nutzereingabe, und ein Attract-Modus
    /// ("bei Tastendruck abbrechen") wuerde sich selbst sofort beenden.
    auto_injected_keys: Vec<i32>,
    /// Gemerkte Anzeigenamen je Tastencode (siehe `key_name`).
    key_names: HashMap<i64, String>,
    // Post-Processing (Shader): die Szene wird in `scene_rt` gerendert und beim
    // FLIP per Fragment-Shader (Index in `shaders`) auf den Screen praesentiert.
    shaders: Vec<Shader>,
    /// Zusaetzliche Sampler je Shader: (Uniform-Position, Textur).
    /// Sie werden ERST beim Zeichnen gesetzt -- `SetShaderValueTexture` ruft
    /// intern `glUniform1i` und wirkt damit auf das GERADE AKTIVE Programm.
    /// Ausserhalb von `BeginShaderMode` landet die Zuweisung also am falschen
    /// Shader und der Sampler bleibt schwarz.
    shader_textures: HashMap<usize, Vec<(i32, raylib::ffi::Texture2D)>>,
    post_shader_idx: Option<usize>,
    /// Stapel fuer GFX_PUSH/GFX_POP.
    gfx_stack: Vec<GfxState>,
    /// Wie viele Clip-Rechtecke gerade offen sind. Wird pro Bild
    /// zurueckgesetzt -- der Abspieler raeumt unbalancierte Clips ohnehin
    /// am Bildende weg, der Zaehler muss ihm darin folgen.
    clip_tiefe: u32,
    scene_rt: Option<RenderTexture2D>,
}

/// Welche Zeichen in einen geladenen Font gebacken werden: ASCII, der ganze
/// Latin-1-Bereich (deutsche Umlaute, Akzente franzoesischer und spanischer
/// Vereinsnamen) und gaengige Typografie/Pfeile. raylib backt ohne diese
/// Liste nur die 95 ASCII-Glyphen und zeichnet fuer alles andere ein `?`.
fn zeichensatz() -> String {
    let mut chars = String::new();
    for c in 0x20u32..=0x7Eu32 { chars.push(char::from_u32(c).unwrap()); }
    for c in 0xA0u32..=0xFFu32 { chars.push(char::from_u32(c).unwrap()); }
    chars.push_str("…–—„“”‚‘’·•°→←↑↓×÷≈≠≤≥");
    chars
}

/// Fuehrt `f` aus, waehrend raylib nur Fehler meldet.
///
/// Beim Laden einer Schrift zaehlt raylib nach, wie viele der angeforderten
/// Zeichen wirklich drin sind, und meldet jedes fehlende als Warnung auf
/// stdout: "Requested codepoints glyphs found: [209/213]". Eine Pixel-Schrift
/// ohne französische Akzente ist aber kein Fehler, und stdout gehoert der
/// Programmausgabe -- ein PRINT-Ergebnis darf nicht zwischen Font-Meldungen
/// stehen.
fn ohne_warnungen<T>(f: impl FnOnce() -> T) -> T {
    use raylib::consts::TraceLogLevel;
    unsafe { raylib::ffi::SetTraceLogLevel(TraceLogLevel::LOG_ERROR as i32); }
    let r = f();
    unsafe { raylib::ffi::SetTraceLogLevel(TraceLogLevel::LOG_WARNING as i32); }
    r
}

/// Handle des Ausweich-Fonts in einem Zeichen-Befehl. Nicht -1 (eingebaute
/// Schrift) und kein Index in `fonts`, damit die von LOADFONT vergebenen
/// Nummern unveraendert bei 0, 1, 2 ... beginnen.
const FONT_AUSWEICH: i64 = -2;

/// Kandidaten fuer den Ausweich-Font: die eingebaute raylib-Bitmapschrift
/// kennt nur ASCII, deshalb braucht "Köln" eine echte Schrift vom System.
/// Der erste vorhandene Pfad gewinnt.
fn ausweich_schriften() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    { &["C:/Windows/Fonts/segoeui.ttf", "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/tahoma.ttf"] }
    #[cfg(target_os = "macos")]
    { &["/System/Library/Fonts/SFNS.ttf", "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf"] }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    { &["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf"] }
}

/// GB-Farbe (0xRRGGBB INTEGER) -> raylib Color.
fn col(c: i64) -> Color {
    let v = c as u32;
    // Oberes Byte = Alpha. 0 bedeutet DECKEND (255) -- so bleiben die alten
    // 24-bit-Farben `&Hrrggbb` / RGB(r,g,b) voll deckend, waehrend
    // RGBA(r,g,b,a) bzw. `&Haarrggbb` mit a>0 echte Transparenz liefert.
    let a = ((v >> 24) & 0xFF) as u8;
    let a = if a == 0 { 255 } else { a };
    Color::new(((v >> 16) & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, (v & 0xFF) as u8, a)
}

impl Graphics {
    /// Lazy-Init ohne SCREEN: ein verstecktes Fenster, nur fuer den GL-Kontext
    /// (LOADIMAGE/imgfx-Texturen, Kamera-/Sprite-Logik ohne sichtbares Fenster).
    /// Damit funktionieren LOADIMAGE & Co. auch VOR (oder ganz ohne) SCREEN.
    /// Ein spaeteres SCREEN macht das Fenster via `reconfigure` sichtbar.
    pub fn new_headless() -> Graphics {
        Graphics::new_impl(64, 64, "Drachenhauch", 1, true, false)
    }
    /// Fenster mit transparentem Framebuffer (SCREEN_TRANSPARENT). Das Flag muss
    /// schon bei der Fenster-Erzeugung gesetzt sein -- nicht nachtraeglich machbar.
    pub fn new_transparent(width: i32, height: i32, title: &str, scale: i32) -> Graphics {
        Graphics::new_impl(width.max(1), height.max(1), title, scale, false, true)
    }
    /// Das (bereits erzeugte) Fenster den ganzen aktuellen Monitor abdecken lassen:
    /// auf native Aufloesung umkonfigurieren und an die Monitor-Ecke setzen. Fuer
    /// transparente Vollbild-Overlays (Monitor-Query braucht ein offenes Fenster,
    /// darum NACH der Erzeugung). DHRT_SCALE wird wie bei SCREEN_NATIVE respektiert.
    pub fn cover_current_monitor(&mut self, title: &str) {
        let m = get_current_monitor();
        let mw = get_monitor_width(m).max(1);
        let mh = get_monitor_height(m).max(1);
        let scale = std::env::var("DHRT_SCALE").ok()
            .and_then(|s| s.parse::<i32>().ok()).filter(|&n| n >= 1).unwrap_or(1);
        let lw = (mw / scale).max(1);
        let lh = (mh / scale).max(1);
        self.reconfigure_raw(lw, lh, title, scale);
        let pos = get_monitor_position(m);
        self.rl.set_window_position(pos.x as i32, pos.y as i32);
    }

    pub fn new(width: i32, height: i32, title: &str, scale: i32) -> Graphics {
        Graphics::new_impl(width, height, title, scale, false, false)
    }

    fn new_impl(width: i32, height: i32, title: &str, scale: i32, hidden: bool, transparent: bool) -> Graphics {
        // DHRT_SCALE erlaubt es, JEDEN SCREEN-Aufruf hochskaliert zu rendern
        // (z.B. fuer scharfe Buch-Screenshots), ohne die .dh-Quelle zu aendern.
        let scale = std::env::var("DHRT_SCALE").ok()
            .and_then(|s| s.parse::<i32>().ok()).filter(|&n| n >= 1).unwrap_or(scale);
        let win_w = width * scale;
        let win_h = height * scale;
        // raylib loggt sonst seinen INFO-Startup-Spam auf stdout und verschmutzt
        // die Konsolen-Ausgabe (TW ist sauber). WARNING zeigt weiter echte
        // Warnungen/Fehler (z.B. fehlgeschlagenes Texture-Load).
        let mut builder = raylib::init();
        builder.size(win_w, win_h)
            .title(title)
            .log_level(raylib::consts::TraceLogLevel::LOG_WARNING);
        // FLAG_WINDOW_TRANSPARENT MUSS vor build() gesetzt sein (GLFW-Hint).
        if transparent { builder.transparent(); }
        let (mut rl, thread) = builder.build();
        if hidden {
            rl.set_window_state(WindowState::default().set_window_hidden(true));
        }
        rl.set_target_fps(60);
        // Web: die HTML-Leinwand auf die Fenstergroesse bringen.
        //
        // Nachgemessen im Browser: die Laufzeit meldet nach SCREEN(480,320)
        // brav 480x320 -- der PUFFER der <canvas> blieb aber 1x1, und per CSS
        // auf volle Breite gestreckt wurde daraus eine einzige Farbflaeche.
        // raylibs Web-Pfad setzt die Groesse hier nicht durch; emscriptens
        // eigener Aufruf tut es zuverlaessig.
        #[cfg(target_os = "emscripten")]
        web_leinwand_groesse(win_w, win_h);
        // Headless-Verifizierung: DHRT_FRAMES begrenzt die Frames, DHRT_SCREENSHOT
        // legt den PNG-Pfad fest (Screenshot beim letzten Frame).
        let max_frames = std::env::var("DHRT_FRAMES").ok().and_then(|s| s.parse().ok());
        let screenshot = std::env::var("DHRT_SCREENSHOT").ok();
        // Kontaktbogen: DHRT_CONTACT=pfad.png. Ohne eigene Angaben verteilt er
        // DHRT_CONTACT_MAX Bilder gleichmaessig ueber DHRT_FRAMES -- man setzt
        // also im Normalfall nur zwei Umgebungsvariablen und bekommt einen
        // Ueberblick ueber den ganzen Lauf.
        let contact_path = std::env::var("DHRT_CONTACT").ok();
        let contact_max: usize = std::env::var("DHRT_CONTACT_MAX").ok()
            .and_then(|s| s.parse().ok()).filter(|&n: &usize| n > 0).unwrap_or(12);
        let contact_cols: usize = std::env::var("DHRT_CONTACT_COLS").ok()
            .and_then(|s| s.parse().ok()).filter(|&n: &usize| n > 0).unwrap_or(4);
        let contact_every: u64 = std::env::var("DHRT_CONTACT_EVERY").ok()
            .and_then(|s| s.parse().ok()).filter(|&n: &u64| n > 0)
            .or_else(|| max_frames.map(|mx: u64| (mx / contact_max as u64).max(1)))
            .unwrap_or(30);
        let mut layer_names = HashMap::new();
        layer_names.insert(String::new(), 0usize); // Main-Layer
        // Szene-Render-Target (Fenstergroesse) fuer Post-Processing.
        let scene_rt = rl.load_render_texture(&thread, win_w as u32, win_h as u32).ok();
        let mut g = Graphics {
            rl, thread, width, height, scale,
            fullscreen: false, pre_fullscreen: None,
            shaders: Vec::new(), shader_textures: HashMap::new(),
            post_shader_idx: None, gfx_stack: Vec::new(), clip_tiefe: 0, scene_rt,
            layers: vec![Layer { z: 0, cmds: Vec::new() }],
            layer_names,
            active: 0,
            render_targets: Vec::new(),
            active_rt: None,
            // Transparente Fenster starten voll durchsichtig (Alpha 0), normale deckend schwarz.
            clear_color: if transparent { Color::new(0, 0, 0, 0) } else { Color::BLACK },
            transparent,
            cam_x: 0.0, cam_y: 0.0, cam_zoom: 1.0, cam_rotation: 0.0,
            shake_amp: 0.0, shake_dur_ms: 0.0, shake_start: None,
            shake_x: 0.0, shake_y: 0.0,
            cmds3d: Vec::new(),
            models: Vec::new(),
            model_anims: Vec::new(),
            light_shader: None,
            normal_mapped: std::collections::HashSet::new(),
            loc_use_normal: -1,
            pbr_params: std::collections::HashMap::new(),
            loc_metalness: -1,
            loc_roughness: -1,
            emissive: std::collections::HashMap::new(),
            loc_emissive: -1,
            lights: Vec::new(),
            light_ambient: [0.1, 0.1, 0.1, 1.0],
            light_fog: [0.0, 0.0, 0.0, 1.0],
            light_fog_density: 0.0,
            env_sky: [0.5, 0.7, 1.0],
            env_ground: [0.2, 0.2, 0.2],
            env_intensity: 0.0,
            loc_env_sky: -1,
            loc_env_ground: -1,
            loc_env_intensity: -1,
            ibl_irradiance: 0,
            ibl_prefilter: 0,
            ibl_brdf: 0,
            ibl_env: 0,
            skybox_enabled: false,
            skybox_shader: None,
            skybox_loc_proj: -1,
            skybox_loc_view: -1,
            use_ibl_maps: false,
            loc_use_ibl: -1,
            loc_irradiance: -1,
            loc_prefilter: -1,
            loc_brdf: -1,
            loc_view: -1,
            loc_ambient: -1,
            loc_fog_color: -1,
            loc_fog_density: -1,
            shadow_enabled: false,
            shadow_fbo: 0,
            shadow_depth: 0,
            shadow_res: 1024,
            shadow_area: 25.0,
            shadow_dist: 50.0,
            shadow_target: [0.0, 0.0, 0.0],
            loc_light_vp: -1,
            loc_shadow_map: -1,
            loc_shadow_res: -1,
            loc_shadows_on: -1,
            inst_shader: None,
            inst_loc_view: -1,
            inst_loc_ambient: -1,
            inst_loc_count: -1,
            inst_light_locs: Vec::new(),
            // Default-Blick: schraeg von vorn-oben auf den Ursprung.
            cam3d: Camera3D::perspective(
                Vector3::new(6.0, 5.0, 6.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                45.0),
            cam3d_view: None,
            cam3d_proj: None,
            text_size: 20,
            fonts: Vec::new(),
            font_sizes: Vec::new(),
            active_font: -1,
            fallback: None,
            text_spacing: 0.0,
            textures: Vec::new(),
            image_cache: HashMap::new(),
            tex_frei: Vec::new(),
            atlases: Vec::new(),
            frame_count: 0,
            max_frames,
            screenshot,
            contact_path, contact_every, contact_cols, contact_max,
            contact_shots: Vec::new(), contact_written: false,
            shot_taken: false,
            auto_list: None,
            auto_recording: false,
            auto_path: None,
            auto_events: Vec::new(),
            auto_playing: false,
            auto_play_idx: 0,
            auto_play_frame: 0,
            auto_play_base: 0,
            auto_injected_keys: Vec::new(),
            key_names: HashMap::new(),
        };
        // DHRT_FONT setzt einen TTF als Default-Font (scharfe Schrift in
        // Screenshots statt der pixeligen raylib-Bitmap-Schrift). Basis-Groesse
        // an die Skala gekoppelt, damit Text bei sz*scale knackig bleibt.
        if let Ok(fp) = std::env::var("DHRT_FONT") {
            if !fp.is_empty() {
                let bake = (32 * scale.max(1)).clamp(32, 256);
                if let Ok(h) = g.load_font_ext(&fp, bake) {
                    let _ = g.set_font(h);
                }
            }
        }
        // Ausweich-Font fuer Umlaute laden, solange kein Default gesetzt wurde.
        // Einmal beim Fensterstart, nicht beim ersten Umlaut: TEXT_WIDTH muss
        // dieselbe Schrift messen, die TEXT spaeter zeichnet -- sonst rechnet
        // ein Layout mit anderen Breiten, als am Ende dastehen.
        if g.active_font < 0 {
            let bake = (32 * scale.max(1)).clamp(32, 256);
            let chars = zeichensatz();
            for pfad in ausweich_schriften() {
                if !std::path::Path::new(pfad).exists() { continue; }
                let rl = &mut g.rl;
                let thread = &g.thread;
                if let Ok(f) = ohne_warnungen(|| rl.load_font_ex(thread, pfad, bake, Some(&chars))) {
                    unsafe { raylib::ffi::SetTextureFilter(f.texture, 1 /*BILINEAR*/); }
                    g.fallback = Some(f);
                }
                break;
            }
        }
        g
    }

    /// Welcher Font zeichnet diesen Text? Normalerweise der aktive. Steht
    /// keiner (eingebaute Bitmapschrift) und enthaelt der Text Zeichen
    /// jenseits von ASCII, springt der Ausweich-Font ein -- als Handle
    /// `FONT_AUSWEICH`, das nicht in `fonts` steht.
    fn font_fuer(&self, s: &str) -> i64 {
        if self.active_font >= 0 { return self.active_font; }
        if self.fallback.is_some() && !s.is_ascii() { return FONT_AUSWEICH; }
        -1
    }

    /// Font zu einem Handle -- inklusive des Ausweich-Fonts, den `fonts`
    /// nicht enthaelt. `None` = eingebaute Schrift.
    fn font_von(&self, h: i64) -> Option<&Font> {
        if h == FONT_AUSWEICH { return self.fallback.as_ref(); }
        if h < 0 { return None; }
        self.fonts.get(h as usize)
    }

    /// SCREEN nach einem Lazy-Init (oder erneutes SCREEN): das bestehende Fenster
    /// sichtbar machen und auf die gewuenschte Groesse/Titel umstellen, statt ein
    /// zweites raylib-Fenster zu erzeugen (raylib paniced bei Doppel-Init).
    pub fn reconfigure(&mut self, width: i32, height: i32, title: &str, scale: i32) {
        let scale = std::env::var("DHRT_SCALE").ok()
            .and_then(|s| s.parse::<i32>().ok()).filter(|&n| n >= 1).unwrap_or(scale);
        self.reconfigure_raw(width, height, title, scale);
    }

    /// Wie `reconfigure`, aber `scale` wird direkt uebernommen (kein DHRT_SCALE-
    /// Override). Gemeinsame Basis fuer SCREEN und SCREEN_NATIVE.
    fn reconfigure_raw(&mut self, width: i32, height: i32, title: &str, scale: i32) {
        self.width = width;
        self.height = height;
        self.scale = scale;
        let win_w = width * scale;
        let win_h = height * scale;
        self.rl.clear_window_state(WindowState::default().set_window_hidden(true));
        self.rl.set_window_size(win_w, win_h);
        self.rl.set_window_title(&self.thread, title);
        // Szene-Render-Target an die neue Groesse anpassen (Post-Processing).
        self.scene_rt = self.rl.load_render_texture(&self.thread, win_w as u32, win_h as u32).ok();
    }

    /// SCREEN_NATIVE([titel$]) -- Vollbild in der ECHTEN Aufloesung des aktuellen
    /// Monitors. Anders als `SCREEN(w,h)` + `SET_FULLSCREEN(TRUE)` (das ein kleines
    /// Backbuffer auf den Monitor hochskaliert -> unscharf) rendert die Szene hier
    /// 1:1 in nativen Pixeln: logisches Raster = Monitor-Aufloesung, scene_rt und
    /// Replay-Viewport sind nativ gross. SCREENWIDTH()/HEIGHT() liefern dann die
    /// Monitor-Aufloesung. DHRT_SCALE (Dev-Screenshot-Knopf) wird respektiert:
    /// das logische Raster wird entsprechend geteilt, das Fenster bleibt nativ.
    ///
    /// Wir nutzen BORDERLESS-WINDOWED (randloses Fenster ueber den ganzen Monitor),
    /// nicht exklusives Vollbild (`toggle_fullscreen`): letzteres macht auf manchen
    /// Setups einen unsauberen Video-Mode-Wechsel (GLFW „failed to query video mode",
    /// Hoehe landet z.B. bei 1421 statt 1440). Borderless wechselt KEINEN Video-Modus
    /// und deckt den Monitor exakt nativ ab.
    pub fn screen_native(&mut self, title: &str) {
        let m = get_current_monitor();
        let mw = get_monitor_width(m).max(1);
        let mh = get_monitor_height(m).max(1);
        let scale = std::env::var("DHRT_SCALE").ok()
            .and_then(|s| s.parse::<i32>().ok()).filter(|&n| n >= 1).unwrap_or(1);
        let lw = (mw / scale).max(1);
        let lh = (mh / scale).max(1);
        self.reconfigure_raw(lw, lh, title, scale);
        self.rl.toggle_borderless_windowed();
    }

    fn emit(&mut self, c: Cmd) {
        // Ist ein Render-Target aktiv (RENDERTARGET_BEGIN), gehen alle Draws in
        // dessen Command-Buffer statt in den aktiven Layer.
        if let Some(rt) = self.active_rt {
            self.render_targets[rt].cmds.push(c);
        } else {
            let a = self.active;
            self.layers[a].cmds.push(c);
        }
    }

    // --- Render-Targets (RENDERTARGET_*) ---
    /// Legt ein neues Render-Target (RenderTexture2D) an -> Handle (Index).
    pub fn rendertarget_new(&mut self, w: i32, h: i32, behalten: bool) -> Result<i64, String> {
        let rt = self.rl.load_render_texture(&self.thread, w.max(1) as u32, h.max(1) as u32)
            .map_err(|e| format!("RENDERTARGET_NEW: {}", e))?;
        self.render_targets.push(RenderTarget { rt, cmds: Vec::new(), behalten });
        Ok((self.render_targets.len() - 1) as i64)
    }

    /// RENDERTARGET_CLEAR(rt [, farbe]): ein behaltenes Target von Hand leeren.
    /// Ohne Farbe transparent -- so wie es ein normales Target jedes Bild tut.
    pub fn rendertarget_clear(&mut self, idx: i64, farbe: Option<i64>) -> Result<(), String> {
        let i = self.check_rt(idx, "RENDERTARGET_CLEAR")?;
        let c = match farbe { Some(v) => col(v), None => Color::new(0, 0, 0, 0) };
        // Als erster Befehl des Puffers -- alles, was in diesem Bild schon
        // hineingezeichnet wurde, soll ja auch weg sein.
        self.render_targets[i].cmds.insert(0, Cmd::Clear(c));
        Ok(())
    }
    fn check_rt(&self, idx: i64, fn_: &str) -> Result<usize, String> {
        let i = idx as usize;
        if idx < 0 || i >= self.render_targets.len() {
            return Err(format!("{}: ungueltiges RENDERTARGET-Handle {}", fn_, idx));
        }
        Ok(i)
    }
    /// Folgende Draws gehen in das Target, bis RENDERTARGET_END.
    pub fn rendertarget_begin(&mut self, idx: i64) -> Result<(), String> {
        let i = self.check_rt(idx, "RENDERTARGET_BEGIN")?;
        self.active_rt = Some(i);
        Ok(())
    }
    pub fn rendertarget_end(&mut self) { self.active_rt = None; }
    /// Zeichnet das Target (seine Textur) an Position x,y, skaliert + getoent.
    pub fn rendertarget_draw(&mut self, idx: i64, x: i32, y: i32, scale: f64, tint: Option<i64>, flip_v: bool) -> Result<(), String> {
        let i = self.check_rt(idx, "RENDERTARGET_DRAW")?;
        let (x, y) = self.w2s(x, y);
        let tcol = match tint { Some(c) => col(c), None => Color::WHITE };
        self.emit(Cmd::RtDraw(i, x, y, (scale * self.cam_zoom).max(0.0) as f32, tcol, flip_v));
        Ok(())
    }

    // --- Kamera (Modul `camera`) ---
    // World->Screen: sx = int((x - cam_x) * zoom). Bei Identitaet (0,0,1) No-Op.
    //
    // Review-Fund: while RENDERTARGET_BEGIN/END ist aktiv, liefen Draws bisher
    // trotzdem durch w2s()/ssize() -- Inhalt, der IN ein Target gezeichnet
    // wird, bekam so die aktuelle Kamera-Transformation eingebrannt. Wird das
    // Target danach per RENDERTARGET_DRAW auf den Screen gestempelt, ist DAS
    // selbst wiederum ein camera-aware Draw (eigener w2s()/ssize()-Aufruf in
    // rendertarget_draw) -- die Kamera wirkte damit doppelt. Ein Render-Target
    // ist konzeptionell eine eigene, kamera-unabhaengige Leinwand (das Stempeln
    // selbst bleibt camera-aware, weil active_rt dabei bereits None ist) --
    // daher hier gaten: waehrend active_rt gesetzt ist, No-Op/Identitaet.
    fn w2s(&self, x: i32, y: i32) -> (i32, i32) {
        if self.active_rt.is_some() { return (x, y); }
        let px = (x as f64 - self.cam_x + self.shake_x) * self.cam_zoom;
        let py = (y as f64 - self.cam_y + self.shake_y) * self.cam_zoom;
        if self.cam_rotation == 0.0 { return (px as i32, py as i32); }
        let (rx, ry) = self.rotate_around_screen_center(px, py, -self.cam_rotation);
        (rx as i32, ry as i32)
    }
    fn ssize(&self, s: i32) -> i32 {
        if self.active_rt.is_some() { return s.max(0); }
        ((s as f64 * self.cam_zoom) as i32).max(0)
    }
    /// Dreht einen bereits translatierten/gezoomten Screen-Punkt um den
    /// logischen Bildschirm-Mittelpunkt (width/height/2) -- der Punkt genau
    /// im Zentrum bleibt fix, weshalb CAMERA_FOLLOW's "target landet in der
    /// Mitte"-Rechnung auch bei aktiver Rotation unveraendert stimmt.
    fn rotate_around_screen_center(&self, px: f64, py: f64, deg: f64) -> (f64, f64) {
        rotate_point_around(px, py, self.width as f64 / 2.0, self.height as f64 / 2.0, deg)
    }
    pub fn set_camera(&mut self, x: f64, y: f64, zoom: f64) { self.cam_x = x; self.cam_y = y; self.cam_zoom = zoom; }
    pub fn reset_camera(&mut self) { self.cam_x = 0.0; self.cam_y = 0.0; self.cam_zoom = 1.0; self.cam_rotation = 0.0; }
    pub fn set_camera_rotation(&mut self, deg: f64) { self.cam_rotation = deg; }
    pub fn camera_rotation(&self) -> f64 { self.cam_rotation }

    /// CAMERA_SHAKE(staerke, dauer_ms) -- zufaelliger Kamera-Ruckel-Offset
    /// (Welt-Pixel), klingt linear ueber dauer_ms ab. Liegt im selben
    /// World->Screen-Pfad wie CAMERA_SET und wirkt damit auf ALLE 2D-Draws
    /// (bewusst: der ganze Screen wackelt, inkl. HUD -- das ist der Effekt).
    /// staerke=0 stoppt einen laufenden Shake sofort.
    pub fn camera_shake(&mut self, amp: f64, dur_ms: f64) {
        if amp <= 0.0 {
            self.shake_start = None;
            self.shake_x = 0.0;
            self.shake_y = 0.0;
            return;
        }
        self.shake_amp = amp;
        self.shake_dur_ms = dur_ms;
        self.shake_start = Some(std::time::Instant::now());
        self.update_shake();              // wirkt schon im aktuellen Frame
    }
    /// Pro Frame (flip-Ende) den Offset fuer den naechsten Frame wuerfeln.
    fn update_shake(&mut self) {
        if let Some(start) = self.shake_start {
            let rem = shake_remaining(start.elapsed().as_secs_f64() * 1000.0, self.shake_dur_ms);
            if rem <= 0.0 {
                self.shake_start = None;
                self.shake_x = 0.0;
                self.shake_y = 0.0;
            } else {
                let a = self.shake_amp * rem;
                self.shake_x = crate::builtins::rng_uniform(-a, a);
                self.shake_y = crate::builtins::rng_uniform(-a, a);
            }
        }
    }
    pub fn camera(&self) -> (f64, f64, f64) { (self.cam_x, self.cam_y, self.cam_zoom) }
    pub fn s2w_x(&self, sx: f64) -> f64 { if self.cam_zoom == 0.0 { sx } else { sx / self.cam_zoom + self.cam_x } }
    pub fn s2w_y(&self, sy: f64) -> f64 { if self.cam_zoom == 0.0 { sy } else { sy / self.cam_zoom + self.cam_y } }
    /// Rotations-bewusste Umkehrung von w2s: braucht BEIDE Screen-Koordinaten
    /// (Rotation mischt x/y), darum eigene _rot-Varianten statt s2w_x/y direkt
    /// zu aendern -- CAMERA_S2W_X(sx) ohne sy bleibt bei Rotation=0 exakt wie
    /// bisher (Abwaertskompatibel fuer alle Scripte ohne CAMERA_SET_ROTATION).
    pub fn s2w_x_rot(&self, sx: f64, sy: f64) -> f64 {
        if self.cam_rotation == 0.0 { return self.s2w_x(sx); }
        let (ux, _) = self.rotate_around_screen_center(sx, sy, self.cam_rotation);
        self.s2w_x(ux)
    }
    pub fn s2w_y_rot(&self, sx: f64, sy: f64) -> f64 {
        if self.cam_rotation == 0.0 { return self.s2w_y(sy); }
        let (_, uy) = self.rotate_around_screen_center(sx, sy, self.cam_rotation);
        self.s2w_y(uy)
    }

    // --- 3D (Modul `g3d`) ---
    #[allow(clippy::too_many_arguments)]
    pub fn set_camera3d(&mut self, px: f32, py: f32, pz: f32,
                        tx: f32, ty: f32, tz: f32, fovy: f32) {
        self.cam3d = Camera3D::perspective(
            Vector3::new(px, py, pz),
            Vector3::new(tx, ty, tz),
            Vector3::new(0.0, 1.0, 0.0),
            fovy);
        // Standard-Perspektive -> etwaige Matrix-Overrides verwerfen.
        self.cam3d_view = None;
        self.cam3d_proj = None;
    }
    /// Orbit-Kamera: blickt aus `radius` Abstand auf das Ziel, gesteuert ueber
    /// `yaw`/`pitch` (Grad). Spart die manuelle Kugelkoordinaten-Trigonometrie.
    /// `fovy <= 0` behaelt die aktuelle Brennweite (sonst 45).
    pub fn camera_orbit(&mut self, tx: f32, ty: f32, tz: f32,
                        radius: f32, yaw_deg: f32, pitch_deg: f32, fovy: f32) {
        // Pitch knapp unter +-90 halten -> kein Gimbal-Flip am Pol.
        let pitch = pitch_deg.clamp(-89.9, 89.9).to_radians();
        let yaw = yaw_deg.to_radians();
        let cp = pitch.cos();
        let px = tx + radius * cp * yaw.sin();
        let py = ty + radius * pitch.sin();
        let pz = tz + radius * cp * yaw.cos();
        let f = if fovy > 0.0 { fovy }
                else if self.cam3d.fovy > 0.0 { self.cam3d.fovy }
                else { 45.0 };
        self.set_camera3d(px, py, pz, tx, ty, tz, f);
    }
    /// Modul m3d: View-Matrix-Override (CAMERA3D_VIEW). `mat` column-major.
    pub fn set_camera3d_view(&mut self, mat: [f32; 16]) { self.cam3d_view = Some(mat); }
    /// Modul m3d: Projektions-Matrix-Override (CAMERA3D_PROJECTION).
    pub fn set_camera3d_projection(&mut self, mat: [f32; 16]) { self.cam3d_proj = Some(mat); }
    /// Kamera per raylib-Controller bewegen (liest Tastatur/Maus). mode:
    /// 1=free, 2=orbital, 3=first_person, 4=third_person (sonst custom/no-op).
    /// cam3d bleibt zwischen Frames erhalten -> CAMERA3D einmal initial setzen,
    /// dann jeden Frame CAMERA3D_UPDATE(mode).
    pub fn camera3d_update(&mut self, mode: i64) {
        use raylib::consts::CameraMode::*;
        let m = match mode {
            1 => CAMERA_FREE,
            2 => CAMERA_ORBITAL,
            3 => CAMERA_FIRST_PERSON,
            4 => CAMERA_THIRD_PERSON,
            _ => CAMERA_CUSTOM,
        };
        self.cam3d.update_camera(m);
    }
    pub fn cam3d_pos(&self) -> (f64, f64, f64) {
        (self.cam3d.position.x as f64, self.cam3d.position.y as f64, self.cam3d.position.z as f64)
    }
    pub fn cam3d_target(&self) -> (f64, f64, f64) {
        (self.cam3d.target.x as f64, self.cam3d.target.y as f64, self.cam3d.target.z as f64)
    }

    fn emit3d(&mut self, c: Cmd3D) { self.cmds3d.push(c); }

    #[allow(clippy::too_many_arguments)]
    pub fn cube(&mut self, x: f32, y: f32, z: f32, w: f32, h: f32, d: f32, col_: i64, wires: bool) {
        let c = col(col_);
        if wires { self.emit3d(Cmd3D::CubeWires(x, y, z, w, h, d, c)); }
        else { self.emit3d(Cmd3D::Cube(x, y, z, w, h, d, c)); }
    }
    pub fn sphere(&mut self, x: f32, y: f32, z: f32, r: f32, col_: i64, wires: bool) {
        let c = col(col_);
        if wires { self.emit3d(Cmd3D::SphereWires(x, y, z, r, c)); }
        else { self.emit3d(Cmd3D::Sphere(x, y, z, r, c)); }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn cylinder(&mut self, x: f32, y: f32, z: f32, rt: f32, rb: f32, h: f32, col_: i64) {
        self.emit3d(Cmd3D::Cylinder(x, y, z, rt, rb, h, col(col_)));
    }
    pub fn plane(&mut self, x: f32, y: f32, z: f32, sx: f32, sz: f32, col_: i64) {
        self.emit3d(Cmd3D::Plane(x, y, z, sx, sz, col(col_)));
    }
    #[allow(clippy::too_many_arguments)]
    pub fn line3d(&mut self, x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32, col_: i64) {
        self.emit3d(Cmd3D::Line(x1, y1, z1, x2, y2, z2, col(col_)));
    }
    pub fn point3d(&mut self, x: f32, y: f32, z: f32, col_: i64) {
        self.emit3d(Cmd3D::Point(x, y, z, col(col_)));
    }
    pub fn grid3d(&mut self, slices: i32, spacing: f32) {
        self.emit3d(Cmd3D::Grid(slices.max(0), spacing));
    }

    // --- 3D-Modelle ---
    /// Laedt ein Modell von Datei (OBJ/GLTF/...) -> MODEL-Handle (Index).
    pub fn load_model(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        let m = self.rl.load_model(&self.thread, path)
            .map_err(|e| format!("LOADMODEL: '{}' nicht ladbar: {}", path, e))?;
        self.models.push(m);
        Ok((self.models.len() - 1) as i64)
    }
    /// Laedt Skelett-Animationen (GLTF/IQM/M3D) aus einer Datei -> ANIM_SET-Index.
    pub fn load_model_anims(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let anims = self.rl.load_model_animations(&self.thread, resolved.as_str())
            .map_err(|_| format!("MODEL_LOAD_ANIMS: '{}' enthaelt keine Animationen", path))?;
        self.model_anims.push(anims);
        Ok((self.model_anims.len() - 1) as i64)
    }
    fn check_anim(&self, set: i64, idx: i64, fn_: &str) -> Result<(usize, usize), String> {
        let s = set as usize;
        if set < 0 || s >= self.model_anims.len() {
            return Err(format!("{}: ungueltiges ANIM_SET-Handle {}", fn_, set));
        }
        let cnt = self.model_anims[s].len() as i64;
        if idx < 0 || idx >= cnt {
            return Err(format!("{}: Animations-Index {} ausserhalb [0..{}]", fn_, idx, cnt - 1));
        }
        Ok((s, idx as usize))
    }
    /// Anzahl Animationen im Set.
    pub fn anim_count(&self, set: i64) -> Result<i64, String> {
        let s = set as usize;
        if set < 0 || s >= self.model_anims.len() {
            return Err(format!("MODEL_ANIM_COUNT: ungueltiges ANIM_SET-Handle {}", set));
        }
        Ok(self.model_anims[s].len() as i64)
    }
    /// Frame-Anzahl einer Animation.
    pub fn anim_frames(&self, set: i64, idx: i64) -> Result<i64, String> {
        let (s, a) = self.check_anim(set, idx, "MODEL_ANIM_FRAMES")?;
        Ok(self.model_anims[s][a].keyframeCount as i64)
    }
    /// Name einer Animation (leer falls keiner gesetzt).
    pub fn anim_name(&self, set: i64, idx: i64) -> Result<String, String> {
        let (s, a) = self.check_anim(set, idx, "MODEL_ANIM_NAME")?;
        let raw = self.model_anims[s][a].name;
        let bytes: Vec<u8> = raw.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    /// Setzt das Modell auf Frame `frame` der Animation `anim_idx` aus `set`.
    pub fn model_animate(&mut self, model_idx: i64, set: i64, anim_idx: i64, frame: i32) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_ANIMATE")?;
        let (s, a) = self.check_anim(set, anim_idx, "MODEL_ANIMATE")?;
        let frames = self.model_anims[s][a].keyframeCount.max(1);
        let f = frame.rem_euclid(frames) as f32;                   // loopt automatisch
        self.rl.update_model_animation(&self.thread, &mut self.models[mi], &self.model_anims[s][a], f);
        Ok(())
    }
    /// Blendet zwischen zwei Animationen desselben Sets (neu in raylib 6.0:
    /// `UpdateModelAnimationEx`). `blend` 0.0 = ganz `anim_a`, 1.0 = ganz
    /// `anim_b`, dazwischen interpoliert -- fuer weiche Uebergaenge (z.B.
    /// Walk->Run) statt hartem Anim-Wechsel.
    #[allow(clippy::too_many_arguments)]
    pub fn model_animate_blend(&mut self, model_idx: i64, set: i64,
                                anim_a: i64, frame_a: i32,
                                anim_b: i64, frame_b: i32, blend: f32) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_ANIMATE_BLEND")?;
        let (s, ia) = self.check_anim(set, anim_a, "MODEL_ANIMATE_BLEND")?;
        let (_, ib) = self.check_anim(set, anim_b, "MODEL_ANIMATE_BLEND")?;
        let frames_a = self.model_anims[s][ia].keyframeCount.max(1);
        let fa = frame_a.rem_euclid(frames_a) as f32;
        let frames_b = self.model_anims[s][ib].keyframeCount.max(1);
        let fb = frame_b.rem_euclid(frames_b) as f32;
        let blend = blend.clamp(0.0, 1.0);
        self.rl.update_model_animation_ex(&self.thread, &mut self.models[mi],
            &self.model_anims[s][ia], fa, &self.model_anims[s][ib], fb, blend);
        Ok(())
    }
    /// Baut aus einem generierten Mesh ein Modell und gibt das Handle zurueck.
    fn push_model_from_mesh(&mut self, mesh: Mesh, fn_: &str) -> Result<i64, String> {
        // load_model_from_mesh uebernimmt das Mesh (WeakMesh = kein Drop).
        let weak = unsafe { mesh.make_weak() };
        let m = self.rl.load_model_from_mesh(&self.thread, weak)
            .map_err(|e| format!("{}: Mesh-Modell fehlgeschlagen: {}", fn_, e))?;
        self.models.push(m);
        Ok((self.models.len() - 1) as i64)
    }
    // Review-Fund: Segment-Zahlen wurden nur nach UNTEN geklemmt (.max(3)
    // etc.), nie nach oben -- `MESH_SPHERE(1, 100000, 100000)` fordert von
    // raylib/par_shapes ~1e10 Vertices an (Allocator-Abort/Hang), direkt aus
    // gewoehnlichem BASIC-Code heraus. Dimensionen (w/h/d/r/size) wurden
    // GAR NICHT geprueft -- negative/NaN-Werte gingen unveraendert an den
    // C-Generator. Ein Deckel + eine Endlichkeits-/Positiv-Pruefung schliessen
    // beides an einer Stelle.
    const MAX_MESH_SEGMENTS: i32 = 256;
    fn check_mesh_dim(name: &str, label: &str, v: f32) -> Result<(), String> {
        if !v.is_finite() || v <= 0.0 {
            return Err(format!("{}: '{}' muss eine endliche Zahl > 0 sein, erhalten {}", name, label, v));
        }
        Ok(())
    }
    pub fn mesh_cube(&mut self, w: f32, h: f32, d: f32) -> Result<i64, String> {
        Self::check_mesh_dim("MESH_CUBE", "w", w)?;
        Self::check_mesh_dim("MESH_CUBE", "h", h)?;
        Self::check_mesh_dim("MESH_CUBE", "d", d)?;
        let mesh = Mesh::gen_mesh_cube(&self.thread, w, h, d);
        self.push_model_from_mesh(mesh, "MESH_CUBE")
    }
    pub fn mesh_sphere(&mut self, r: f32, rings: i32, slices: i32) -> Result<i64, String> {
        Self::check_mesh_dim("MESH_SPHERE", "r", r)?;
        let mesh = Mesh::gen_mesh_sphere(&self.thread, r,
            rings.clamp(3, Self::MAX_MESH_SEGMENTS), slices.clamp(3, Self::MAX_MESH_SEGMENTS));
        self.push_model_from_mesh(mesh, "MESH_SPHERE")
    }
    pub fn mesh_cylinder(&mut self, r: f32, h: f32, slices: i32) -> Result<i64, String> {
        Self::check_mesh_dim("MESH_CYLINDER", "r", r)?;
        Self::check_mesh_dim("MESH_CYLINDER", "h", h)?;
        let mesh = Mesh::gen_mesh_cylinder(&self.thread, r, h, slices.clamp(3, Self::MAX_MESH_SEGMENTS));
        self.push_model_from_mesh(mesh, "MESH_CYLINDER")
    }
    pub fn mesh_torus(&mut self, r: f32, size: f32, rad_seg: i32, sides: i32) -> Result<i64, String> {
        Self::check_mesh_dim("MESH_TORUS", "r", r)?;
        Self::check_mesh_dim("MESH_TORUS", "size", size)?;
        let mesh = Mesh::gen_mesh_torus(&self.thread, r, size,
            rad_seg.clamp(3, Self::MAX_MESH_SEGMENTS), sides.clamp(3, Self::MAX_MESH_SEGMENTS));
        self.push_model_from_mesh(mesh, "MESH_TORUS")
    }
    pub fn mesh_knot(&mut self, r: f32, size: f32, rad_seg: i32, sides: i32) -> Result<i64, String> {
        Self::check_mesh_dim("MESH_KNOT", "r", r)?;
        Self::check_mesh_dim("MESH_KNOT", "size", size)?;
        let mesh = Mesh::gen_mesh_knot(&self.thread, r, size,
            rad_seg.clamp(3, Self::MAX_MESH_SEGMENTS), sides.clamp(3, Self::MAX_MESH_SEGMENTS));
        self.push_model_from_mesh(mesh, "MESH_KNOT")
    }
    pub fn mesh_plane(&mut self, w: f32, l: f32, res_x: i32, res_z: i32) -> Result<i64, String> {
        Self::check_mesh_dim("MESH_PLANE", "w", w)?;
        Self::check_mesh_dim("MESH_PLANE", "l", l)?;
        let mesh = Mesh::gen_mesh_plane(&self.thread, w, l,
            res_x.clamp(1, Self::MAX_MESH_SEGMENTS), res_z.clamp(1, Self::MAX_MESH_SEGMENTS));
        self.push_model_from_mesh(mesh, "MESH_PLANE")
    }
    /// Terrain-Mesh aus einer (Graustufen-)Image (LOADIMAGE-Handle): Helligkeit
    /// = Hoehe. size = (Breite, Hoehenskalierung, Tiefe) in Welt-Einheiten.
    pub fn mesh_heightmap(&mut self, tex_idx: i64, sx: f32, sy: f32, sz: f32) -> Result<i64, String> {
        let ti = tex_idx as usize;
        if !self.tex_ok(tex_idx) { return Err(self.tex_fehler(tex_idx, "MESH_HEIGHTMAP")); }
        // Mesh aus dem CPU-Image bauen (haelt danach keinen Borrow auf self).
        let mesh = {
            let img = &self.textures[ti].img;
            Mesh::gen_mesh_heightmap(&self.thread, img, Vector3::new(sx, sy, sz))
        };
        self.push_model_from_mesh(mesh, "MESH_HEIGHTMAP")
    }
    fn check_model(&self, idx: i64, fn_: &str) -> Result<usize, String> {
        let i = idx as usize;
        if idx < 0 || i >= self.models.len() {
            return Err(format!("{}: ungueltiges MODEL-Handle {}", fn_, idx));
        }
        Ok(i)
    }
    pub fn draw_model(&mut self, idx: i64, x: f32, y: f32, z: f32, scale: f32, col_: i64) -> Result<(), String> {
        let i = self.check_model(idx, "MODEL")?;
        self.emit3d(Cmd3D::Model(i, x, y, z, scale, col(col_)));
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn draw_model_ex(&mut self, idx: i64, x: f32, y: f32, z: f32,
                         ax: f32, ay: f32, az: f32, angle: f32, scale: f32, col_: i64) -> Result<(), String> {
        let i = self.check_model(idx, "MODEL_EX")?;
        self.emit3d(Cmd3D::ModelEx(i, x, y, z, ax, ay, az, angle, scale, col(col_)));
        Ok(())
    }
    pub fn draw_model_wires(&mut self, idx: i64, x: f32, y: f32, z: f32, scale: f32, col_: i64) -> Result<(), String> {
        let i = self.check_model(idx, "MODEL_WIRES")?;
        self.emit3d(Cmd3D::ModelWires(i, x, y, z, scale, col(col_)));
        Ok(())
    }
    /// Modul m3d: Modell mit beliebiger Welt-Matrix zeichnen (MODEL_MATRIX).
    /// `mat` ist column-major (OpenGL-Order, wie m3d MAT4 sie liefert).
    pub fn draw_model_matrix(&mut self, idx: i64, mat: Rc<[f32; 16]>, col_: i64) -> Result<(), String> {
        let i = self.check_model(idx, "MODEL_MATRIX")?;
        self.emit3d(Cmd3D::ModelMatrix(i, mat, col(col_)));
        Ok(())
    }
    /// Laedt den Instancing-Shader (einmal). Das `instanceTransform`-Attribut wird
    /// auf SHADER_LOC_MATRIX_MODEL gelegt (raylib bindet dort die Per-Instance-VBO);
    /// view/ambient/lightCount-Locations werden gecacht.
    fn ensure_inst_shader(&mut self) {
        if self.inst_shader.is_some() { return; }
        let mut sh = self.rl.load_shader_from_memory(&self.thread,
            Some(&fuer_ziel_uebersetzen(INST_VS)), Some(&fuer_ziel_uebersetzen(INST_FS)));
        // instanceTransform-Attribut-Location -> SHADER_LOC_MATRIX_MODEL.
        let cname = std::ffi::CString::new("instanceTransform").unwrap();
        let attr = unsafe { raylib::ffi::rlGetLocationAttrib(sh.id, cname.as_ptr()) };
        sh.locs_mut()[raylib::consts::ShaderLocationIndex::SHADER_LOC_MATRIX_MODEL as usize] = attr;
        self.inst_loc_view = sh.get_shader_location("viewPos");
        self.inst_loc_ambient = sh.get_shader_location("ambient");
        self.inst_loc_count = sh.get_shader_location("lightCount");
        self.inst_shader = Some(sh);
        self.inst_light_locs.clear();   // wird in update_inst_light_uniforms gefuellt
    }
    /// Modul m3d: GPU-Instancing -- dasselbe Modell mit N Welt-Matrizen in EINEM
    /// Draw-Call (raylib DrawMeshInstanced). `mats` column-major (OpenGL-Order).
    pub fn draw_model_instanced(&mut self, idx: i64, mats: Vec<[f32; 16]>, col_: i64) -> Result<(), String> {
        let i = self.check_model(idx, "MODEL_INSTANCED")?;
        if mats.is_empty() { return Ok(()); }   // nichts zu zeichnen
        self.ensure_inst_shader();
        self.emit3d(Cmd3D::ModelInstanced(i, Rc::new(mats), col(col_)));
        Ok(())
    }
    /// Legt eine via LOADIMAGE geladene Textur als Diffuse-/Albedo-Map an.
    pub fn model_set_texture(&mut self, model_idx: i64, tex_idx: i64) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_TEXTURE")?;
        let ti = tex_idx as usize;
        if !self.tex_ok(tex_idx) { return Err(self.tex_fehler(tex_idx, "MODEL_TEXTURE")); }
        let mats = self.models[mi].materials_mut();
        if mats.is_empty() {
            return Err("MODEL_TEXTURE: Modell hat kein Material".into());
        }
        mats[0].set_material_texture(
            raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO,
            &self.textures[ti].tex);
        Ok(())
    }

    /// Billboard: eine Textur (LOADIMAGE-Handle), die im 3D-Raum immer zur
    /// Kamera zeigt -- ideal fuer Baeume/Sprites/Funken in 3D.
    pub fn billboard(&mut self, tex_idx: i64, x: f32, y: f32, z: f32, size: f32, col_: i64) -> Result<(), String> {
        let i = tex_idx as usize;
        if !self.tex_ok(tex_idx) { return Err(self.tex_fehler(tex_idx, "BILLBOARD")); }
        self.emit3d(Cmd3D::Billboard(i, x, y, z, size, col(col_)));
        Ok(())
    }

    // --- Ray-Kollision / Picking (3D) ---
    // Liefern die Distanz vom Ray-Ursprung zum Treffer (Welt-Einheiten) oder
    // -1.0 bei keinem Treffer. Trefferpunkt = ursprung + richtung * distanz.
    #[allow(clippy::too_many_arguments)]
    pub fn ray_hit_box(&self, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32,
                       cx: f32, cy: f32, cz: f32, sx: f32, sy: f32, sz: f32) -> f64 {
        let ray = Ray::new(Vector3::new(ox, oy, oz), Vector3::new(dx, dy, dz));
        let bb = BoundingBox::new(
            Vector3::new(cx - sx / 2.0, cy - sy / 2.0, cz - sz / 2.0),
            Vector3::new(cx + sx / 2.0, cy + sy / 2.0, cz + sz / 2.0));
        let rc = bb.get_ray_collision_box(ray);
        if rc.hit { rc.distance as f64 } else { -1.0 }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn ray_hit_sphere(&self, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32,
                          cx: f32, cy: f32, cz: f32, r: f32) -> f64 {
        // Review-Fund: raylibs GetRayCollisionSphere ist ein reiner
        // Abstand-zur-Gerade-Test (`hit = d >= 0.0`), OHNE zu pruefen, ob der
        // Treffer VOR dem Ursprung liegt -- ein Objekt HINTER der Kamera,
        // das die rueckwaertige Verlaengerung des Mausstrahls kreuzt, liefert
        // `hit=true` mit einer NEGATIVEN `distance`. Der dokumentierte
        // Vertrag ("Distanz oder -1") wird dadurch verletzt: `PICK_SPHERE(...)
        // <> -1` waehlt so ein Objekt hinter der Kamera aus. Ausserdem
        // erwartet die zugrundeliegende Formel eine NORMALISIERTE Richtung --
        // ein unnormalisierter Vektor (aus GB-Code plausibel, z.B. eine
        // Differenz zweier Punkte) liefert nicht nur einen skalierten,
        // sondern einen tatsaechlich FALSCHEN Treffer/Kein-Treffer-Ausschlag.
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        if len < 1e-6 { return -1.0; }
        let (ndx, ndy, ndz) = (dx / len, dy / len, dz / len);
        let ray = Ray::new(Vector3::new(ox, oy, oz), Vector3::new(ndx, ndy, ndz));
        let rc = get_ray_collision_sphere(ray, Vector3::new(cx, cy, cz), r);
        if rc.hit && rc.distance >= 0.0 { rc.distance as f64 } else { -1.0 }
    }
    /// Strahl aus Ursprung + Richtung mit NORMALISIERTER Richtung. Alle
    /// raylib-Strahl-Tests liefern die Distanz in Vielfachen der Richtungs-
    /// laenge -- nur bei Einheitslaenge ist das Ergebnis eine Weltdistanz.
    /// Eine Nullrichtung gibt es nicht als Strahl -> None.
    fn unit_ray(ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32) -> Option<Ray> {
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        if len < 1e-6 { return None; }
        Some(Ray::new(Vector3::new(ox, oy, oz),
                      Vector3::new(dx / len, dy / len, dz / len)))
    }
    /// Strahl gegen ein einzelnes Dreieck (Moeller-Trumbore). raylib cullt
    /// bewusst NICHT -- ein Treffer von der Rueckseite zaehlt genauso. Punkte
    /// je (x,y,z), Ergebnis Distanz oder -1.
    #[allow(clippy::too_many_arguments)]
    pub fn ray_hit_tri(&self, o: [f32; 3], d: [f32; 3], p: [[f32; 3]; 3]) -> f64 {
        let Some(ray) = Self::unit_ray(o[0], o[1], o[2], d[0], d[1], d[2]) else { return -1.0; };
        let v = |q: [f32; 3]| Vector3::new(q[0], q[1], q[2]);
        let rc = get_ray_collision_triangle(ray, v(p[0]), v(p[1]), v(p[2]));
        if rc.hit { rc.distance as f64 } else { -1.0 }
    }
    /// Strahl gegen ein Viereck (zwei Dreiecke). Die Punkte muessen REIHUM
    /// liegen (p1-p2-p3-p4 im Kreis) -- bei einer Ueberkreuz-Reihenfolge
    /// testet raylib zwei sich ueberlappende Dreiecke und der Treffer fehlt.
    pub fn ray_hit_quad(&self, o: [f32; 3], d: [f32; 3], p: [[f32; 3]; 4]) -> f64 {
        let Some(ray) = Self::unit_ray(o[0], o[1], o[2], d[0], d[1], d[2]) else { return -1.0; };
        let v = |q: [f32; 3]| Vector3::new(q[0], q[1], q[2]);
        let rc = get_ray_collision_quad(ray, v(p[0]), v(p[1]), v(p[2]), v(p[3]));
        if rc.hit { rc.distance as f64 } else { -1.0 }
    }
    /// Mausstrahl durch die aktuelle 3D-Kamera (Fenster-Pixel -> Welt-Ray).
    fn mouse_ray(&self) -> Ray {
        let m = self.rl.get_mouse_position();
        self.rl.get_screen_to_world_ray(m, self.cam3d)
    }
    /// Schnittpunkt des Mausstrahls mit der horizontalen Ebene y = plane_y.
    /// Liefert (welt_x, welt_z, getroffen). Bei (fast) parallelem Strahl oder
    /// Treffer "hinter" der Kamera: Ursprung-xz + getroffen=false.
    pub fn mouse_ground(&self, plane_y: f32) -> (f32, f32, bool) {
        let r = self.mouse_ray();
        let dy = r.direction.y;
        if dy.abs() < 1e-6 {
            return (r.position.x, r.position.z, false);
        }
        let t = (plane_y - r.position.y) / dy;
        if t < 0.0 {
            return (r.position.x, r.position.z, false);
        }
        (r.position.x + t * r.direction.x, r.position.z + t * r.direction.z, true)
    }
    pub fn pick_box(&self, cx: f32, cy: f32, cz: f32, sx: f32, sy: f32, sz: f32) -> f64 {
        let r = self.mouse_ray();
        self.ray_hit_box(r.position.x, r.position.y, r.position.z,
                         r.direction.x, r.direction.y, r.direction.z, cx, cy, cz, sx, sy, sz)
    }
    pub fn pick_sphere(&self, cx: f32, cy: f32, cz: f32, radius: f32) -> f64 {
        let r = self.mouse_ray();
        self.ray_hit_sphere(r.position.x, r.position.y, r.position.z,
                            r.direction.x, r.direction.y, r.direction.z, cx, cy, cz, radius)
    }
    pub fn pick_tri(&self, p: [[f32; 3]; 3]) -> f64 {
        let r = self.mouse_ray();
        self.ray_hit_tri([r.position.x, r.position.y, r.position.z],
                         [r.direction.x, r.direction.y, r.direction.z], p)
    }
    pub fn pick_quad(&self, p: [[f32; 3]; 4]) -> f64 {
        let r = self.mouse_ray();
        self.ray_hit_quad([r.position.x, r.position.y, r.position.z],
                          [r.direction.x, r.direction.y, r.direction.z], p)
    }
    /// 3D-Weltpunkt -> Bildschirm-Pixel (durch die aktuelle 3D-Kamera).
    pub fn world_to_screen(&self, wx: f32, wy: f32, wz: f32) -> (f32, f32) {
        let v = self.rl.get_world_to_screen(Vector3::new(wx, wy, wz), self.cam3d);
        (v.x, v.y)
    }
    /// Strahl-RICHTUNG vom Screen-Punkt durch die 3D-Kamera. Der Ursprung des
    /// Strahls ist die Kameraposition (CAMERA3D_X/Y/Z).
    pub fn screen_ray_dir(&self, sx: f32, sy: f32) -> (f32, f32, f32) {
        let r = self.rl.get_screen_to_world_ray(Vector2::new(sx, sy), self.cam3d);
        (r.direction.x, r.direction.y, r.direction.z)
    }
    /// Raycast gegen ein geladenes Modell (alle Meshes), platziert bei
    /// (px,py,pz) mit uniformer Skalierung `scale`. Liefert die Distanz zum
    /// naechsten Treffer oder -1 bei Verfehlen.
    #[allow(clippy::too_many_arguments)]
    pub fn ray_hit_model(&self, idx: i64, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32,
                         px: f32, py: f32, pz: f32, scale: f32) -> f64 {
        if idx < 0 || idx as usize >= self.models.len() { return -1.0; }
        let ray = Ray::new(Vector3::new(ox, oy, oz), Vector3::new(dx, dy, dz));
        // Gleiche Transform-Reihenfolge wie DrawModel: erst skalieren, dann verschieben.
        let transform = Matrix::scale(scale, scale, scale) * Matrix::translate(px, py, pz);
        let mut best = -1.0f64;
        for mesh in self.models[idx as usize].meshes() {
            // WeakMesh und Mesh sind beide #[repr(transparent)] ueber ffi::Mesh.
            let m: &Mesh = unsafe { std::mem::transmute::<&WeakMesh, &Mesh>(mesh) };
            let rc = get_ray_collision_model(ray, m, &transform);
            if rc.hit {
                let d = rc.distance as f64;
                if best < 0.0 || d < best { best = d; }
            }
        }
        best
    }
    /// Wie ray_hit_model, aber mit dem Mausstrahl (analog PICK_BOX/PICK_SPHERE).
    pub fn pick_model(&self, idx: i64, px: f32, py: f32, pz: f32, scale: f32) -> f64 {
        let r = self.mouse_ray();
        self.ray_hit_model(idx, r.position.x, r.position.y, r.position.z,
                           r.direction.x, r.direction.y, r.direction.z, px, py, pz, scale)
    }
    /// Pixelfarbe eines geladenen Bildes lesen -> 0xRRGGBB, -1 wenn ungueltig.
    pub fn get_pixel(&mut self, idx: i64, x: i32, y: i32) -> i64 {
        if !self.tex_ok(idx) { return -1; }
        let img = &mut self.textures[idx as usize].img;
        if x < 0 || y < 0 || x >= img.width || y >= img.height { return -1; }
        let c = img.get_color(x, y);
        ((c.r as i64) << 16) | ((c.g as i64) << 8) | (c.b as i64)
    }

    // --- Beleuchtung (Blinn-Phong via rlights-Shader) ---
    /// Laedt den Lighting-Shader (einmal) und aktiviert die Beleuchtung. Die
    /// Uniform-Locations fuer viewPos/ambient werden gecacht.
    pub fn light_enable(&mut self) {
        if self.light_shader.is_some() { return; }
        let mut sh = self.rl.load_shader_from_memory(&self.thread,
            Some(&fuer_ziel_uebersetzen(LIGHT_VS)), Some(&fuer_ziel_uebersetzen(LIGHT_FS)));
        // matModel fuer fragPosition (Weltkoordinaten) explizit binden.
        let loc_model = sh.get_shader_location("matModel");
        sh.locs_mut()[raylib::consts::ShaderLocationIndex::SHADER_LOC_MATRIX_MODEL as usize] = loc_model;
        self.loc_view = sh.get_shader_location("viewPos");
        self.loc_ambient = sh.get_shader_location("ambient");
        self.loc_fog_color = sh.get_shader_location("fogColor");
        self.loc_fog_density = sh.get_shader_location("fogDensity");
        self.loc_use_normal = sh.get_shader_location("useNormalMap");
        self.loc_metalness = sh.get_shader_location("metalness");
        self.loc_roughness = sh.get_shader_location("roughness");
        self.loc_emissive = sh.get_shader_location("emissive");
        self.loc_env_sky = sh.get_shader_location("envSky");
        self.loc_env_ground = sh.get_shader_location("envGround");
        self.loc_env_intensity = sh.get_shader_location("envIntensity");
        self.loc_use_ibl = sh.get_shader_location("useIBLMaps");
        self.loc_irradiance = sh.get_shader_location("irradianceMap");
        self.loc_prefilter = sh.get_shader_location("prefilterMap");
        self.loc_brdf = sh.get_shader_location("brdfLUT");
        self.light_shader = Some(sh);
    }
    pub fn light_fog(&mut self, col_: i64, density: f64) {
        let v = col_ as u32;
        self.light_fog = [((v >> 16) & 0xFF) as f32 / 255.0, ((v >> 8) & 0xFF) as f32 / 255.0,
                          (v & 0xFF) as f32 / 255.0, 1.0];
        self.light_fog_density = density.max(0.0) as f32;
    }
    fn rgb3(col_: i64) -> [f32; 3] {
        let v = col_ as u32;
        [((v >> 16) & 0xFF) as f32 / 255.0, ((v >> 8) & 0xFF) as f32 / 255.0, (v & 0xFF) as f32 / 255.0]
    }
    /// Analytisches Environment-Lighting (IBL): Sky-/Ground-Farbe + Intensitaet
    /// (0 = aus). Metalle reflektieren die Umgebung, roughness verwischt sie.
    pub fn light_env(&mut self, sky: i64, ground: i64, intensity: f64) {
        self.env_sky = Self::rgb3(sky);
        self.env_ground = Self::rgb3(ground);
        self.env_intensity = intensity.max(0.0) as f32;
    }
    // --- Echtes HDR-Cubemap-IBL (LIGHT_ENV_HDR) ---
    // Port von raylibs `shaders_basic_pbr`-Beispiel: equirect -> Cubemap,
    // Irradiance-Convolution, Prefilter-Mips, BRDF-LUT. Gerendert ueber FBOs +
    // `rlLoadDrawCube`/`rlLoadDrawQuad` (immediate glDrawArrays, kein Render-
    // Batch -> die per rlSetUniformMatrix gesetzten View/Proj-Uniforms bleiben
    // stehen). Wie das Shadow-Mapping rein rlgl-/ffi-basiert.
    //
    // WICHTIG: raylibs `rlLoadTextureCubemap` legt fuer NULL-Daten KEINE Float-
    // Cubemaps an (R32/R16 werden abgelehnt) -> wir nutzen R8G8B8A8 (LDR). Sehr
    // helle Werte (Sonne) clampen; fuer Reflexionen/IBL visuell ausreichend.

    /// Die 6 Cubemap-View-Matrizen (Blick aus dem Zentrum auf jede Achse),
    /// direkt als ffi::Matrix (Copy) fuer rlSetUniformMatrix.
    fn ibl_cube_views() -> [raylib::ffi::Matrix; 6] {
        let o = Vector3::zero();
        [
            Matrix::look_at(o, Vector3::new( 1.0,  0.0,  0.0), Vector3::new(0.0, -1.0,  0.0)).into(),
            Matrix::look_at(o, Vector3::new(-1.0,  0.0,  0.0), Vector3::new(0.0, -1.0,  0.0)).into(),
            Matrix::look_at(o, Vector3::new( 0.0,  1.0,  0.0), Vector3::new(0.0,  0.0,  1.0)).into(),
            Matrix::look_at(o, Vector3::new( 0.0, -1.0,  0.0), Vector3::new(0.0,  0.0, -1.0)).into(),
            Matrix::look_at(o, Vector3::new( 0.0,  0.0,  1.0), Vector3::new(0.0, -1.0,  0.0)).into(),
            Matrix::look_at(o, Vector3::new( 0.0,  0.0, -1.0), Vector3::new(0.0, -1.0,  0.0)).into(),
        ]
    }

    /// 90deg-Projektion fuer die Cubemap-Passes (aspect 1).
    fn ibl_cube_proj() -> raylib::ffi::Matrix {
        let (near, far) = unsafe {
            (raylib::ffi::rlGetCullDistanceNear(), raylib::ffi::rlGetCullDistanceFar())
        };
        Matrix::perspective(90.0_f64.to_radians(), 1.0, near, far).into()
    }

    /// Rendert eine einfache (1-Mip) Cubemap mit `fs` ueber die 6 Faces. Quelle ist
    /// eine 2D-Textur (equirect) oder eine Cubemap (irradiance). Liefert die GL-ID.
    fn ibl_render_cube(&mut self, fs: &str, src_id: u32, src_cubemap: bool, size: i32) -> Result<u32, String> {
        let sh = self.rl.load_shader_from_memory(&self.thread,
            Some(&fuer_ziel_uebersetzen(CUBEMAP_VS)), Some(&fuer_ziel_uebersetzen(fs)));
        let id = sh.id;
        if id == 0 {
            // Der Grund steht dabei, weil er nicht am Aufruf liegt: unser
            // Cubemap-Shader ist Desktop-GLSL (#version 330) und uebersetzt auf
            // WebGL nicht. Ein Programm kann das abfangen (TRY/CATCH) und auf
            // das analytische LIGHT_ENV ausweichen -- die Demo tut genau das.
            return Err("LIGHT_ENV_HDR: Cubemap-Shader nicht ladbar \
                        (auf dieser Grafik-Schnittstelle nicht uebersetzbar, z.B. WebGL)".into());
        }
        let loc_proj = sh.get_shader_location("matProjection");
        let loc_view = sh.get_shader_location("matView");
        let views = Self::ibl_cube_views();
        let proj = Self::ibl_cube_proj();
        let (win_w, win_h) = (self.width * self.scale, self.height * self.scale);
        let cubemap;
        unsafe {
            raylib::ffi::rlDisableBackfaceCulling();
            let rbo = raylib::ffi::rlLoadTextureDepth(size, size, true);   // Renderbuffer
            // R8G8B8A8 (=7): einzige fuer leere Cubemaps unterstuetzte Form.
            cubemap = raylib::ffi::rlLoadTextureCubemap(std::ptr::null(), size, 7, 1);
            let fbo = raylib::ffi::rlLoadFramebuffer();
            // RL_ATTACHMENT_DEPTH=100, RL_ATTACHMENT_RENDERBUFFER=200.
            raylib::ffi::rlFramebufferAttach(fbo, rbo, 100, 200, 0);
            // RL_ATTACHMENT_COLOR_CHANNEL0=0, RL_ATTACHMENT_CUBEMAP_POSITIVE_X=0.
            raylib::ffi::rlFramebufferAttach(fbo, cubemap, 0, 0, 0);
            raylib::ffi::rlEnableShader(id);
            raylib::ffi::rlSetUniformMatrix(loc_proj, proj);
            raylib::ffi::rlActiveTextureSlot(0);
            if src_cubemap { raylib::ffi::rlEnableTextureCubemap(src_id); }
            else { raylib::ffi::rlEnableTexture(src_id); }
            raylib::ffi::rlViewport(0, 0, size, size);
            for i in 0..6 {
                raylib::ffi::rlSetUniformMatrix(loc_view, views[i as usize]);
                raylib::ffi::rlFramebufferAttach(fbo, cubemap, 0, i, 0);   // +X + i
                // rlFramebufferAttach unbindet das FBO (glBindFramebuffer 0) ->
                // pro Face neu binden, sonst landet der Cube auf dem Screen.
                raylib::ffi::rlEnableFramebuffer(fbo);
                raylib::ffi::rlClearScreenBuffers();
                raylib::ffi::rlLoadDrawCube();
            }
            raylib::ffi::rlDisableShader();
            if src_cubemap { raylib::ffi::rlDisableTextureCubemap(); }
            else { raylib::ffi::rlDisableTexture(); }
            raylib::ffi::rlDisableFramebuffer();
            raylib::ffi::rlUnloadFramebuffer(fbo);
            raylib::ffi::rlEnableBackfaceCulling();
            raylib::ffi::rlViewport(0, 0, win_w, win_h);
        }
        Ok(cubemap)
    }

    /// Prefilter-Cubemap: GGX-Importance-Sampling pro Roughness-Mip-Level.
    /// `rlLoadTextureCubemap(.., mips)` legt die Mip-Speicher direkt an (kein
    /// rlGenTextureMipmaps fuer Cubemaps moeglich -> bindet GL_TEXTURE_2D).
    fn ibl_render_prefilter(&mut self, env_cubemap: u32, size: i32) -> Result<u32, String> {
        const MIPS: i32 = 5;
        let sh = self.rl.load_shader_from_memory(&self.thread,
            Some(&fuer_ziel_uebersetzen(CUBEMAP_VS)), Some(&fuer_ziel_uebersetzen(PREFILTER_FS)));
        let id = sh.id;
        if id == 0 { return Err("LIGHT_ENV_HDR: Prefilter-Shader nicht ladbar".into()); }
        let loc_proj = sh.get_shader_location("matProjection");
        let loc_view = sh.get_shader_location("matView");
        let loc_rough = sh.get_shader_location("roughness");
        let views = Self::ibl_cube_views();
        let proj = Self::ibl_cube_proj();
        let (win_w, win_h) = (self.width * self.scale, self.height * self.scale);
        let prefilter;
        // Voller Mip-Chain (128->1 = 8 Level), damit die Cubemap mit
        // LINEAR_MIPMAP_LINEAR *mipmap-complete* ist (sonst sampelt sie komplett
        // schwarz). Prefiltert werden nur die ersten MIPS Roughness-Level
        // (MAX_REFLECTION_LOD im PBR-Shader = MIPS-1); hoehere Mips bleiben leer,
        // werden aber nie gesampelt (max lod = MIPS-1).
        let full_mips = (32 - (size as u32).leading_zeros()) as i32;
        unsafe {
            raylib::ffi::rlDisableBackfaceCulling();
            prefilter = raylib::ffi::rlLoadTextureCubemap(std::ptr::null(), size, 7, full_mips);
            let rbo = raylib::ffi::rlLoadTextureDepth(size, size, true);
            let fbo = raylib::ffi::rlLoadFramebuffer();
            raylib::ffi::rlFramebufferAttach(fbo, rbo, 100, 200, 0);   // DEPTH, RENDERBUFFER
            raylib::ffi::rlEnableShader(id);
            raylib::ffi::rlSetUniformMatrix(loc_proj, proj);
            raylib::ffi::rlActiveTextureSlot(0);
            raylib::ffi::rlEnableTextureCubemap(env_cubemap);
            for mip in 0..MIPS {
                let msize = (size as f32 * 0.5f32.powi(mip)).max(1.0) as i32;
                raylib::ffi::rlViewport(0, 0, msize, msize);
                let roughness = mip as f32 / (MIPS - 1) as f32;
                raylib::ffi::rlSetUniform(
                    loc_rough, &roughness as *const f32 as *const std::os::raw::c_void, 0 /*FLOAT*/, 1);
                for i in 0..6 {
                    raylib::ffi::rlSetUniformMatrix(loc_view, views[i as usize]);
                    raylib::ffi::rlFramebufferAttach(fbo, prefilter, 0, i, mip);
                    raylib::ffi::rlEnableFramebuffer(fbo);
                    raylib::ffi::rlClearScreenBuffers();
                    raylib::ffi::rlLoadDrawCube();
                }
            }
            raylib::ffi::rlDisableShader();
            raylib::ffi::rlDisableTextureCubemap();
            raylib::ffi::rlDisableFramebuffer();
            raylib::ffi::rlUnloadFramebuffer(fbo);
            raylib::ffi::rlEnableBackfaceCulling();
            raylib::ffi::rlViewport(0, 0, win_w, win_h);
        }
        Ok(prefilter)
    }

    /// BRDF-Integrations-LUT (2D, NdotV x roughness) via Fullscreen-Quad.
    fn ibl_render_brdf(&mut self, size: i32) -> Result<u32, String> {
        let sh = self.rl.load_shader_from_memory(&self.thread,
            Some(&fuer_ziel_uebersetzen(BRDF_VS)), Some(&fuer_ziel_uebersetzen(BRDF_FS)));
        let id = sh.id;
        if id == 0 { return Err("LIGHT_ENV_HDR: BRDF-Shader nicht ladbar".into()); }
        let (win_w, win_h) = (self.width * self.scale, self.height * self.scale);
        let brdf;
        unsafe {
            brdf = raylib::ffi::rlLoadTexture(std::ptr::null(), size, size, 7, 1);
            let fbo = raylib::ffi::rlLoadFramebuffer();
            raylib::ffi::rlFramebufferAttach(fbo, brdf, 0, 100, 0);   // COLOR0, TEXTURE2D
            raylib::ffi::rlEnableFramebuffer(fbo);
            raylib::ffi::rlViewport(0, 0, size, size);
            raylib::ffi::rlEnableShader(id);
            raylib::ffi::rlClearScreenBuffers();
            raylib::ffi::rlLoadDrawQuad();
            raylib::ffi::rlDisableShader();
            raylib::ffi::rlDisableFramebuffer();
            raylib::ffi::rlUnloadFramebuffer(fbo);
            raylib::ffi::rlViewport(0, 0, win_w, win_h);
        }
        Ok(brdf)
    }

    /// Laedt ein `.hdr`-Equirect-Panorama und berechnet die drei IBL-Maps
    /// (Irradiance + Prefilter + BRDF-LUT) einmalig. Aktiviert den Cubemap-IBL-
    /// Pfad im Shader (useIBLMaps=1). `intensity` skaliert den IBL-Beitrag
    /// (= envIntensity; 0 = aus). Idempotent: weitere Aufrufe setzen nur die
    /// Intensitaet (kein Neu-Generieren).
    pub fn light_env_hdr(&mut self, path: &str, intensity: f64) -> Result<(), String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        if self.light_shader.is_none() { self.light_enable(); }
        self.env_intensity = intensity.max(0.0) as f32;
        if self.use_ibl_maps { return Ok(()); }
        // 1) HDR-Equirect laden. raylib-sys ist OHNE SUPPORT_FILEFORMAT_HDR
        // gebaut -> wir dekodieren das Radiance-RGBE selbst zu RGBA32F und laden
        // es als 2D-Float-Textur (rlLoadTexture unterstuetzt Float mit Daten).
        let (hdr, hw, hh) = load_hdr_rgbe(path)
            .map_err(|e| format!("LIGHT_ENV_HDR: '{}' -- {}", path, e))?;
        let pano_id = unsafe {
            // RL_PIXELFORMAT_UNCOMPRESSED_R32G32B32A32 = 10.
            raylib::ffi::rlLoadTexture(hdr.as_ptr() as *const std::os::raw::c_void, hw, hh, 10, 1)
        };
        if pano_id == 0 { return Err("LIGHT_ENV_HDR: Panorama-Textur fehlgeschlagen".into()); }
        unsafe {
            // Bilineare Filterung + Clamp fuer sauberes equirect-Sampling.
            let tex = raylib::ffi::Texture2D { id: pano_id, width: hw, height: hh, mipmaps: 1, format: 10 };
            raylib::ffi::SetTextureFilter(tex, 1 /*BILINEAR*/);
            raylib::ffi::SetTextureWrap(tex, 1 /*CLAMP*/);
        }
        // Review-Fund: die vier folgenden Schritte erzeugen der Reihe nach
        // GPU-Cubemaps/-Texturen; schlaegt ein SPAETERER Schritt fehl (z.B.
        // Shader-Kompilierung), leakten die bereits erzeugten Texturen der
        // vorherigen Schritte -- nur der Erfolgspfad rief bislang
        // rlUnloadTexture auf. `built` sammelt jede erfolgreich erzeugte
        // Textur-ID (inkl. der Panorama-Textur); im Fehlerfall werden alle
        // hier explizit freigegeben, bevor der Fehler propagiert wird.
        let mut built: Vec<u32> = vec![pano_id];
        let step = |g: &mut Self, built: &mut Vec<u32>| -> Result<(u32, u32, u32, u32), String> {
            // 2) equirect -> Cubemap (512).
            let env = g.ibl_render_cube(EQUIRECT_FS, pano_id, false, 512)?;
            built.push(env);
            // 3) Irradiance-Cubemap (32, diffuse).
            let irradiance = g.ibl_render_cube(IRRADIANCE_FS, env, true, 32)?;
            built.push(irradiance);
            // 4) Prefilter-Cubemap (128 + Roughness-Mips, specular).
            let prefilter = g.ibl_render_prefilter(env, 128)?;
            built.push(prefilter);
            // 5) BRDF-LUT (512, 2D).
            let brdf = g.ibl_render_brdf(512)?;
            built.push(brdf);
            Ok((env, irradiance, prefilter, brdf))
        };
        match step(self, &mut built) {
            Ok((env, irradiance, prefilter, brdf)) => {
                // Equirect freigeben; env-Cubemap fuer die Skybox aufbewahren.
                unsafe { raylib::ffi::rlUnloadTexture(pano_id); }
                self.ibl_env = env;
                self.ibl_irradiance = irradiance;
                self.ibl_prefilter = prefilter;
                self.ibl_brdf = brdf;
                self.use_ibl_maps = true;
                Ok(())
            }
            Err(e) => {
                for id in built { unsafe { raylib::ffi::rlUnloadTexture(id); } }
                Err(e)
            }
        }
    }

    /// Skybox an/aus: zeichnet die env-Cubemap (von LIGHT_ENV_HDR) als 3D-
    /// Hintergrund. Ohne vorheriges LIGHT_ENV_HDR (ibl_env == 0) ein No-Op.
    pub fn skybox(&mut self, on: bool) {
        if on && self.skybox_shader.is_none() {
            let sh = self.rl.load_shader_from_memory(&self.thread,
                Some(&fuer_ziel_uebersetzen(SKYBOX_VS)), Some(&fuer_ziel_uebersetzen(SKYBOX_FS)));
            self.skybox_loc_proj = sh.get_shader_location("matProjection");
            self.skybox_loc_view = sh.get_shader_location("matView");
            self.skybox_shader = Some(sh);
        }
        self.skybox_enabled = on;
    }

    pub fn light_ambient(&mut self, col_: i64, intensity: f64) {
        let v = col_ as u32;
        let f = intensity as f32;
        self.light_ambient = [
            ((v >> 16) & 0xFF) as f32 / 255.0 * f,
            ((v >> 8) & 0xFF) as f32 / 255.0 * f,
            (v & 0xFF) as f32 / 255.0 * f,
            1.0,
        ];
    }
    /// Fuegt ein Licht hinzu (kind: 0=directional, 1=point). Liefert den Index
    /// oder -1 wenn MAX_LIGHTS erreicht. Bei directional ist (x,y,z) die
    /// Richtung (Ziel), die Position liegt im Ursprung.
    pub fn light_add(&mut self, kind: i64, x: f32, y: f32, z: f32, col_: i64) -> i64 {
        if self.light_shader.is_none() { self.light_enable(); }
        if self.lights.len() >= MAX_LIGHTS { return -1; }
        let i = self.lights.len();
        let v = col_ as u32;
        let color = [((v >> 16) & 0xFF) as f32 / 255.0, ((v >> 8) & 0xFF) as f32 / 255.0,
                     (v & 0xFF) as f32 / 255.0, 1.0];
        let (pos, target) = if kind == 0 {
            ([0.0, 0.0, 0.0], [x, y, z])   // directional: Richtung = target
        } else {
            ([x, y, z], [0.0, 0.0, 0.0])   // point: Position
        };
        let sh = self.light_shader.as_mut().unwrap();
        let ld = LightData {
            enabled: true, kind: kind as i32, pos, target, color,
            loc_enabled: sh.get_shader_location(&format!("lights[{}].enabled", i)),
            loc_type:    sh.get_shader_location(&format!("lights[{}].type", i)),
            loc_pos:     sh.get_shader_location(&format!("lights[{}].position", i)),
            loc_target:  sh.get_shader_location(&format!("lights[{}].target", i)),
            loc_color:   sh.get_shader_location(&format!("lights[{}].color", i)),
        };
        self.lights.push(ld);
        i as i64
    }
    pub fn light_set_pos(&mut self, idx: i64, x: f32, y: f32, z: f32) -> Result<(), String> {
        let l = self.lights.get_mut(idx as usize).ok_or("LIGHT_SET_POS: ungueltiger Licht-Index")?;
        if l.kind == 0 { l.target = [x, y, z]; } else { l.pos = [x, y, z]; }
        Ok(())
    }
    pub fn light_set_color(&mut self, idx: i64, col_: i64) -> Result<(), String> {
        let v = col_ as u32;
        let l = self.lights.get_mut(idx as usize).ok_or("LIGHT_SET_COLOR: ungueltiger Licht-Index")?;
        l.color = [((v >> 16) & 0xFF) as f32 / 255.0, ((v >> 8) & 0xFF) as f32 / 255.0, (v & 0xFF) as f32 / 255.0, 1.0];
        Ok(())
    }
    pub fn light_set_enabled(&mut self, idx: i64, on: bool) -> Result<(), String> {
        let l = self.lights.get_mut(idx as usize).ok_or("LIGHT_SET_ENABLED: ungueltiger Licht-Index")?;
        l.enabled = on;
        Ok(())
    }
    /// Haengt den Lighting-Shader an alle Materialien eines Modells.
    /// Tangenten werden NICHT hier erzeugt -- raylibs `GenMeshTangents` setzt ein
    /// nicht-indiziertes Mesh voraus und warnt sonst ("vertexCount expected to be
    /// a multiple of 3"). Tangenten braucht nur Normal-Mapping, daher erzeugt sie
    /// `MODEL_TEXTURE_NORMAL` (model_set_normal) erst bei Bedarf.
    pub fn model_lit(&mut self, model_idx: i64) -> Result<(), String> {
        if self.light_shader.is_none() {
            return Err("MODEL_LIT: zuerst LIGHT_ENABLE() / ein Licht hinzufuegen".into());
        }
        let mi = self.check_model(model_idx, "MODEL_LIT")?;
        let sh_ffi = *self.light_shader.as_ref().unwrap().as_ref();   // ffi::Shader (Copy)
        for mat in self.models[mi].materials_mut() {
            unsafe { mat.as_raw_mut().shader = sh_ffi; }
        }
        Ok(())
    }
    /// Legt eine via LOADIMAGE geladene Textur als Normal-Map (MATERIAL_MAP_NORMAL).
    /// Aktiviert useNormalMap fuer dieses Modell.
    pub fn model_set_normal(&mut self, model_idx: i64, tex_idx: i64) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_TEXTURE_NORMAL")?;
        let ti = tex_idx as usize;
        if !self.tex_ok(tex_idx) { return Err(self.tex_fehler(tex_idx, "MODEL_TEXTURE_NORMAL")); }
        // Tangenten erzeugen (TBN-Basis fuer Normal-Mapping). Nur hier noetig --
        // nicht pauschal in MODEL_LIT, das spart die raylib-"vertexCount"-Warnung
        // fuer indizierte Meshes (Plane/Cube/...) ohne Normal-Map.
        for m in self.models[mi].meshes_mut() {
            m.gen_mesh_tangents(&self.thread);
        }
        for mat in self.models[mi].materials_mut() {
            mat.set_material_texture(raylib::consts::MaterialMapIndex::MATERIAL_MAP_NORMAL, &self.textures[ti].tex);
        }
        self.normal_mapped.insert(mi);
        Ok(())
    }
    /// Setzt die PBR-Materialparameter eines Modells (metalness/roughness 0..1).
    pub fn model_pbr(&mut self, model_idx: i64, metalness: f64, roughness: f64) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_PBR")?;
        self.pbr_params.insert(mi, (metalness.clamp(0.0, 1.0) as f32, roughness.clamp(0.0, 1.0) as f32));
        Ok(())
    }
    /// Setzt Eigenleuchten eines Modells (Farbe 0xRRGGBB + Staerke; 0 = aus).
    /// Wirkt auf MODEL_LIT-Modelle; mit Bloom-POSTFX entsteht echter Glow.
    pub fn model_emissive(&mut self, model_idx: i64, color: i64, strength: f64) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_EMISSIVE")?;
        let r = ((color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((color >> 8) & 0xFF) as f32 / 255.0;
        let b = (color & 0xFF) as f32 / 255.0;
        self.emissive.insert(mi, (r, g, b, strength.max(0.0) as f32));
        Ok(())
    }
    /// Setzt pro Frame viewPos + ambient + alle Licht-Uniforms. Wird in flip()
    /// vor dem 3D-Pass gerufen, solange Beleuchtung aktiv ist.
    fn update_light_uniforms(&mut self) {
        if self.light_shader.is_none() { return; }
        let view = [self.cam3d.position.x, self.cam3d.position.y, self.cam3d.position.z];
        let ambient = self.light_ambient;
        let loc_view = self.loc_view;
        let loc_ambient = self.loc_ambient;
        let (fog_color, fog_density) = (self.light_fog, self.light_fog_density);
        let (loc_fog_color, loc_fog_density) = (self.loc_fog_color, self.loc_fog_density);
        let (env_sky, env_ground, env_int) = (self.env_sky, self.env_ground, self.env_intensity);
        let (loc_esky, loc_eground, loc_eint) = (self.loc_env_sky, self.loc_env_ground, self.loc_env_intensity);
        // HDR-Cubemap-IBL: useIBLMaps-Gate + Sampler-Unit-Locs (Binden in render_scene).
        let use_ibl = self.use_ibl_maps;
        let (loc_use_ibl, loc_irr, loc_pre, loc_brdf) =
            (self.loc_use_ibl, self.loc_irradiance, self.loc_prefilter, self.loc_brdf);
        // Licht-Daten lokal kopieren (Borrow-Trennung zum Shader).
        let lights: Vec<(i32,i32,i32,i32,i32, i32, i32, [f32;3], [f32;3], [f32;4])> =
            self.lights.iter().map(|l| (
                l.loc_enabled, l.loc_type, l.loc_pos, l.loc_target, l.loc_color,
                if l.enabled {1} else {0}, l.kind, l.pos, l.target, l.color)).collect();
        let sh = self.light_shader.as_mut().unwrap();
        if loc_view >= 0 { sh.set_shader_value(loc_view, view); }
        if loc_ambient >= 0 { sh.set_shader_value(loc_ambient, ambient); }
        if loc_fog_color >= 0 { sh.set_shader_value(loc_fog_color, fog_color); }
        if loc_fog_density >= 0 { sh.set_shader_value(loc_fog_density, fog_density); }
        if loc_esky >= 0 { sh.set_shader_value(loc_esky, env_sky); }
        if loc_eground >= 0 { sh.set_shader_value(loc_eground, env_ground); }
        if loc_eint >= 0 { sh.set_shader_value(loc_eint, env_int); }
        // HDR-Cubemap-IBL: useIBLMaps-Gate + Sampler-Units setzen. Das eigentliche
        // Binden der Maps (Slots 11/12/13) passiert in render_scene im Draw-Kontext
        // (rlFramebufferAttach/begin_drawing wuerden eine fruehere Bindung loesen).
        if loc_use_ibl >= 0 { sh.set_shader_value(loc_use_ibl, if use_ibl { 1i32 } else { 0i32 }); }
        // Die Einheiten IMMER setzen, nicht nur mit IBL. Ohne das blieben die
        // beiden Cubemap-Sampler auf ihrer Vorgabe 0 stehen -- auf derselben
        // Einheit wie `texture0` (sampler2D). Zwei VERSCHIEDENE Sampler-Arten
        // auf einer Einheit sind in WebGL 2 ein INVALID_OPERATION: der
        // Zeichenaufruf wird verworfen, ohne dass Shader-Uebersetzung oder
        // Verlinkung etwas melden. Genau daran blieb jedes MODEL_LIT-Modell im
        // Browser unsichtbar. Desktop-Treiber sind an der Stelle nachsichtig --
        // die Zuweisung schadet dort nichts (11-13 nutzt sonst niemand).
        if loc_irr >= 0 { sh.set_shader_value(loc_irr, 11i32); }
        if loc_pre >= 0 { sh.set_shader_value(loc_pre, 12i32); }
        if loc_brdf >= 0 { sh.set_shader_value(loc_brdf, 13i32); }
        for (le, lt, lp, ltg, lc, en, kind, pos, target, color) in lights {
            if le >= 0 { sh.set_shader_value(le, en); }
            if lt >= 0 { sh.set_shader_value(lt, kind); }
            if lp >= 0 { sh.set_shader_value(lp, pos); }
            if ltg >= 0 { sh.set_shader_value(ltg, target); }
            if lc >= 0 { sh.set_shader_value(lc, color); }
        }
    }

    /// Wie update_light_uniforms, aber fuer den Instancing-Shader (INST_VS/FS).
    /// Laedt viewPos/ambient/lightCount + alle Licht-Uniforms. Die pro-Licht-
    /// Locations werden lazy (re)aufgeloest, wenn sich die Lichter-Anzahl aendert
    /// (Lichter koennen nach dem Shader-Load via LIGHT_* hinzukommen).
    fn update_inst_light_uniforms(&mut self) {
        if self.inst_shader.is_none() { return; }
        // Licht-Locations bei Bedarf (neu) aufloesen.
        if self.inst_light_locs.len() != self.lights.len() {
            let sh = self.inst_shader.as_mut().unwrap();
            self.inst_light_locs = (0..self.lights.len()).map(|i| [
                sh.get_shader_location(&format!("lights[{}].enabled", i)),
                sh.get_shader_location(&format!("lights[{}].type", i)),
                sh.get_shader_location(&format!("lights[{}].position", i)),
                sh.get_shader_location(&format!("lights[{}].target", i)),
                sh.get_shader_location(&format!("lights[{}].color", i)),
            ]).collect();
        }
        let view = [self.cam3d.position.x, self.cam3d.position.y, self.cam3d.position.z];
        let ambient = self.light_ambient;
        let (loc_view, loc_ambient, loc_count) =
            (self.inst_loc_view, self.inst_loc_ambient, self.inst_loc_count);
        let active = self.lights.iter().filter(|l| l.enabled).count() as i32;
        // Licht-Daten + ihre Instancing-Locations lokal kopieren (Borrow-Trennung).
        let lights: Vec<([i32; 5], i32, i32, [f32; 3], [f32; 3], [f32; 4])> =
            self.lights.iter().zip(self.inst_light_locs.iter()).map(|(l, locs)| (
                *locs, if l.enabled {1} else {0}, l.kind, l.pos, l.target, l.color)).collect();
        let sh = self.inst_shader.as_mut().unwrap();
        if loc_view >= 0 { sh.set_shader_value(loc_view, view); }
        if loc_ambient >= 0 { sh.set_shader_value(loc_ambient, ambient); }
        if loc_count >= 0 { sh.set_shader_value(loc_count, active); }
        for (locs, en, kind, pos, target, color) in lights {
            if locs[0] >= 0 { sh.set_shader_value(locs[0], en); }
            if locs[1] >= 0 { sh.set_shader_value(locs[1], kind); }
            if locs[2] >= 0 { sh.set_shader_value(locs[2], pos); }
            if locs[3] >= 0 { sh.set_shader_value(locs[3], target); }
            if locs[4] >= 0 { sh.set_shader_value(locs[4], color); }
        }
    }

    pub fn cls(&mut self, color: i64) {
        // Innerhalb eines Render-Targets: Clear-Cmd in dessen Buffer (der Buffer
        // wird beim FLIP ohnehin transparent vorgecleart -> Clear setzt die Farbe).
        if let Some(rt) = self.active_rt {
            self.render_targets[rt].cmds.push(Cmd::Clear(col(color)));
            return;
        }
        // CLS setzt die Hintergrundfarbe (beim FLIP gecleart) und leert den
        // aktiven Layer (Wipe). Die Layer werden ohnehin pro FLIP geleert.
        let bg = if self.transparent {
            // Transparentes Fenster (SCREEN_TRANSPARENT): Alpha-Byte WOERTLICH nehmen
            // -- `CLS()`/`CLS(0)` -> voll durchsichtig (Desktop scheint durch),
            // `CLS(&Haarrggbb)` mit a<255 -> getoenter, durchscheinender Hintergrund.
            let v = color as u32;
            Color::new(((v >> 16) & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, (v & 0xFF) as u8, ((v >> 24) & 0xFF) as u8)
        } else {
            // Normaler Szenen-Hintergrund ist IMMER deckend -- ein Alpha-Anteil
            // wuerde sonst beim PostFX/RenderTexture-Compositing die Szene durchscheinen lassen.
            let mut b = col(color);
            b.a = 255;
            b
        };
        // Review-Fund: `clear_color` ist die GLOBALE Frame-Hintergrundfarbe
        // (FLIP macht `d.clear_background(clear_color)`), nicht pro Layer --
        // CLS() innerhalb einer NICHT-Haupt-Layer (`LAYER("ui") :
        // CLS(RGB(0,0,0))`, gedacht als reines Wipe dieser einen Layer)
        // ueberschrieb bisher trotzdem die globale Hintergrundfarbe und damit
        // effektiv den `bg`-Layer, unabhaengig von dessen z-Reihenfolge. Nur
        // auf der Haupt-Layer (active==0, kein LAYER(...) aktiv) darf CLS
        // die globale Hintergrundfarbe setzen; jede benannte Layer wird nur
        // per Cmds-Clear gewischt (das war schon vorher korrekt).
        if self.active == 0 {
            self.clear_color = bg;
        }
        let a = self.active;
        self.layers[a].cmds.clear();
    }

    // --- Z-Layer ---
    pub fn layer_define(&mut self, name: &str, z: i32) {
        if let Some(&i) = self.layer_names.get(name) {
            self.layers[i].z = z;
        } else {
            self.layers.push(Layer { z, cmds: Vec::new() });
            self.layer_names.insert(name.to_string(), self.layers.len() - 1);
        }
    }
    pub fn layer(&mut self, name: &str) {
        if let Some(&i) = self.layer_names.get(name) {
            self.active = i;
        } else {
            // Auto-Define mit Auto-Z (hoechstes + 10).
            let z = self.layers.iter().map(|l| l.z).max().unwrap_or(0) + 10;
            self.layers.push(Layer { z, cmds: Vec::new() });
            let idx = self.layers.len() - 1;
            self.layer_names.insert(name.to_string(), idx);
            self.active = idx;
        }
    }
    pub fn layer_end(&mut self) { self.active = 0; }
    pub fn layer_clear(&mut self, name: &str) {
        if let Some(&i) = self.layer_names.get(name) { self.layers[i].cmds.clear(); }
    }

    pub fn plot(&mut self, x: i32, y: i32, c: i64) { let (x, y) = self.w2s(x, y); self.emit(Cmd::Pixel(x, y, col(c))); }
    pub fn line(&mut self, a: i32, b: i32, cc: i32, d: i32, c: i64) { let (a, b) = self.w2s(a, b); let (cc, d) = self.w2s(cc, d); self.emit(Cmd::Line(a, b, cc, d, col(c))); }
    pub fn box_fill(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) { let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); self.emit(Cmd::BoxFill(x1, y1, x2, y2, col(c))); }
    pub fn rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) { let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); self.emit(Cmd::RectLines(x1, y1, x2, y2, col(c))); }
    pub fn circle(&mut self, x: i32, y: i32, r: i32, c: i64) { let (x, y) = self.w2s(x, y); let r = self.ssize(r); self.emit(Cmd::Circle(x, y, r as f32, col(c))); }
    /// Kreis-Kontur (Gegenstueck zum gefuellten CIRCLE). Nutzt die Ellipsen-
    /// Kontur mit gleichen Radien -> kein eigener Cmd noetig.
    pub fn circle_outline(&mut self, x: i32, y: i32, r: i32, c: i64) { let (x, y) = self.w2s(x, y); let r = self.ssize(r); self.emit(Cmd::Ellipse(x, y, r, r, col(c), false)); }
    // --- 2D-Extras (Batch 1) ---
    #[allow(clippy::too_many_arguments)]
    pub fn line_thick(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, w: f64, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        self.emit(Cmd::LineEx(x1, y1, x2, y2, (w * self.cam_zoom).max(1.0) as f32, col(c)));
    }
    #[allow(clippy::too_many_arguments)]
    pub fn round_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, radius: i32, c: i64, filled: bool) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        self.emit(Cmd::RoundRect(x1, y1, x2, y2, self.ssize(radius), col(c), filled));
    }
    #[allow(clippy::too_many_arguments)]
    pub fn gradient_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c1: i64, c2: i64, vertical: bool) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        self.emit(Cmd::GradientRect(x1, y1, x2, y2, col(c1), col(c2), vertical));
    }
    pub fn spline(&mut self, xs: &[i32], ys: &[i32], w: f64, c: i64) {
        let pts: Vec<(i32, i32)> = xs.iter().zip(ys).map(|(&x, &y)| self.w2s(x, y)).collect();
        self.emit(Cmd::Spline(pts, (w * self.cam_zoom).max(1.0) as f32, col(c)));
    }
    /// Rundes Rechteck mit senkrechtem Verlauf (siehe `Cmd::RoundGradient`).
    #[allow(clippy::too_many_arguments)]
    pub fn round_gradient(&mut self, x1: i32, y1: i32, x2: i32, y2: i32,
                          radius: i32, oben: i64, unten: i64) {
        let (x1, y1) = self.w2s(x1, y1);
        let (x2, y2) = self.w2s(x2, y2);
        self.emit(Cmd::RoundGradient(x1, y1, x2, y2, self.ssize(radius), col(oben), col(unten)));
    }

    /// Kreisring-Ausschnitt (Kuchenstueck bei `r_in` = 0, Donut/Tacho-Bogen
    /// sonst). Winkel in Grad, 0 = rechts, wachsend im Uhrzeigersinn.
    #[allow(clippy::too_many_arguments)]
    pub fn ring(&mut self, cx: i32, cy: i32, r_in: i32, r_out: i32,
                von: f64, bis: f64, c: i64, filled: bool) {
        let (cx, cy) = self.w2s(cx, cy);
        let (ri, ro) = (self.ssize(r_in), self.ssize(r_out));
        self.emit(Cmd::Ring(cx, cy, ri as f32, ro as f32, von as f32, bis as f32, col(c), filled));
    }

    // --- Blend-Modes (Batch 2) ---
    pub fn blend_mode(&mut self, mode: i32) { self.emit(Cmd::BlendMode(mode)); }

    // --- Prozedurale Texturen (Batch 3): liefern ein IMAGE-Handle ---

    /// Obergrenze fuer GENTEX_*(w, h, ...) -- ohne Cap loest ein versehentlich
    /// riesiges w/h (Tippfehler, vertauschte Parameter, z.B. GENTEX_COLOR mit
    /// vertauschten Skalierungs-/Pixel-Einheiten) eine Multi-GB/TB-Allokation
    /// in raylibs Image-Generatoren aus, die bei Fehlschlag den Prozess
    /// abbricht statt einen fangbaren Fehler zu liefern.
    const MAX_GENTEX_CELLS: i64 = 64_000_000; // z.B. 8000x8000

    fn check_gentex_dims(fn_: &str, w: i32, h: i32) -> Result<(i32, i32), String> {
        let w = w.max(1);
        let h = h.max(1);
        match (w as i64).checked_mul(h as i64) {
            Some(cells) if cells <= Self::MAX_GENTEX_CELLS => Ok((w, h)),
            _ => Err(format!(
                "{}: Breite*Hoehe ueberschreitet das Limit von {} Pixeln (erhalten {}x{})",
                fn_, Self::MAX_GENTEX_CELLS, w, h
            )),
        }
    }

    pub fn gen_tex_perlin(&mut self, w: i32, h: i32, scale: f64) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("GENTEX_PERLIN", w, h)?;
        let img = Image::gen_image_perlin_noise(w, h, 0, 0, scale.max(0.1) as f32);
        self.push_tex_from_image(img)
    }
    /// GENTEX_CELLULAR(w, h, kachel): Voronoi-/Zellrauschen -- Steinboden,
    /// Rissmuster, organische Strukturen. Kleinere Kachel = feiner.
    pub fn gen_tex_cellular(&mut self, w: i32, h: i32, tile: i64) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("GENTEX_CELLULAR", w, h)?;
        let img = Image::gen_image_cellular(w, h, tile.clamp(1, 4096) as i32);
        self.push_tex_from_image(img)
    }
    /// GENTEX_NOISE(w, h, anteil): Weissrauschen, `anteil` 0..1 = Anteil
    /// weisser Pixel. Grundlage fuer Sternenfelder, Korn, Dither-Masken.
    pub fn gen_tex_noise(&mut self, w: i32, h: i32, factor: f64) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("GENTEX_NOISE", w, h)?;
        let f = if factor.is_finite() { factor.clamp(0.0, 1.0) } else { 0.5 };
        let img = Image::gen_image_white_noise(w, h, f as f32);
        self.push_tex_from_image(img)
    }
    /// GENTEX_GRADIENT_BOX(w, h, dichte, c1, c2): rechteckiger Verlauf von der
    /// Mitte nach aussen -- Vignetten, Rahmen-Schatten (das eckige Gegenstueck
    /// zu GENTEX_RADIAL).
    pub fn gen_tex_gradient_square(&mut self, w: i32, h: i32, density: f64,
                                   c1: i64, c2: i64) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("GENTEX_GRADIENT_BOX", w, h)?;
        let d = if density.is_finite() { density.clamp(0.0, 1.0) } else { 0.5 };
        let img = Image::gen_image_gradient_square(w, h, d as f32, col(c1), col(c2));
        self.push_tex_from_image(img)
    }
    pub fn gen_tex_gradient(&mut self, w: i32, h: i32, c1: i64, c2: i64, vertical: bool) -> Result<i64, String> {
        // direction in Grad: 0 = vertikal (oben->unten), 90 = horizontal.
        let (w, h) = Self::check_gentex_dims("GENTEX_GRADIENT", w, h)?;
        let dir = if vertical { 0 } else { 90 };
        let img = Image::gen_image_gradient_linear(w, h, dir, col(c1), col(c2));
        self.push_tex_from_image(img)
    }
    pub fn gen_tex_checked(&mut self, w: i32, h: i32, cx: i32, cy: i32, c1: i64, c2: i64) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("GENTEX_CHECKED", w, h)?;
        let img = Image::gen_image_checked(w, h, cx.max(1), cy.max(1), col(c1), col(c2));
        self.push_tex_from_image(img)
    }
    pub fn gen_tex_color(&mut self, w: i32, h: i32, c: i64) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("GENTEX_COLOR", w, h)?;
        let img = Image::gen_image_color(w, h, col(c));
        self.push_tex_from_image(img)
    }
    /// Radialer Farbverlauf (Mitte `inner` -> Rand `outer`). `density` 0..1
    /// steuert die Groesse des hellen Kerns (0 = nur im Zentrum). Perfekt fuer
    /// weiche Glows/Lichter/Vignetten -- additiv gezeichnet ein sauberer Schein
    /// ohne die harten Kanten gestapelter Kreise.
    pub fn gen_tex_radial(&mut self, w: i32, h: i32, inner: i64, outer: i64, density: f64) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("GENTEX_RADIAL", w, h)?;
        let img = Image::gen_image_gradient_radial(
            w, h, density.clamp(0.0, 1.0) as f32, col(inner), col(outer));
        self.push_tex_from_image(img)
    }

    // --- Clipboard + Drag&Drop (Batch 5) ---
    pub fn clipboard_get(&self) -> String { self.rl.get_clipboard_text().unwrap_or_default() }
    pub fn clipboard_set(&mut self, s: &str) { let _ = self.rl.set_clipboard_text(s); }
    pub fn dropped_files(&self) -> Vec<String> {
        if !self.rl.is_file_dropped() { return Vec::new(); }
        self.rl.load_dropped_files().paths().iter().map(|s| s.to_string()).collect()
    }
    pub fn triangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); let (x3, y3) = self.w2s(x3, y3);
        self.emit(Cmd::Triangle(x1, y1, x2, y2, x3, y3, col(c)));
    }
    pub fn triangle_outline(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2); let (x3, y3) = self.w2s(x3, y3);
        self.emit(Cmd::TriLines(x1, y1, x2, y2, x3, y3, col(c)));
    }
    fn ellipse_box(x1: i32, y1: i32, x2: i32, y2: i32) -> (i32, i32, i32, i32) {
        let x = x1.min(x2); let y = y1.min(y2);
        let w = (x2 - x1).abs() + 1; let h = (y2 - y1).abs() + 1;
        (x + w / 2, y + h / 2, w / 2, h / 2) // cx, cy, rh, rv
    }
    pub fn ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        let (cx, cy, rh, rv) = Self::ellipse_box(x1, y1, x2, y2);
        self.emit(Cmd::Ellipse(cx, cy, rh, rv, col(c), true));
    }
    pub fn ellipse_outline(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        let (cx, cy, rh, rv) = Self::ellipse_box(x1, y1, x2, y2);
        self.emit(Cmd::Ellipse(cx, cy, rh, rv, col(c), false));
    }
    /// `width`: `None` = klassischer 1px-Strich (zoom-unabhaengig, wie vor
    /// Einfuehrung des Width-Parameters). `Some(w)` skaliert wie bei
    /// `line_thick`/`spline` mit `cam_zoom`.
    pub fn arc(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, start: f64, end: f64, width: Option<f64>, c: i64) {
        let (x1, y1) = self.w2s(x1, y1); let (x2, y2) = self.w2s(x2, y2);
        let (cx, cy, rh, rv) = Self::ellipse_box(x1, y1, x2, y2);
        let n = 48;
        let mut pts = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let t = start + (end - start) * (i as f64) / (n as f64);
            let px = cx as f64 + rh as f64 * t.cos();
            let py = cy as f64 - rv as f64 * t.sin(); // y nach unten -> CCW = minus sin
            pts.push((px.round() as i32, py.round() as i32));
        }
        let thick = match width { Some(w) => (w * self.cam_zoom).max(1.0) as f32, None => 1.0 };
        self.emit(Cmd::Poly(pts, thick, col(c), false));
    }
    pub fn polygon(&mut self, flat: &[i32], c: i64, filled: bool) -> Result<(), String> {
        if flat.len() < 6 || flat.len() % 2 != 0 {
            return Err("POLYGON: braucht mindestens 3 Punkte (6 Werte)".into());
        }
        let pts: Vec<(i32, i32)> = flat.chunks(2).map(|p| { let (x, y) = self.w2s(p[0], p[1]); (x, y) }).collect();
        if filled { self.emit(Cmd::FillPoly(pts, col(c))); }
        else { self.emit(Cmd::Poly(pts, 1.0, col(c), true)); }
        Ok(())
    }
    pub fn text(&mut self, x: i32, y: i32, s: String, c: i64) {
        // Position via w2s (inkl. Zoom), aber Font-Groesse bleibt -- wie Python.
        let (x, y) = self.w2s(x, y);
        let sz = self.text_size;
        let font = self.font_fuer(&s);
        let spacing = self.text_spacing;
        self.emit(Cmd::Text(x, y, s, sz, col(c), font, spacing));
    }
    pub fn set_text_size(&mut self, sz: i32) { self.text_size = sz.max(1); }

    /// TEXTROT(x, y, s$, winkel[, skala[, farbe]]) -- Text ZENTRIERT auf
    /// (x, y), um das Zentrum gedreht (Grad, Konvention wie DRAWIMAGEROT)
    /// und skaliert. Nutzt aktiven Font/Groesse/Spacing; Position laeuft
    /// durch die Camera (w2s) wie TEXT.
    pub fn text_rot(&mut self, x: i32, y: i32, s: String, angle_deg: f32, scale: f32, c: i64) {
        let (x, y) = self.w2s(x, y);
        let font = self.font_fuer(&s);
        self.emit(Cmd::TextRot(x, y, s, self.text_size, col(c), font,
                               self.text_spacing, angle_deg, scale.max(0.0001)));
    }

    /// Gedrehter Text mit EXPLIZITEM Font-Handle + Groesse.
    ///
    /// Gegenstueck zu `text_styled`: `text_rot` nimmt `active_font`/`text_size`,
    /// womit ein Aufrufer mit eigener Schrift (Modul `chart`) seinen gedrehten
    /// Text in einer ANDEREN Schrift bekaeme als den waagerechten.
    #[allow(clippy::too_many_arguments)]
    pub fn text_rot_styled(&mut self, x: i32, y: i32, s: String, angle_deg: f32,
                           c: i64, font: i64, size: i32) {
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::TextRot(x, y, s, size.max(1), col(c), font,
                               self.text_spacing, angle_deg, 1.0));
    }

    /// Text mit explizitem Font-Handle + Groesse (umgeht active_font/text_size).
    /// `font` = -1 -> Default-Font. Fuer per-Widget-Styling (Modul `gui`).
    pub fn text_styled(&mut self, x: i32, y: i32, s: String, c: i64, font: i64, size: i32) {
        let (x, y) = self.w2s(x, y);
        // Auch hier ausweichen: ein Widget ohne eigene Schrift (font = -1)
        // zeigt sonst "K?ln" in der Tabelle.
        let font = if font < 0 && self.fallback.is_some() && !s.is_ascii() {
            FONT_AUSWEICH
        } else { font };
        self.emit(Cmd::Text(x, y, s, size.max(1), col(c), font, self.text_spacing));
    }

    /// Laedt einen TTF/OTF-Font in der gegebenen Basis-Groesse -> FONT-Handle.
    ///
    /// Mit erweitertem Zeichensatz: raylib backt sonst nur die 95 ASCII-
    /// Glyphen, und jedes deutsche Wort mit Umlaut kaeme als "K?ln" heraus.
    pub fn load_font(&mut self, path: &str, size: i32) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        let chars = zeichensatz();
        let rl = &mut self.rl;
        let thread = &self.thread;
        let f = ohne_warnungen(|| rl.load_font_ex(thread, path, size.max(4), Some(&chars)))
            .map_err(|e| format!("LOADFONT: Font '{}' nicht ladbar: {}", path, e))?;
        // Bilinear filtern -> skalierter Text bleibt glatt statt pixelig/jaggy
        // (Default ist Nearest; sichtbar v.a. bei kleiner UI-Schrift).
        unsafe { raylib::ffi::SetTextureFilter(f.texture, 1 /*BILINEAR*/); }
        self.fonts.push(f);
        self.font_sizes.push(size.max(4));   // SETFONT uebernimmt diese Groesse
        Ok((self.fonts.len() - 1) as i64)
    }

    /// Wie `load_font`, aber ohne den Umweg ueber `resolve_asset_path` und mit
    /// Sentinel-Groesse: fuer den per `DHRT_FONT` gesetzten Default-Font.
    fn load_font_ext(&mut self, path: &str, size: i32) -> Result<i64, String> {
        let chars = zeichensatz();
        let rl = &mut self.rl;
        let thread = &self.thread;
        let f = ohne_warnungen(|| rl.load_font_ex(thread, path, size.max(4), Some(&chars)))
            .map_err(|e| format!("DHRT_FONT '{}' nicht ladbar: {}", path, e))?;
        unsafe { raylib::ffi::SetTextureFilter(f.texture, 1 /*BILINEAR*/); }
        self.fonts.push(f);
        self.font_sizes.push(0);   // Sentinel: Default-Font wendet seine Groesse NICHT an
        Ok((self.fonts.len() - 1) as i64)
    }
    /// Aktiven Font setzen (-1 = Default). Ungueltige Handles -> Fehler.
    /// Ein per LOADFONT geladener Font setzt zugleich text_size auf seine
    /// Lade-Groesse (so "wirkt" die LOADFONT-Groesse direkt); TEXT_SIZE
    /// danach uebersteuert weiterhin.
    pub fn set_font(&mut self, h: i64) -> Result<(), String> {
        if h < -1 || h >= self.fonts.len() as i64 {
            return Err(format!("SETFONT: ungueltiges FONT-Handle {}", h));
        }
        self.active_font = h;
        if h >= 0 {
            if let Some(&sz) = self.font_sizes.get(h as usize) {
                if sz > 0 { self.text_size = sz; }
            }
        }
        Ok(())
    }
    pub fn set_text_spacing(&mut self, px: i32) { self.text_spacing = px as f32; }

    pub fn text_width(&self, s: &str) -> i32 {
        // font_fuer: dieselbe Wahl wie beim Zeichnen, sonst misst ein Layout
        // die Standardschrift und bekommt am Ende den Ausweich-Font zu sehen.
        if let Some(f) = self.font_von(self.font_fuer(s)) {
            return f.measure_text(s, self.text_size as f32, self.text_spacing).x as i32;
        }
        let c = std::ffi::CString::new(s).unwrap_or_default();
        unsafe { raylib::ffi::MeasureText(c.as_ptr(), self.text_size) }
    }
    pub fn text_height(&self) -> i32 { self.text_size }

    /// Textbreite bei EXPLIZITER Groesse (statt `text_size`) -- Gegenstueck zu
    /// `text_styled`, das ebenfalls an `text_size` vorbeischreibt. Ohne das
    /// koennte ein Aufrufer mit eigener Schriftgroesse (Modul `chart`) seinen
    /// Text nicht mittig setzen.
    /// Textbreite bei expliziter Groesse UND explizitem Font-Handle.
    ///
    /// Gegenstueck zu `text_styled`, das ebenfalls ein Handle nimmt:
    /// `font` >= 0 ist ein geladener Font, -1 die Standardschrift. Wichtig,
    /// weil `text_width_at` mit dem AKTIVEN Font misst -- wer einem einzelnen
    /// Element eine eigene Schrift gegeben hat (Modul `gui`: GUI_SET_FONT),
    /// bekaeme sonst die Breite einer ANDEREN Schrift, und alles was darauf
    /// zentriert oder beschnitten wird saesse schief.
    /// Gerade aktive Schrift (-1 = Standardschrift). Wer mit explizitem
    /// Handle zeichnet, aber "wie global eingestellt" meint, braucht sie.
    pub fn active_font(&self) -> i64 { self.active_font }

    pub fn text_width_in(&self, s: &str, size: i32, font: i64) -> i32 {
        let size = size.max(1);
        // Gegenstueck zu text_styled, das bei Umlauten ebenfalls ausweicht.
        let font = if font < 0 && self.fallback.is_some() && !s.is_ascii() {
            FONT_AUSWEICH
        } else { font };
        if let Some(f) = self.font_von(font) {
            return f.measure_text(s, size as f32, self.text_spacing).x as i32;
        }
        // -1 (oder ein ungueltiges Handle) = Standardschrift.
        let c = std::ffi::CString::new(s).unwrap_or_default();
        unsafe { raylib::ffi::MeasureText(c.as_ptr(), size) }
    }

    pub fn text_width_at(&self, s: &str, size: i32) -> i32 {
        let size = size.max(1);
        if let Some(f) = self.font_von(self.font_fuer(s)) {
            return f.measure_text(s, size as f32, self.text_spacing).x as i32;
        }
        let c = std::ffi::CString::new(s).unwrap_or_default();
        unsafe { raylib::ffi::MeasureText(c.as_ptr(), size) }
    }

    // Bewusst NICHT uebernommen: animierte GIFs (`LoadImageAnim`). raylib haengt
    // die weiteren Bilder an `image.data` an, laesst `width`/`height` aber bei
    // EINEM Bild -- eine Textur daraus zeigt nur Bild 0. Korrigieren liesse sich
    // das nur, indem man `height` im Bild-Struct ueberschreibt; raylib-rs macht
    // `Image` dafuer aber ausdruecklich `readonly` (kein DerefMut, innerer Wert
    // `pub(crate)`, kein Konstruktor aus einer rohen ffi::Image). Der einzige Weg
    // waere rohes FFI am Wrapper vorbei -- genau das Muster, das hier schon
    // einmal einen Use-after-free verursacht hat (siehe MODEL_ANIMATE). Fuer
    // Animationen gibt es den Sprite-Blatt-Weg (ATLAS_*, `sprite`-Modul, der
    // Sprite-Editor exportiert Blaetter) -- der ist ohnehin der schnellere.

    // Bewusst NICHT uebernommen: raylibs `GetClipboardImage` ist ausdruecklich
    // Windows-only ("Do not use if you plan to compile to other platforms").
    // Ein Builtin, das nur auf einem der drei unterstuetzten Systeme etwas tut,
    // waere ein Rueckschritt gegenueber der Cross-Platform-Arbeit -- und CLIPBOARD_GET
    // (Text) funktioniert ueberall.

    /// LOADFONT_IMAGE(bild, trennfarbe, erstes_zeichen) -> FONT: Bitmap-Font
    /// aus einem PNG. Die Zeichen stehen nebeneinander, durch `trennfarbe`
    /// getrennt -- das klassische Verfahren fuer Pixel-Schriften, die
    /// LOADFONT (TTF) nicht scharf hinbekommt.
    pub fn load_font_image(&mut self, img: i64, key: i64, first: i64) -> Result<i64, String> {
        let src = self.src_image(img, "LOADFONT_IMAGE")?;
        let f = self.rl.load_font_from_image(&self.thread, &src, col(key), first as i32)
            .map_err(|e| format!("LOADFONT_IMAGE: {}", e))?;
        // Nearest lassen: Pixel-Schrift soll pixelig bleiben (anders als bei
        // LOADFONT, wo bilinear die skalierte TTF-Schrift glaettet).
        let size = f.baseSize.max(4);
        self.fonts.push(f);
        self.font_sizes.push(size);
        Ok((self.fonts.len() - 1) as i64)
    }
    /// TEXT_LINE_SPACING(px): Zeilenabstand fuer mehrzeiligen Text.
    pub fn text_line_spacing(&mut self, px: i64) {
        self.rl.set_text_line_spacing(px.clamp(0, 4096) as i32);
    }

    pub fn load_texture(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        if let Some(&h) = self.image_cache.get(path) { return Ok(h); }
        // CPU-Image laden (fuer imgfx) + GPU-Textur daraus.
        let img = Image::load_image(path).map_err(|e| format!("LOADIMAGE: {}", e))?;
        let tex = self.rl.load_texture_from_image(&self.thread, &img).map_err(|e| format!("LOADIMAGE: {}", e))?;
        self.textures.push(Tex { tex, img });
        self.tex_frei.push(false);
        let h = (self.textures.len() - 1) as i64;
        self.image_cache.insert(path.to_string(), h);
        if let Ok(abs) = std::fs::canonicalize(path) {
            self.image_cache.insert(abs.to_string_lossy().to_string(), h);
        }
        Ok(h)
    }

    fn push_tex_from_image(&mut self, img: Image) -> Result<i64, String> {
        let tex = self.rl.load_texture_from_image(&self.thread, &img).map_err(|e| format!("IMAGE: {}", e))?;
        self.textures.push(Tex { tex, img });
        self.tex_frei.push(false);
        Ok((self.textures.len() - 1) as i64)
    }

    /// Lebt dieses IMAGE-Handle noch?
    fn tex_ok(&self, idx: i64) -> bool {
        idx >= 0 && (idx as usize) < self.textures.len() && !self.tex_frei[idx as usize]
    }

    /// Die passende Meldung zu einem nicht benutzbaren Handle. "Freigegeben"
    /// und "gab es nie" sind verschiedene Fehler, und wer den einen fuer den
    /// anderen haelt, sucht an der falschen Stelle.
    fn tex_fehler(&self, idx: i64, fn_: &str) -> String {
        if idx >= 0 && (idx as usize) < self.textures.len() && self.tex_frei[idx as usize] {
            format!("{}: dieses Bild wurde mit IMAGE_FREE freigegeben", fn_)
        } else {
            format!("{}: ungueltiges IMAGE-Handle {}", fn_, idx)
        }
    }

    /// Ein Bild freigeben (IMAGE_FREE).
    ///
    /// Der Platz wird nicht neu vergeben: ein stehengebliebenes Handle soll
    /// sich melden statt still auf ein fremdes Bild zu zeigen. Zurueck bleibt
    /// ein 1x1-Platzhalter -- das eigentliche Bild und seine GPU-Textur sind
    /// weg, sobald das alte `Tex` fallengelassen wird.
    ///
    /// **Was das NICHT aufraeumt:** eine Textur, die per MODEL_TEXTURE an ein
    /// Modell gegeben wurde, lebt dort als reine ID weiter (raylibs
    /// Texture2D ist ein Struct ohne Zaehlung). Ein solches Modell zeigt
    /// danach auf eine freigegebene Textur. Dasselbe gilt fuer Bilder, die
    /// noch in einem Widget oder einem Sprite-Atlas stecken -- die melden
    /// sich immerhin, weil ihr Zeichenweg das Handle prueft.
    pub fn image_free(&mut self, idx: i64) -> Result<(), String> {
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGE_FREE")); }
        let i = idx as usize;
        // Aus dem Pfad-Cache werfen: sonst gibt LOADIMAGE fuer denselben Pfad
        // weiter dieses Handle heraus, und der naechste Aufruf haette ein
        // freigegebenes Bild in der Hand.
        self.image_cache.retain(|_, h| *h != idx);
        let leer = Image::gen_image_color(1, 1, Color::new(0, 0, 0, 0));
        let tex = self.rl.load_texture_from_image(&self.thread, &leer)
            .map_err(|e| format!("IMAGE_FREE: {}", e))?;
        self.textures[i] = Tex { tex, img: leer };
        self.tex_frei[i] = true;
        Ok(())
    }

    fn src_image(&self, idx: i64, fn_: &str) -> Result<Image, String> {
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, fn_)); }
        Ok(self.textures[idx as usize].img.clone())
    }

    // --- imgfx (immutable: liefern neues IMAGE-Handle) ---
    pub fn image_scale(&mut self, idx: i64, w: i32, h: i32) -> Result<i64, String> {
        if w <= 0 || h <= 0 { return Err("IMAGE_SCALE: w und h muessen > 0 sein".into()); }
        let mut img = self.src_image(idx, "IMAGE_SCALE")?;
        img.resize(w, h);
        self.push_tex_from_image(img)
    }
    /// IMAGE_SCALE_NN: Skalieren OHNE Interpolation (Nearest-Neighbour).
    /// `IMAGE_SCALE` glaettet bilinear -- fuer Pixelgrafik ist das falsch: aus
    /// harten Kanten werden Farbverlaeufe, ein 8x8-Sprite auf 32x32 ist danach
    /// matschig statt gross. Diese Variante behaelt die Bloecke.
    pub fn image_scale_nn(&mut self, idx: i64, w: i32, h: i32) -> Result<i64, String> {
        if w <= 0 || h <= 0 { return Err("IMAGE_SCALE_NN: w und h muessen > 0 sein".into()); }
        let mut img = self.src_image(idx, "IMAGE_SCALE_NN")?;
        img.resize_nn(w, h);
        self.push_tex_from_image(img)
    }
    pub fn image_rotate(&mut self, idx: i64, degrees: f32) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_ROTATE")?;
        img.rotate(degrees as i32);
        self.push_tex_from_image(img)
    }
    /// IMAGE_ROTATE_CW / IMAGE_ROTATE_CCW: EXAKTE Vierteldrehung.
    ///
    /// `IMAGE_ROTATE` rechnet trigonometrisch und tastet dabei neu ab -- fuer
    /// Pixelgrafik ist das falsch, und zwar auch bei 90 Grad: gemessen an
    /// einem 16x16-Bild mit vier Eckpunkten verschwinden nach
    /// `IMAGE_ROTATE(b, 90.0)` ALLE vier, und selbst die einfarbige Flaeche
    /// kommt verwaschen zurueck (0x141414 -> 0x131413). Dieselbe Falle wie
    /// `IMAGE_SCALE` gegen `IMAGE_SCALE_NN`, nur faellt sie hier noch mehr
    /// auf, weil eine Vierteldrehung eigentlich verlustfrei ist.
    ///
    /// raylib hat dafuer eigene Funktionen (`ImageRotateCW`/`CCW`), die die
    /// Punkte nur UMSORTIEREN. Breite und Hoehe tauschen dabei.
    pub fn image_rotate_cw(&mut self, idx: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_ROTATE_CW")?;
        img.rotate_cw();
        self.push_tex_from_image(img)
    }
    pub fn image_rotate_ccw(&mut self, idx: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_ROTATE_CCW")?;
        img.rotate_ccw();
        self.push_tex_from_image(img)
    }
    pub fn image_flip(&mut self, idx: i64, fx: bool, fy: bool) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_FLIP")?;
        if fx { img.flip_horizontal(); }
        if fy { img.flip_vertical(); }
        self.push_tex_from_image(img)
    }
    pub fn image_tint(&mut self, idx: i64, color: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_TINT")?;
        img.color_tint(col(color));
        self.push_tex_from_image(img)
    }
    pub fn image_copy(&mut self, idx: i64) -> Result<i64, String> {
        let img = self.src_image(idx, "IMAGE_COPY")?;
        self.push_tex_from_image(img)
    }
    pub fn image_crop(&mut self, idx: i64, x: i32, y: i32, w: i32, h: i32) -> Result<i64, String> {
        if w <= 0 || h <= 0 { return Err("IMAGE_CROP: w und h muessen > 0 sein".into()); }
        let mut img = self.src_image(idx, "IMAGE_CROP")?;
        img.crop(Rectangle::new(x as f32, y as f32, w as f32, h as f32));
        self.push_tex_from_image(img)
    }
    pub fn image_resize_canvas(&mut self, idx: i64, w: i32, h: i32, offx: i32, offy: i32, fill: i64) -> Result<i64, String> {
        if w <= 0 || h <= 0 { return Err("IMAGE_RESIZE_CANVAS: w und h muessen > 0 sein".into()); }
        let mut img = self.src_image(idx, "IMAGE_RESIZE_CANVAS")?;
        img.resize_canvas(w, h, offx, offy, col(fill));
        self.push_tex_from_image(img)
    }
    pub fn image_blur(&mut self, idx: i64, radius: i32) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_BLUR")?;
        img.blur_gaussian(radius.max(0));
        self.push_tex_from_image(img)
    }
    /// IMAGE_CONVOLVE(bild, kern): freie Faltung mit einem quadratischen Kern
    /// (3x3, 5x5, ...). Damit sind Schaerfen, Kantenerkennung, Praegen und
    /// eigene Weichzeichner moeglich -- `IMAGE_BLUR` kann nur Gauss.
    /// Der Kern kommt als flaches ARRAY OF FLOAT in Zeilenfolge.
    pub fn image_convolve(&mut self, idx: i64, kernel: &[f64]) -> Result<i64, String> {
        let n = kernel.len();
        let side = (n as f64).sqrt().round() as usize;
        if n == 0 || side * side != n {
            return Err(format!(
                "IMAGE_CONVOLVE: Kern muss quadratisch sein (9 = 3x3, 25 = 5x5, ...), \
hat aber {} Werte", n));
        }
        if side % 2 == 0 {
            return Err(format!(
                "IMAGE_CONVOLVE: Kern-Seitenlaenge muss ungerade sein (3, 5, 7, ...), ist {}",
                side));
        }
        let k: Vec<f32> = kernel.iter().map(|v| if v.is_finite() { *v as f32 } else { 0.0 }).collect();
        let mut img = self.src_image(idx, "IMAGE_CONVOLVE")?;
        img.kernel_convolution(&k)
            .map_err(|e| format!("IMAGE_CONVOLVE: {}", e))?;
        self.push_tex_from_image(img)
    }
    /// IMAGE_ALPHA_MASK(bild, maske): Graustufen der Maske werden zum
    /// Alphakanal -- der uebliche Weg fuer weiche Raender und Uebergaenge.
    pub fn image_alpha_mask(&mut self, idx: i64, mask: i64) -> Result<i64, String> {
        let m = self.src_image(mask, "IMAGE_ALPHA_MASK")?;
        let mut img = self.src_image(idx, "IMAGE_ALPHA_MASK")?;
        img.alpha_mask(&m);
        self.push_tex_from_image(img)
    }
    /// IMAGE_ALPHA_CROP(bild, schwelle): voellig durchsichtigen Rand abschneiden
    /// (Sprites aus einem zu grossen Blatt eng zuschneiden).
    pub fn image_alpha_crop(&mut self, idx: i64, threshold: f64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_ALPHA_CROP")?;
        img.alpha_crop(if threshold.is_finite() { threshold.clamp(0.0, 1.0) as f32 } else { 0.0 });
        self.push_tex_from_image(img)
    }
    /// IMAGE_ALPHA_PREMULTIPLY(bild): Farbe mit Alpha vormultiplizieren --
    /// beseitigt die dunklen Saeume an weichen Raendern beim Skalieren.
    pub fn image_alpha_premultiply(&mut self, idx: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_ALPHA_PREMULTIPLY")?;
        img.alpha_premultiply();
        self.push_tex_from_image(img)
    }
    /// IMAGE_DITHER(bild, r, g, b, a): Farbtiefe pro Kanal in Bit reduzieren,
    /// mit Floyd-Steinberg-Fehlerverteilung -- der Retro-Look.
    ///
    /// raylib setzt NUR die drei echten 16-Bit-Formate um; bei jeder anderen
    /// Kombination warnt es bloss und laesst das Bild mit ungueltigem Format
    /// zurueck (die Textur wird dann schwarz). Deshalb hier hart abgelehnt --
    /// Validierung gehoert in den Wrapper, nicht ins Backend.
    pub fn image_dither(&mut self, idx: i64, r: i64, g: i64, b: i64, al: i64) -> Result<i64, String> {
        if !matches!((r, g, b, al), (5, 6, 5, 0) | (5, 5, 5, 1) | (4, 4, 4, 4)) {
            return Err(format!(
                "IMAGE_DITHER: nur 5,6,5,0 (RGB565) / 5,5,5,1 (RGB5551) / 4,4,4,4 (RGBA4444) \
moeglich -- bekam {},{},{},{}", r, g, b, al));
        }
        let mut img = self.src_image(idx, "IMAGE_DITHER")?;
        // raylib dithert nur aus R8G8B8(A8) heraus; GENTEX_*-Bilder koennen ein
        // anderes Format haben -> vorher angleichen, sonst passiert nichts.
        img.set_format(raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        img.dither(r as i32, g as i32, b as i32, al as i32);
        self.push_tex_from_image(img)
    }
    /// IMAGE_PALETTE(bild, max): die haeufigsten Farben als ARRAY OF INTEGER
    /// (0xRRGGBB) -- fuer automatische Paletten, Farbschema-Ableitung, Retro-
    /// Umfaerbung.
    pub fn image_palette(&mut self, idx: i64, max: i64) -> Result<Vec<i64>, String> {
        let img = self.src_image(idx, "IMAGE_PALETTE")?;
        let pal = img.extract_palette(max.clamp(1, 4096) as u32);
        Ok(pal.iter()
            .filter(|c| c.a > 0)                  // voll durchsichtige Fuellwerte weglassen
            .map(|c| ((c.r as i64) << 16) | ((c.g as i64) << 8) | c.b as i64)
            .collect())
    }
    pub fn image_brightness(&mut self, idx: i64, n: i32) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_BRIGHTNESS")?;
        img.color_brightness(n.clamp(-255, 255));
        self.push_tex_from_image(img)
    }
    pub fn image_contrast(&mut self, idx: i64, n: f32) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_CONTRAST")?;
        img.color_contrast(n.clamp(-100.0, 100.0));
        self.push_tex_from_image(img)
    }
    pub fn image_grayscale(&mut self, idx: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_GRAYSCALE")?;
        img.color_grayscale();
        self.push_tex_from_image(img)
    }
    pub fn image_invert(&mut self, idx: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_INVERT")?;
        img.color_invert();
        self.push_tex_from_image(img)
    }
    pub fn image_replace_color(&mut self, idx: i64, from: i64, to: i64) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_REPLACE_COLOR")?;
        img.color_replace(col(from), col(to));
        self.push_tex_from_image(img)
    }

    // --- imgfx In-Image-Zeichnen (MUTIEREND: veraendert das uebergebene Image
    // direkt, anders als die immutablen Filter oben). Nach jeder Mutation wird
    // die GPU-Textur neu hochgeladen, damit DRAWIMAGE das Ergebnis zeigt. Daher:
    // zum Aufbauen eines Bildes (LOADIMAGE-Zeit), nicht pro Frame. ---
    fn reupload_tex(&mut self, i: usize) -> Result<(), String> {
        // Review-Fund: baute hier bisher IMMER eine komplett NEUE GPU-Textur
        // (neue GL-Textur-ID) und ersetzte `self.textures[i].tex` -- das
        // droppt die ALTE Texture2D (UnloadTexture). raylibs Texture2D ist
        // nur ein {id, width, height, ...}-Struct OHNE Refcounting: jedes
        // Model/Material, das vorher per MODEL_TEXTURE/MODEL_TEXTURE_NORMAL
        // auf diese Textur-ID zeigte, haelt eine reine Kopie der ALTEN ID und
        // zeigt danach auf eine freigegebene/von der GPU wiederverwendete
        // Textur (Model rendert falsch/leer/garbage). IMAGE_DRAW_* aendert
        // Bild-Dimensionen/-Format nie (nur einzelne Pixel werden mutiert) --
        // ein In-Place-Update der BESTEHENDEN Textur via UpdateTexture (GL-ID
        // bleibt unveraendert bestehen) ist daher moeglich und vermeidet den
        // Unload+Reload-Zyklus komplett.
        let t = &mut self.textures[i];
        let size = t.img.get_pixel_data_size();
        let bytes: &[u8] = unsafe { std::slice::from_raw_parts(t.img.data as *const u8, size) };
        t.tex.update_texture(bytes)
            .map_err(|e| format!("IMAGE_DRAW: Textur-Update fehlgeschlagen: {:?}", e))
    }
    pub fn image_draw_line(&mut self, idx: i64, x1: i32, y1: i32, x2: i32, y2: i32, color: i64) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGE_DRAW_LINE")); }
        let t = &mut self.textures[i];
        t.img.draw_line(x1, y1, x2, y2, col(color));
        self.reupload_tex(i)
    }
    pub fn image_draw_circle(&mut self, idx: i64, cx: i32, cy: i32, r: i32, color: i64) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGE_DRAW_CIRCLE")); }
        let t = &mut self.textures[i];
        t.img.draw_circle(cx, cy, r.max(0), col(color));
        self.reupload_tex(i)
    }
    pub fn image_draw_rect(&mut self, idx: i64, x: i32, y: i32, w: i32, h: i32, color: i64) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGE_DRAW_RECT")); }
        let t = &mut self.textures[i];
        t.img.draw_rectangle(x, y, w.max(0), h.max(0), col(color));
        self.reupload_tex(i)
    }
    pub fn image_draw_text(&mut self, idx: i64, x: i32, y: i32, text: &str, size: i32, color: i64) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGE_DRAW_TEXT")); }
        let t = &mut self.textures[i];
        t.img.draw_text(text, x, y, size.max(1), col(color));
        self.reupload_tex(i)
    }
    /// Ein neues Bild anlegen (IMAGE_NEW). Ohne Farbe: vollstaendig DURCHSICHTIG.
    ///
    /// `GENTEX_COLOR` kann das nicht, und zwar aus einem Grund, der sich nicht
    /// umgehen laesst: die Farbkonvention deutet Alpha 0 als DECKEND (damit
    /// `&Hrrggbb` und `RGB(r,g,b)` deckend bleiben, siehe `col`). Ein wirklich
    /// durchsichtiges Bild ist ueber eine FARBE also gar nicht auszudruecken --
    /// es braucht einen eigenen Weg. Fuer eine leere Ebene ist genau das der
    /// Normalfall.
    pub fn image_new(&mut self, w: i32, h: i32, farbe: Option<i64>) -> Result<i64, String> {
        let (w, h) = Self::check_gentex_dims("IMAGE_NEW", w, h)?;
        let c = match farbe { Some(v) => col(v), None => Color::new(0, 0, 0, 0) };
        self.push_tex_from_image(Image::gen_image_color(w, h, c))
    }

    /// Einen Bereich vollstaendig durchsichtig machen (IMAGE_CLEAR) -- der
    /// Radierer. Ohne Rechteck das ganze Bild.
    ///
    /// Geht ueber `draw_rectangle`, weil raylibs `ImageDrawRectangle` die Farbe
    /// SCHREIBT statt sie einzumischen. Wuerde es mischen, waere ein
    /// durchsichtiges Rechteck ein Nichts-Tun -- und Radieren unmoeglich.
    pub fn image_clear(&mut self, idx: i64, r: Option<(i32, i32, i32, i32)>) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGE_CLEAR")); }
        let t = &mut self.textures[i];
        let (x, y, w, h) = r.unwrap_or((0, 0, t.img.width, t.img.height));
        t.img.draw_rectangle(x, y, w.max(0), h.max(0), Color::new(0, 0, 0, 0));
        self.reupload_tex(i)
    }

    /// Ein Bild in ein anderes zeichnen (IMAGE_DRAW_IMAGE), mit Alpha gemischt.
    ///
    /// Damit lassen sich Ebenen zu einem Bild verrechnen und ein kopierter
    /// Ausschnitt einsetzen -- beides ging vorher gar nicht, weil die
    /// `IMAGE_DRAW_*`-Familie nur Formen und Text kennt.
    ///
    /// Die Quelle wird KOPIERT geholt, bevor das Ziel veraenderlich geliehen
    /// wird. Nebenbei ist damit `ziel = quelle` erlaubt (ein Bild auf sich
    /// selbst versetzt zeichnen) statt ein Ausleih-Fehler.
    pub fn image_draw_image(&mut self, dst: i64, src: i64, x: i32, y: i32,
                            quelle: Option<(i32, i32, i32, i32)>, tint: i64)
                            -> Result<(), String> {
        let s = self.src_image(src, "IMAGE_DRAW_IMAGE")?;
        let (sx, sy, sw, sh) = quelle.unwrap_or((0, 0, s.width, s.height));
        if sw <= 0 || sh <= 0 {
            return Err("IMAGE_DRAW_IMAGE: Quellbreite und -hoehe muessen > 0 sein".into());
        }
        if !self.tex_ok(dst) { return Err(self.tex_fehler(dst, "IMAGE_DRAW_IMAGE")); }
        let i = dst as usize;
        let t = &mut self.textures[i];
        t.img.draw(&s,
                   Rectangle::new(sx as f32, sy as f32, sw as f32, sh as f32),
                   Rectangle::new(x as f32, y as f32, sw as f32, sh as f32),
                   col(tint));
        self.reupload_tex(i)
    }

    /// Deckkraft eines Bildpunkts (GETALPHA), 0..255; -1 ausserhalb.
    ///
    /// `GETPIXEL` liefert nur die drei Farbkanaele -- und kann es auch nicht
    /// anders: der Rueckgabewert ist wieder eine FARBE, und dort bedeutet ein
    /// oberstes Byte 0 "deckend". Ein durchsichtiger Punkt kaeme als deckendes
    /// Schwarz zurueck. Deshalb ein eigener Getter statt eines vierten Bytes.
    pub fn get_alpha(&mut self, idx: i64, x: i32, y: i32) -> i64 {
        if !self.tex_ok(idx) { return -1; }
        let img = &self.textures[idx as usize].img;
        if x < 0 || y < 0 || x >= img.width || y >= img.height { return -1; }
        img.get_color(x, y).a as i64
    }

    /// Mehrere Bilder als animiertes GIF schreiben (IMAGE_SAVE_GIF).
    ///
    /// `verzoegerung` in Hundertstelsekunden. Die Bilder werden HIER in
    /// RGBA umgewandelt -- der Schreiber selbst kennt raylib nicht und ist
    /// damit fuer sich testbar (siehe gifschreiber.rs).
    pub fn image_save_gif(&mut self, handles: &[i64], pfad: &str,
                          verzoegerung: u16, wiederholen: bool) -> Result<(), String> {
        let mut bilder = Vec::with_capacity(handles.len());
        for &idx in handles {
            if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGE_SAVE_GIF")); }
            let img = &self.textures[idx as usize].img;
            let (w, h) = (img.width, img.height);
            if w <= 0 || h <= 0 || w > u16::MAX as i32 || h > u16::MAX as i32 {
                return Err(std::format!("IMAGE_SAVE_GIF: Bildgroesse {}x{} geht nicht", w, h));
            }
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for y in 0..h {
                for x in 0..w {
                    let c = img.get_color(x, y);
                    rgba.extend_from_slice(&[c.r, c.g, c.b, c.a]);
                }
            }
            bilder.push(crate::gifschreiber::Bild {
                breite: w as u16, hoehe: h as u16, rgba,
            });
        }
        crate::gifschreiber::schreiben(pfad, &bilder, verzoegerung, wiederholen)
    }

    /// Ein Bild in eine Datei schreiben (IMAGE_SAVE).
    ///
    /// Das Format bestimmt die Endung. Ton konnte sich seit dem SFX-Editor
    /// sichern (`AUDIO_SAVE_WAV`), Bild bis hierher gar nicht -- `SAVESCREENSHOT`
    /// sichert den BILDSCHIRM, nicht ein Bild.
    pub fn image_save(&mut self, idx: i64, pfad: &str) -> Result<(), String> {
        const ENDUNGEN: [&str; 4] = ["png", "bmp", "jpg", "tga"];
        let endung = std::path::Path::new(pfad).extension()
            .and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        if !ENDUNGEN.contains(&endung.as_str()) {
            return Err(std::format!(
                "IMAGE_SAVE: '{}' -- die Endung entscheidet ueber das Format, moeglich sind {}",
                pfad, ENDUNGEN.join(", ")));
        }
        // Ob die Datei vorher schon da war, muss VOR dem Schreiben feststehen:
        // die Bindung wirft raylibs Erfolgs-Flag weg (`export_image` liefert
        // `()`), es bleibt also nur, hinterher an der Datei nachzusehen -- und
        // eine stehengebliebene alte Datei sonst als Erfolg zu lesen.
        let vorher = std::fs::metadata(pfad).ok().map(|m| (m.len(), m.modified().ok()));
        let img = self.src_image(idx, "IMAGE_SAVE")?;
        img.export_image(pfad);
        match std::fs::metadata(pfad) {
            Ok(m) if m.len() > 0
                && Some((m.len(), m.modified().ok())) != vorher => Ok(()),
            _ => Err(std::format!(
                "IMAGE_SAVE: '{}' liess sich nicht schreiben (Verzeichnis vorhanden? schreibbar?)",
                pfad)),
        }
    }

    fn cache_image_alias(&mut self, alias: &str, handle: i64) {
        self.image_cache.insert(alias.to_string(), handle);
    }
    pub fn draw_image(&mut self, idx: i64, x: i32, y: i32) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "DRAWIMAGE")); }
        let (dw, dh) = (self.textures[i].tex.width, self.textures[i].tex.height);
        let (dw, dh) = (self.ssize(dw), self.ssize(dh));
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::Texture(i, x, y, dw, dh));
        Ok(())
    }
    /// Textur skaliert in ein Ziel-Rechteck (Modul `gui` Image-Widget).
    /// Bounds-safe (ungueltiges Handle / idx<0 -> No-Op beim Rendern).
    pub fn draw_image_rect(&mut self, idx: i64, x: i32, y: i32, w: i32, h: i32) {
        if idx < 0 { return; }
        let (x, y) = self.w2s(x, y);
        let (w, h) = (self.ssize(w), self.ssize(h));
        self.emit(Cmd::TextureRect(idx as usize, x, y, w, h));
    }
    pub fn draw_image_part(&mut self, idx: i64, sx: i32, sy: i32, sw: i32, sh: i32, dx: i32, dy: i32) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "DRAWIMAGEPART")); }
        let (dw, dh) = (self.ssize(sw), self.ssize(sh));
        let (dx, dy) = self.w2s(dx, dy);
        self.emit(Cmd::TexturePart(i, sx, sy, sw, sh, dx, dy, dw, dh));
        Ok(())
    }
    /// Wie draw_image_part, aber mit Ziel-Groesse (dw,dh) -> skaliertes Blitten
    /// eines Sub-Rechtecks (z. B. Sprite-Sheet-Kachel gross zeichnen).
    pub fn draw_image_part_ex(&mut self, idx: i64, sx: i32, sy: i32, sw: i32, sh: i32,
                              dx: i32, dy: i32, dw: i32, dh: i32) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "DRAWIMAGEPARTEX")); }
        let (dw, dh) = (self.ssize(dw), self.ssize(dh));
        let (dx, dy) = self.w2s(dx, dy);
        self.emit(Cmd::TexturePartEx(i, sx, sy, sw, sh, dx, dy, dw, dh));
        Ok(())
    }
    /// 9-Slice: ein Bild auf eine beliebige Groesse ziehen, ohne dass die
    /// Raender mitverzerren.
    ///
    /// Das Bild wird in neun Stuecke geteilt (`rand` breit): die vier Ecken
    /// bleiben unveraendert, die Kanten dehnen sich nur ENTLANG ihrer Achse,
    /// die Mitte in beide. Genau das braucht eine Knopf-Grafik, die fuer
    /// jede Beschriftungslaenge passen soll -- ein schlicht skaliertes Bild
    /// wuerde seine runden Ecken zu Ellipsen ziehen.
    #[allow(clippy::too_many_arguments)]
    pub fn nine_slice(&mut self, idx: i64, x: i32, y: i32, w: i32, h: i32,
                      rand: i32) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "9-Slice")); }
        let (bw, bh) = (self.textures[i].tex.width, self.textures[i].tex.height);
        let r = neun_rand(bw, bh, w, h, rand);
        if r <= 0 {
            return self.draw_image_part_ex(idx, 0, 0, bw, bh, x, y, w, h);
        }
        let (sx, sw) = neun_spannen(0, bw, r);
        let (sy, sh) = neun_spannen(0, bh, r);
        let (dx, dw) = neun_spannen(x, w, r);
        let (dy, dh) = neun_spannen(y, h, r);
        for c in 0..3 {
            for z in 0..3 {
                if sw[c] <= 0 || sh[z] <= 0 || dw[c] <= 0 || dh[z] <= 0 {
                    continue;
                }
                self.draw_image_part_ex(idx, sx[c], sy[z], sw[c], sh[z],
                                        dx[c], dy[z], dw[c], dh[z])?;
            }
        }
        Ok(())
    }

    pub fn draw_image_flipped(&mut self, idx: i64, x: i32, y: i32, fh: bool, fv: bool) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "DRAWIMAGEFLIPPED")); }
        let (dw, dh) = (self.textures[i].tex.width, self.textures[i].tex.height);
        let (dw, dh) = (self.ssize(dw), self.ssize(dh));
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::TextureFlipped(i, x, y, dw, dh, fh, fv));
        Ok(())
    }
    /// Rotierter Sprite-Blit: zeichnet das Bild **zentriert** auf (x,y), gedreht
    /// um `angle_deg` Grad (um das Zentrum), skaliert mit `scale`, getoent mit
    /// `tint` (None = unveraendert). Ideal fuer physics2d (`x,y = BODY_X/Y`,
    /// `angle = DEG(PHYS2D_BODY_ANGLE(...))`).
    pub fn draw_image_rot(&mut self, idx: i64, x: i32, y: i32, angle_deg: f32,
                          scale: f32, tint: Option<i64>) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "DRAWIMAGEROT")); }
        let (x, y) = self.w2s(x, y);
        let c = tint.map(col).unwrap_or(Color::WHITE);
        // `scale` ist der User-Skalierungsfaktor -- cam_zoom hier mit
        // hineinmultiplizieren (wie ssize() es fuer Integer-Groessen tut),
        // sonst ignoriert DRAWIMAGEROT die Kamera-Zoomstufe komplett.
        let scl = (scale.max(0.0001) as f64 * self.cam_zoom).max(0.0001) as f32;
        self.emit(Cmd::TextureRot(i, x, y, angle_deg, scl, c));
        Ok(())
    }
    /// Zeichnet eine 2D-Tilemap (flache row-major `values`; Tile < 0 =
    /// transparent). Tileset wird als gerasterter Strip interpretiert
    /// (tiles_per_row = tileset_breite / tw). Jedes Tile geht durch
    /// `draw_image_part`, d.h. Camera (Translation + Zoom) wirkt korrekt.
    pub fn draw_tilemap(&mut self, idx: i64, values: &[i64], rows: i32, cols: i32,
                        tw: i32, th: i32, sx: i32, sy: i32) -> Result<(), String> {
        let i = idx as usize;
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "DRAWTILEMAP")); }
        let tex_w = self.textures[i].tex.width as i32;
        let tiles_per_row = (tex_w / tw.max(1)).max(1);
        for r in 0..rows {
            let base = (r * cols) as usize;
            for c in 0..cols {
                let tile = values[base + c as usize];
                if tile < 0 { continue; }
                let tile = tile as i32;
                let sc = tile % tiles_per_row;
                let sr = tile / tiles_per_row;
                self.draw_image_part(idx, sc * tw, sr * th, tw, th, sx + c * tw, sy + r * th)?;
            }
        }
        Ok(())
    }

    pub fn image_width(&self, idx: i64) -> Result<i64, String> {
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGEWIDTH")); }
        Ok(self.textures[idx as usize].tex.width as i64)
    }
    pub fn image_height(&self, idx: i64) -> Result<i64, String> {
        if !self.tex_ok(idx) { return Err(self.tex_fehler(idx, "IMAGEHEIGHT")); }
        Ok(self.textures[idx as usize].tex.height as i64)
    }

    // --- LOAD_ASSETS: Bilder aus JSON-Manifest vorladen (Alias/Pfad-Cache) ---
    pub fn load_assets(&mut self, manifest_path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(manifest_path);
        let manifest_path = resolved.as_str();
        let text = std::fs::read_to_string(manifest_path)
            .map_err(|_| format!("LOAD_ASSETS: Manifest nicht gefunden: {}", manifest_path))?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("LOAD_ASSETS: Manifest-Lesefehler '{}': {}", manifest_path, e))?;
        let dir = std::path::Path::new(manifest_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let mut count = 0i64;
        if let Some(images) = json.get("images") {
            if let Some(obj) = images.as_object() {
                for (alias, rel) in obj {
                    let rel = rel.as_str().ok_or("LOAD_ASSETS: Pfad muss STRING sein")?;
                    let full = dir.join(rel);
                    let h = self.load_texture(&full.to_string_lossy())?;
                    self.cache_image_alias(alias, h);
                    count += 1;
                }
            } else if let Some(arr) = images.as_array() {
                for rel in arr {
                    let rel = rel.as_str().ok_or("LOAD_ASSETS: Pfad muss STRING sein")?;
                    let full = dir.join(rel);
                    self.load_texture(&full.to_string_lossy())?;
                    count += 1;
                }
            }
        }
        // "sounds" wird ignoriert (Audio = spaeterer Schritt).
        Ok(count)
    }

    // --- Sprite-Atlas ---
    pub fn atlas_load(&mut self, manifest_path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(manifest_path);
        let manifest_path = resolved.as_str();
        let text = std::fs::read_to_string(manifest_path)
            .map_err(|_| format!("ATLAS_LOAD: Manifest nicht gefunden: {}", manifest_path))?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("ATLAS_LOAD: {}", e))?;
        let dir = std::path::Path::new(manifest_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let image = json.get("image").and_then(|v| v.as_str())
            .ok_or("ATLAS_LOAD: 'image' fehlt im Manifest")?;
        let full = dir.join(image);
        let tex_idx = self.load_texture(&full.to_string_lossy())? as usize;
        let mut frames = HashMap::new();
        if let Some(sprites) = json.get("sprites").and_then(|v| v.as_object()) {
            for (name, rect) in sprites {
                let arr = rect.as_array().ok_or("ATLAS_LOAD: rect muss [x,y,w,h] sein")?;
                if arr.len() != 4 { return Err(format!("ATLAS_LOAD: rect '{}' braucht 4 Werte", name)); }
                let g = |i: usize| arr[i].as_i64().unwrap_or(0) as i32;
                frames.insert(name.clone(), (g(0), g(1), g(2), g(3)));
            }
        }
        self.atlases.push(Atlas { tex_idx, frames });
        Ok((self.atlases.len() - 1) as i64)
    }
    pub fn atlas_draw(&mut self, atlas: i64, name: &str, x: i32, y: i32, flip_h: bool,
                      flip_v: bool, tint: Option<i64>) -> Result<(), String> {
        let (tex, sx, sy, sw, sh) = {
            let a = self.atlases.get(atlas as usize).ok_or("ATLAS_DRAW: ungueltiges Atlas-Handle")?;
            let &(sx, sy, sw, sh) = a.frames.get(name)
                .ok_or_else(|| format!("ATLAS_DRAW: Sprite '{}' nicht im Atlas", name))?;
            (a.tex_idx, sx, sy, sw, sh)
        };
        let (dw, dh) = (self.ssize(sw), self.ssize(sh));
        let (x, y) = self.w2s(x, y);
        let tcol = match tint { Some(c) => col(c), None => Color::WHITE };
        self.emit(Cmd::AtlasDraw(tex, sx, sy, sw, sh, x, y, dw, dh, flip_h, flip_v, tcol));
        Ok(())
    }

    /// SPRITE_DRAW: aktuelles Frame als Sheet-Sub-Rect, Camera-aware.
    pub fn draw_sprite(&mut self, tex_idx: i64, frame: i32, fw: i32, fh: i32,
                       x: i32, y: i32, flip_x: bool, flip_y: bool,
                       scale_x: f64, scale_y: f64, tint: Option<i64>) -> Result<(), String> {
        let i = tex_idx as usize;
        if !self.tex_ok(i as i64) { return Err(self.tex_fehler(i as i64, "SPRITE_DRAW")); }
        let tex_w = self.textures[i].tex.width;
        let cols = (tex_w / fw).max(1);
        let (gcol, grow) = (frame % cols, frame / cols);
        let (sx, sy) = (gcol * fw, grow * fh);
        let (dx, dy) = self.w2s(x, y);
        // Zielgroesse: Frame * Sprite-Scale * Camera-Zoom (Screen-Scale `s` im Replay).
        let dw = (fw as f64 * scale_x * self.cam_zoom).max(1.0) as i32;
        let dh = (fh as f64 * scale_y * self.cam_zoom).max(1.0) as i32;
        let tint_col = match tint {
            Some(c) => col(c),
            None => Color::WHITE,
        };
        self.emit(Cmd::SpriteDraw(i, sx, sy, fw, fh, dx, dy, dw, dh, flip_x, flip_y, tint_col));
        Ok(())
    }

    pub fn key_down(&self, code: i64) -> bool {
        use raylib::consts::KeyboardKey::*;
        // Negative Codes = Gamepad-Buttons/DPad (siehe DEFAULT_KEYS joy_*).
        if code < 0 { return self.joy_button_down(code); }
        // "+"/"-" (ASCII 43/45): pygame liefert layout-aware Keysyms, raylib aber
        // PHYSISCHE US-Positionen. Daher pro Code mehrere moegliche Tasten
        // pruefen: US-Haupttaste, Ziffernblock und die dt.-Layout-Position
        // (auf DE liegt "+" an US "]", "-" an US "/").
        match code {
            43 => return self.rl.is_key_down(KEY_EQUAL)
                       || self.rl.is_key_down(KEY_KP_ADD)
                       || self.rl.is_key_down(KEY_RIGHT_BRACKET),
            45 => return self.rl.is_key_down(KEY_MINUS)
                       || self.rl.is_key_down(KEY_KP_SUBTRACT)
                       || self.rl.is_key_down(KEY_SLASH),
            _ => {}
        }
        match map_key(code) { Some(k) => self.rl.is_key_down(k), None => false }
    }

    /// Flankengetriggert: ist die Taste in DIESEM Frame neu gedrueckt worden?
    /// (raylib is_key_pressed). Fuer Caret-Bewegung u.ae., damit ein Tastendruck
    /// nicht jeden Frame ausloest.
    pub fn key_pressed(&self, code: i64) -> bool {
        match map_key(code) { Some(k) => self.rl.is_key_pressed(k), None => false }
    }
    /// Ist eine Shift-Taste gedrueckt? (fuer Text-Selektion via Shift+Pfeil)
    pub fn key_shift(&self) -> bool {
        use raylib::consts::KeyboardKey::*;
        self.rl.is_key_down(KEY_LEFT_SHIFT) || self.rl.is_key_down(KEY_RIGHT_SHIFT)
    }
    /// Ist eine Strg/Ctrl-Taste gedrueckt? (fuer Strg+A/C/V/X im Textfeld)
    pub fn key_ctrl(&self) -> bool {
        use raylib::consts::KeyboardKey::*;
        self.rl.is_key_down(KEY_LEFT_CONTROL) || self.rl.is_key_down(KEY_RIGHT_CONTROL)
    }

    // --- Gamepad (Modul input: INPUT_JOY_*) ---
    /// Anzahl zusammenhaengend verfuegbarer Pads ab Slot 0.
    pub fn joy_count(&self) -> i64 {
        let mut n = 0;
        while self.rl.is_gamepad_available(n) { n += 1; }
        n as i64
    }
    pub fn joy_name(&self, idx: i64) -> String {
        if idx < 0 || !self.rl.is_gamepad_available(idx as i32) { return String::new(); }
        self.rl.get_gamepad_name(idx as i32).unwrap_or_default()
    }
    /// Rohe Achsen-Bewegung (-1..+1) fuer Achs-Index 0..5 (Deadzone macht der Aufrufer).
    pub fn joy_axis(&self, pad: i64, axis_idx: i32) -> f64 {
        use raylib::consts::GamepadAxis::*;
        if pad < 0 || !self.rl.is_gamepad_available(pad as i32) { return 0.0; }
        let ax = match axis_idx {
            0 => GAMEPAD_AXIS_LEFT_X, 1 => GAMEPAD_AXIS_LEFT_Y,
            2 => GAMEPAD_AXIS_RIGHT_X, 3 => GAMEPAD_AXIS_RIGHT_Y,
            4 => GAMEPAD_AXIS_LEFT_TRIGGER, 5 => GAMEPAD_AXIS_RIGHT_TRIGGER,
            _ => return 0.0,
        };
        self.rl.get_gamepad_axis_movement(pad as i32, ax) as f64
    }
    /// Ist der durch den negativen Bind-Code bezeichnete Button/DPad an
    /// IRGENDEINEM verbundenen Pad gedrueckt? (wie Pythons _poll_joysticks_into)
    pub fn joy_button_down(&self, code: i64) -> bool {
        use raylib::consts::GamepadButton::*;
        let btn = match code {
            -100 => GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
            -101 => GAMEPAD_BUTTON_RIGHT_FACE_RIGHT,
            -102 => GAMEPAD_BUTTON_RIGHT_FACE_LEFT,
            -103 => GAMEPAD_BUTTON_RIGHT_FACE_UP,
            -104 => GAMEPAD_BUTTON_LEFT_TRIGGER_1,
            -105 => GAMEPAD_BUTTON_RIGHT_TRIGGER_1,
            -106 => GAMEPAD_BUTTON_MIDDLE_LEFT,
            -107 => GAMEPAD_BUTTON_MIDDLE_RIGHT,
            -108 => GAMEPAD_BUTTON_LEFT_THUMB,
            -109 => GAMEPAD_BUTTON_RIGHT_THUMB,
            -200 => GAMEPAD_BUTTON_LEFT_FACE_UP,
            -201 => GAMEPAD_BUTTON_LEFT_FACE_DOWN,
            -202 => GAMEPAD_BUTTON_LEFT_FACE_LEFT,
            -203 => GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
            _ => return false,
        };
        let mut i = 0;
        while self.rl.is_gamepad_available(i) {
            if self.rl.is_gamepad_button_down(i, btn) { return true; }
            i += 1;
        }
        false
    }
    // --- Core-Tastatur: INKEY$ / WAITKEY ---
    /// INKEY$: naechstes getipptes (druckbares) Zeichen oder leerer String.
    /// Non-blocking; liest aus raylibs Tipp-Queue (gefuellt beim letzten FLIP).
    pub fn inkey(&mut self) -> String {
        match self.rl.get_char_pressed() {
            Some(c) => c.to_string(),
            None => String::new(),
        }
    }
    /// WAITKEY: blockiert bis eine Taste gedrueckt wird, liefert den raylib-
    /// Keycode (INTEGER). -1 wenn das Fenster geschlossen wird. (Der Code-Wert
    /// folgt raylibs KeyboardKey-Enum, nicht den SDL-Codes der KEY_*-
    /// Konstanten -- historisch so gewachsen, dokumentiert.)
    pub fn waitkey(&mut self) -> i64 {
        loop {
            // window_should_close() ruft intern PollInputEvents -> fuellt die
            // Key-Queue, ohne dass ein FLIP noetig ist.
            if self.rl.window_should_close() { return -1; }
            if let Some(k) = self.rl.get_key_pressed() { return k as i64; }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    }

    // --- Core-Joystick (JOYSTICK_*): direkte raylib-Gamepad-Abfrage ---
    // Ein ungueltiger Joystick-INDEX wirft, ein ungueltiger Achsen-/Button-/
    // Hat-Unterindex liefert dagegen 0/false (kein Fehler).
    pub fn joystick_count(&self) -> i64 { self.joy_count() }
    fn joystick_check(&self, idx: i64, fn_: &str) -> Result<(), String> {
        if idx < 0 || !self.rl.is_gamepad_available(idx as i32) {
            let n = self.joy_count();
            return Err(if n > 0 {
                format!("{}: Joystick-Index {} ausserhalb [0..{}]", fn_, idx, n - 1)
            } else {
                format!("{}: Joystick-Index {} - kein Gamepad angeschlossen", fn_, idx)
            });
        }
        Ok(())
    }
    pub fn joystick_name(&self, idx: i64) -> Result<String, String> {
        self.joystick_check(idx, "JOYSTICK_NAME")?;
        Ok(self.joy_name(idx))
    }
    /// JOYSTICK_AXIS(idx, axis): rohe Achse (-1..+1), Achs-Index 0..5.
    pub fn joystick_axis(&self, idx: i64, axis: i64) -> Result<f64, String> {
        self.joystick_check(idx, "JOYSTICK_AXIS")?;
        Ok(self.joy_axis(idx, axis as i32))
    }
    /// JOYSTICK_BUTTON(idx, btn): btn folgt raylibs GamepadButton-Reihenfolge
    /// (0..17). Best-effort -- die Roh-Index-Zuordnung weicht von pygame ab;
    /// fuer praezise Bindings IMPORT "input" (JOY_BUTTON_*) nutzen.
    pub fn joystick_button(&self, idx: i64, btn: i64) -> Result<bool, String> {
        self.joystick_check(idx, "JOYSTICK_BUTTON")?;
        Ok(match Self::joy_btn_enum(btn) {
            Some(b) => self.rl.is_gamepad_button_down(idx as i32, b),
            None => false,
        })
    }
    /// Roh-Index (0..17) -> raylib-GamepadButton. Von JOYSTICK_BUTTON und den
    /// Flanken-Varianten JOYSTICK_HIT/RELEASED geteilt.
    fn joy_btn_enum(btn: i64) -> Option<raylib::consts::GamepadButton> {
        use raylib::consts::GamepadButton::*;
        Some(match btn {
            0 => GAMEPAD_BUTTON_UNKNOWN,
            1 => GAMEPAD_BUTTON_LEFT_FACE_UP,
            2 => GAMEPAD_BUTTON_LEFT_FACE_RIGHT,
            3 => GAMEPAD_BUTTON_LEFT_FACE_DOWN,
            4 => GAMEPAD_BUTTON_LEFT_FACE_LEFT,
            5 => GAMEPAD_BUTTON_RIGHT_FACE_UP,
            6 => GAMEPAD_BUTTON_RIGHT_FACE_RIGHT,
            7 => GAMEPAD_BUTTON_RIGHT_FACE_DOWN,
            8 => GAMEPAD_BUTTON_RIGHT_FACE_LEFT,
            9 => GAMEPAD_BUTTON_LEFT_TRIGGER_1,
            10 => GAMEPAD_BUTTON_LEFT_TRIGGER_2,
            11 => GAMEPAD_BUTTON_RIGHT_TRIGGER_1,
            12 => GAMEPAD_BUTTON_RIGHT_TRIGGER_2,
            13 => GAMEPAD_BUTTON_MIDDLE_LEFT,
            14 => GAMEPAD_BUTTON_MIDDLE,
            15 => GAMEPAD_BUTTON_MIDDLE_RIGHT,
            16 => GAMEPAD_BUTTON_LEFT_THUMB,
            17 => GAMEPAD_BUTTON_RIGHT_THUMB,
            _ => return None,
        })
    }
    /// JOYSTICK_HAT_X(idx, hat): nur hat 0 -- aus dem D-Pad abgeleitet
    /// (raylib hat keine Hats). +1 rechts, -1 links, 0 sonst.
    pub fn joystick_hat_x(&self, idx: i64, hat: i64) -> Result<i64, String> {
        use raylib::consts::GamepadButton::*;
        self.joystick_check(idx, "JOYSTICK_HAT_X")?;
        if hat != 0 { return Ok(0); }
        let r = self.rl.is_gamepad_button_down(idx as i32, GAMEPAD_BUTTON_LEFT_FACE_RIGHT);
        let l = self.rl.is_gamepad_button_down(idx as i32, GAMEPAD_BUTTON_LEFT_FACE_LEFT);
        Ok((r as i64) - (l as i64))
    }
    /// JOYSTICK_HAT_Y(idx, hat): +1 oben, -1 unten (D-Pad, hat 0).
    pub fn joystick_hat_y(&self, idx: i64, hat: i64) -> Result<i64, String> {
        use raylib::consts::GamepadButton::*;
        self.joystick_check(idx, "JOYSTICK_HAT_Y")?;
        if hat != 0 { return Ok(0); }
        let u = self.rl.is_gamepad_button_down(idx as i32, GAMEPAD_BUTTON_LEFT_FACE_UP);
        let d = self.rl.is_gamepad_button_down(idx as i32, GAMEPAD_BUTTON_LEFT_FACE_DOWN);
        Ok((u as i64) - (d as i64))
    }
    /// JOYSTICK_RUMBLE(idx, links, rechts, dauer_s): Vibrationsmotoren ansteuern
    /// (0.0..1.0 je Motor). NaN/Infinity/negative Werte werden wie bei
    /// SERIAL_SET_TIMEOUT geklemmt statt den zugrunde liegenden C-Aufruf mit
    /// Unsinn zu fuettern.
    pub fn joystick_rumble(&mut self, idx: i64, left: f64, right: f64, duration: f64) -> Result<(), String> {
        self.joystick_check(idx, "JOYSTICK_RUMBLE")?;
        let clamp01 = |v: f64| if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 };
        let dur = if duration.is_finite() { duration.clamp(0.0, 60.0) } else { 0.0 };
        self.rl.set_gamepad_vibration(idx as i32, clamp01(left) as f32, clamp01(right) as f32, dur as f32);
        Ok(())
    }

    /// Leert raylibs Tipp-Zeichen-Queue dieses Frames und liefert die getippten
    /// Zeichen als String (für UI_TEXTFIELD). Wie pygames Text-Input-Puffer.
    pub fn pop_text_input(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.rl.get_char_pressed() {
            s.push(c);
        }
        s
    }
    pub fn mouse_x(&self) -> i64 { (self.rl.get_mouse_x() / self.scale) as i64 }
    pub fn mouse_y(&self) -> i64 { (self.rl.get_mouse_y() / self.scale) as i64 }
    pub fn mouse_button(&self, b: i64) -> bool {
        match Self::mouse_btn(b) { Some(x) => self.rl.is_mouse_button_down(x), None => false }
    }

    // --- Eingabe-FLANKEN -----------------------------------------------------
    // `MOUSEBUTTON` und `KEYPRESSED` liefern beide "wird gehalten". Damit fehlte
    // im Kern ausgerechnet der haeufigste Fall: "genau in DIESEM Frame
    // gedrueckt". Dafuer musste man bisher das `input`-Modul mit Bindings +
    // INPUT_UPDATE bemuehen, obwohl raylib die Flanken direkt kennt.
    fn mouse_btn(b: i64) -> Option<MouseButton> {
        Some(match b {
            0 => MouseButton::MOUSE_BUTTON_LEFT,
            1 => MouseButton::MOUSE_BUTTON_RIGHT,
            2 => MouseButton::MOUSE_BUTTON_MIDDLE,
            3 => MouseButton::MOUSE_BUTTON_SIDE,
            4 => MouseButton::MOUSE_BUTTON_EXTRA,
            _ => return None,
        })
    }
    /// MOUSE_HIT(b): in DIESEM Frame gedrueckt worden?
    pub fn mouse_hit(&self, b: i64) -> bool {
        match Self::mouse_btn(b) { Some(x) => self.rl.is_mouse_button_pressed(x), None => false }
    }
    /// MOUSE_RELEASED(b): in DIESEM Frame losgelassen worden?
    pub fn mouse_released(&self, b: i64) -> bool {
        match Self::mouse_btn(b) { Some(x) => self.rl.is_mouse_button_released(x), None => false }
    }

    /// Wie `key_down`, aber mit frei waehlbarem raylib-Test -- inklusive der
    /// "+"/"-"-Sonderbehandlung (siehe key_down: pygame-Keysym vs. physische
    /// US-Position), damit die Flanken-Varianten dieselben Tasten treffen.
    fn key_test(&self, code: i64, f: impl Fn(raylib::consts::KeyboardKey) -> bool) -> bool {
        use raylib::consts::KeyboardKey::*;
        if code < 0 { return false; }              // Gamepad-Codes: siehe JOYSTICK_HIT
        match code {
            43 => return f(KEY_EQUAL) || f(KEY_KP_ADD) || f(KEY_RIGHT_BRACKET),
            45 => return f(KEY_MINUS) || f(KEY_KP_SUBTRACT) || f(KEY_SLASH),
            _ => {}
        }
        match map_key(code) { Some(k) => f(k), None => false }
    }
    /// KEYHIT(code): in DIESEM Frame gedrueckt (Blitz-BASIC-Sprech). `KEYPRESSED`
    /// bleibt "gehalten" -- der Name ist historisch, ihn umzudeuten wuerde
    /// bestehende Programme still kaputtmachen.
    pub fn key_hit(&self, code: i64) -> bool {
        self.key_test(code, |k| self.rl.is_key_pressed(k))
    }
    /// KEYRELEASED(code): in DIESEM Frame losgelassen.
    pub fn key_released_edge(&self, code: i64) -> bool {
        self.key_test(code, |k| self.rl.is_key_released(k))
    }
    /// KEYREPEAT(code): erster Druck ODER System-Auto-Repeat (Textcursor,
    /// Mengen-Eingabe) -- haelt man die Taste, feuert es wiederholt.
    pub fn key_repeat(&self, code: i64) -> bool {
        self.key_test(code, |k| self.rl.is_key_pressed(k) || self.rl.is_key_pressed_repeat(k))
    }

    /// Relative Mausbewegung seit dem letzten Frame -- Grundlage fuer
    /// Maus-Blick-Steuerung. Ohne sie war `MOUSE_LOCK` kaum zu gebrauchen: bei
    /// gefangenem Cursor stehen MOUSEX/MOUSEY still. Durch `scale` geteilt,
    /// damit die Einheit zu MOUSEX/MOUSEY passt.
    pub fn mouse_delta_x(&self) -> f64 { self.rl.get_mouse_delta().x as f64 / self.scale.max(1) as f64 }
    pub fn mouse_delta_y(&self) -> f64 { self.rl.get_mouse_delta().y as f64 / self.scale.max(1) as f64 }
    /// MOUSE_SET_POS(x, y): Zeiger setzen (z.B. Recentern bei Maus-Blick).
    pub fn mouse_set_pos(&mut self, x: i64, y: i64) {
        let s = self.scale.max(1) as f32;
        self.rl.set_mouse_position(Vector2::new(x as f32 * s, y as f32 * s));
    }
    /// MOUSE_ON_SCREEN(): liegt der Zeiger ueber dem Fenster?
    pub fn mouse_on_screen(&self) -> bool { self.rl.is_cursor_on_screen() }
    /// MOUSEWHEEL_X/Y(): Rad-Bewegung als KOMMAZAHL und in beiden Achsen.
    /// `MOUSEWHEEL` liefert nur die vertikale Achse als ganze Zahl -- ein
    /// horizontales Rad (viele Maeuse, jedes Touchpad) war damit unerreichbar,
    /// und feine Touchpad-Schritte (0.25) fielen auf 0 herunter.
    pub fn mouse_wheel_x(&self) -> f64 { self.rl.get_mouse_wheel_move_v().x as f64 }
    pub fn mouse_wheel_y(&self) -> f64 { self.rl.get_mouse_wheel_move_v().y as f64 }

    /// KEY_NAME$(code): Anzeigename einer Taste fuer Belegungsdialoge.
    /// raylib/GLFW kennt nur die Namen der DRUCKBAREN Tasten (und die
    /// layout-abhaengig: auf einer deutschen Tastatur heisst KEY_Y "z") --
    /// fuer Sondertasten liefert es nichts. Genau die will ein
    /// Belegungsdialog aber anzeigen, daher die eigene Ersatztabelle.
    ///
    /// Gemerkt wird das Ergebnis je Tastencode: ein Belegungsdialog fragt den
    /// Namen pro Bild ab, und die Antwort aendert sich nicht. Auf Plattformen
    /// ohne `GetKeyName` (Web) warnt raylib bei JEDEM Aufruf -- ungemerkt
    /// flutete das die Ausgabe und verdeckte alles andere.
    pub fn key_name(&mut self, code: i64) -> String {
        if let Some(n) = self.key_names.get(&code) { return n.clone(); }
        let Some(k) = map_key(code) else { return String::new(); };
        let name = match self.rl.get_key_name(k) {
            Some(n) if !n.trim().is_empty() => n.to_uppercase(),
            _ => key_label(k).to_string(),
        };
        self.key_names.insert(code, name.clone());
        name
    }
    // --- Eingabe aufzeichnen / abspielen (AUTOMATION_*) ---------------------
    // raylib zeichnet in `EndDrawing` den kompletten Eingabe-Zustand des Frames
    // auf (Tasten, Maus, Rad, Gamepad, Touch) und kann ihn spaeter wieder in
    // seinen Eingabe-Zustand einspeisen. Damit sind Demo-Modus ("Attract"),
    // Bug-Berichte zum Nachspielen und automatische Spieltests moeglich --
    // vorher hatte GB dafuer gar nichts.
    //
    // WICHTIG: raylib gibt die Wiedergabe NICHT selbst getaktet, das muss der
    // Aufrufer tun (`PlayAutomationEvent` je faelligem Ereignis). Genau das
    // macht `automation_tick()` am Ende jedes FLIP -- direkt NACH dem
    // Einlesen der echten Eingabe, damit die eingespeisten Werte den Frame
    // gewinnen, den das GB-Programm als naechstes liest.

    /// AUTOMATION_RECORD(datei$): Aufnahme starten. Eine laufende Wiedergabe
    /// wird beendet (raylib spielt waehrend einer Aufnahme ohnehin nichts ab).
    pub fn automation_record(&mut self, path: &str) -> Result<(), String> {
        if path.trim().is_empty() {
            return Err("AUTOMATION_RECORD: Dateiname fehlt".into());
        }
        self.auto_playing = false;
        let mut list = Box::new(self.rl.load_automation_event_list(None));
        self.rl.set_automation_event_list(&mut list);
        self.rl.start_automation_event_recording();
        self.auto_list = Some(list);
        self.auto_path = Some(path.to_string());
        self.auto_recording = true;
        Ok(())
    }
    /// AUTOMATION_STOP(): beendet, was gerade laeuft. Eine Aufnahme wird dabei
    /// in ihre Datei geschrieben; Rueckgabe = Anzahl der Ereignisse (bei
    /// gestoppter Wiedergabe 0).
    pub fn automation_stop(&mut self) -> Result<i64, String> {
        self.auto_playing = false;
        if !self.auto_recording { return Ok(0); }
        self.rl.stop_automation_event_recording();
        self.auto_recording = false;
        let path = self.auto_path.take().unwrap_or_default();
        let Some(list) = self.auto_list.as_ref() else { return Ok(0); };
        let count = list.count() as i64;
        if !list.export(&path) {
            return Err(format!("AUTOMATION_STOP: '{}' nicht schreibbar", path));
        }
        Ok(count)
    }
    /// AUTOMATION_PLAY(datei$): Aufnahme laden und ab dem naechsten Frame
    /// abspielen. Rueckgabe = Anzahl geladener Ereignisse.
    pub fn automation_play(&mut self, path: &str) -> Result<i64, String> {
        if self.auto_recording {
            return Err("AUTOMATION_PLAY: laeuft noch eine Aufnahme (erst AUTOMATION_STOP)".into());
        }
        if !std::path::Path::new(path).exists() {
            return Err(format!("AUTOMATION_PLAY: '{}' nicht gefunden", path));
        }
        // Hinweis: die alte Liste wird beim Zuweisen unten freigegeben. raylib
        // haelt zwar noch seinen internen Zeiger darauf, liest ihn aber NUR
        // waehrend einer laufenden Aufnahme -- und eine solche schliesst der
        // Wachtposten oben aus. Jede neue Aufnahme setzt den Zeiger neu.
        let list = Box::new(self.rl.load_automation_event_list(Some(path.into())));
        self.auto_events = list.events();
        // Aufsteigend nach Frame -- die Wiedergabe laeuft strikt vorwaerts.
        self.auto_events.sort_by_key(|e| e.frame());
        self.auto_play_base = self.auto_events.first().map(|e| e.frame()).unwrap_or(0);
        self.auto_play_idx = 0;
        self.auto_play_frame = 0;
        self.auto_playing = !self.auto_events.is_empty();
        self.auto_list = Some(list);
        Ok(self.auto_events.len() as i64)
    }
    pub fn automation_recording(&self) -> bool { self.auto_recording }
    pub fn automation_playing(&self) -> bool { self.auto_playing }
    /// AUTOMATION_FRAME(): Frame-Nummer innerhalb der Wiedergabe (0 = erster).
    pub fn automation_frame(&self) -> i64 { self.auto_play_frame as i64 }
    /// AUTOMATION_COUNT(): Ereignisse in der zuletzt geladenen/aufgenommenen Liste.
    pub fn automation_count(&self) -> i64 {
        if self.auto_recording {
            return self.auto_list.as_ref().map(|l| l.count() as i64).unwrap_or(0);
        }
        self.auto_events.len() as i64
    }
    /// Am Ende jedes FLIP: die fuer diesen Frame aufgezeichneten Ereignisse in
    /// raylibs Eingabe-Zustand einspeisen. Ereignisse, deren Frame uebersprungen
    /// wurde (Aufnahme mit anderer Bildrate), werden nachgeholt statt verworfen.
    fn automation_tick(&mut self) {
        self.auto_injected_keys.clear();
        if !self.auto_playing { return; }
        while self.auto_play_idx < self.auto_events.len() {
            let e = &self.auto_events[self.auto_play_idx];
            if e.frame().saturating_sub(self.auto_play_base) > self.auto_play_frame { break; }
            // Merken, welche TASTEN wir gleich einspeisen -- KEY_ANY_HIT soll nur
            // echte Eingabe melden (raylibs AutomationEventType-Nummern aus
            // rcore.c: 2 = INPUT_KEY_DOWN, 3 = INPUT_KEY_PRESSED). Gamepad-Knoepfe
            // brauchen das NICHT: deren Wiedergabe setzt nur `currentButtonState`,
            // nicht raylibs "zuletzt gedrueckter Knopf" -- JOYSTICK_ANY_BUTTON
            // sieht die Demo also ohnehin nicht.
            if matches!(e.get_type(), 2 | 3) {
                self.auto_injected_keys.push(e.params()[0]);
            }
            e.play();
            self.auto_play_idx += 1;
        }
        self.auto_play_frame += 1;
        if self.auto_play_idx >= self.auto_events.len() { self.auto_playing = false; }
    }

    /// KEY_ANY_HIT(): GB-Code der zuletzt gedrueckten Taste, -1 wenn keine.
    /// Das Gegenstueck zu JOYSTICK_ANY_BUTTON -- zusammen mit `KEY_NAME$` ist
    /// ein Belegungsdialog ("Druecke eine Taste ...") damit in drei Zeilen
    /// gebaut, statt alle Konstanten einzeln mit KEYHIT abzuklappern.
    pub fn key_any_hit(&mut self) -> i64 {
        // raylib fuehrt eine Warteschlange. Wir nehmen die erste Taste dieses
        // Frames, die GB ueberhaupt kennt, und leeren den Rest -- ein Dialog
        // will genau eine Belegung, keine Sammlung.
        let mut found = -1;
        while let Some(k) = self.rl.get_key_pressed() {
            // Von der laufenden Wiedergabe eingespeiste Tasten sind KEINE
            // Nutzereingabe -- sonst braeche ein Attract-Modus, der "bei
            // Tastendruck abbrechen" prueft, an seiner eigenen Demo ab.
            if self.auto_injected_keys.contains(&(k as i32)) { continue; }
            if found < 0 {
                if let Some(code) = gb_key_code(k) { found = code; }
            }
        }
        found
    }
    /// JOYSTICK_MAPPINGS(text$): SDL-GameControllerDB-Zeilen nachladen, damit
    /// auch exotische Pads die richtige Knopf-Belegung bekommen. Liefert
    /// raylibs Rueckgabe (Anzahl erkannter Zuordnungen, -1 bei Fehler).
    pub fn joystick_mappings(&self, text: &str) -> i64 {
        let Ok(cs) = std::ffi::CString::new(text) else { return -1; };
        let bytes: Vec<std::os::raw::c_char> =
            cs.as_bytes_with_nul().iter().map(|&b| b as std::os::raw::c_char).collect();
        self.rl.set_gamepad_mappings(&bytes) as i64
    }
    /// WINDOW_DPI_X/Y(): Skalierungsfaktor des Bildschirms (1.0 = normal,
    /// 2.0 = HiDPI/Retina). Ohne den weiss ein Programm nicht, ob seine
    /// Pixelgroessen auf dem Zielgeraet winzig herauskommen.
    pub fn window_dpi_x(&self) -> f64 { self.rl.get_window_scale_dpi().x as f64 }
    pub fn window_dpi_y(&self) -> f64 { self.rl.get_window_scale_dpi().y as f64 }

    /// MOUSE_CURSOR(form$): Systemcursor umschalten -- Hand ueber Knoepfen,
    /// Textmarke ueber Eingabefeldern, Groesse-Pfeile an Kanten.
    pub fn mouse_cursor(&mut self, name: &str) -> Result<(), String> {
        use raylib::consts::MouseCursor::*;
        let c = match name.to_ascii_lowercase().as_str() {
            "default" | "arrow" => MOUSE_CURSOR_DEFAULT,
            "ibeam" | "text" => MOUSE_CURSOR_IBEAM,
            "crosshair" | "cross" => MOUSE_CURSOR_CROSSHAIR,
            "hand" | "pointer" => MOUSE_CURSOR_POINTING_HAND,
            "resize_ew" => MOUSE_CURSOR_RESIZE_EW,
            "resize_ns" => MOUSE_CURSOR_RESIZE_NS,
            "resize_nwse" => MOUSE_CURSOR_RESIZE_NWSE,
            "resize_nesw" => MOUSE_CURSOR_RESIZE_NESW,
            "resize_all" | "move" => MOUSE_CURSOR_RESIZE_ALL,
            "not_allowed" | "no" => MOUSE_CURSOR_NOT_ALLOWED,
            other => return Err(format!(
                "MOUSE_CURSOR: unbekannte Form '{}' -- erwartet default/ibeam/crosshair/\
hand/resize_ew/resize_ns/resize_nwse/resize_nesw/resize_all/not_allowed", other)),
        };
        self.rl.set_mouse_cursor(c);
        Ok(())
    }

    /// JOYSTICK_HIT / JOYSTICK_RELEASED: Flanken analog zu JOYSTICK_BUTTON.
    pub fn joystick_hit(&self, idx: i64, btn: i64) -> Result<bool, String> {
        self.joystick_check(idx, "JOYSTICK_HIT")?;
        Ok(match Self::joy_btn_enum(btn) {
            Some(b) => self.rl.is_gamepad_button_pressed(idx as i32, b), None => false })
    }
    pub fn joystick_released(&self, idx: i64, btn: i64) -> Result<bool, String> {
        self.joystick_check(idx, "JOYSTICK_RELEASED")?;
        Ok(match Self::joy_btn_enum(btn) {
            Some(b) => self.rl.is_gamepad_button_released(idx as i32, b), None => false })
    }
    /// JOYSTICK_ANY_BUTTON(): zuletzt gedrueckter Gamepad-Knopf, -1 = keiner.
    /// Fuer "Druecke einen Knopf"-Belegungsdialoge.
    pub fn joystick_any_button(&self) -> i64 {
        // raylib-rs liefert bereits `None`, wenn nichts anliegt (es filtert
        // GAMEPAD_BUTTON_UNKNOWN heraus) -> -1 durchreichen.
        self.rl.get_gamepad_button_pressed().map(|b| b as i64).unwrap_or(-1)
    }
    /// JOYSTICK_AXIS_COUNT(idx): wie viele Achsen hat das Pad wirklich?
    pub fn joystick_axis_count(&self, idx: i64) -> Result<i64, String> {
        self.joystick_check(idx, "JOYSTICK_AXIS_COUNT")?;
        Ok(self.rl.get_gamepad_axis_count(idx as i32) as i64)
    }

    // --- Touch + Gesten ------------------------------------------------------
    // Bisher komplett ungenutzt (0 von 12 raylib-Funktionen). Auf einem
    // Touchscreen meldet raylib den ersten Finger zwar zusaetzlich als Maus,
    // aber Multitouch, Wisch und Pinch waren gar nicht erreichbar.
    pub fn touch_count(&self) -> i64 { self.rl.get_touch_point_count() as i64 }
    pub fn touch_x(&self, i: i64) -> f64 {
        self.rl.get_touch_position(i.max(0) as u32).x as f64 / self.scale.max(1) as f64
    }
    pub fn touch_y(&self, i: i64) -> f64 {
        self.rl.get_touch_position(i.max(0) as u32).y as f64 / self.scale.max(1) as f64
    }
    /// TOUCH_ID(i): stabile Finger-Kennung -- damit laesst sich ein Finger ueber
    /// Frames hinweg verfolgen, auch wenn ein anderer dazwischen losgelassen wird.
    pub fn touch_id(&self, i: i64) -> i64 { self.rl.get_touch_point_id(i.max(0) as u32) as i64 }

    /// GESTURE$(): erkannte Geste dieses Frames als Name ("" = keine).
    /// Namen statt Zahlen, damit BASIC-Code lesbar bleibt.
    pub fn gesture(&self) -> String {
        use raylib::consts::Gesture::*;
        let g = self.rl.get_gesture_detected();
        match g {
            GESTURE_TAP => "tap", GESTURE_DOUBLETAP => "doubletap", GESTURE_HOLD => "hold",
            GESTURE_DRAG => "drag", GESTURE_SWIPE_RIGHT => "swipe_right",
            GESTURE_SWIPE_LEFT => "swipe_left", GESTURE_SWIPE_UP => "swipe_up",
            GESTURE_SWIPE_DOWN => "swipe_down", GESTURE_PINCH_IN => "pinch_in",
            GESTURE_PINCH_OUT => "pinch_out", _ => "",
        }.to_string()
    }
    pub fn gesture_drag_x(&self) -> f64 { self.rl.get_gesture_drag_vector().x as f64 }
    pub fn gesture_drag_y(&self) -> f64 { self.rl.get_gesture_drag_vector().y as f64 }
    pub fn gesture_drag_angle(&self) -> f64 { self.rl.get_gesture_drag_angle() as f64 }
    pub fn gesture_pinch_x(&self) -> f64 { self.rl.get_gesture_pinch_vector().x as f64 }
    pub fn gesture_pinch_y(&self) -> f64 { self.rl.get_gesture_pinch_vector().y as f64 }
    pub fn gesture_pinch_angle(&self) -> f64 { self.rl.get_gesture_pinch_angle() as f64 }
    /// GESTURE_HOLD_TIME(): wie lange wird schon gehalten (Sekunden)?
    pub fn gesture_hold_time(&self) -> f64 { self.rl.get_gesture_hold_duration() as f64 }

    /// Mausrad-Delta dieses Frames (raylib liefert es pro Frame; "pop" =
    /// einmal lesen). Positiv = nach oben/vorn.
    pub fn pop_mouse_wheel(&self) -> i64 { self.rl.get_mouse_wheel_move() as i64 }

    /// Logische Fenster-Breite/Hoehe (wie an SCREEN uebergeben).
    // Live-Fenstergroesse (logisch, d.h. ohne Scale) -- spiegelt eine evtl. vom
    // Nutzer geaenderte Groesse bei resizeable Fenstern wider. Bei nicht-
    // resizeable Fenstern == konfigurierte Groesse (kein Verhaltensbruch).
    pub fn screen_width(&self) -> i64 { (self.rl.get_screen_width() / self.scale.max(1)) as i64 }
    pub fn screen_height(&self) -> i64 { (self.rl.get_screen_height() / self.scale.max(1)) as i64 }

    // --- Game-Loop-Grundlagen ---
    // Headless (DHRT_FRAMES gesetzt): fester Schritt 1/60 s -> zeitbasierte
    // Spiele laufen deterministisch und sind per Screenshot/Frame testbar
    // (echtes get_frame_time ist ohne Vsync ~0). Sonst echte Frame-Zeit.
    pub fn delta(&self) -> f64 {
        if self.max_frames.is_some() { 1.0 / 60.0 } else { self.rl.get_frame_time() as f64 }
    }
    pub fn fps(&self) -> i64 { self.rl.get_fps() as i64 }
    pub fn set_target_fps(&mut self, n: i64) { self.rl.set_target_fps(n.max(0) as u32); }
    pub fn set_window_title(&mut self, title: &str) { self.rl.set_window_title(&self.thread, title); }
    pub fn save_screenshot(&mut self, path: &str) { self.write_screenshot(path); }

    /// Schreibt einen Screenshot robust unter `path` (relativ = zum cwd).
    /// raylibs `take_screenshot` stellt intern `CORE.Storage.basePath` voran und
    /// kann daher weder absolute Pfade noch (nach canonicalize) `\\?\`-cwd-Pfade
    /// korrekt schreiben. Stattdessen lesen wir die Screen-Pixel selbst und
    /// exportieren sie via `ExportImage` (das KEINEN Prefix voranstellt) unter
    /// einem absoluten, vom `\\?\`-Prefix bereinigten Pfad.
    /// Ein Bild fuer den Kontaktbogen aufnehmen (verkleinert, damit das
    /// Raster nicht ins Uferlose waechst).
    fn contact_capture(&mut self) {
        if self.contact_shots.len() >= self.contact_max { return; }
        let mut img = self.rl.load_image_from_screen(&self.thread);
        // Auf hoechstens 480 Pixel Breite herunterrechnen -- lesbar genug, um
        // Bewegung und Ablauf zu beurteilen, und ein Raster aus zwoelf solchen
        // Bildern bleibt eine handliche Datei.
        let bw = 480i32.min(img.width);
        if img.width > bw {
            let bh = (img.height as f32 * bw as f32 / img.width as f32).round() as i32;
            img.resize(bw, bh.max(1));
        }
        let nr = self.frame_count;
        self.contact_shots.push((nr, img));
    }

    /// Die gesammelten Bilder als beschriftetes Raster in EINE PNG schreiben.
    fn contact_write(&mut self) {
        if self.contact_written || self.contact_shots.is_empty() { return; }
        self.contact_written = true;
        let Some(path) = self.contact_path.clone() else { return };

        const RAND: i32 = 8;          // Abstand zwischen den Kacheln
        const BESCHRIFTUNG: i32 = 18; // Platz fuer die Bildnummer
        let (tw, th) = {
            let f = &self.contact_shots[0].1;
            (f.width, f.height)
        };
        let spalten = self.contact_cols.min(self.contact_shots.len()).max(1) as i32;
        let zeilen = ((self.contact_shots.len() as i32) + spalten - 1) / spalten;
        let breite = spalten * tw + (spalten + 1) * RAND;
        let hoehe = zeilen * (th + BESCHRIFTUNG) + (zeilen + 1) * RAND;

        let mut blatt = Image::gen_image_color(breite, hoehe, Color::new(18, 20, 26, 255));
        for (i, (nr, bild)) in self.contact_shots.iter().enumerate() {
            let sp = (i as i32) % spalten;
            let ze = (i as i32) / spalten;
            let x = RAND + sp * (tw + RAND);
            let y = RAND + ze * (th + BESCHRIFTUNG + RAND);
            blatt.draw(bild,
                       Rectangle::new(0.0, 0.0, bild.width as f32, bild.height as f32),
                       Rectangle::new(x as f32, y as f32, tw as f32, th as f32),
                       Color::WHITE);
            blatt.draw_text(&format!("Bild {}", nr), x + 2, y + th + 3, 14,
                            Color::new(150, 170, 200, 255));
        }

        let p = std::path::Path::new(&path);
        let abs = if p.is_absolute() { p.to_path_buf() }
                  else { std::env::current_dir().map(|d| d.join(p)).unwrap_or_else(|_| p.to_path_buf()) };
        let abs = crate::strip_extended_prefix(abs);
        blatt.export_image(&abs.to_string_lossy());
    }

    // --- GFX_PUSH / GFX_POP ------------------------------------------------
    /// Zeichenzustand auf den Stapel legen. Was genau dazugehoert, steht bei
    /// `GfxState`.
    pub fn gfx_push(&mut self) {
        // Obergrenze, damit eine PUSH-Schleife ohne POP nicht den Speicher
        // auffrisst -- ein Programmierfehler soll auffallen, nicht wachsen.
        if self.gfx_stack.len() >= 64 { return; }
        let st = GfxState {
            cam: (self.cam_x, self.cam_y, self.cam_zoom, self.cam_rotation),
            shake: (self.shake_amp, self.shake_dur_ms, self.shake_start, self.shake_x, self.shake_y),
            active_layer: self.active,
            clear_color: self.clear_color,
            light_ambient: self.light_ambient,
            light_fog: self.light_fog,
            light_fog_density: self.light_fog_density,
            lights: self.lights.iter()
                .map(|l| (l.enabled, l.kind, l.pos, l.target, l.color)).collect(),
            env: (self.env_sky, self.env_ground, self.env_intensity),
            use_ibl_maps: self.use_ibl_maps,
            skybox_enabled: self.skybox_enabled,
            shadow: (self.shadow_enabled, self.shadow_area, self.shadow_dist, self.shadow_target),
            cam3d: self.cam3d,
            cam3d_view: self.cam3d_view,
            cam3d_proj: self.cam3d_proj,
            text: (self.text_size, self.active_font, self.text_spacing),
            post_shader_idx: self.post_shader_idx,
        };
        self.gfx_stack.push(st);
    }

    /// Zustand vom Stapel zurueckholen. Liefert `false`, wenn der Stapel leer
    /// ist -- daraus macht die VM eine klare Meldung statt eines stillen No-Ops.
    pub fn gfx_pop(&mut self) -> bool {
        let Some(st) = self.gfx_stack.pop() else { return false };
        let (cx, cy, cz, cr) = st.cam;
        self.cam_x = cx; self.cam_y = cy; self.cam_zoom = cz; self.cam_rotation = cr;
        let (sa, sd, ss, sx, sy) = st.shake;
        self.shake_amp = sa; self.shake_dur_ms = sd; self.shake_start = ss;
        self.shake_x = sx; self.shake_y = sy;
        // Layer-Index nur uebernehmen, wenn es ihn noch gibt (eine Szene darf
        // waehrenddessen neue Layer angelegt haben, entfernt werden sie nie).
        if st.active_layer < self.layers.len() { self.active = st.active_layer; }
        self.clear_color = st.clear_color;
        self.light_ambient = st.light_ambient;
        self.light_fog = st.light_fog;
        self.light_fog_density = st.light_fog_density;
        // Lichter: gespeicherte Werte zurueckschreiben, spaeter hinzugekommene
        // ABSCHALTEN -- sonst leuchtet ein Licht aus der Szene weiter.
        for (i, l) in self.lights.iter_mut().enumerate() {
            match st.lights.get(i) {
                Some(&(en, kind, pos, target, color)) => {
                    l.enabled = en; l.kind = kind; l.pos = pos; l.target = target; l.color = color;
                }
                None => l.enabled = false,
            }
        }
        let (sky, ground, int) = st.env;
        self.env_sky = sky; self.env_ground = ground; self.env_intensity = int;
        self.use_ibl_maps = st.use_ibl_maps;
        self.skybox_enabled = st.skybox_enabled;
        let (sh_on, sh_area, sh_dist, sh_target) = st.shadow;
        self.shadow_enabled = sh_on; self.shadow_area = sh_area;
        self.shadow_dist = sh_dist; self.shadow_target = sh_target;
        self.cam3d = st.cam3d;
        self.cam3d_view = st.cam3d_view;
        self.cam3d_proj = st.cam3d_proj;
        let (ts, af, tsp) = st.text;
        self.text_size = ts; self.text_spacing = tsp;
        // Schrift nur setzen, wenn sie noch existiert (Fonts werden nie
        // entladen, aber ein Handle aus einem anderen Lauf waere ungueltig).
        if af < 0 || (af as usize) < self.fonts.len() { self.active_font = af; }
        // Post-Effekt zurueck. Ohne diese Zeile bliebe der Shader der
        // verlassenen Szene ueber allem Folgenden liegen -- genau der Fehler,
        // gegen den PUSH/POP gebaut ist.
        self.post_shader_idx = match st.post_shader_idx {
            Some(i) if i < self.shaders.len() => Some(i),
            Some(_) => None,   // Shader gibt es nicht mehr -> lieber ohne
            None => None,
        };
        true
    }

    /// Wie tief ist der Stapel? (Zum Pruefen im Test und fuer Fehlersuche.)
    pub fn gfx_depth(&self) -> i64 { self.gfx_stack.len() as i64 }

    fn write_screenshot(&mut self, path: &str) {
        let p = std::path::Path::new(path);
        let abs = if p.is_absolute() {
            p.to_path_buf()
        } else {
            std::env::current_dir().map(|d| d.join(p)).unwrap_or_else(|_| p.to_path_buf())
        };
        let abs = crate::strip_extended_prefix(abs);
        let img = self.rl.load_image_from_screen(&self.thread);
        img.export_image(&abs.to_string_lossy());
    }
    /// SET_FULLSCREEN(an) -- Vollbild an/aus, OHNE raylibs `toggle_fullscreen()`
    /// (exklusiver Video-Mode-Wechsel): der ist auf manchen Setups unzuverlaessig
    /// (siehe screen_native()-Kommentar -- GLFW "failed to query video mode") und
    /// aendert dabei NICHT die tatsaechliche Fenstergroesse auf die native
    /// Monitor-Aufloesung. Ergebnis war: das logische SCREEN(w,h)-Bild blieb in
    /// Fenstergroesse in der Bildschirmecke stehen, der Rest des Monitors blieb
    /// schwarz/leer statt das Bild auszufuellen.
    ///
    /// Stattdessen: groesstmoegliche GANZZAHLIGE Pixel-Skalierung waehlen, die
    /// noch in den Monitor passt (scharf, kein Weichzeichnen) -- randlos machen
    /// via `toggle_borderless_windowed` (erwiesenermassen zuverlaessig, anders
    /// als toggle_fullscreen). ACHTUNG: toggle_borderless_windowed zwingt das
    /// Fenster auf die VOLLE Monitor-Groesse, unabhaengig von einer vorher per
    /// `set_window_size` gesetzten kleineren Groesse -- bei einem Seitenverhaeltnis-
    /// Mismatch (Monitor 16:9, SCREEN(...) z.B. 4:3) bleibt darum auf einer Achse
    /// mehr Platz als `width*k`. Um SCREENWIDTH()/HEIGHT() (== live Fenstergroesse
    /// / scale) dabei NICHT zu verfaelschen, wird die tatsaechliche Fenstergroesse
    /// NACH dem Toggle abgefragt und `width`/`height` darauf angepasst -- der
    /// zusaetzliche Platz wird so zu benutzbarer logischer Zeichenflaeche statt
    /// eines falschen SCREENWIDTH()-Werts. Bei exakt passendem Seitenverhaeltnis
    /// (wie in der Praxis meist: 1920x1080/2560x1440-Monitor + 16:9-SCREEN) aendert
    /// sich dadurch nichts.
    pub fn set_fullscreen(&mut self, fs: bool) {
        if fs == self.fullscreen { return; }
        if fs {
            self.pre_fullscreen = Some((self.width, self.height, self.scale));
            let m = get_current_monitor();
            let mw = get_monitor_width(m).max(1);
            let mh = get_monitor_height(m).max(1);
            let k = (mw / self.width.max(1)).min(mh / self.height.max(1)).max(1);
            self.resize_keep_title(self.width, self.height, k);
            self.rl.toggle_borderless_windowed();
            let aw = self.rl.get_screen_width().max(k);
            let ah = self.rl.get_screen_height().max(k);
            self.width = aw / k;
            self.height = ah / k;
            self.scene_rt = self.rl.load_render_texture(&self.thread, aw as u32, ah as u32).ok();
        } else {
            self.rl.toggle_borderless_windowed();
            if let Some((w, h, sc)) = self.pre_fullscreen.take() {
                self.resize_keep_title(w, h, sc);
            }
        }
        self.fullscreen = fs;
    }

    /// Fenster + Szene-Render-Target auf `width*scale x height*scale` bringen,
    /// ohne (anders als reconfigure_raw) den Fenstertitel anzufassen -- fuer
    /// SET_FULLSCREEN-Toggles, die den vom User gesetzten Titel nicht
    /// ueberschreiben duerfen.
    fn resize_keep_title(&mut self, width: i32, height: i32, scale: i32) {
        self.width = width;
        self.height = height;
        self.scale = scale;
        let win_w = width * scale;
        let win_h = height * scale;
        self.rl.set_window_size(win_w, win_h);
        self.scene_rt = self.rl.load_render_texture(&self.thread, win_w as u32, win_h as u32).ok();
    }

    /// WINDOW_ESC_QUIT(an) -- ESC als Fenster-Schliessen-Taste an/aus.
    /// raylib-Default: ESC schliesst das Fenster (QUITREQUESTED wird true).
    /// Mit `an=false` ist ESC eine GANZ NORMALE Taste -> nur noch das Fenster-X
    /// bzw. Alt+F4 beenden. Fuer Spiele, die ESC fuers Pause-/Hauptmenue nutzen.
    pub fn set_esc_quit(&mut self, on: bool) {
        use raylib::consts::KeyboardKey::*;
        self.rl.set_exit_key(if on { Some(KEY_ESCAPE) } else { None });
    }

    /// MOUSE_VISIBLE(an) -- OS-Maus-Cursor zeigen/verstecken (Spiele mit
    /// eigenem Fadenkreuz-/Cursor-Sprite verstecken ihn und zeichnen selbst).
    pub fn mouse_visible(&mut self, v: bool) {
        if v { self.rl.show_cursor(); } else { self.rl.hide_cursor(); }
    }
    /// MOUSE_LOCK(an) -- Cursor fangen: verstecken + im Fenster einsperren
    /// (raylib DisableCursor, relative Bewegung). Fuer First-Person-/
    /// Kamera-Maussteuerung; MOUSEX/Y laufen weiter mit. MOUSE_LOCK(FALSE)
    /// gibt den Cursor frei und zeigt ihn wieder.
    pub fn mouse_lock(&mut self, locked: bool) {
        if locked { self.rl.disable_cursor(); } else { self.rl.enable_cursor(); }
    }
    /// MOUSE_HIDDEN() -- ist der Cursor gerade versteckt/gefangen?
    pub fn mouse_hidden(&self) -> bool { self.rl.is_cursor_hidden() }

    // --- Natives OS-Fenster (das SCREEN-Fenster selbst) ---
    /// Das Programmfenster vom OS aus groessenveraenderbar machen (Default: aus).
    pub fn window_resizable(&mut self, f: bool) {
        let ws = WindowState::default().set_window_resizable(true);
        if f { self.rl.set_window_state(ws); } else { self.rl.clear_window_state(ws); }
    }
    /// Fensterrahmen/Titelleiste aus- oder einblenden (randloses Overlay).
    pub fn window_undecorated(&mut self, f: bool) {
        let ws = WindowState::default().set_window_undecorated(true);
        if f { self.rl.set_window_state(ws); } else { self.rl.clear_window_state(ws); }
    }
    /// Fenster immer im Vordergrund halten (Topmost) -- fuer Desktop-Overlays.
    pub fn window_topmost(&mut self, f: bool) {
        let ws = WindowState::default().set_window_topmost(true);
        if f { self.rl.set_window_state(ws); } else { self.rl.clear_window_state(ws); }
    }
    /// Maus-Klicks durch das Fenster zum Desktop DURCHREICHEN
    /// (FLAG_WINDOW_MOUSE_PASSTHROUGH) -- fuer klick-durchlaessige Overlays/Widgets.
    /// Braucht ein randloses Fenster (WINDOW_UNDECORATED). raylib-rs bietet dafuer
    /// keinen WindowState-Setter, darum direkt per FFI. Hinweis: die Tastatur (ESC)
    /// erreicht das Fenster nur, solange es den Fokus hat -- nach einem Desktop-Klick
    /// zum Beenden Alt+F4.
    pub fn window_passthrough(&mut self, f: bool) {
        let flag = raylib::ffi::ConfigFlags::FLAG_WINDOW_MOUSE_PASSTHROUGH as u32;
        unsafe {
            if f { raylib::ffi::SetWindowState(flag); } else { raylib::ffi::ClearWindowState(flag); }
        }
    }
    pub fn window_min_size(&mut self, w: i32, h: i32) {
        self.rl.set_window_min_size(w * self.scale.max(1), h * self.scale.max(1));
    }
    pub fn window_max_size(&mut self, w: i32, h: i32) {
        self.rl.set_window_max_size(w * self.scale.max(1), h * self.scale.max(1));
    }
    pub fn window_maximize(&mut self) { self.rl.maximize_window(); }
    pub fn window_minimize(&mut self) { self.rl.minimize_window(); }
    pub fn window_restore(&mut self) { self.rl.restore_window(); }
    /// Wurde das Fenster seit dem letzten FLIP vom Nutzer/OS in der Groesse geaendert?
    pub fn window_resized(&self) -> bool { self.rl.is_window_resized() }

    // --- Fenster-Zustand + Politur ------------------------------------------
    /// WINDOW_FOCUSED(): hat das Fenster den Tastaturfokus? Damit laesst sich
    /// ein Spiel pausieren, sobald der Nutzer wegklickt.
    pub fn window_focused(&self) -> bool { self.rl.is_window_focused() }
    pub fn window_minimized(&self) -> bool { self.rl.is_window_minimized() }
    pub fn window_maximized(&self) -> bool { self.rl.is_window_maximized() }
    pub fn window_hidden(&self) -> bool { self.rl.is_window_hidden() }
    pub fn window_fullscreen_state(&self) -> bool { self.rl.is_window_fullscreen() }
    /// WINDOW_FOCUS(): Fenster nach vorne holen.
    pub fn window_focus(&mut self) { self.rl.set_window_focused(); }
    /// WINDOW_OPACITY(0..1): ganzes Fenster durchscheinend (Overlays, Fade-ins).
    pub fn window_opacity(&mut self, v: f64) {
        let o = if v.is_finite() { v.clamp(0.0, 1.0) } else { 1.0 };
        self.rl.set_window_opacity(o as f32);
    }
    /// WINDOW_ICON(bild): Fenster-/Taskleisten-Symbol setzen. Ohne das trug
    /// JEDES exportierte Spiel das raylib-Standardsymbol.
    pub fn window_icon(&mut self, img: i64) -> Result<(), String> {
        let t = self.textures.get(img.max(0) as usize)
            .ok_or_else(|| self.tex_fehler(img, "WINDOW_ICON"))?;
        // raylib verlangt RGBA8 fuers Icon und ignoriert andere Formate still
        // -> auf einer Kopie konvertieren, das Original bleibt unangetastet.
        let mut copy = t.img.clone();
        copy.set_format(raylib::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        self.rl.set_window_icon(&copy);
        Ok(())
    }
    /// GET_TIME(): Sekunden seit Programmstart (monoton, unabhaengig von DELTA).
    pub fn get_time(&self) -> f64 { self.rl.get_time() }

    /// OPENURL(adresse$): Adresse im Standardbrowser oeffnen (Itch-Seite,
    /// Anleitung, Mitmach-Link aus dem Spiel heraus).
    ///
    /// Absichtlich auf http/https begrenzt: raylibs OpenURL reicht die
    /// Zeichenkette an die Shell weiter, und ein `file:`- oder gar
    /// Programm-Schema waere damit ein Weg, aus einem harmlos wirkenden
    /// GB-Programm heraus Beliebiges zu starten.
    pub fn open_url(&self, url: &str) -> Result<(), String> {
        let low = url.trim().to_ascii_lowercase();
        if !(low.starts_with("http://") || low.starts_with("https://")) {
            return Err(format!(
                "OPENURL: nur http:// und https:// erlaubt (bekam '{}')", url));
        }
        raylib::misc::open_url(url.trim());
        Ok(())
    }

    // --- Monitore / Display-Infos (raylib GetMonitor*) ---
    // Alle Monitor-Masse sind ECHTE OS-Pixel (kein Screen-Scale), denn sie
    // beschreiben die Hardware, nicht das logische SCREEN-Raster. Monitor-Index
    // 0..MONITOR_COUNT()-1; raylibs Get-Funktionen klemmen ungueltige Indizes
    // intern selbst ab und liefern dann 0 bzw. "".
    /// Anzahl angeschlossener Monitore.
    pub fn monitor_count(&self) -> i64 { get_monitor_count() as i64 }
    /// Index des Monitors, auf dem das Fenster gerade ueberwiegend liegt.
    pub fn current_monitor(&self) -> i64 { get_current_monitor() as i64 }
    /// Native Breite des Monitors `i` in Pixeln.
    pub fn monitor_width(&self, i: i64) -> i64 { get_monitor_width(i as i32) as i64 }
    /// Native Hoehe des Monitors `i` in Pixeln.
    pub fn monitor_height(&self, i: i64) -> i64 { get_monitor_height(i as i32) as i64 }
    /// Bildwiederholrate (Hz) des Monitors `i`.
    pub fn monitor_refresh(&self, i: i64) -> i64 { get_monitor_refresh_rate(i as i32) as i64 }
    /// Anzeigename des Monitors `i` (leer, wenn nicht ermittelbar).
    /// NICHT raylib-rs' `get_monitor_name` benutzen: das ruft `CString::from_raw`
    /// auf den von `GetMonitorName` gelieferten Zeiger und GIBT IHN BEIM DROP FREI
    /// -- der Speicher gehoert aber GLFW, nicht Rust -> Heap-Korruption (0xC0000374).
    /// Wir leihen den Zeiger nur aus (CStr, kein free) und kopieren in einen String.
    pub fn monitor_name(&self, i: i64) -> String {
        unsafe {
            let p = raylib::ffi::GetMonitorName(i as i32);
            if p.is_null() { return String::new(); }
            std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }
    /// X-Position des Monitors `i` im virtuellen Desktop (OS-Pixel).
    pub fn monitor_x(&self, i: i64) -> i64 { get_monitor_position(i as i32).x as i64 }
    /// Y-Position des Monitors `i` im virtuellen Desktop (OS-Pixel).
    pub fn monitor_y(&self, i: i64) -> i64 { get_monitor_position(i as i32).y as i64 }
    /// Fenster auf Monitor `i` schieben (raylibs Vollbild-Zielmonitor). Out-of-range
    /// wird ignoriert -- raylibs set_window_monitor hat ein debug_assert, das in
    /// Debug-Builds sonst paniken wuerde.
    pub fn set_window_monitor(&mut self, i: i64) {
        if i >= 0 && i < get_monitor_count() as i64 { self.rl.set_window_monitor(i as i32); }
    }
    /// X-Position der linken oberen Fensterecke (OS-Pixel).
    pub fn window_x(&self) -> i64 { self.rl.get_window_position().x as i64 }
    /// Y-Position der linken oberen Fensterecke (OS-Pixel).
    pub fn window_y(&self) -> i64 { self.rl.get_window_position().y as i64 }
    /// Fenster an OS-Pixelposition (x, y) setzen.
    pub fn set_window_pos(&mut self, x: i64, y: i64) { self.rl.set_window_position(x as i32, y as i32); }

    /// Clip-Rechteck auf den Stack legen (Scissor). Koordinaten werden wie bei
    /// allen Draws kamera-transformiert; der Screen-Scale kommt beim Replay.
    pub fn push_clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let (x, y) = self.w2s(x, y);
        let (w, h) = (self.ssize(w), self.ssize(h));
        self.clip_tiefe += 1;
        self.emit(Cmd::ScissorPush(x, y, w, h));
    }
    /// `false`, wenn gar kein Clip offen ist -- der Aufrufer meldet das.
    /// Ohne diese Pruefung naehme ein zu viel gesetztes SCISSOR_END dem
    /// umgebenden Code (z.B. einem `gui`-Fenster) seinen Clip weg, und der
    /// Fehler zeigte sich erst als Zeichnen ueber den Rand hinaus.
    pub fn pop_clip(&mut self) -> bool {
        if self.clip_tiefe == 0 { return false; }
        self.clip_tiefe -= 1;
        self.emit(Cmd::ScissorPop);
        true
    }
    /// Wie viele Clips gerade offen sind (fuer SCISSOR_DEPTH).
    pub fn clip_depth(&self) -> i64 { self.clip_tiefe as i64 }

    // --- Shader / Post-Processing ---
    /// Laedt einen Fragment-Shader (GLSL-Quelltext) -> Handle (Index) oder -1.
    pub fn load_shader(&mut self, code: &str) -> i64 {
        let sh = self.rl.load_shader_from_memory(&self.thread, None,
                                        Some(&fuer_ziel_uebersetzen(code)));
        if !sh.is_shader_valid() { return -1; }
        self.shaders.push(sh);
        (self.shaders.len() - 1) as i64
    }
    pub fn shader_set_float(&mut self, h: i64, name: &str, v: f64) {
        if let Some(sh) = self.shaders.get_mut(h as usize) {
            let loc = sh.get_shader_location(name);
            if loc >= 0 { sh.set_shader_value(loc, v as f32); }
        }
    }
    pub fn shader_set_vec2(&mut self, h: i64, name: &str, x: f64, y: f64) {
        if let Some(sh) = self.shaders.get_mut(h as usize) {
            let loc = sh.get_shader_location(name);
            if loc >= 0 { sh.set_shader_value(loc, [x as f32, y as f32]); }
        }
    }
    pub fn shader_set_vec3(&mut self, h: i64, name: &str, x: f64, y: f64, z: f64) {
        if let Some(sh) = self.shaders.get_mut(h as usize) {
            let loc = sh.get_shader_location(name);
            if loc >= 0 { sh.set_shader_value(loc, [x as f32, y as f32, z as f32]); }
        }
    }
    /// SHADER_SET_ARRAY(shader, name$, werte): `uniform float[]` fuellen.
    /// Bisher liess sich pro Aufruf nur EIN Wert setzen -- damit waren
    /// Effekte, die eine Liste brauchen (Lichtpositionen, Farbverlaufs-Stufen,
    /// Wellen-Parameter), gar nicht anzusteuern.
    pub fn shader_set_array(&mut self, h: i64, name: &str, vals: &[f64]) -> Result<(), String> {
        if vals.is_empty() {
            return Err("SHADER_SET_ARRAY: Werte-Array ist leer".into());
        }
        let sh = self.shaders.get_mut(h as usize)
            .ok_or("SHADER_SET_ARRAY: ungueltiges SHADER-Handle")?;
        let loc = sh.get_shader_location(name);
        if loc < 0 {
            // Kein Fehler: ein Uniform, das der Shader wegoptimiert hat, ist
            // ein haeufiger und harmloser Fall (wie bei SHADER_SET).
            return Ok(());
        }
        let v: Vec<f32> = vals.iter().map(|x| if x.is_finite() { *x as f32 } else { 0.0 }).collect();
        sh.set_shader_value_v(loc, &v);
        Ok(())
    }
    /// SHADER_SET_TEXTURE(shader, name$, bild): zweiten Sampler belegen.
    /// Ohne das war jeder Shader auf die EINE Textur beschraenkt, die raylib
    /// selbst bindet -- Masken, Paletten-Nachschlagetabellen, Normal-Maps in
    /// 2D und Ueberblendungen zwischen zwei Bildern waren unmoeglich.
    pub fn shader_set_texture(&mut self, h: i64, name: &str, img: i64) -> Result<(), String> {
        // ffi::Texture2D ist Copy -- vorher herauskopieren, damit `self.textures`
        // nicht ausgeliehen bleibt, waehrend `self.shaders` veraendert wird.
        let tex: raylib::ffi::Texture2D = *self.textures.get(img.max(0) as usize)
            .ok_or_else(|| self.tex_fehler(img, "SHADER_SET_TEXTURE"))?
            .tex;
        let idx = h as usize;
        let sh = self.shaders.get(idx)
            .ok_or("SHADER_SET_TEXTURE: ungueltiges SHADER-Handle")?;
        let loc = sh.get_shader_location(name);
        if loc < 0 { return Ok(()); }            // Uniform wegoptimiert: harmlos
        // Nur vormerken -- gesetzt wird beim Zeichnen (siehe Feld-Kommentar).
        let slots = self.shader_textures.entry(idx).or_default();
        match slots.iter_mut().find(|(l, _)| *l == loc) {
            Some(slot) => slot.1 = tex,
            None => slots.push((loc, tex)),
        }
        Ok(())
    }
    /// SHADER_SET_MATRIX(shader, name$, mat): MAT4 aus dem `m3d`-Modul als
    /// `uniform mat4` -- eigene Projektionen/Bone-Transformationen im Shader.
    pub fn shader_set_matrix(&mut self, h: i64, name: &str, m: &[f32; 16]) -> Result<(), String> {
        let sh = self.shaders.get_mut(h as usize)
            .ok_or("SHADER_SET_MATRIX: ungueltiges SHADER-Handle")?;
        let loc = sh.get_shader_location(name);
        if loc >= 0 {
            // m3d liefert column-major (wie OpenGL), raylibs Matrix ist
            // row-major -> beim Umfuellen transponieren.
            sh.set_shader_value_matrix(loc, raylib::math::Matrix {
                m0: m[0], m1: m[1], m2: m[2], m3: m[3],
                m4: m[4], m5: m[5], m6: m[6], m7: m[7],
                m8: m[8], m9: m[9], m10: m[10], m11: m[11],
                m12: m[12], m13: m[13], m14: m[14], m15: m[15],
            });
        }
        Ok(())
    }

    /// Aktiven Post-Processing-Shader setzen (-1 = aus).
    pub fn set_postfx(&mut self, h: i64) {
        self.post_shader_idx = if h >= 0 && (h as usize) < self.shaders.len() {
            Some(h as usize)
        } else { None };
    }

    pub fn quit_requested(&self) -> bool {
        if let Some(mx) = self.max_frames {
            if self.frame_count >= mx { return true; }
        }
        self.rl.window_should_close()
    }

    // --- Shadow-Mapping ---
    /// Aktiviert Shadow-Mapping: legt ein sampleable Depth-FBO an und cached die
    /// Shader-Locations. Braucht Beleuchtung (LIGHT_ENABLE wird ggf. nachgeholt).
    pub fn shadow_enable(&mut self, res: i32) -> Result<(), String> {
        if self.light_shader.is_none() { self.light_enable(); }
        let res = res.clamp(256, 4096);
        if self.shadow_fbo == 0 {
            unsafe {
                let fbo = raylib::ffi::rlLoadFramebuffer();
                if fbo == 0 { return Err("SHADOW_ENABLE: Framebuffer fehlgeschlagen".into()); }
                raylib::ffi::rlEnableFramebuffer(fbo);
                let depth = raylib::ffi::rlLoadTextureDepth(res, res, false);
                // RL_ATTACHMENT_DEPTH=100, RL_ATTACHMENT_TEXTURE2D=100.
                raylib::ffi::rlFramebufferAttach(fbo, depth, 100, 100, 0);
                raylib::ffi::rlDisableFramebuffer();
                self.shadow_fbo = fbo;
                self.shadow_depth = depth;
            }
        } else if res != self.shadow_res {
            // Review-Fund: ein ZWEITER SHADOW_ENABLE-Aufruf mit anderer
            // Aufloesung aktualisierte bisher nur `self.shadow_res` (das
            // Viewport/Shader-Uniform in render_shadow_map lesen) -- die
            // tatsaechliche Depth-Textur blieb in ihrer URSPRUENGLICHEN
            // Groesse angelegt. Viewport/Shader gingen danach von einer
            // Aufloesung aus, die nicht zur echten Textur passte (falsch
            // skalierte/abgeschnittene/aliasing Schatten). Depth-Attachment
            // am bestehenden FBO neu anlegen statt nur die Zahl zu aendern.
            unsafe {
                raylib::ffi::rlEnableFramebuffer(self.shadow_fbo);
                raylib::ffi::rlUnloadTexture(self.shadow_depth);
                let depth = raylib::ffi::rlLoadTextureDepth(res, res, false);
                raylib::ffi::rlFramebufferAttach(self.shadow_fbo, depth, 100, 100, 0);
                raylib::ffi::rlDisableFramebuffer();
                self.shadow_depth = depth;
            }
        }
        self.shadow_res = res;
        let (vp, map, r, on) = {
            let sh = self.light_shader.as_ref().unwrap();
            (sh.get_shader_location("lightVP"), sh.get_shader_location("shadowMap"),
             sh.get_shader_location("shadowMapResolution"), sh.get_shader_location("shadowsEnabled"))
        };
        self.loc_light_vp = vp; self.loc_shadow_map = map;
        self.loc_shadow_res = r; self.loc_shadows_on = on;
        self.shadow_enabled = true;
        Ok(())
    }
    pub fn shadow_area(&mut self, size: f64, dist: f64) {
        self.shadow_area = (size as f32).max(1.0);
        self.shadow_dist = (dist as f32).max(1.0);
    }
    pub fn shadow_target(&mut self, x: f32, y: f32, z: f32) { self.shadow_target = [x, y, z]; }

    /// Rendert die beleuchteten Modelle aus Sicht des (ersten) directional
    /// Lights in die Depth-Map und setzt lightVP + bindet die Map an Slot 10.
    fn render_shadow_map(&mut self) {
        if !self.shadow_enabled || self.light_shader.is_none() { return; }
        // Schattenwerfendes Licht = erstes AKTIVES directional (kind 0). Sonst
        // aus. Review-Fund: ohne den enabled-Check waehlte diese Suche ein per
        // LIGHT_SET_ENABLED(false) deaktiviertes directional Light an Index 0
        // weiter aus, obwohl update_light_uniforms() die Szene bereits korrekt
        // nur aus den AKTIVEN Lichtern beleuchtet -- der Schatten zeigte dann
        // in eine Richtung, die nicht zum sichtbaren Licht passte.
        let dir = match self.lights.iter().find(|l| l.kind == 0 && l.enabled) {
            Some(l) => {
                let d = [l.target[0] - l.pos[0], l.target[1] - l.pos[1], l.target[2] - l.pos[2]];
                let len = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt().max(1e-6);
                [d[0]/len, d[1]/len, d[2]/len]
            }
            None => {
                if self.loc_shadows_on >= 0 {
                    let loc = self.loc_shadows_on;
                    self.light_shader.as_mut().unwrap().set_shader_value(loc, 0i32);
                }
                return;
            }
        };
        let t = self.shadow_target;
        let cam_pos = Vector3::new(t[0] - dir[0]*self.shadow_dist,
                                   t[1] - dir[1]*self.shadow_dist,
                                   t[2] - dir[2]*self.shadow_dist);
        let light_cam = Camera3D::orthographic(
            cam_pos, Vector3::new(t[0], t[1], t[2]), Vector3::new(0.0, 1.0, 0.0), self.shadow_area*2.0);
        let ffi_cam: raylib::ffi::Camera3D = light_cam.into();
        let res = self.shadow_res;
        let (win_w, win_h) = (self.width*self.scale, self.height*self.scale);
        let white = raylib::ffi::Color { r: 255, g: 255, b: 255, a: 255 };
        let (lv, lp);
        unsafe {
            raylib::ffi::rlEnableFramebuffer(self.shadow_fbo);
            raylib::ffi::rlViewport(0, 0, res, res);
            raylib::ffi::rlClearScreenBuffers();
            raylib::ffi::BeginMode3D(ffi_cam);
            lv = raylib::ffi::rlGetMatrixModelview();
            lp = raylib::ffi::rlGetMatrixProjection();
            for cmd in &self.cmds3d {
                match cmd {
                    Cmd3D::Model(i, x, y, z, sc, _) | Cmd3D::ModelWires(i, x, y, z, sc, _) => {
                        if let Some(m) = self.models.get(*i) {
                            raylib::ffi::DrawModel(*m.as_ref(),
                                raylib::ffi::Vector3 { x: *x, y: *y, z: *z }, *sc, white);
                        }
                    }
                    Cmd3D::ModelEx(i, x, y, z, ax, ay, az, ang, sc, _) => {
                        if let Some(m) = self.models.get(*i) {
                            raylib::ffi::DrawModelEx(*m.as_ref(),
                                raylib::ffi::Vector3 { x: *x, y: *y, z: *z },
                                raylib::ffi::Vector3 { x: *ax, y: *ay, z: *az }, *ang,
                                raylib::ffi::Vector3 { x: *sc, y: *sc, z: *sc }, white);
                        }
                    }
                    _ => {}
                }
            }
            raylib::ffi::EndMode3D();
            raylib::ffi::rlDisableFramebuffer();
            raylib::ffi::rlViewport(0, 0, win_w, win_h);
        }
        // lightVP = lightView * lightProj (== raylibs MatrixMultiply(view, proj)).
        let lvp = Matrix::from(lv) * Matrix::from(lp);
        let (loc_vp, loc_res, loc_on, loc_map, depth) =
            (self.loc_light_vp, self.loc_shadow_res, self.loc_shadows_on, self.loc_shadow_map, self.shadow_depth);
        let sh = self.light_shader.as_mut().unwrap();
        if loc_vp >= 0 { sh.set_shader_value_matrix(loc_vp, lvp); }
        if loc_res >= 0 { sh.set_shader_value(loc_res, res); }
        if loc_on >= 0 { sh.set_shader_value(loc_on, 1i32); }
        if loc_map >= 0 { sh.set_shader_value(loc_map, 10i32); }   // Sampler -> Texture-Unit 10
        // Depth-Textur an Unit 10 binden (Material-Maps nutzen 0..2 -> kein Clash).
        unsafe {
            raylib::ffi::rlActiveTextureSlot(10);
            raylib::ffi::rlEnableTexture(depth);
        }
    }

    pub fn flip(&mut self) {
        // Licht-Uniforms (viewPos/ambient/Lichter) vor dem 3D-Pass aktualisieren.
        self.update_light_uniforms();
        self.update_inst_light_uniforms();
        self.render_shadow_map();
        let s = self.scale;
        let clear_color = self.clear_color;
        let mut order: Vec<usize> = (0..self.layers.len()).collect();
        order.sort_by_key(|&i| self.layers[i].z);
        // RT-Groesse = Fenstergroesse (bekannt, ohne mehrdeutigen Textur-Query).
        let (tw, th) = ((self.width * self.scale) as f32, (self.height * self.scale) as f32);
        // HDR-IBL-Maps: im Draw-Kontext (binnen begin_drawing) an Slots 11/12/13
        // binden, damit die Bindung sicher bis zum Modell-Draw steht.
        let ibl = (self.use_ibl_maps, self.ibl_irradiance, self.ibl_prefilter, self.ibl_brdf);
        // Skybox-Info (Shader-ID + Locs + env-Cubemap), falls aktiv + HDR geladen.
        let skybox = if self.skybox_enabled && self.ibl_env != 0 {
            self.skybox_shader.as_ref().map(|s| (s.id, self.skybox_loc_proj, self.skybox_loc_view, self.ibl_env))
        } else { None };
        // m3d-Kamera-Overrides vor dem (borrowenden) Destructure kopieren (Copy).
        let cam_view = self.cam3d_view;
        let cam_proj = self.cam3d_proj;
        // Instancing-Shader (ffi::Shader = Copy) fuer den DrawMeshInstanced-Pfad.
        let inst_ffi = self.inst_shader.as_ref().map(|s| *s.as_ref());
        let Graphics { rl, thread, layers, textures, fonts, fallback, cmds3d, cam3d, models,
            light_shader, normal_mapped, loc_use_normal, pbr_params, loc_metalness, loc_roughness,
            emissive, loc_emissive,
            scene_rt, shaders, shader_textures, post_shader_idx, render_targets, .. } = self;
        let mat_locs = (*loc_use_normal, *loc_metalness, *loc_roughness, *loc_emissive);
        let ausweich: Option<&Font> = fallback.as_ref();
        let nmap_set: &std::collections::HashSet<usize> = normal_mapped;
        let pbr_ref: &std::collections::HashMap<usize, (f32, f32)> = pbr_params;
        let emis_ref: &std::collections::HashMap<usize, (f32, f32, f32, f32)> = emissive;
        let cam = *cam3d;
        // Render-Targets zuerst auf ihre Texturen rendern (nur die, in die dieser
        // Frame gezeichnet wurde -- leere behalten ihren Inhalt). 2D-only: eigene
        // synthetische Ein-Layer-Szene, transparent gecleart, kein 3D/Licht/RtDraw.
        {
            let empty_set = std::collections::HashSet::new();
            let empty_map = std::collections::HashMap::new();
            let empty_emis = std::collections::HashMap::new();
            let clear_rt = Color::new(0, 0, 0, 0);
            for i in 0..render_targets.len() {
                if render_targets[i].cmds.is_empty() { continue; }
                let cmds = std::mem::take(&mut render_targets[i].cmds);
                let synth = [Layer { z: 0, cmds }];
                // Ein behaltenes Target wird NICHT geleert -- der neue Inhalt
                // legt sich ueber den alten. Das ist der ganze Trick hinter
                // Rueckkopplung/Nachzieheffekten.
                let behalten = render_targets[i].behalten;
                let mut tx = rl.begin_texture_mode(thread, &mut render_targets[i].rt);
                let clear = if behalten { None } else { Some(clear_rt) };
                // Maßstab 1, NICHT der Fenster-Maßstab `s`: ein Render-Target
                // hat seine EIGENE Pixelgroesse. Mit `s` wurde der Inhalt im
                // Vollbild (s=2) doppelt so gross in ein Ziel fester Groesse
                // gezeichnet -- alles rechts/unten davon fiel weg, und in einem
                // behaltenen Target blieben die abgeschnittenen Raender stehen
                // ("die Kurven kleben am rechten Rand"). Hochskaliert wird beim
                // Stempeln (Cmd::RtDraw rechnet dort mit `s`), nicht hier.
                render_scene(&mut tx, 1, clear, &synth, &[0], textures, fonts, ausweich,
                    &[], cam, &[], None, (-1, -1, -1, -1), &empty_set, &empty_map, &empty_emis,
                    (false, 0, 0, 0), &[], None, None, None, None);
            }
        }
        let rts: &[RenderTarget] = render_targets;
        // Post-FX aktiv? -> (Shader-Index, Render-Target). Sonst direkt zeichnen.
        let postfx = match *post_shader_idx {
            Some(i) if i < shaders.len() => scene_rt.as_mut().map(|rt| (i, rt)),
            _ => None,
        };
        if let Some((idx, rt)) = postfx {
            // 1) Szene in die RenderTexture rendern.
            {
                let mut tx = rl.begin_texture_mode(thread, rt);
                render_scene(&mut tx, s, Some(clear_color), layers, &order, textures, fonts, ausweich, cmds3d, cam, models, light_shader.as_mut(), mat_locs, nmap_set, pbr_ref, emis_ref, ibl, rts, skybox, cam_view, cam_proj, inst_ffi);
            }
            // 2) RT per Fragment-Shader auf den Screen praesentieren (Y-flip).
            let src = Rectangle::new(0.0, 0.0, tw, -th);
            let dst = Rectangle::new(0.0, 0.0, tw, th);
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            {
                // ffi::Shader ist Copy -- vor dem mutablen Ausleihen kopieren.
                let sh_ffi = *shaders[idx].as_ref();
                let mut sm = d.begin_shader_mode(&mut shaders[idx]);
                // Zusaetzliche Sampler JETZT setzen -- `SetShaderValueTexture`
                // ruft glUniform1i auf dem gerade aktiven Programm, vorher waere
                // es am falschen gelandet (Sampler bliebe schwarz).
                if let Some(slots) = shader_textures.get(&idx) {
                    for (loc, tex) in slots {
                        unsafe { raylib::ffi::SetShaderValueTexture(sh_ffi, *loc, *tex); }
                    }
                }
                sm.draw_texture_pro(&*rt, src, dst, Vector2::zero(), 0.0, Color::WHITE);
            }
        } else {
            let mut d = rl.begin_drawing(thread);
            render_scene(&mut d, s, Some(clear_color), layers, &order, textures, fonts, ausweich, cmds3d, cam, models, light_shader.as_mut(), mat_locs, nmap_set, pbr_ref, emis_ref, ibl, rts, skybox, cam_view, cam_proj, inst_ffi);
        }
        // Web (emscripten): nach dem Praesentieren (EndDrawing oben beim Drop des
        // Draw-Handles) ans Browser-Event-Loop yielden -- sonst blockiert der
        // GB-Render-Loop den Main-Thread und der Tab haengt. ASYNCIFY wickelt den
        // Stack ab; beim naechsten Frame geht es hier weiter.
        #[cfg(target_os = "emscripten")]
        unsafe { emscripten_sleep(0); }

        // Aufgezeichnete Eingabe einspeisen. Muss HIER stehen: das Draw-Handle
        // ist gerade gedroppt worden (= EndDrawing), raylib hat die echte
        // Eingabe fuer den naechsten Frame schon eingelesen -- die eingespeisten
        // Werte ueberschreiben sie also und gelten fuer genau diesen Frame.
        self.automation_tick();

        // Layer + 3D-Befehle fuer den naechsten Frame leeren (Immediate-Mode).
        for l in self.layers.iter_mut() { l.cmds.clear(); }
        self.cmds3d.clear();
        // Der Abspieler schliesst offen gebliebene Clips am Bildende; der
        // Zaehler muss ihm folgen, sonst schleppte ein vergessenes SCISSOR
        // seine Tiefe ins naechste Bild und SCISSOR_END traefe dort ins Leere.
        self.clip_tiefe = 0;
        // Review-Fund: CLAUDE.md/docs dokumentieren "LAYER_END() -- zurueck
        // zum Main-Buffer (optional, FLIP macht's auch)" -- das stimmte
        // bisher nicht: weder `active` (aktive Layer) noch `active_rt`
        // (aktives Render-Target) wurden hier zurueckgesetzt. Ein Frame, der
        // mit `LAYER("ui") : TEXT(...) : FLIP()` endet (ohne explizites
        // LAYER_END()), liess ALLE Draws des naechsten Frames -- inklusive
        // CLS() -- weiter in die "ui"-Layer laufen, die Haupt-/Hintergrund-
        // Zeichnungen verschwanden dadurch effektiv hinter der UI-Layer.
        // Ein vergessenes RENDERTARGET_END() verschluckte auf dieselbe Art
        // alle folgenden Frames komplett (sie liefen weiter ins Render-
        // Target statt auf den Screen).
        self.active = 0;
        self.active_rt = None;
        self.frame_count += 1;
        // Web: raylib setzt die Leinwand nach dem Anlegen noch einmal selbst --
        // die ersten Bilder lang nachziehen, danach steht sie.
        #[cfg(target_os = "emscripten")]
        if self.frame_count <= 8 {
            web_leinwand_groesse(self.width * self.scale, self.height * self.scale);
        }
        // CAMERA_SHAKE: Offset fuer den naechsten Frame wuerfeln/abklingen.
        self.update_shake();
        // Headless-Screenshot beim Erreichen der Frame-Grenze.
        if let (Some(mx), Some(path), false) = (self.max_frames, self.screenshot.clone(), self.shot_taken) {
            if self.frame_count >= mx {
                self.write_screenshot(&path);
                self.shot_taken = true;
            }
        }
        // Kontaktbogen: in festen Abstaenden aufnehmen, am Ende zusammensetzen.
        if self.contact_path.is_some() && !self.contact_written {
            if self.frame_count % self.contact_every == 0 { self.contact_capture(); }
            let voll = self.contact_shots.len() >= self.contact_max;
            let ende = self.max_frames.map(|mx| self.frame_count >= mx).unwrap_or(false);
            if voll || ende { self.contact_write(); }
        }
    }
}

/// m3d-MAT4 ([f32;16] column-major / OpenGL-Float-Order) -> raylib::ffi::Matrix.
/// Feld m{k} = arr[k] (arr ist die MatrixToFloatV-Reihenfolge m0..m15).
fn m3d_arr_to_ffi(a: &[f32; 16]) -> raylib::ffi::Matrix {
    raylib::ffi::Matrix {
        m0: a[0], m1: a[1], m2: a[2], m3: a[3],
        m4: a[4], m5: a[5], m6: a[6], m7: a[7],
        m8: a[8], m9: a[9], m10: a[10], m11: a[11],
        m12: a[12], m13: a[13], m14: a[14], m15: a[15],
    }
}

/// Spielt 3D-Befehle (begin_mode3D) + 2D-Layer (mit Scissor-Clip-Stack) auf ein
/// beliebiges Draw-Ziel ab -- den Screen ODER eine RenderTexture (beide impl
/// `RaylibDraw`). So laeuft derselbe Replay-Code mit und ohne Post-Shader.
/// `clear = None` laesst den Zielinhalt stehen (behaltene Render-Targets --
/// Voraussetzung fuer Rueckkopplungs-/Nachzieheffekte).
/// Font-Handle eines Zeichen-Befehls aufloesen. `None` = eingebaute Schrift.
/// FONT_AUSWEICH steht fuer den Umlaut-Ausweichfont, der nicht in `fonts` liegt.
fn font_zu_handle<'a>(h: i64, fonts: &'a [Font], ausweich: Option<&'a Font>) -> Option<&'a Font> {
    if h == FONT_AUSWEICH { return ausweich; }
    if h < 0 { return None; }
    fonts.get(h as usize)
}

fn render_scene<D: RaylibDraw>(
    d: &mut D, s: i32, clear: Option<Color>,
    layers: &[Layer], order: &[usize], textures: &[Tex], fonts: &[Font],
    ausweich: Option<&Font>,
    cmds3d: &[Cmd3D], cam3d: Camera3D, models: &[Model],
    mut light_shader: Option<&mut Shader>, mat_locs: (i32, i32, i32, i32),
    normal_mapped: &std::collections::HashSet<usize>,
    pbr_params: &std::collections::HashMap<usize, (f32, f32)>,
    emissive_params: &std::collections::HashMap<usize, (f32, f32, f32, f32)>,
    ibl: (bool, u32, u32, u32),
    render_targets: &[RenderTarget],
    skybox: Option<(u32, i32, i32, u32)>,   // (shader_id, loc_proj, loc_view, env_cubemap)
    cam_view: Option<[f32; 16]>,            // m3d CAMERA3D_VIEW-Override (column-major)
    cam_proj: Option<[f32; 16]>,            // m3d CAMERA3D_PROJECTION-Override
    inst_shader: Option<raylib::ffi::Shader>,   // m3d MODEL_INSTANCED (DrawMeshInstanced)
) {
    let (loc_use_normal, loc_metalness, loc_roughness, loc_emissive) = mat_locs;
    // Per-Modell-Material-Uniforms (useNormalMap + metalness/roughness + emissive) vor dem Draw.
    let set_material = |ls: &mut Option<&mut Shader>, idx: usize| {
        if let Some(sh) = ls.as_mut() {
            if loc_use_normal >= 0 {
                sh.set_shader_value(loc_use_normal, if normal_mapped.contains(&idx) { 1i32 } else { 0i32 });
            }
            let (m, r) = pbr_params.get(&idx).copied().unwrap_or((0.0, 0.6));
            if loc_metalness >= 0 { sh.set_shader_value(loc_metalness, m); }
            if loc_roughness >= 0 { sh.set_shader_value(loc_roughness, r); }
            if loc_emissive >= 0 {
                let (er, eg, eb, es) = emissive_params.get(&idx).copied().unwrap_or((0.0, 0.0, 0.0, 0.0));
                sh.set_shader_value(loc_emissive, [er, eg, eb, es]);
            }
        }
    };
    let mut clip_stack: Vec<(i32, i32, i32, i32)> = Vec::new();
    let mut cur_blend = 0i32;   // aktiver Blend-Mode (0 = Default/alpha)
    if let Some(c) = clear { d.clear_background(c); }
            // HDR-IBL-Maps im Draw-Kontext an Slots 11/12/13 binden (Cubemaps via
            // rlEnableTextureCubemap, BRDF-LUT 2D). Hier statt update_light_uniforms,
            // damit die Bindung garantiert bis zum Modell-Draw steht.
            if ibl.0 {
                unsafe {
                    raylib::ffi::rlActiveTextureSlot(11);
                    raylib::ffi::rlEnableTextureCubemap(ibl.1);
                    raylib::ffi::rlActiveTextureSlot(12);
                    raylib::ffi::rlEnableTextureCubemap(ibl.2);
                    raylib::ffi::rlActiveTextureSlot(13);
                    raylib::ffi::rlEnableTexture(ibl.3);
                    raylib::ffi::rlActiveTextureSlot(0);
                }
            }
            // 3D-Pass zuerst (in einem begin_mode3D-Block), 2D-HUD danach obenauf.
            if !cmds3d.is_empty() || skybox.is_some() {
                let mut d3 = d.begin_mode3D(cam3d);
                // m3d-Overrides: begin_mode3D hat View/Projektion aus cam3d gesetzt;
                // hier ggf. durch die benutzerdefinierten Matrizen ersetzen (Ortho,
                // Custom-Frustum, Shadow-Tricks). Gilt fuer Skybox + alle 3D-Draws.
                unsafe {
                    if let Some(p) = &cam_proj { raylib::ffi::rlSetMatrixProjection(m3d_arr_to_ffi(p)); }
                    if let Some(v) = &cam_view { raylib::ffi::rlSetMatrixModelview(m3d_arr_to_ffi(v)); }
                }
                // Skybox ganz zuerst (Hintergrund): env-Cubemap in Blickrichtung,
                // ohne Depth-Write (Modelle zeichnen darueber), Cube von innen.
                if let Some((sid, lproj, lview, env)) = skybox {
                    unsafe {
                        let view = raylib::ffi::rlGetMatrixModelview();
                        let proj = raylib::ffi::rlGetMatrixProjection();
                        raylib::ffi::rlDisableBackfaceCulling();
                        raylib::ffi::rlDisableDepthMask();
                        raylib::ffi::rlEnableShader(sid);
                        raylib::ffi::rlSetUniformMatrix(lproj, proj);
                        raylib::ffi::rlSetUniformMatrix(lview, view);
                        raylib::ffi::rlActiveTextureSlot(0);
                        raylib::ffi::rlEnableTextureCubemap(env);
                        raylib::ffi::rlLoadDrawCube();
                        raylib::ffi::rlDisableTextureCubemap();
                        raylib::ffi::rlDisableShader();
                        raylib::ffi::rlEnableDepthMask();
                        raylib::ffi::rlEnableBackfaceCulling();
                    }
                }
                for c in cmds3d.iter() {
                    match c {
                        Cmd3D::Cube(x, y, z, w, h, dd, col) =>
                            d3.draw_cube(Vector3::new(*x, *y, *z), *w, *h, *dd, *col),
                        Cmd3D::CubeWires(x, y, z, w, h, dd, col) =>
                            d3.draw_cube_wires(Vector3::new(*x, *y, *z), *w, *h, *dd, *col),
                        Cmd3D::Sphere(x, y, z, r, col) =>
                            d3.draw_sphere(Vector3::new(*x, *y, *z), *r, *col),
                        Cmd3D::SphereWires(x, y, z, r, col) =>
                            d3.draw_sphere_wires(Vector3::new(*x, *y, *z), *r, 12, 12, *col),
                        Cmd3D::Cylinder(x, y, z, rt, rb, h, col) =>
                            d3.draw_cylinder(Vector3::new(*x, *y, *z), *rt, *rb, *h, 16, *col),
                        Cmd3D::Plane(x, y, z, sx, sz, col) =>
                            d3.draw_plane(Vector3::new(*x, *y, *z), Vector2::new(*sx, *sz), *col),
                        Cmd3D::Line(x1, y1, z1, x2, y2, z2, col) =>
                            d3.draw_line3D(Vector3::new(*x1, *y1, *z1), Vector3::new(*x2, *y2, *z2), *col),
                        Cmd3D::Point(x, y, z, col) =>
                            d3.draw_point3D(Vector3::new(*x, *y, *z), *col),
                        Cmd3D::Grid(slices, spacing) =>
                            d3.draw_grid(*slices, *spacing),
                        Cmd3D::Model(i, x, y, z, sc, col) => {
                            if let Some(m) = models.get(*i) {
                                set_material(&mut light_shader, *i);
                                d3.draw_model(m, Vector3::new(*x, *y, *z), *sc, *col);
                            }
                        }
                        Cmd3D::ModelEx(i, x, y, z, ax, ay, az, ang, sc, col) => {
                            if let Some(m) = models.get(*i) {
                                set_material(&mut light_shader, *i);
                                d3.draw_model_ex(m, Vector3::new(*x, *y, *z),
                                    Vector3::new(*ax, *ay, *az), *ang,
                                    Vector3::new(*sc, *sc, *sc), *col);
                            }
                        }
                        Cmd3D::ModelWires(i, x, y, z, sc, col) => {
                            if let Some(m) = models.get(*i) {
                                set_material(&mut light_shader, *i);
                                d3.draw_model_wires(m, Vector3::new(*x, *y, *z), *sc, *col);
                            }
                        }
                        Cmd3D::ModelMatrix(i, mat, col) => {
                            if let Some(m) = models.get(*i) {
                                set_material(&mut light_shader, *i);
                                // Welt-Matrix auf den rl-Transform-Stack; DrawModel mit
                                // pos=0/scale=1 -> nur unsere Matrix wirkt (DrawMesh liest
                                // rlGetMatrixTransform()). matModel-Uniform = unsere Matrix.
                                unsafe {
                                    raylib::ffi::rlPushMatrix();
                                    raylib::ffi::rlMultMatrixf(mat.as_ptr());
                                }
                                d3.draw_model(m, Vector3::new(0.0, 0.0, 0.0), 1.0, *col);
                                unsafe { raylib::ffi::rlPopMatrix(); }
                            }
                        }
                        Cmd3D::ModelInstanced(i, mats, col) => {
                            // GPU-Instancing: dasselbe Mesh mit N Welt-Matrizen in
                            // EINEM Draw-Call pro Mesh (raylib DrawMeshInstanced).
                            // Material temporaer auf den Instancing-Shader + tint
                            // umstellen, danach wiederherstellen (sonst broeche der
                            // non-instanced Pfad mit demselben Modell).
                            if let (Some(m), Some(sh)) = (models.get(*i), inst_shader) {
                                let tr: Vec<raylib::ffi::Matrix> =
                                    mats.iter().map(|a| m3d_arr_to_ffi(a)).collect();
                                let fc = raylib::ffi::Color { r: col.r, g: col.g, b: col.b, a: col.a };
                                unsafe {
                                    let mdl: &raylib::ffi::Model = m.as_ref();
                                    for mi in 0..mdl.meshCount as isize {
                                        let mesh = *mdl.meshes.offset(mi);
                                        let mat_idx = if mdl.meshMaterial.is_null() { 0 }
                                            else { *mdl.meshMaterial.offset(mi) as isize };
                                        let mat_ptr = mdl.materials.offset(mat_idx);
                                        let map_ptr = (*mat_ptr).maps; // [0] = ALBEDO/DIFFUSE
                                        let saved_shader = (*mat_ptr).shader;
                                        let saved_col = (*map_ptr).color;
                                        (*mat_ptr).shader = sh;
                                        (*map_ptr).color = fc;
                                        raylib::ffi::DrawMeshInstanced(
                                            mesh, *mat_ptr, tr.as_ptr(), tr.len() as i32);
                                        (*mat_ptr).shader = saved_shader;
                                        (*map_ptr).color = saved_col;
                                    }
                                }
                            }
                        }
                        Cmd3D::Billboard(i, x, y, z, size, col) => {
                            if let Some(t) = textures.get(*i) {
                                d3.draw_billboard(cam3d, &t.tex, Vector3::new(*x, *y, *z), *size, *col);
                            }
                        }
                    }
                }
            }
            for &li in order {
              // Review-Fund: Blend-Mode/Scissor-Clip wurden bisher nur EINMAL
              // vor der gesamten Layer-Schleife deklariert und nie zwischen
              // Layern zurueckgesetzt -- ein BLEND_MODE("add")/PUSH_CLIP auf
              // Layer A ohne passenden Reset lief in Layer B (und jede
              // weitere, spaeter im z-Order gezeichnete Layer) hinein, weil
              // alle Layer-Cmds hier in EINEM flachen Replay ablaufen. Vor
              // jeder neuen Layer auf Default zurueckstellen, statt erst am
              // Ende der kompletten Schleife (das schuetzte bisher nur den
              // naechsten FRAME, nicht die naechste Layer im selben Frame).
              if cur_blend != 0 { unsafe { raylib::ffi::EndBlendMode(); } cur_blend = 0; }
              if !clip_stack.is_empty() { unsafe { raylib::ffi::EndScissorMode(); } clip_stack.clear(); }
              for c in layers[li].cmds.iter() {
                match c {
                    Cmd::Clear(col) => d.clear_background(*col),
                    Cmd::Pixel(x, y, col) => {
                        if s == 1 { d.draw_pixel(*x, *y, *col); }
                        else { d.draw_rectangle(x * s, y * s, s, s, *col); }
                    }
                    Cmd::Line(a, b, cc, dd, col) => d.draw_line(a * s, b * s, cc * s, dd * s, *col),
                    Cmd::BoxFill(x1, y1, x2, y2, col) => {
                        let x = (*x1).min(*x2) * s; let y = (*y1).min(*y2) * s;
                        let w = ((x2 - x1).abs() + 1) * s; let h = ((y2 - y1).abs() + 1) * s;
                        d.draw_rectangle(x, y, w, h, *col);
                    }
                    Cmd::RectLines(x1, y1, x2, y2, col) => {
                        let x = (*x1).min(*x2) * s; let y = (*y1).min(*y2) * s;
                        let w = ((x2 - x1).abs() + 1) * s; let h = ((y2 - y1).abs() + 1) * s;
                        d.draw_rectangle_lines(x, y, w, h, *col);
                    }
                    Cmd::Circle(x, y, r, col) => d.draw_circle(x * s, y * s, r * s as f32, *col),
                    Cmd::Triangle(x1, y1, x2, y2, x3, y3, col) => {
                        // raylib `DrawTriangle` cullt nach Wicklung (erwartet CCW
                        // im y-down-Screen-Space). Damit TRIANGLE wicklungs-
                        // unabhaengig ist: signed area pruefen, bei CW (>0) die
                        // letzten beiden Vertices intern tauschen statt nichts
                        // zu zeichnen.
                        let (x1, y1, mut x2, mut y2, mut x3, mut y3) = (*x1, *y1, *x2, *y2, *x3, *y3);
                        let area2 = (x2 - x1) as i64 * (y3 - y1) as i64
                                  - (x3 - x1) as i64 * (y2 - y1) as i64;
                        if area2 > 0 {
                            std::mem::swap(&mut x2, &mut x3);
                            std::mem::swap(&mut y2, &mut y3);
                        }
                        d.draw_triangle(
                            Vector2::new((x1 * s) as f32, (y1 * s) as f32),
                            Vector2::new((x2 * s) as f32, (y2 * s) as f32),
                            Vector2::new((x3 * s) as f32, (y3 * s) as f32),
                            *col,
                        );
                    }
                    Cmd::TriLines(x1, y1, x2, y2, x3, y3, col) => {
                        d.draw_triangle_lines(
                            Vector2::new((x1 * s) as f32, (y1 * s) as f32),
                            Vector2::new((x2 * s) as f32, (y2 * s) as f32),
                            Vector2::new((x3 * s) as f32, (y3 * s) as f32),
                            *col,
                        );
                    }
                    Cmd::Ellipse(cx, cy, rh, rv, col, filled) => {
                        if *filled { d.draw_ellipse(cx * s, cy * s, (*rh * s) as f32, (*rv * s) as f32, *col); }
                        else { d.draw_ellipse_lines(cx * s, cy * s, (*rh * s) as f32, (*rv * s) as f32, *col); }
                    }
                    Cmd::Poly(pts, thick, col, closed) => {
                        let n = pts.len();
                        if n >= 2 {
                            if *thick > 1.0 {
                                // `DrawSplineLinear` zeichnet jedes Segment als eigenes,
                                // ungemitertes Quad (raylib ohne SUPPORT_SPLINE_MITERS) --
                                // an den Gelenken zwischen den vielen kurzen Arc-Segmenten
                                // reisst die Innenseite der Kurve dadurch sichtbar auf
                                // (Loecher). Stattdessen jedes Segment per `draw_line_ex`
                                // zeichnen und an jedem Gelenk einen gefuellten Kreis
                                // (Radius = halbe Dicke) draufsetzen -- klassisches
                                // Round-Join-Pattern, luecken-frei unabhaengig vom Winkel.
                                let thick_s = *thick * s as f32;
                                let v: Vec<Vector2> = pts.iter()
                                    .map(|p| Vector2::new((p.0 * s) as f32, (p.1 * s) as f32)).collect();
                                for i in 0..n - 1 {
                                    d.draw_line_ex(v[i], v[i + 1], thick_s, *col);
                                }
                                if *closed {
                                    d.draw_line_ex(v[n - 1], v[0], thick_s, *col);
                                }
                                let r = thick_s * 0.5;
                                // Offene Linie: nur die inneren Gelenke runden (Enden
                                // bleiben flach, wie bei einem einzelnen draw_line_ex).
                                // Geschlossen: JEDER Punkt ist ein Gelenk, inkl. v[0].
                                let joints: &[Vector2] = if *closed { &v[..] } else { &v[1..n - 1] };
                                for p in joints {
                                    d.draw_circle_v(*p, r, *col);
                                }
                            } else {
                                for i in 0..n - 1 {
                                    d.draw_line(pts[i].0 * s, pts[i].1 * s, pts[i + 1].0 * s, pts[i + 1].1 * s, *col);
                                }
                                if *closed {
                                    d.draw_line(pts[n - 1].0 * s, pts[n - 1].1 * s, pts[0].0 * s, pts[0].1 * s, *col);
                                }
                            }
                        }
                    }
                    Cmd::FillPoly(pts, col) => {
                        // Triangle-Fan (korrekt fuer konvexe Polygone).
                        // `DrawTriangleFan` cullt wie `DrawTriangle` nach Wicklung
                        // -> bei CW-Eingabe (signed area >0) die Punkt-Reihenfolge
                        // umdrehen, damit POLYGON wicklungsunabhaengig fuellt.
                        if pts.len() >= 3 {
                            let n = pts.len();
                            let mut area2: i64 = 0;
                            for i in 0..n {
                                let (ax, ay) = pts[i];
                                let (bx, by) = pts[(i + 1) % n];
                                area2 += ax as i64 * by as i64 - bx as i64 * ay as i64;
                            }
                            let mut v: Vec<Vector2> = pts.iter()
                                .map(|p| Vector2::new((p.0 * s) as f32, (p.1 * s) as f32)).collect();
                            if area2 > 0 { v.reverse(); }
                            d.draw_triangle_fan(&v, *col);
                        }
                    }
                    Cmd::Text(x, y, txt, sz, col, font, spacing) => {
                        match font_zu_handle(*font, fonts, ausweich) {
                            Some(f) => d.draw_text_ex(
                                f, txt, Vector2::new((x * s) as f32, (y * s) as f32),
                                (sz * s) as f32, spacing * s as f32, *col),
                            None => d.draw_text(txt, x * s, y * s, sz * s, *col),
                        }
                    }
                    Cmd::TextRot(cx, cy, txt, sz, col, font, spacing, ang, scl) => {
                        // Zentriert auf (cx,cy), Rotation um das Text-Zentrum
                        // (DrawTextPro: origin = halbe Textbox).
                        let fsize = (sz * s) as f32 * scl;
                        let pos = Vector2::new((cx * s) as f32, (cy * s) as f32);
                        match font_zu_handle(*font, fonts, ausweich) {
                            Some(f) => {
                                let fspacing = spacing * s as f32 * scl;
                                let m = f.measure_text(txt, fsize, fspacing);
                                d.draw_text_pro(f, txt, pos, Vector2::new(m.x / 2.0, m.y / 2.0),
                                                *ang, fsize, fspacing, *col);
                            }
                            _ => {
                                // Default-Font: raylib-Spacing-Konvention = Groesse/10 (wie
                                // DrawText). Via ffi, weil der generische Draw-Kontext kein
                                // get_font_default hat.
                                let fspacing = fsize / 10.0;
                                let c_txt = std::ffi::CString::new(txt.as_str()).unwrap_or_default();
                                unsafe {
                                    let df = raylib::ffi::GetFontDefault();
                                    let m = raylib::ffi::MeasureTextEx(df, c_txt.as_ptr(), fsize, fspacing);
                                    raylib::ffi::DrawTextPro(
                                        df, c_txt.as_ptr(),
                                        raylib::ffi::Vector2 { x: pos.x, y: pos.y },
                                        raylib::ffi::Vector2 { x: m.x / 2.0, y: m.y / 2.0 },
                                        *ang, fsize, fspacing, (*col).into());
                                }
                            }
                        }
                    }
                    Cmd::Texture(i, x, y, dw, dh) => {
                        // Review-Fund: `dw`/`dh` sind bereits beim Emit ueber
                        // ssize() mit cam_zoom skaliert -- hier nur noch der
                        // SCREEN()-Skalierungsfaktor `s`, analog zu TextureRect.
                        let t = &textures[*i].tex;
                        let src = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32);
                        let dst = Rectangle::new((x * s) as f32, (y * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(t, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::TexturePart(i, sx, sy, sw, sh, dx, dy, dw, dh) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32, *sw as f32, *sh as f32);
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::TexturePartEx(i, sx, sy, sw, sh, dx, dy, dw, dh) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32, *sw as f32, *sh as f32);
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::TextureRect(i, dx, dy, dw, dh) => {
                        if let Some(t) = textures.get(*i) {
                            let src = Rectangle::new(0.0, 0.0, t.tex.width as f32, t.tex.height as f32);
                            let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                            d.draw_texture_pro(&t.tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                        }
                    }
                    Cmd::TextureFlipped(i, x, y, dw, dh, fh, fv) => {
                        let t = &textures[*i].tex;
                        let sw = if *fh { -(t.width as f32) } else { t.width as f32 };
                        let sh = if *fv { -(t.height as f32) } else { t.height as f32 };
                        let src = Rectangle::new(0.0, 0.0, sw, sh);
                        let dst = Rectangle::new((x * s) as f32, (y * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(t, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::TextureRot(i, cx, cy, ang, scl, tint) => {
                        let t = &textures[*i].tex;
                        let w = t.width as f32 * scl * s as f32;
                        let h = t.height as f32 * scl * s as f32;
                        let src = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32);
                        // dst-Position = Zentrum; origin = halbe Groesse -> Drehung um die Mitte.
                        let dst = Rectangle::new((cx * s) as f32, (cy * s) as f32, w, h);
                        d.draw_texture_pro(t, src, dst, Vector2::new(w / 2.0, h / 2.0), *ang, *tint);
                    }
                    Cmd::AtlasDraw(i, sx, sy, sw, sh, dx, dy, dw, dh, fh, fv, tint) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32,
                            if *fh { -(*sw as f32) } else { *sw as f32 },
                            if *fv { -(*sh as f32) } else { *sh as f32 });
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0, *tint);
                    }
                    Cmd::SpriteDraw(i, sx, sy, sw, sh, dx, dy, dw, dh, fx, fy, tint) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32,
                            if *fx { -(*sw as f32) } else { *sw as f32 },
                            if *fy { -(*sh as f32) } else { *sh as f32 });
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0,*tint);
                    }
                    Cmd::LineEx(x1, y1, x2, y2, thick, col) => {
                        d.draw_line_ex(Vector2::new((x1 * s) as f32, (y1 * s) as f32),
                            Vector2::new((x2 * s) as f32, (y2 * s) as f32), thick * s as f32, *col);
                    }
                    Cmd::RoundRect(x1, y1, x2, y2, radius, col, filled) => {
                        let x = (*x1).min(*x2) * s; let y = (*y1).min(*y2) * s;
                        let w = ((x2 - x1).abs() + 1) * s; let h = ((y2 - y1).abs() + 1) * s;
                        // raylib-roundness = Bruchteil der halben kuerzeren Seite (0..1).
                        let half = (w.min(h) as f32) * 0.5;
                        let roundness = if half > 0.0 { ((*radius * s) as f32 / half).clamp(0.0, 1.0) } else { 0.0 };
                        let rec = Rectangle::new(x as f32, y as f32, w as f32, h as f32);
                        if *filled { d.draw_rectangle_rounded(rec, roundness, 12, *col); }
                        else { d.draw_rectangle_rounded_lines(rec, roundness, 12, *col); }
                    }
                    Cmd::RoundGradient(x1, y1, x2, y2, radius, c1, c2) => {
                        let x = (*x1).min(*x2) * s;
                        let y = (*y1).min(*y2) * s;
                        let w = ((x2 - x1).abs() + 1) * s;
                        let h = ((y2 - y1).abs() + 1) * s;
                        let r = (*radius * s).clamp(0, w / 2).min(h / 2);
                        for zeile in 0..h {
                            let einzug = ecken_einzug(r, h, zeile);
                            let breite = w - 2 * einzug;
                            if breite <= 0 {
                                continue;
                            }
                            let t = if h > 1 { zeile as f32 / (h - 1) as f32 } else { 0.0 };
                            let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
                            let c = Color::new(lerp(c1.r, c2.r), lerp(c1.g, c2.g),
                                               lerp(c1.b, c2.b), lerp(c1.a, c2.a));
                            d.draw_rectangle(x + einzug, y + zeile, breite, 1, c);
                        }
                    }
                    Cmd::Ring(cx, cy, ri, ro, von, bis, col, filled) => {
                        let mitte = Vector2::new((cx * s) as f32, (cy * s) as f32);
                        let (ri, ro) = (ri * s as f32, ro * s as f32);
                        // Ein Segment je 4 Grad haelt auch grosse Kreise rund,
                        // ohne bei schmalen Kuchenstuecken Dreiecke zu verschwenden.
                        let seg = (((bis - von).abs() / 4.0).ceil() as i32).clamp(6, 180);
                        if *filled { d.draw_ring(mitte, ri, ro, *von, *bis, seg, *col); }
                        else { d.draw_ring_lines(mitte, ri, ro, *von, *bis, seg, *col); }
                    }
                    Cmd::GradientRect(x1, y1, x2, y2, c1, c2, vertical) => {
                        let x = (*x1).min(*x2) * s; let y = (*y1).min(*y2) * s;
                        let w = ((x2 - x1).abs() + 1) * s; let h = ((y2 - y1).abs() + 1) * s;
                        if *vertical { d.draw_rectangle_gradient_v(x, y, w, h, *c1, *c2); }
                        else { d.draw_rectangle_gradient_h(x, y, w, h, *c1, *c2); }
                    }
                    Cmd::Spline(pts, thick, col) => {
                        if pts.len() >= 2 {
                            let v: Vec<Vector2> = pts.iter()
                                .map(|p| Vector2::new((p.0 * s) as f32, (p.1 * s) as f32)).collect();
                            // Catmull-Rom braucht >= 4 Punkte; sonst dicke Linie.
                            // Review-Fund: raylibs DrawSplineCatmullRom behandelt
                            // den ERSTEN und LETZTEN Punkt nur als Tangenten-
                            // Kontrollpunkte (startet bei points[1], endet vor
                            // points[n-1]) -- die Kurve lief bisher NICHT durch
                            // die tatsaechlich uebergebenen Start-/Endpunkte,
                            // obwohl SPLINE laut Doku "Catmull-Rom DURCH Punkte"
                            // verspricht (sichtbar im shipped-Demo
                            // examples/100_2d_extras.dh: die Kurve endete
                            // sichtbar vor den letzten Stuetzpunkt-Markern).
                            // Phantom-Duplikat von erstem/letztem Punkt ist der
                            // uebliche Catmull-Rom-Trick dagegen.
                            if v.len() >= 4 {
                                let mut vv = Vec::with_capacity(v.len() + 2);
                                vv.push(v[0]);
                                vv.extend_from_slice(&v);
                                vv.push(*v.last().unwrap());
                                d.draw_spline_catmull_rom(&vv, thick * s as f32, *col);
                            }
                            else {
                                for i in 0..v.len() - 1 {
                                    d.draw_line_ex(v[i], v[i + 1], thick * s as f32, *col);
                                }
                            }
                        }
                    }
                    Cmd::BlendMode(m) => {
                        // Folgende Draws bis zum naechsten BlendMode/Frame-Ende.
                        // m == 0 (alpha) => zurueck auf Default (EndBlendMode).
                        unsafe {
                            if *m == 0 { raylib::ffi::EndBlendMode(); }
                            else { raylib::ffi::BeginBlendMode(*m); }
                        }
                        cur_blend = *m;
                    }
                    Cmd::RtDraw(i, x, y, scale, tint, flip_v) => {
                        if let Some(rtgt) = render_targets.get(*i) {
                            let tex = rtgt.rt.texture();   // &WeakTexture2D
                            let tw = tex.width as f32; let th = tex.height as f32;
                            // RenderTexture ist y-gespiegelt -> normalerweise negative
                            // Quell-Hoehe (aufrecht). flip_v=true laesst die Spiegelung
                            // stehen -> vertikal gespiegelte Ausgabe (Boden-Reflexion).
                            let src_h = if *flip_v { th } else { -th };
                            let src = Rectangle::new(0.0, 0.0, tw, src_h);
                            let dst = Rectangle::new((x * s) as f32, (y * s) as f32,
                                tw * scale * s as f32, th * scale * s as f32);
                            d.draw_texture_pro(tex, src, dst, Vector2::zero(), 0.0, *tint);
                        }
                    }
                    Cmd::ScissorPush(x, y, w, h) => {
                        // Logische -> Screen-Pixel, dann mit aktuellem Clip schneiden.
                        let mut rx = x * s; let mut ry = y * s;
                        let mut rw = w * s; let mut rh = h * s;
                        if let Some(&(cx, cy, cw, ch)) = clip_stack.last() {
                            let nx = rx.max(cx); let ny = ry.max(cy);
                            let nx2 = (rx + rw).min(cx + cw); let ny2 = (ry + rh).min(cy + ch);
                            rx = nx; ry = ny; rw = (nx2 - nx).max(0); rh = (ny2 - ny).max(0);
                        }
                        clip_stack.push((rx, ry, rw, rh));
                        unsafe { raylib::ffi::BeginScissorMode(rx, ry, rw, rh); }
                    }
                    Cmd::ScissorPop => {
                        clip_stack.pop();
                        match clip_stack.last() {
                            Some(&(rx, ry, rw, rh)) => unsafe { raylib::ffi::BeginScissorMode(rx, ry, rw, rh); },
                            None => unsafe { raylib::ffi::EndScissorMode(); },
                        }
                    }
                }
              }
            }
            // Sicherheit: unbalancierte Clips/Blend nicht in den naechsten Frame lecken.
            if !clip_stack.is_empty() { unsafe { raylib::ffi::EndScissorMode(); } }
            if cur_blend != 0 { unsafe { raylib::ffi::EndBlendMode(); } }
}


// === Radiance-`.hdr`-(RGBE)-Loader ===
// raylib-sys 5.5.1 ist ohne SUPPORT_FILEFORMAT_HDR gebaut, daher dekodieren wir
// das equirectangulare HDR-Panorama selbst zu RGBA32F (row-major, A=1) fuer
// LIGHT_ENV_HDR. Unterstuetzt die neue (2,2,..) und alte/flache RLE-Form.

fn rgbe_to_float(r: u8, g: u8, b: u8, e: u8) -> (f32, f32, f32) {
    if e == 0 { return (0.0, 0.0, 0.0); }
    let f = (2.0f32).powi(e as i32 - (128 + 8));
    (r as f32 * f, g as f32 * f, b as f32 * f)
}

/// Dekodiert eine Scanline (W Pixel RGBE) -> `scan` (W*4 Bytes).
fn hdr_decode_scanline(b: &[u8], pos: &mut usize, w: usize, scan: &mut [u8]) -> Result<(), String> {
    if *pos + 4 > b.len() { return Err("Scanline-Ende".into()); }
    let (h0, h1, h2, h3) = (b[*pos], b[*pos + 1], b[*pos + 2], b[*pos + 3]);
    let new_rle = h0 == 2 && h1 == 2 && (((h2 as usize) << 8) | h3 as usize) == w
        && (8..=0x7fff).contains(&w);
    if !new_rle {
        // Flache RGBE-Pixel (mit optionaler alter RLE: (1,1,1,n) wiederholt vorigen).
        let mut x = 0usize;
        let mut prev = [0u8; 4];
        let mut shift = 0u32;
        while x < w {
            if *pos + 4 > b.len() { return Err("vorzeitiges Ende (flat)".into()); }
            let px = [b[*pos], b[*pos + 1], b[*pos + 2], b[*pos + 3]];
            *pos += 4;
            if px[0] == 1 && px[1] == 1 && px[2] == 1 {
                let count = (px[3] as usize) << shift;
                for _ in 0..count {
                    if x >= w { break; }
                    scan[x * 4..x * 4 + 4].copy_from_slice(&prev);
                    x += 1;
                }
                shift += 8;
            } else {
                scan[x * 4..x * 4 + 4].copy_from_slice(&px);
                prev = px;
                x += 1;
                shift = 0;
            }
        }
        return Ok(());
    }
    *pos += 4; // RLE-Header ueberspringen
    // Neue RLE: 4 Kanaele getrennt, je W Bytes (Run wenn count > 128).
    for ch in 0..4 {
        let mut x = 0usize;
        while x < w {
            if *pos >= b.len() { return Err("RLE-Ende".into()); }
            let count = b[*pos];
            *pos += 1;
            if count > 128 {
                let n = (count - 128) as usize;
                if *pos >= b.len() { return Err("RLE-Run-Ende".into()); }
                let val = b[*pos];
                *pos += 1;
                for _ in 0..n {
                    if x >= w { return Err("RLE-Ueberlauf".into()); }
                    scan[x * 4 + ch] = val;
                    x += 1;
                }
            } else {
                for _ in 0..count as usize {
                    if x >= w || *pos >= b.len() { return Err("RLE-Literal-Ende".into()); }
                    scan[x * 4 + ch] = b[*pos];
                    *pos += 1;
                    x += 1;
                }
            }
        }
    }
    Ok(())
}

/// Laedt ein Radiance-`.hdr` -> (RGBA32F-Pixel, Breite, Hoehe).
fn load_hdr_rgbe(path: &str) -> Result<(Vec<f32>, i32, i32), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("nicht lesbar: {}", e))?;
    let mut pos = 0usize;
    let read_line = |b: &[u8], p: &mut usize| -> String {
        let start = *p;
        while *p < b.len() && b[*p] != b'\n' { *p += 1; }
        let s = String::from_utf8_lossy(&b[start..*p]).to_string();
        if *p < b.len() { *p += 1; }
        s
    };
    let magic = read_line(&bytes, &mut pos);
    if !magic.starts_with("#?") { return Err("kein Radiance-Header (#?RADIANCE)".into()); }
    // Header-Zeilen bis Leerzeile.
    loop {
        if pos >= bytes.len() { return Err("unerwartetes Ende im Header".into()); }
        let line = read_line(&bytes, &mut pos);
        if line.is_empty() { break; }
    }
    // Aufloesungszeile, ueblich "-Y H +X W".
    let res = read_line(&bytes, &mut pos);
    let parts: Vec<&str> = res.split_whitespace().collect();
    if parts.len() != 4 { return Err(format!("unerwartete Aufloesungszeile '{}'", res)); }
    let height: i32 = parts[1].parse().map_err(|_| "Hoehe ungueltig".to_string())?;
    let width: i32 = parts[3].parse().map_err(|_| "Breite ungueltig".to_string())?;
    if width <= 0 || height <= 0 { return Err("ungueltige Dimension".into()); }
    let top_down = parts[0] == "-Y"; // Standard: erste Scanline = oben
    let (w, h) = (width as usize, height as usize);
    let mut out = vec![0f32; w * h * 4];
    let mut scan = vec![0u8; w * 4];
    for y in 0..h {
        hdr_decode_scanline(&bytes, &mut pos, w, &mut scan)?;
        // GL-Texturen erwarten Zeile 0 = unten; raylib-Bilder Zeile 0 = oben.
        // top_down (-Y): erste Scanline ist oben -> nach unten gespiegelt ablegen.
        let row = if top_down { h - 1 - y } else { y };
        for x in 0..w {
            let (fr, fg, fb) = rgbe_to_float(scan[x*4], scan[x*4+1], scan[x*4+2], scan[x*4+3]);
            let o = (row * w + x) * 4;
            out[o] = fr; out[o+1] = fg; out[o+2] = fb; out[o+3] = 1.0;
        }
    }
    Ok((out, width, height))
}

/// SDL/pygame-Keycode (Wert der GB-KEY_*-Konstanten) -> raylib KeyboardKey.
/// Randbreite fuer 9-Slice, auf ein sinnvolles Mass gestutzt.
///
/// Der Rand darf nie mehr als die halbe Bild- ODER Zielseite belegen: sonst
/// ueberlappten sich gegenueberliegende Ecken und die Mittelstuecke haetten
/// negative Groesse. Das trifft genau dann zu, wenn ein Widget kleiner wird
/// als seine Skin-Raender -- und dort soll es zusammenschrumpfen, nicht
/// kaputtgehen.
fn neun_rand(bw: i32, bh: i32, w: i32, h: i32, rand: i32) -> i32 {
    rand.max(0).min(bw / 2).min(bh / 2).min(w / 2).min(h / 2)
}

/// Die drei Abschnitte einer Achse: Anfang fest, Mitte gedehnt, Ende fest.
/// Liefert (Positionen, Laengen).
fn neun_spannen(start: i32, laenge: i32, r: i32) -> ([i32; 3], [i32; 3]) {
    ([start, start + r, start + laenge - r], [r, laenge - 2 * r, r])
}

/// Seitlicher Einzug einer Zeile in einem runden Rechteck.
///
/// Fuer `Cmd::RoundGradient`: die Flaeche wird zeilenweise gefuellt, und in
/// den obersten/untersten `r` Zeilen muss der Streifen um den Eckenbogen
/// eingerueckt werden. `dy` ist der senkrechte Abstand zur Mitte des
/// Eckenkreises (Zeilenmitte, daher die 0.5), der Einzug folgt daraus per
/// Kreisgleichung.
fn ecken_einzug(r: i32, h: i32, zeile: i32) -> i32 {
    if r <= 0 {
        return 0;
    }
    let dy = if zeile < r {
        (r - zeile) as f32 - 0.5
    } else if zeile >= h - r {
        (zeile - (h - r)) as f32 + 0.5
    } else {
        return 0;
    };
    if dy <= 0.0 {
        return 0;
    }
    let k = ((r * r) as f32 - dy * dy).max(0.0).sqrt();
    ((r as f32 - k).round() as i32).clamp(0, r)
}

fn map_key(code: i64) -> Option<KeyboardKey> {
    use KeyboardKey::*;
    Some(match code {
        27 => KEY_ESCAPE,
        13 => KEY_ENTER,
        32 => KEY_SPACE,
        9 => KEY_TAB,
        8 => KEY_BACKSPACE,
        127 => KEY_DELETE,
        1073741904 => KEY_LEFT,
        1073741903 => KEY_RIGHT,
        1073741906 => KEY_UP,
        1073741905 => KEY_DOWN,
        1073741898 => KEY_HOME,
        1073741901 => KEY_END,
        1073741897 => KEY_INSERT,
        1073741899 => KEY_PAGE_UP,
        1073741902 => KEY_PAGE_DOWN,
        1073741881 => KEY_CAPS_LOCK,
        // Umschalt-/Steuertasten (SDL-Keycodes 224..230 | Scancode-Maske)
        1073742048 => KEY_LEFT_CONTROL,
        1073742049 => KEY_LEFT_SHIFT,
        1073742050 => KEY_LEFT_ALT,
        1073742051 => KEY_LEFT_SUPER,
        1073742052 => KEY_RIGHT_CONTROL,
        1073742053 => KEY_RIGHT_SHIFT,
        1073742054 => KEY_RIGHT_ALT,
        1073742055 => KEY_RIGHT_SUPER,
        // Ziffernblock: SDL zaehlt KP_1..KP_9 aufsteigend, KP_0 kommt DANACH.
        1073741908 => KEY_KP_DIVIDE,
        1073741909 => KEY_KP_MULTIPLY,
        1073741910 => KEY_KP_SUBTRACT,
        1073741911 => KEY_KP_ADD,
        1073741912 => KEY_KP_ENTER,
        1073741913 => KEY_KP_1, 1073741914 => KEY_KP_2, 1073741915 => KEY_KP_3,
        1073741916 => KEY_KP_4, 1073741917 => KEY_KP_5, 1073741918 => KEY_KP_6,
        1073741919 => KEY_KP_7, 1073741920 => KEY_KP_8, 1073741921 => KEY_KP_9,
        1073741922 => KEY_KP_0,
        1073741923 => KEY_KP_DECIMAL,
        // Buchstaben. pygame zaehlt sie als KLEINbuchstaben (97..122), raylib
        // als GROSSE (65..90) -- beide Schreibweisen werden angenommen.
        //
        // Frueher galten nur die kleinen, und `KEYHIT(ASC("S"))` traf still
        // GAR NICHTS: kein Fehler, keine Warnung, die Taste existierte fuer
        // das Programm einfach nicht. Genau darueber ist der Tilemap-Editor
        // gestolpert -- jedes seiner Tastenkuerzel war tot, und weil ein
        // nicht reagierendes Kuerzel wie ein vergessener Aufruf aussieht,
        // sucht man den Fehler im eigenen Programm. Ein Bereich, der ohnehin
        // auf `None` lief, kann durch Annehmen niemand brechen.
        97..=122 => return key_from_i32((code - 32) as i32),
        65..=90 => return key_from_i32(code as i32),
        // Satzzeichen (SDL-Keycodes). Ohne sie lief `KEYHIT(ASC("-"))` ins
        // Leere: die Taste existierte fuer Drachenhauch schlicht nicht, ohne
        // Fehlermeldung. raylib benennt die Tasten nach ihrer PHYSISCHEN Lage
        // im US-Layout -- auf anderen Belegungen sitzt das Zeichen also
        // moeglicherweise woanders (deshalb sind Ziffern und Buchstaben die
        // verlaesslichere Wahl fuer Steuertasten).
        39 => KEY_APOSTROPHE,
        44 => KEY_COMMA,
        45 => KEY_MINUS,
        46 => KEY_PERIOD,
        47 => KEY_SLASH,
        59 => KEY_SEMICOLON,
        // SDLK_PLUS (43) hat in raylib keine eigene Taste -- auf US-Layout
        // entsteht "+" als Umschalt+"=", darum dieselbe Taste wie 61.
        43 | 61 => KEY_EQUAL,
        91 => KEY_LEFT_BRACKET,
        92 => KEY_BACKSLASH,
        93 => KEY_RIGHT_BRACKET,
        96 => KEY_GRAVE,
        // Ziffern: pygame 48..57 == raylib KEY_ZERO..KEY_NINE.
        48..=57 => return key_from_i32(code as i32),
        // F1..F12: pygame 1073741882.. -> raylib KEY_F1=290..
        1073741882..=1073741893 => return key_from_i32((290 + (code - 1073741882)) as i32),
        _ => return None,
    })
}

fn key_from_i32(v: i32) -> Option<KeyboardKey> {
    // raylib-rs bietet kein from_i32 stabil ueber alle Versionen; wir mappen
    // die hier benoetigten Bereiche (Buchstaben/Ziffern/F-Keys) explizit.
    use KeyboardKey::*;
    Some(match v {
        65 => KEY_A, 66 => KEY_B, 67 => KEY_C, 68 => KEY_D, 69 => KEY_E, 70 => KEY_F,
        71 => KEY_G, 72 => KEY_H, 73 => KEY_I, 74 => KEY_J, 75 => KEY_K, 76 => KEY_L,
        77 => KEY_M, 78 => KEY_N, 79 => KEY_O, 80 => KEY_P, 81 => KEY_Q, 82 => KEY_R,
        83 => KEY_S, 84 => KEY_T, 85 => KEY_U, 86 => KEY_V, 87 => KEY_W, 88 => KEY_X,
        89 => KEY_Y, 90 => KEY_Z,
        48 => KEY_ZERO, 49 => KEY_ONE, 50 => KEY_TWO, 51 => KEY_THREE, 52 => KEY_FOUR,
        53 => KEY_FIVE, 54 => KEY_SIX, 55 => KEY_SEVEN, 56 => KEY_EIGHT, 57 => KEY_NINE,
        290 => KEY_F1, 291 => KEY_F2, 292 => KEY_F3, 293 => KEY_F4, 294 => KEY_F5,
        295 => KEY_F6, 296 => KEY_F7, 297 => KEY_F8, 298 => KEY_F9, 299 => KEY_F10,
        300 => KEY_F11, 301 => KEY_F12,
        _ => return None,
    })
}

/// Umkehrung von `map_key`: raylib-Taste -> GB-Tastencode (SDL-Konvention).
/// Buchstaben/Ziffern/F-Tasten rechnet die Nummerierung selbst um, alles
/// andere kommt aus der Tabelle -- so muss hier KEIN roher raylib-Zahlenwert
/// geraten werden (die Enum-Variante ist die Quelle).
fn gb_key_code(k: KeyboardKey) -> Option<i64> {
    use KeyboardKey::*;
    let v = k as u32 as i64;
    match v {
        65..=90 => return Some(v + 32),        // A..Z -> 97..122 (SDL: klein)
        48..=57 => return Some(v),             // 0..9 identisch
        290..=301 => return Some(1073741882 + (v - 290)),   // F1..F12
        _ => {}
    }
    Some(match k {
        KEY_ESCAPE => 27, KEY_ENTER => 13, KEY_SPACE => 32, KEY_TAB => 9,
        KEY_BACKSPACE => 8, KEY_DELETE => 127, KEY_INSERT => 1073741897,
        KEY_LEFT => 1073741904, KEY_RIGHT => 1073741903,
        KEY_UP => 1073741906, KEY_DOWN => 1073741905,
        KEY_HOME => 1073741898, KEY_END => 1073741901,
        KEY_PAGE_UP => 1073741899, KEY_PAGE_DOWN => 1073741902,
        KEY_CAPS_LOCK => 1073741881,
        KEY_LEFT_CONTROL => 1073742048, KEY_LEFT_SHIFT => 1073742049,
        KEY_LEFT_ALT => 1073742050, KEY_LEFT_SUPER => 1073742051,
        KEY_RIGHT_CONTROL => 1073742052, KEY_RIGHT_SHIFT => 1073742053,
        KEY_RIGHT_ALT => 1073742054, KEY_RIGHT_SUPER => 1073742055,
        KEY_KP_DIVIDE => 1073741908, KEY_KP_MULTIPLY => 1073741909,
        KEY_KP_SUBTRACT => 1073741910, KEY_KP_ADD => 1073741911,
        KEY_KP_ENTER => 1073741912, KEY_KP_DECIMAL => 1073741923,
        KEY_KP_1 => 1073741913, KEY_KP_2 => 1073741914, KEY_KP_3 => 1073741915,
        KEY_KP_4 => 1073741916, KEY_KP_5 => 1073741917, KEY_KP_6 => 1073741918,
        KEY_KP_7 => 1073741919, KEY_KP_8 => 1073741920, KEY_KP_9 => 1073741921,
        KEY_KP_0 => 1073741922,
        _ => return None,
    })
}

/// Anzeigename der Tasten, fuer die GLFW keinen liefert (alles Nicht-
/// Druckbare). Bewusst kurze, in Spielen uebliche Beschriftungen; leer, wenn
/// auch hier nichts Sinnvolles steht (dann zeigt der Aufrufer den Code).
fn key_label(k: KeyboardKey) -> &'static str {
    use KeyboardKey::*;
    match k {
        KEY_SPACE => "LEER", KEY_ENTER | KEY_KP_ENTER => "ENTER", KEY_ESCAPE => "ESC",
        KEY_TAB => "TAB", KEY_BACKSPACE => "RUECK", KEY_DELETE => "ENTF",
        KEY_INSERT => "EINFG", KEY_HOME => "POS1", KEY_END => "ENDE",
        KEY_PAGE_UP => "BILD-AUF", KEY_PAGE_DOWN => "BILD-AB",
        KEY_LEFT => "LINKS", KEY_RIGHT => "RECHTS", KEY_UP => "HOCH", KEY_DOWN => "RUNTER",
        KEY_LEFT_SHIFT | KEY_RIGHT_SHIFT => "UMSCHALT",
        KEY_LEFT_CONTROL | KEY_RIGHT_CONTROL => "STRG",
        KEY_LEFT_ALT | KEY_RIGHT_ALT => "ALT",
        KEY_LEFT_SUPER | KEY_RIGHT_SUPER => "SUPER",
        KEY_CAPS_LOCK => "FESTSTELL", KEY_NUM_LOCK => "NUM", KEY_SCROLL_LOCK => "ROLLEN",
        KEY_PRINT_SCREEN => "DRUCK", KEY_PAUSE => "PAUSE",
        KEY_F1 => "F1", KEY_F2 => "F2", KEY_F3 => "F3", KEY_F4 => "F4",
        KEY_F5 => "F5", KEY_F6 => "F6", KEY_F7 => "F7", KEY_F8 => "F8",
        KEY_F9 => "F9", KEY_F10 => "F10", KEY_F11 => "F11", KEY_F12 => "F12",
        _ => "",
    }
}

/// Restanteil eines CAMERA_SHAKE 1..0 (pure, fuer #[test]): linearer Abfall
/// ueber die Dauer, danach 0.
fn shake_remaining(elapsed_ms: f64, dur_ms: f64) -> f64 {
    if dur_ms <= 0.0 { return 0.0; }
    (1.0 - elapsed_ms / dur_ms).clamp(0.0, 1.0)
}

/// Dreht (px,py) um den Pivot (cx,cy) um `deg` Grad gegen den Uhrzeigersinn
/// (Standard-Mathe-Konvention; Screen-Y-nach-unten wird bewusst NICHT extra
/// gespiegelt -- w2s/s2w_*_rot sind sich beide konsistent in dieser
/// Konvention, siehe Kommentar an cam_rotation). Pure Funktion (fuer #[test]
/// ohne echte Graphics/raylib-Instanz).
fn rotate_point_around(px: f64, py: f64, cx: f64, cy: f64, deg: f64) -> (f64, f64) {
    let (s, c) = deg.to_radians().sin_cos();
    let dx = px - cx;
    let dy = py - cy;
    (cx + dx * c - dy * s, cy + dx * s + dy * c)
}

#[cfg(test)]
mod shake_tests {
    use super::shake_remaining;

    #[test]
    fn shake_decays_linearly_and_clamps() {
        assert_eq!(shake_remaining(0.0, 400.0), 1.0);
        assert!((shake_remaining(200.0, 400.0) - 0.5).abs() < 1e-9);
        assert_eq!(shake_remaining(400.0, 400.0), 0.0);
        assert_eq!(shake_remaining(999.0, 400.0), 0.0);   // ueber Ende
        assert_eq!(shake_remaining(10.0, 0.0), 0.0);      // dur=0 -> aus
    }
}

#[cfg(test)]
mod camera_rotation_tests {
    use super::rotate_point_around;

    #[test]
    fn zero_degrees_is_identity() {
        let (x, y) = rotate_point_around(37.0, -12.5, 100.0, 100.0, 0.0);
        assert!((x - 37.0).abs() < 1e-9);
        assert!((y - (-12.5)).abs() < 1e-9);
    }

    #[test]
    fn center_point_is_fixed_under_any_rotation() {
        // Der Pivot selbst bleibt immer an Ort und Stelle -- Grundlage dafuer,
        // dass CAMERA_FOLLOW's "target landet auf Bildschirm-Mitte" bei
        // aktiver Rotation weiterhin stimmt.
        for deg in [15.0, 90.0, 180.0, 271.0, -40.0] {
            let (x, y) = rotate_point_around(100.0, 100.0, 100.0, 100.0, deg);
            assert!((x - 100.0).abs() < 1e-9, "deg={deg}");
            assert!((y - 100.0).abs() < 1e-9, "deg={deg}");
        }
    }

    #[test]
    fn ninety_degrees_matches_hand_calculation() {
        // Weltpunkt liegt vor der Rotation (nach Translate+Zoom) bei
        // Screen-(50,0) auf einem 200x200-Screen (Pivot (100,100)).
        // -90 Grad ist die w2s-Konvention fuer CAMERA_SET_ROTATION(90).
        let (x, y) = rotate_point_around(50.0, 0.0, 100.0, 100.0, -90.0);
        assert!((x - 0.0).abs() < 1e-6, "x={x}");
        assert!((y - 150.0).abs() < 1e-6, "y={y}");
    }

    #[test]
    fn hundred_eighty_degrees_is_point_reflection() {
        let (x, y) = rotate_point_around(0.0, 0.0, 100.0, 100.0, 180.0);
        assert!((x - 200.0).abs() < 1e-6, "x={x}");
        assert!((y - 200.0).abs() < 1e-6, "y={y}");
    }

    #[test]
    fn forward_and_inverse_round_trip() {
        // w2s dreht um -cam_rotation, s2w_*_rot um +cam_rotation -- muss sich
        // exakt aufheben (Grundvoraussetzung fuer korrektes Maus-Picking).
        let (cx, cy) = (100.0, 100.0);
        let (px, py) = (17.0, 233.0);
        for deg in [0.0, 30.0, 90.0, 145.0, 260.0] {
            let (sx, sy) = rotate_point_around(px, py, cx, cy, -deg);
            let (ux, uy) = rotate_point_around(sx, sy, cx, cy, deg);
            assert!((ux - px).abs() < 1e-6, "deg={deg} ux={ux}");
            assert!((uy - py).abs() < 1e-6, "deg={deg} uy={uy}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ecken_einzug, neun_rand, neun_spannen};

    #[test]
    fn neun_spannen_deckt_die_achse_luecken_und_ueberlappungsfrei_ab() {
        let (pos, len) = neun_spannen(10, 100, 12);
        // Jeder Abschnitt beginnt, wo der vorige endet ...
        assert_eq!(pos[0] + len[0], pos[1]);
        assert_eq!(pos[1] + len[1], pos[2]);
        // ... und zusammen ergeben sie genau die Gesamtlaenge.
        assert_eq!(len.iter().sum::<i32>(), 100);
        assert_eq!(pos[2] + len[2], 110);
        // Aussen fest, innen der Rest.
        assert_eq!(len[0], 12);
        assert_eq!(len[2], 12);
        assert_eq!(len[1], 76);
    }

    #[test]
    fn neun_rand_verhindert_ueberlappende_ecken() {
        // Passt bequem -> unveraendert.
        assert_eq!(neun_rand(48, 48, 200, 60, 12), 12);
        // Ziel schmaler als zwei Raender -> auf die halbe Zielbreite gestutzt.
        assert_eq!(neun_rand(48, 48, 20, 60, 12), 10);
        // Auch das Quellbild deckelt.
        assert_eq!(neun_rand(16, 48, 200, 60, 12), 8);
        // Kein negativer Rand.
        assert_eq!(neun_rand(48, 48, 200, 60, -5), 0);
    }

    #[test]
    fn neun_teile_bleiben_bei_gestutztem_rand_gueltig() {
        // Der Fall, in dem es frueher kaputtging: Widget kleiner als die
        // Skin-Raender. Mit dem gestutzten Rand darf KEIN Stueck eine
        // negative Groesse bekommen.
        let r = neun_rand(48, 48, 20, 14, 12);
        for (start, laenge) in [(0, 20), (0, 14)] {
            let (_, len) = neun_spannen(start, laenge, r);
            for l in len {
                assert!(l >= 0, "negatives Teilstueck bei Rand {}: {:?}", r, len);
            }
            assert_eq!(len.iter().sum::<i32>(), laenge);
        }
    }

    #[test]
    fn ecken_einzug_rundet_oben_und_unten_gleich() {
        let (r, h) = (8, 40);
        // Die oberste und die unterste Zeile muessen gleich stark eingezogen
        // sein -- sonst sitzt der Verlauf sichtbar schief in der Form.
        for k in 0..r {
            assert_eq!(
                ecken_einzug(r, h, k),
                ecken_einzug(r, h, h - 1 - k),
                "Zeile {} oben != unten", k
            );
        }
    }

    #[test]
    fn ecken_einzug_ist_in_der_mitte_null() {
        let (r, h) = (8, 40);
        for zeile in r..(h - r) {
            assert_eq!(ecken_einzug(r, h, zeile), 0, "Zeile {} unnoetig eingerueckt", zeile);
        }
    }

    #[test]
    fn ecken_einzug_waechst_zur_kante_hin_monoton() {
        let (r, h) = (10, 60);
        let mut vorher = i32::MAX;
        for zeile in 0..r {
            let e = ecken_einzug(r, h, zeile);
            assert!(e <= vorher, "Zeile {}: Einzug waechst wieder ({} > {})", zeile, e, vorher);
            assert!(e <= r, "Einzug groesser als der Radius");
            vorher = e;
        }
        // In der obersten Zeile ist der Einzug r - sqrt(r - 1/4) -- NICHT der
        // volle Radius: die Zeilenmitte liegt bei 0.5, dort ist der Kreis
        // schon ein Stueck breit. Fuer r = 10 sind das 7.
        let erwartet = (r as f32 - (r as f32 - 0.25).sqrt()).round() as i32;
        assert_eq!(ecken_einzug(r, h, 0), erwartet, "oberste Zeile folgt nicht dem Kreisbogen");
        // Am Ende der Rundung ist er praktisch null.
        assert!(ecken_einzug(r, h, r - 1) <= 1, "Rundung endet zu spaet");
    }

    #[test]
    fn ecken_einzug_ohne_radius_ist_immer_null() {
        for zeile in 0..10 {
            assert_eq!(ecken_einzug(0, 10, zeile), 0);
        }
    }
}
