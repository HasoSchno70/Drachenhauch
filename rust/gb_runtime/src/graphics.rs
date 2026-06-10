//! raylib-Grafik-Backend (Schritt 4). Nur mit `--features graphics`.
//!
//! Modell: Draw-Builtins zeichnen NICHT sofort, sondern haengen ein `Cmd`
//! an eine Liste. `CLS` leert die Liste + merkt die Clear-Farbe. `FLIP`
//! rendert alle Cmds in einem `begin_drawing`/`end_drawing`-Block und
//! praesentiert. Das vermeidet, raylibs Draw-Handle ueber Builtin-Aufrufe
//! hinweg zu halten (Borrow-Checker) und braucht keine Render-Texture.
//!
//! Bit-Identitaet gilt hier NICHT fuer Pixel (anderer Renderer als pygame) --
//! nur `PRINT`/stdout bleibt bit-identisch. Verifikation per Screenshot.

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
    Poly(Vec<(i32, i32)>, Color, bool),       // points, color, closed
    FillPoly(Vec<(i32, i32)>, Color),
    // x, y, text, size, color, font_idx (-1 = Default), spacing
    Text(i32, i32, String, i32, Color, i64, f32),
    Texture(usize, i32, i32),
    TexturePart(usize, i32, i32, i32, i32, i32, i32), // tex, sx,sy,sw,sh, dx,dy
    TextureRect(usize, i32, i32, i32, i32),           // tex skaliert in dx,dy,dw,dh (bounds-safe)
    TextureFlipped(usize, i32, i32, bool, bool),       // tex, x, y, flip_h, flip_v
    AtlasDraw(usize, i32, i32, i32, i32, i32, i32, bool, Color), // tex, sx,sy,sw,sh, dx,dy, flip_h, tint
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
    BlendMode(i32),                                    // 0=alpha,1=additive,2=multiplied,4=subtract
    RtDraw(usize, i32, i32, f32, Color),               // render-target idx, x, y, scale, tint
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

/// Besitzt das von raylib `LoadModelAnimations` allokierte Array roh. Wir nutzen
/// die ffi direkt, weil der raylib-rs-Wrapper die Structs flach kopiert und dann
/// `UnloadModelAnimations` ruft (gibt bones/framePoses frei) -> Use-after-free.
/// Hier bleibt das Array am Leben und wird erst beim Drop sauber freigegeben.
struct AnimSet {
    ptr: *mut raylib::ffi::ModelAnimation,
    count: i32,
}
impl Drop for AnimSet {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { raylib::ffi::UnloadModelAnimations(self.ptr, self.count); }
        }
    }
}

pub struct Graphics {
    rl: RaylibHandle,
    thread: RaylibThread,
    width: i32,
    height: i32,
    scale: i32,
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
    // Kamera (Modul `camera`): World->Screen-Transform fuer alle Draws.
    cam_x: f64,
    cam_y: f64,
    cam_zoom: f64,
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
    // Skelett-Animationen (MODEL_LOAD_ANIMS): je Set ein rohes raylib-Array.
    model_anims: Vec<AnimSet>,
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
    active_font: i64,
    text_spacing: f32,
    textures: Vec<Tex>,
    image_cache: HashMap<String, i64>,
    atlases: Vec<Atlas>,
    pub frame_count: u64,
    max_frames: Option<u64>,
    screenshot: Option<String>,
    shot_taken: bool,
    // Post-Processing (Shader): die Szene wird in `scene_rt` gerendert und beim
    // FLIP per Fragment-Shader (Index in `shaders`) auf den Screen praesentiert.
    shaders: Vec<Shader>,
    post_shader_idx: Option<usize>,
    scene_rt: Option<RenderTexture2D>,
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
    /// Spiegelt das pygame-Lazy-Init des Tree-Walkers (LOADIMAGE etc. ohne SCREEN).
    /// Ein spaeteres SCREEN macht das Fenster via `reconfigure` sichtbar.
    pub fn new_headless() -> Graphics {
        Graphics::new_impl(64, 64, "GameBasic", 1, true)
    }

    pub fn new(width: i32, height: i32, title: &str, scale: i32) -> Graphics {
        Graphics::new_impl(width, height, title, scale, false)
    }

    fn new_impl(width: i32, height: i32, title: &str, scale: i32, hidden: bool) -> Graphics {
        let win_w = width * scale;
        let win_h = height * scale;
        // raylib loggt sonst seinen INFO-Startup-Spam auf stdout und verschmutzt
        // die Konsolen-Ausgabe (TW ist sauber). WARNING zeigt weiter echte
        // Warnungen/Fehler (z.B. fehlgeschlagenes Texture-Load).
        let (mut rl, thread) = raylib::init()
            .size(win_w, win_h)
            .title(title)
            .log_level(raylib::consts::TraceLogLevel::LOG_WARNING)
            .build();
        if hidden {
            rl.set_window_state(WindowState::default().set_window_hidden(true));
        }
        rl.set_target_fps(60);
        // Headless-Verifizierung: GBRT_FRAMES begrenzt die Frames, GBRT_SCREENSHOT
        // legt den PNG-Pfad fest (Screenshot beim letzten Frame).
        let max_frames = std::env::var("GBRT_FRAMES").ok().and_then(|s| s.parse().ok());
        let screenshot = std::env::var("GBRT_SCREENSHOT").ok();
        let mut layer_names = HashMap::new();
        layer_names.insert(String::new(), 0usize); // Main-Layer
        // Szene-Render-Target (Fenstergroesse) fuer Post-Processing.
        let scene_rt = rl.load_render_texture(&thread, win_w as u32, win_h as u32).ok();
        Graphics {
            rl, thread, width, height, scale,
            shaders: Vec::new(), post_shader_idx: None, scene_rt,
            layers: vec![Layer { z: 0, cmds: Vec::new() }],
            layer_names,
            active: 0,
            render_targets: Vec::new(),
            active_rt: None,
            clear_color: Color::BLACK,
            cam_x: 0.0, cam_y: 0.0, cam_zoom: 1.0,
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
            active_font: -1,
            text_spacing: 0.0,
            textures: Vec::new(),
            image_cache: HashMap::new(),
            atlases: Vec::new(),
            frame_count: 0,
            max_frames,
            screenshot,
            shot_taken: false,
        }
    }

    /// SCREEN nach einem Lazy-Init (oder erneutes SCREEN): das bestehende Fenster
    /// sichtbar machen und auf die gewuenschte Groesse/Titel umstellen, statt ein
    /// zweites raylib-Fenster zu erzeugen (raylib paniced bei Doppel-Init).
    pub fn reconfigure(&mut self, width: i32, height: i32, title: &str, scale: i32) {
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
    pub fn rendertarget_new(&mut self, w: i32, h: i32) -> Result<i64, String> {
        let rt = self.rl.load_render_texture(&self.thread, w.max(1) as u32, h.max(1) as u32)
            .map_err(|e| format!("RENDERTARGET_NEW: {}", e))?;
        self.render_targets.push(RenderTarget { rt, cmds: Vec::new() });
        Ok((self.render_targets.len() - 1) as i64)
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
    pub fn rendertarget_draw(&mut self, idx: i64, x: i32, y: i32, scale: f64, tint: Option<i64>) -> Result<(), String> {
        let i = self.check_rt(idx, "RENDERTARGET_DRAW")?;
        let (x, y) = self.w2s(x, y);
        let tcol = match tint { Some(c) => col(c), None => Color::WHITE };
        self.emit(Cmd::RtDraw(i, x, y, (scale * self.cam_zoom).max(0.0) as f32, tcol));
        Ok(())
    }

    // --- Kamera (Modul `camera`) ---
    // World->Screen: sx = int((x - cam_x) * zoom). Bei Identitaet (0,0,1) No-Op.
    fn w2s(&self, x: i32, y: i32) -> (i32, i32) {
        (((x as f64 - self.cam_x) * self.cam_zoom) as i32,
         ((y as f64 - self.cam_y) * self.cam_zoom) as i32)
    }
    fn ssize(&self, s: i32) -> i32 { ((s as f64 * self.cam_zoom) as i32).max(0) }
    pub fn set_camera(&mut self, x: f64, y: f64, zoom: f64) { self.cam_x = x; self.cam_y = y; self.cam_zoom = zoom; }
    pub fn reset_camera(&mut self) { self.cam_x = 0.0; self.cam_y = 0.0; self.cam_zoom = 1.0; }
    pub fn camera(&self) -> (f64, f64, f64) { (self.cam_x, self.cam_y, self.cam_zoom) }
    pub fn s2w_x(&self, sx: f64) -> f64 { if self.cam_zoom == 0.0 { sx } else { sx / self.cam_zoom + self.cam_x } }
    pub fn s2w_y(&self, sy: f64) -> f64 { if self.cam_zoom == 0.0 { sy } else { sy / self.cam_zoom + self.cam_y } }

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
        self.rl.update_camera(&mut self.cam3d, m);
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
        let c = std::ffi::CString::new(resolved.as_str())
            .map_err(|_| "MODEL_LOAD_ANIMS: ungueltiger Pfad".to_string())?;
        let mut count: i32 = 0;
        let ptr = unsafe { raylib::ffi::LoadModelAnimations(c.as_ptr(), &mut count) };
        if ptr.is_null() || count <= 0 {
            return Err(format!("MODEL_LOAD_ANIMS: '{}' enthaelt keine Animationen", path));
        }
        self.model_anims.push(AnimSet { ptr, count });
        Ok((self.model_anims.len() - 1) as i64)
    }
    fn check_anim(&self, set: i64, idx: i64, fn_: &str) -> Result<(usize, isize), String> {
        let s = set as usize;
        if set < 0 || s >= self.model_anims.len() {
            return Err(format!("{}: ungueltiges ANIM_SET-Handle {}", fn_, set));
        }
        let cnt = self.model_anims[s].count as i64;
        if idx < 0 || idx >= cnt {
            return Err(format!("{}: Animations-Index {} ausserhalb [0..{}]", fn_, idx, cnt - 1));
        }
        Ok((s, idx as isize))
    }
    /// Anzahl Animationen im Set.
    pub fn anim_count(&self, set: i64) -> Result<i64, String> {
        let s = set as usize;
        if set < 0 || s >= self.model_anims.len() {
            return Err(format!("MODEL_ANIM_COUNT: ungueltiges ANIM_SET-Handle {}", set));
        }
        Ok(self.model_anims[s].count as i64)
    }
    /// Frame-Anzahl einer Animation.
    pub fn anim_frames(&self, set: i64, idx: i64) -> Result<i64, String> {
        let (s, a) = self.check_anim(set, idx, "MODEL_ANIM_FRAMES")?;
        Ok(unsafe { (*self.model_anims[s].ptr.offset(a)).frameCount } as i64)
    }
    /// Name einer Animation (leer falls keiner gesetzt).
    pub fn anim_name(&self, set: i64, idx: i64) -> Result<String, String> {
        let (s, a) = self.check_anim(set, idx, "MODEL_ANIM_NAME")?;
        let raw = unsafe { (*self.model_anims[s].ptr.offset(a)).name };
        let bytes: Vec<u8> = raw.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    /// Setzt das Modell auf Frame `frame` der Animation `anim_idx` aus `set`.
    pub fn model_animate(&mut self, model_idx: i64, set: i64, anim_idx: i64, frame: i32) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_ANIMATE")?;
        let (s, a) = self.check_anim(set, anim_idx, "MODEL_ANIMATE")?;
        let anim = unsafe { *self.model_anims[s].ptr.offset(a) };   // ffi::ModelAnimation (Copy)
        let frames = anim.frameCount.max(1);
        let f = frame.rem_euclid(frames);                          // loopt automatisch
        let model_ffi = *self.models[mi].as_mut();                 // ffi::Model (Copy)
        unsafe { raylib::ffi::UpdateModelAnimation(model_ffi, anim, f); }
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
    pub fn mesh_cube(&mut self, w: f32, h: f32, d: f32) -> Result<i64, String> {
        let mesh = Mesh::gen_mesh_cube(&self.thread, w, h, d);
        self.push_model_from_mesh(mesh, "MESH_CUBE")
    }
    pub fn mesh_sphere(&mut self, r: f32, rings: i32, slices: i32) -> Result<i64, String> {
        let mesh = Mesh::gen_mesh_sphere(&self.thread, r, rings.max(3), slices.max(3));
        self.push_model_from_mesh(mesh, "MESH_SPHERE")
    }
    pub fn mesh_cylinder(&mut self, r: f32, h: f32, slices: i32) -> Result<i64, String> {
        let mesh = Mesh::gen_mesh_cylinder(&self.thread, r, h, slices.max(3));
        self.push_model_from_mesh(mesh, "MESH_CYLINDER")
    }
    pub fn mesh_torus(&mut self, r: f32, size: f32, rad_seg: i32, sides: i32) -> Result<i64, String> {
        let mesh = Mesh::gen_mesh_torus(&self.thread, r, size, rad_seg.max(3), sides.max(3));
        self.push_model_from_mesh(mesh, "MESH_TORUS")
    }
    pub fn mesh_knot(&mut self, r: f32, size: f32, rad_seg: i32, sides: i32) -> Result<i64, String> {
        let mesh = Mesh::gen_mesh_knot(&self.thread, r, size, rad_seg.max(3), sides.max(3));
        self.push_model_from_mesh(mesh, "MESH_KNOT")
    }
    pub fn mesh_plane(&mut self, w: f32, l: f32, res_x: i32, res_z: i32) -> Result<i64, String> {
        let mesh = Mesh::gen_mesh_plane(&self.thread, w, l, res_x.max(1), res_z.max(1));
        self.push_model_from_mesh(mesh, "MESH_PLANE")
    }
    /// Terrain-Mesh aus einer (Graustufen-)Image (LOADIMAGE-Handle): Helligkeit
    /// = Hoehe. size = (Breite, Hoehenskalierung, Tiefe) in Welt-Einheiten.
    pub fn mesh_heightmap(&mut self, tex_idx: i64, sx: f32, sy: f32, sz: f32) -> Result<i64, String> {
        let ti = tex_idx as usize;
        if tex_idx < 0 || ti >= self.textures.len() {
            return Err(format!("MESH_HEIGHTMAP: ungueltiges IMAGE-Handle {}", tex_idx));
        }
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
        let mut sh = self.rl.load_shader_from_memory(&self.thread, Some(INST_VS), Some(INST_FS));
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
        if tex_idx < 0 || ti >= self.textures.len() {
            return Err(format!("MODEL_TEXTURE: ungueltiges IMAGE-Handle {}", tex_idx));
        }
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
        if tex_idx < 0 || i >= self.textures.len() {
            return Err(format!("BILLBOARD: ungueltiges IMAGE-Handle {}", tex_idx));
        }
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
        let ray = Ray::new(Vector3::new(ox, oy, oz), Vector3::new(dx, dy, dz));
        let rc = get_ray_collision_sphere(ray, Vector3::new(cx, cy, cz), r);
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

    // --- Beleuchtung (Blinn-Phong via rlights-Shader) ---
    /// Laedt den Lighting-Shader (einmal) und aktiviert die Beleuchtung. Die
    /// Uniform-Locations fuer viewPos/ambient werden gecacht.
    pub fn light_enable(&mut self) {
        if self.light_shader.is_some() { return; }
        let mut sh = self.rl.load_shader_from_memory(&self.thread, Some(LIGHT_VS), Some(LIGHT_FS));
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
            (raylib::ffi::rlGetCullDistanceNear() as f32, raylib::ffi::rlGetCullDistanceFar() as f32)
        };
        Matrix::perspective(90.0_f32.to_radians(), 1.0, near, far).into()
    }

    /// Rendert eine einfache (1-Mip) Cubemap mit `fs` ueber die 6 Faces. Quelle ist
    /// eine 2D-Textur (equirect) oder eine Cubemap (irradiance). Liefert die GL-ID.
    fn ibl_render_cube(&mut self, fs: &str, src_id: u32, src_cubemap: bool, size: i32) -> Result<u32, String> {
        let sh = self.rl.load_shader_from_memory(&self.thread, Some(CUBEMAP_VS), Some(fs));
        let id = sh.id;
        if id == 0 { return Err("LIGHT_ENV_HDR: Cubemap-Shader nicht ladbar".into()); }
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
        let sh = self.rl.load_shader_from_memory(&self.thread, Some(CUBEMAP_VS), Some(PREFILTER_FS));
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
        let sh = self.rl.load_shader_from_memory(&self.thread, Some(BRDF_VS), Some(BRDF_FS));
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
        // 2) equirect -> Cubemap (512).
        let env = self.ibl_render_cube(EQUIRECT_FS, pano_id, false, 512)?;
        // 3) Irradiance-Cubemap (32, diffuse).
        let irradiance = self.ibl_render_cube(IRRADIANCE_FS, env, true, 32)?;
        // 4) Prefilter-Cubemap (128 + Roughness-Mips, specular).
        let prefilter = self.ibl_render_prefilter(env, 128)?;
        // 5) BRDF-LUT (512, 2D).
        let brdf = self.ibl_render_brdf(512)?;
        // Equirect freigeben; env-Cubemap fuer die Skybox aufbewahren.
        unsafe { raylib::ffi::rlUnloadTexture(pano_id); }
        self.ibl_env = env;
        self.ibl_irradiance = irradiance;
        self.ibl_prefilter = prefilter;
        self.ibl_brdf = brdf;
        self.use_ibl_maps = true;
        Ok(())
    }

    /// Skybox an/aus: zeichnet die env-Cubemap (von LIGHT_ENV_HDR) als 3D-
    /// Hintergrund. Ohne vorheriges LIGHT_ENV_HDR (ibl_env == 0) ein No-Op.
    pub fn skybox(&mut self, on: bool) {
        if on && self.skybox_shader.is_none() {
            let sh = self.rl.load_shader_from_memory(&self.thread, Some(SKYBOX_VS), Some(SKYBOX_FS));
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
            mat.as_mut().shader = sh_ffi;
        }
        Ok(())
    }
    /// Legt eine via LOADIMAGE geladene Textur als Normal-Map (MATERIAL_MAP_NORMAL).
    /// Aktiviert useNormalMap fuer dieses Modell.
    pub fn model_set_normal(&mut self, model_idx: i64, tex_idx: i64) -> Result<(), String> {
        let mi = self.check_model(model_idx, "MODEL_TEXTURE_NORMAL")?;
        let ti = tex_idx as usize;
        if tex_idx < 0 || ti >= self.textures.len() {
            return Err(format!("MODEL_TEXTURE_NORMAL: ungueltiges IMAGE-Handle {}", tex_idx));
        }
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
        if use_ibl {
            if loc_irr >= 0 { sh.set_shader_value(loc_irr, 11i32); }
            if loc_pre >= 0 { sh.set_shader_value(loc_pre, 12i32); }
            if loc_brdf >= 0 { sh.set_shader_value(loc_brdf, 13i32); }
        }
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
        // Der Szenen-Hintergrund ist IMMER deckend -- ein Alpha-Anteil (RGBA)
        // wuerde sonst beim PostFX/RenderTexture-Compositing die ganze Szene
        // durchscheinen lassen.
        let mut bg = col(color);
        bg.a = 255;
        self.clear_color = bg;
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

    // --- Blend-Modes (Batch 2) ---
    pub fn blend_mode(&mut self, mode: i32) { self.emit(Cmd::BlendMode(mode)); }

    // --- Prozedurale Texturen (Batch 3): liefern ein IMAGE-Handle ---
    pub fn gen_tex_perlin(&mut self, w: i32, h: i32, scale: f64) -> Result<i64, String> {
        // gen_image_perlin_noise ist in raylib-rs als &self-Methode gebunden
        // (self wird ignoriert) -> auf einem Wegwerf-Image aufrufen.
        let scratch = Image::gen_image_color(1, 1, Color::BLACK);
        let img = scratch.gen_image_perlin_noise(w.max(1), h.max(1), 0, 0, scale.max(0.1) as f32);
        self.push_tex_from_image(img)
    }
    pub fn gen_tex_gradient(&mut self, w: i32, h: i32, c1: i64, c2: i64, vertical: bool) -> Result<i64, String> {
        // direction in Grad: 0 = vertikal (oben->unten), 90 = horizontal.
        let dir = if vertical { 0 } else { 90 };
        let img = Image::gen_image_gradient_linear(w.max(1), h.max(1), dir, col(c1), col(c2));
        self.push_tex_from_image(img)
    }
    pub fn gen_tex_checked(&mut self, w: i32, h: i32, cx: i32, cy: i32, c1: i64, c2: i64) -> Result<i64, String> {
        let img = Image::gen_image_checked(w.max(1), h.max(1), cx.max(1), cy.max(1), col(c1), col(c2));
        self.push_tex_from_image(img)
    }
    pub fn gen_tex_color(&mut self, w: i32, h: i32, c: i64) -> Result<i64, String> {
        let img = Image::gen_image_color(w.max(1), h.max(1), col(c));
        self.push_tex_from_image(img)
    }

    // --- Clipboard + Drag&Drop (Batch 5) ---
    pub fn clipboard_get(&self) -> String { self.rl.get_clipboard_text().unwrap_or_default() }
    pub fn clipboard_set(&mut self, s: &str) { let _ = self.rl.set_clipboard_text(s); }
    pub fn files_dropped(&self) -> bool { self.rl.is_file_dropped() }
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
    pub fn arc(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, start: f64, end: f64, c: i64) {
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
        self.emit(Cmd::Poly(pts, col(c), false));
    }
    pub fn polygon(&mut self, flat: &[i32], c: i64, filled: bool) -> Result<(), String> {
        if flat.len() < 6 || flat.len() % 2 != 0 {
            return Err("POLYGON: braucht mindestens 3 Punkte (6 Werte)".into());
        }
        let pts: Vec<(i32, i32)> = flat.chunks(2).map(|p| { let (x, y) = self.w2s(p[0], p[1]); (x, y) }).collect();
        if filled { self.emit(Cmd::FillPoly(pts, col(c))); }
        else { self.emit(Cmd::Poly(pts, col(c), true)); }
        Ok(())
    }
    pub fn text(&mut self, x: i32, y: i32, s: String, c: i64) {
        // Position via w2s (inkl. Zoom), aber Font-Groesse bleibt -- wie Python.
        let (x, y) = self.w2s(x, y);
        let sz = self.text_size;
        let font = self.active_font;
        let spacing = self.text_spacing;
        self.emit(Cmd::Text(x, y, s, sz, col(c), font, spacing));
    }
    pub fn set_text_size(&mut self, sz: i32) { self.text_size = sz.max(1); }

    /// Text mit explizitem Font-Handle + Groesse (umgeht active_font/text_size).
    /// `font` = -1 -> Default-Font. Fuer per-Widget-Styling (Modul `gui`).
    pub fn text_styled(&mut self, x: i32, y: i32, s: String, c: i64, font: i64, size: i32) {
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::Text(x, y, s, size.max(1), col(c), font, self.text_spacing));
    }

    /// Laedt einen TTF/OTF-Font in der gegebenen Basis-Groesse -> FONT-Handle.
    pub fn load_font(&mut self, path: &str, size: i32) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        let f = self.rl.load_font_ex(&self.thread, path, size.max(4), None)
            .map_err(|e| format!("LOADFONT: Font '{}' nicht ladbar: {}", path, e))?;
        self.fonts.push(f);
        Ok((self.fonts.len() - 1) as i64)
    }
    /// Aktiven Font setzen (-1 = Default). Ungueltige Handles -> Fehler.
    pub fn set_font(&mut self, h: i64) -> Result<(), String> {
        if h < -1 || h >= self.fonts.len() as i64 {
            return Err(format!("SETFONT: ungueltiges FONT-Handle {}", h));
        }
        self.active_font = h;
        Ok(())
    }
    pub fn set_text_spacing(&mut self, px: i32) { self.text_spacing = px as f32; }

    pub fn text_width(&self, s: &str) -> i32 {
        if self.active_font >= 0 {
            if let Some(f) = self.fonts.get(self.active_font as usize) {
                return f.measure_text(s, self.text_size as f32, self.text_spacing).x as i32;
            }
        }
        let c = std::ffi::CString::new(s).unwrap_or_default();
        unsafe { raylib::ffi::MeasureText(c.as_ptr(), self.text_size) }
    }
    pub fn text_height(&self) -> i32 { self.text_size }

    pub fn load_texture(&mut self, path: &str) -> Result<i64, String> {
        let resolved = crate::builtins::resolve_asset_path(path);
        let path = resolved.as_str();
        if let Some(&h) = self.image_cache.get(path) { return Ok(h); }
        // CPU-Image laden (fuer imgfx) + GPU-Textur daraus.
        let img = Image::load_image(path).map_err(|e| format!("LOADIMAGE: {}", e))?;
        let tex = self.rl.load_texture_from_image(&self.thread, &img).map_err(|e| format!("LOADIMAGE: {}", e))?;
        self.textures.push(Tex { tex, img });
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
        Ok((self.textures.len() - 1) as i64)
    }

    fn src_image(&self, idx: i64, fn_: &str) -> Result<Image, String> {
        self.textures.get(idx as usize).map(|t| t.img.clone()).ok_or_else(|| format!("{}: ungueltiges IMAGE-Handle", fn_))
    }

    // --- imgfx (immutable: liefern neues IMAGE-Handle) ---
    pub fn image_scale(&mut self, idx: i64, w: i32, h: i32) -> Result<i64, String> {
        if w <= 0 || h <= 0 { return Err("IMAGE_SCALE: w und h muessen > 0 sein".into()); }
        let mut img = self.src_image(idx, "IMAGE_SCALE")?;
        img.resize(w, h);
        self.push_tex_from_image(img)
    }
    pub fn image_rotate(&mut self, idx: i64, degrees: f32) -> Result<i64, String> {
        let mut img = self.src_image(idx, "IMAGE_ROTATE")?;
        img.rotate(degrees as i32);
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
    fn cache_image_alias(&mut self, alias: &str, handle: i64) {
        self.image_cache.insert(alias.to_string(), handle);
    }
    pub fn draw_image(&mut self, idx: i64, x: i32, y: i32) -> Result<(), String> {
        let i = idx as usize;
        if i >= self.textures.len() { return Err("DRAWIMAGE: ungueltiges IMAGE-Handle".into()); }
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::Texture(i, x, y));
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
        if i >= self.textures.len() { return Err("DRAWIMAGEPART: ungueltiges IMAGE-Handle".into()); }
        let (dx, dy) = self.w2s(dx, dy);
        self.emit(Cmd::TexturePart(i, sx, sy, sw, sh, dx, dy));
        Ok(())
    }
    pub fn draw_image_flipped(&mut self, idx: i64, x: i32, y: i32, fh: bool, fv: bool) -> Result<(), String> {
        let i = idx as usize;
        if i >= self.textures.len() { return Err("DRAWIMAGEFLIPPED: ungueltiges IMAGE-Handle".into()); }
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::TextureFlipped(i, x, y, fh, fv));
        Ok(())
    }
    /// Zeichnet eine 2D-Tilemap (flache row-major `values`; Tile < 0 =
    /// transparent). Tileset wird als gerasterter Strip interpretiert
    /// (tiles_per_row = tileset_breite / tw). Jedes Tile geht durch
    /// `draw_image_part`, d.h. Camera (Translation + Zoom) wirkt korrekt --
    /// identisch zum Tree-Walker-Pfad.
    pub fn draw_tilemap(&mut self, idx: i64, values: &[i64], rows: i32, cols: i32,
                        tw: i32, th: i32, sx: i32, sy: i32) -> Result<(), String> {
        let i = idx as usize;
        if i >= self.textures.len() { return Err("DRAWTILEMAP: ungueltiges IMAGE-Handle".into()); }
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
        self.textures.get(idx as usize).map(|t| t.tex.width as i64).ok_or_else(|| "IMAGEWIDTH: ungueltiges IMAGE-Handle".into())
    }
    pub fn image_height(&self, idx: i64) -> Result<i64, String> {
        self.textures.get(idx as usize).map(|t| t.tex.height as i64).ok_or_else(|| "IMAGEHEIGHT: ungueltiges IMAGE-Handle".into())
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
                      tint: Option<i64>) -> Result<(), String> {
        let (tex, sx, sy, sw, sh) = {
            let a = self.atlases.get(atlas as usize).ok_or("ATLAS_DRAW: ungueltiges Atlas-Handle")?;
            let &(sx, sy, sw, sh) = a.frames.get(name)
                .ok_or_else(|| format!("ATLAS_DRAW: Sprite '{}' nicht im Atlas", name))?;
            (a.tex_idx, sx, sy, sw, sh)
        };
        let (x, y) = self.w2s(x, y);
        let tcol = match tint { Some(c) => col(c), None => Color::WHITE };
        self.emit(Cmd::AtlasDraw(tex, sx, sy, sw, sh, x, y, flip_h, tcol));
        Ok(())
    }

    /// SPRITE_DRAW: aktuelles Frame als Sheet-Sub-Rect, Camera-aware.
    pub fn draw_sprite(&mut self, tex_idx: i64, frame: i32, fw: i32, fh: i32,
                       x: i32, y: i32, flip_x: bool, flip_y: bool,
                       scale_x: f64, scale_y: f64, tint: Option<i64>) -> Result<(), String> {
        let i = tex_idx as usize;
        let tex_w = self.textures.get(i).map(|t| t.tex.width).ok_or("SPRITE_DRAW: ungueltiges IMAGE-Handle")?;
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
    /// folgt raylibs KeyboardKey-Enum, nicht SDL -- Eingabe ist ohnehin nicht
    /// Parity-relevant.)
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
    // Ein ungueltiger Joystick-INDEX wirft (wie der Tree-Walker), ein ungueltiger
    // Achsen-/Button-/Hat-Unterindex liefert dagegen 0/false (kein Fehler).
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
        use raylib::consts::GamepadButton::*;
        self.joystick_check(idx, "JOYSTICK_BUTTON")?;
        let b = match btn {
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
            _ => return Ok(false),
        };
        Ok(self.rl.is_gamepad_button_down(idx as i32, b))
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
        let btn = match b { 0 => MouseButton::MOUSE_BUTTON_LEFT, 1 => MouseButton::MOUSE_BUTTON_RIGHT, 2 => MouseButton::MOUSE_BUTTON_MIDDLE, _ => return false };
        self.rl.is_mouse_button_down(btn)
    }
    pub fn millis(&self) -> i64 { (self.rl.get_time() * 1000.0) as i64 }

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
    pub fn delta(&self) -> f64 { self.rl.get_frame_time() as f64 }
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
    pub fn set_fullscreen(&mut self, fs: bool) {
        if self.rl.is_window_fullscreen() != fs { self.rl.toggle_fullscreen(); }
    }

    // --- Natives OS-Fenster (das SCREEN-Fenster selbst) ---
    /// Das Programmfenster vom OS aus groessenveraenderbar machen (Default: aus).
    pub fn window_resizable(&mut self, f: bool) {
        let ws = WindowState::default().set_window_resizable(true);
        if f { self.rl.set_window_state(ws); } else { self.rl.clear_window_state(ws); }
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

    /// Clip-Rechteck auf den Stack legen (Scissor). Koordinaten werden wie bei
    /// allen Draws kamera-transformiert; der Screen-Scale kommt beim Replay.
    pub fn push_clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let (x, y) = self.w2s(x, y);
        let (w, h) = (self.ssize(w), self.ssize(h));
        self.emit(Cmd::ScissorPush(x, y, w, h));
    }
    pub fn pop_clip(&mut self) { self.emit(Cmd::ScissorPop); }

    // --- Shader / Post-Processing ---
    /// Laedt einen Fragment-Shader (GLSL-Quelltext) -> Handle (Index) oder -1.
    pub fn load_shader(&mut self, code: &str) -> i64 {
        let sh = self.rl.load_shader_from_memory(&self.thread, None, Some(code));
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
        // Schattenwerfendes Licht = erstes directional (kind 0). Sonst aus.
        let dir = match self.lights.iter().find(|l| l.kind == 0) {
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
        let Graphics { rl, thread, layers, textures, fonts, cmds3d, cam3d, models,
            light_shader, normal_mapped, loc_use_normal, pbr_params, loc_metalness, loc_roughness,
            emissive, loc_emissive,
            scene_rt, shaders, post_shader_idx, render_targets, .. } = self;
        let mat_locs = (*loc_use_normal, *loc_metalness, *loc_roughness, *loc_emissive);
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
                let mut tx = rl.begin_texture_mode(thread, &mut render_targets[i].rt);
                render_scene(&mut tx, s, clear_rt, &synth, &[0], textures, fonts,
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
                render_scene(&mut tx, s, clear_color, layers, &order, textures, fonts, cmds3d, cam, models, light_shader.as_mut(), mat_locs, nmap_set, pbr_ref, emis_ref, ibl, rts, skybox, cam_view, cam_proj, inst_ffi);
            }
            // 2) RT per Fragment-Shader auf den Screen praesentieren (Y-flip).
            let src = Rectangle::new(0.0, 0.0, tw, -th);
            let dst = Rectangle::new(0.0, 0.0, tw, th);
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            {
                let mut sm = d.begin_shader_mode(&mut shaders[idx]);
                sm.draw_texture_pro(&*rt, src, dst, Vector2::zero(), 0.0, Color::WHITE);
            }
        } else {
            let mut d = rl.begin_drawing(thread);
            render_scene(&mut d, s, clear_color, layers, &order, textures, fonts, cmds3d, cam, models, light_shader.as_mut(), mat_locs, nmap_set, pbr_ref, emis_ref, ibl, rts, skybox, cam_view, cam_proj, inst_ffi);
        }
        // Web (emscripten): nach dem Praesentieren (EndDrawing oben beim Drop des
        // Draw-Handles) ans Browser-Event-Loop yielden -- sonst blockiert der
        // GB-Render-Loop den Main-Thread und der Tab haengt. ASYNCIFY wickelt den
        // Stack ab; beim naechsten Frame geht es hier weiter.
        #[cfg(target_os = "emscripten")]
        unsafe { emscripten_sleep(0); }

        // Layer + 3D-Befehle fuer den naechsten Frame leeren (Immediate-Mode).
        for l in self.layers.iter_mut() { l.cmds.clear(); }
        self.cmds3d.clear();
        self.frame_count += 1;
        // Headless-Screenshot beim Erreichen der Frame-Grenze.
        if let (Some(mx), Some(path), false) = (self.max_frames, self.screenshot.clone(), self.shot_taken) {
            if self.frame_count >= mx {
                self.write_screenshot(&path);
                self.shot_taken = true;
            }
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
fn render_scene<D: RaylibDraw>(
    d: &mut D, s: i32, clear: Color,
    layers: &[Layer], order: &[usize], textures: &[Tex], fonts: &[Font],
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
    let mut set_material = |ls: &mut Option<&mut Shader>, idx: usize| {
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
    d.clear_background(clear);
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
                            d3.draw_line_3D(Vector3::new(*x1, *y1, *z1), Vector3::new(*x2, *y2, *z2), *col),
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
                        // raylib erwartet CCW; wir uebergeben wie angegeben.
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
                    Cmd::Poly(pts, col, closed) => {
                        let n = pts.len();
                        if n >= 2 {
                            for i in 0..n - 1 {
                                d.draw_line(pts[i].0 * s, pts[i].1 * s, pts[i + 1].0 * s, pts[i + 1].1 * s, *col);
                            }
                            if *closed {
                                d.draw_line(pts[n - 1].0 * s, pts[n - 1].1 * s, pts[0].0 * s, pts[0].1 * s, *col);
                            }
                        }
                    }
                    Cmd::FillPoly(pts, col) => {
                        // Triangle-Fan (korrekt fuer konvexe Polygone).
                        if pts.len() >= 3 {
                            let v: Vec<Vector2> = pts.iter().map(|p| Vector2::new((p.0 * s) as f32, (p.1 * s) as f32)).collect();
                            d.draw_triangle_fan(&v, *col);
                        }
                    }
                    Cmd::Text(x, y, txt, sz, col, font, spacing) => {
                        match fonts.get(*font as usize) {
                            Some(f) if *font >= 0 => d.draw_text_ex(
                                f, txt, Vector2::new((x * s) as f32, (y * s) as f32),
                                (sz * s) as f32, spacing * s as f32, *col),
                            _ => d.draw_text(txt, x * s, y * s, sz * s, *col),
                        }
                    }
                    Cmd::Texture(i, x, y) => {
                        if s == 1 { d.draw_texture(&textures[*i].tex, *x, *y, Color::WHITE); }
                        else { d.draw_texture_ex(&textures[*i].tex, Vector2::new((x * s) as f32, (y * s) as f32), 0.0, s as f32, Color::WHITE); }
                    }
                    Cmd::TexturePart(i, sx, sy, sw, sh, dx, dy) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32, *sw as f32, *sh as f32);
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (sw * s) as f32, (sh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::TextureRect(i, dx, dy, dw, dh) => {
                        if let Some(t) = textures.get(*i) {
                            let src = Rectangle::new(0.0, 0.0, t.tex.width as f32, t.tex.height as f32);
                            let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                            d.draw_texture_pro(&t.tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                        }
                    }
                    Cmd::TextureFlipped(i, x, y, fh, fv) => {
                        let t = &textures[*i].tex;
                        let sw = if *fh { -(t.width as f32) } else { t.width as f32 };
                        let sh = if *fv { -(t.height as f32) } else { t.height as f32 };
                        let src = Rectangle::new(0.0, 0.0, sw, sh);
                        let dst = Rectangle::new((x * s) as f32, (y * s) as f32, (t.width * s) as f32, (t.height * s) as f32);
                        d.draw_texture_pro(t, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::AtlasDraw(i, sx, sy, sw, sh, dx, dy, fh, tint) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32, if *fh { -(*sw as f32) } else { *sw as f32 }, *sh as f32);
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (sw * s) as f32, (sh * s) as f32);
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
                            if v.len() >= 4 { d.draw_spline_catmull_rom(&v, thick * s as f32, *col); }
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
                    Cmd::RtDraw(i, x, y, scale, tint) => {
                        if let Some(rtgt) = render_targets.get(*i) {
                            let tex = rtgt.rt.texture();   // &WeakTexture2D
                            let tw = tex.width as f32; let th = tex.height as f32;
                            // RenderTexture ist y-gespiegelt -> negative Quell-Hoehe.
                            let src = Rectangle::new(0.0, 0.0, tw, -th);
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
fn map_key(code: i64) -> Option<KeyboardKey> {
    use KeyboardKey::*;
    Some(match code {
        27 => KEY_ESCAPE,
        13 => KEY_ENTER,
        32 => KEY_SPACE,
        9 => KEY_TAB,
        8 => KEY_BACKSPACE,
        1073741904 => KEY_LEFT,
        1073741903 => KEY_RIGHT,
        1073741906 => KEY_UP,
        1073741905 => KEY_DOWN,
        // Buchstaben: pygame 97..122 (lowercase ascii) -> raylib 65..90.
        97..=122 => return key_from_i32((code - 32) as i32),
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
