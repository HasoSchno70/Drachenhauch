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

use raylib::prelude::*;
use raylib::core::shaders::RaylibShader;   // get_shader_location auf Shader

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
    TextureFlipped(usize, i32, i32, bool, bool),       // tex, x, y, flip_h, flip_v
    AtlasDraw(usize, i32, i32, i32, i32, i32, i32, bool), // tex, sx,sy,sw,sh, dx,dy, flip_h
    // tex, src(sx,sy,sw,sh), dst(dx,dy,dw,dh), flip_x, flip_y, tint
    SpriteDraw(usize, i32, i32, i32, i32, i32, i32, i32, i32, bool, bool, Color),
    // Clip-Stack (Scissor): Push schneidet mit dem aktuellen Clip, Pop stellt
    // den vorigen wieder her. Koordinaten logisch (pre-scale), beim Replay * s.
    ScissorPush(i32, i32, i32, i32),
    ScissorPop,
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
    // Billboard: Textur (Index), die immer zur Kamera zeigt. idx, x,y,z, size, tint
    Billboard(usize, f32, f32, f32, f32, Color),
}

struct Layer {
    z: i32,
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
uniform vec4 fogColor;
uniform float fogDensity;
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
    vec3 color = ambient.rgb*albedo + Lo;
    color = color/(color + vec3(1.0));        // Reinhard-Tonemapping
    color = pow(color, vec3(1.0/2.2));         // Gamma
    finalColor = vec4(color, 1.0);
    // Exponentieller Tiefen-Fog (fogDensity 0 => kein Effekt).
    float fd = length(viewPos - fragPosition)*fogDensity;
    float fog = clamp(1.0/exp(fd*fd), 0.0, 1.0);
    finalColor = mix(fogColor, finalColor, fog);
}
"#;

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
    clear_color: Color,
    // Kamera (Modul `camera`): World->Screen-Transform fuer alle Draws.
    cam_x: f64,
    cam_y: f64,
    cam_zoom: f64,
    // 3D (Modul `g3d`): Befehlsliste + Perspektiv-Kamera. cmds3d wird pro
    // FLIP geleert; cam3d wird von CAMERA3D gesetzt (sonst Default-Blick).
    cmds3d: Vec<Cmd3D>,
    cam3d: Camera3D,
    // 3D-Modelle (LOADMODEL / MESH_*): bleiben ueber Frames erhalten.
    models: Vec<Model>,
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
    lights: Vec<LightData>,
    light_ambient: [f32; 4],
    light_fog: [f32; 4],
    light_fog_density: f32,
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
    Color::new(((v >> 16) & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, (v & 0xFF) as u8, 255)
}

impl Graphics {
    pub fn new(width: i32, height: i32, title: &str, scale: i32) -> Graphics {
        let win_w = width * scale;
        let win_h = height * scale;
        let (mut rl, thread) = raylib::init().size(win_w, win_h).title(title).build();
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
            clear_color: Color::BLACK,
            cam_x: 0.0, cam_y: 0.0, cam_zoom: 1.0,
            cmds3d: Vec::new(),
            models: Vec::new(),
            light_shader: None,
            normal_mapped: std::collections::HashSet::new(),
            loc_use_normal: -1,
            pbr_params: std::collections::HashMap::new(),
            loc_metalness: -1,
            loc_roughness: -1,
            lights: Vec::new(),
            light_ambient: [0.1, 0.1, 0.1, 1.0],
            light_fog: [0.0, 0.0, 0.0, 1.0],
            light_fog_density: 0.0,
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
            // Default-Blick: schraeg von vorn-oben auf den Ursprung.
            cam3d: Camera3D::perspective(
                Vector3::new(6.0, 5.0, 6.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                45.0),
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

    fn emit(&mut self, c: Cmd) {
        let a = self.active;
        self.layers[a].cmds.push(c);
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
    }
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
        let m = self.rl.load_model(&self.thread, path)
            .map_err(|e| format!("LOADMODEL: '{}' nicht ladbar: {}", path, e))?;
        self.models.push(m);
        Ok((self.models.len() - 1) as i64)
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
        self.light_shader = Some(sh);
    }
    pub fn light_fog(&mut self, col_: i64, density: f64) {
        let v = col_ as u32;
        self.light_fog = [((v >> 16) & 0xFF) as f32 / 255.0, ((v >> 8) & 0xFF) as f32 / 255.0,
                          (v & 0xFF) as f32 / 255.0, 1.0];
        self.light_fog_density = density.max(0.0) as f32;
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
    /// Haengt den Lighting-Shader an alle Materialien eines Modells und erzeugt
    /// die Tangenten (Voraussetzung fuer Normal-Mapping via MODEL_TEXTURE_NORMAL).
    pub fn model_lit(&mut self, model_idx: i64) -> Result<(), String> {
        if self.light_shader.is_none() {
            return Err("MODEL_LIT: zuerst LIGHT_ENABLE() / ein Licht hinzufuegen".into());
        }
        let mi = self.check_model(model_idx, "MODEL_LIT")?;
        // Tangenten erzeugen (sonst ist die TBN-Basis im Shader degeneriert).
        for m in self.models[mi].meshes_mut() {
            m.gen_mesh_tangents(&self.thread);
        }
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
        for (le, lt, lp, ltg, lc, en, kind, pos, target, color) in lights {
            if le >= 0 { sh.set_shader_value(le, en); }
            if lt >= 0 { sh.set_shader_value(lt, kind); }
            if lp >= 0 { sh.set_shader_value(lp, pos); }
            if ltg >= 0 { sh.set_shader_value(ltg, target); }
            if lc >= 0 { sh.set_shader_value(lc, color); }
        }
    }

    pub fn cls(&mut self, color: i64) {
        // CLS setzt die Hintergrundfarbe (beim FLIP gecleart) und leert den
        // aktiven Layer (Wipe). Die Layer werden ohnehin pro FLIP geleert.
        self.clear_color = col(color);
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

    /// Laedt einen TTF/OTF-Font in der gegebenen Basis-Groesse -> FONT-Handle.
    pub fn load_font(&mut self, path: &str, size: i32) -> Result<i64, String> {
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
    pub fn image_width(&self, idx: i64) -> Result<i64, String> {
        self.textures.get(idx as usize).map(|t| t.tex.width as i64).ok_or_else(|| "IMAGEWIDTH: ungueltiges Handle".into())
    }
    pub fn image_height(&self, idx: i64) -> Result<i64, String> {
        self.textures.get(idx as usize).map(|t| t.tex.height as i64).ok_or_else(|| "IMAGEHEIGHT: ungueltiges Handle".into())
    }

    // --- LOAD_ASSETS: Bilder aus JSON-Manifest vorladen (Alias/Pfad-Cache) ---
    pub fn load_assets(&mut self, manifest_path: &str) -> Result<i64, String> {
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
    pub fn atlas_draw(&mut self, atlas: i64, name: &str, x: i32, y: i32, flip_h: bool) -> Result<(), String> {
        let (tex, sx, sy, sw, sh) = {
            let a = self.atlases.get(atlas as usize).ok_or("ATLAS_DRAW: ungueltiges Atlas-Handle")?;
            let &(sx, sy, sw, sh) = a.frames.get(name)
                .ok_or_else(|| format!("ATLAS_DRAW: Sprite '{}' nicht im Atlas", name))?;
            (a.tex_idx, sx, sy, sw, sh)
        };
        let (x, y) = self.w2s(x, y);
        self.emit(Cmd::AtlasDraw(tex, sx, sy, sw, sh, x, y, flip_h));
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
        // Negative Codes = Gamepad-Buttons/DPad (siehe DEFAULT_KEYS joy_*).
        if code < 0 { return self.joy_button_down(code); }
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

    // --- Game-Loop-Grundlagen ---
    pub fn delta(&self) -> f64 { self.rl.get_frame_time() as f64 }
    pub fn fps(&self) -> i64 { self.rl.get_fps() as i64 }
    pub fn set_target_fps(&mut self, n: i64) { self.rl.set_target_fps(n.max(0) as u32); }
    pub fn set_window_title(&mut self, title: &str) { self.rl.set_window_title(&self.thread, title); }
    pub fn save_screenshot(&mut self, path: &str) { self.rl.take_screenshot(&self.thread, path); }
    pub fn set_fullscreen(&mut self, fs: bool) {
        if self.rl.is_window_fullscreen() != fs { self.rl.toggle_fullscreen(); }
    }

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
        self.render_shadow_map();
        let s = self.scale;
        let clear_color = self.clear_color;
        let mut order: Vec<usize> = (0..self.layers.len()).collect();
        order.sort_by_key(|&i| self.layers[i].z);
        // RT-Groesse = Fenstergroesse (bekannt, ohne mehrdeutigen Textur-Query).
        let (tw, th) = ((self.width * self.scale) as f32, (self.height * self.scale) as f32);
        let Graphics { rl, thread, layers, textures, fonts, cmds3d, cam3d, models,
            light_shader, normal_mapped, loc_use_normal, pbr_params, loc_metalness, loc_roughness,
            scene_rt, shaders, post_shader_idx, .. } = self;
        let mat_locs = (*loc_use_normal, *loc_metalness, *loc_roughness);
        let nmap_set: &std::collections::HashSet<usize> = normal_mapped;
        let pbr_ref: &std::collections::HashMap<usize, (f32, f32)> = pbr_params;
        let cam = *cam3d;
        // Post-FX aktiv? -> (Shader-Index, Render-Target). Sonst direkt zeichnen.
        let postfx = match *post_shader_idx {
            Some(i) if i < shaders.len() => scene_rt.as_mut().map(|rt| (i, rt)),
            _ => None,
        };
        if let Some((idx, rt)) = postfx {
            // 1) Szene in die RenderTexture rendern.
            {
                let mut tx = rl.begin_texture_mode(thread, rt);
                render_scene(&mut tx, s, clear_color, layers, &order, textures, fonts, cmds3d, cam, models, light_shader.as_mut(), mat_locs, nmap_set, pbr_ref);
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
            render_scene(&mut d, s, clear_color, layers, &order, textures, fonts, cmds3d, cam, models, light_shader.as_mut(), mat_locs, nmap_set, pbr_ref);
        }
        // Layer + 3D-Befehle fuer den naechsten Frame leeren (Immediate-Mode).
        for l in self.layers.iter_mut() { l.cmds.clear(); }
        self.cmds3d.clear();
        self.frame_count += 1;
        // Headless-Screenshot beim Erreichen der Frame-Grenze.
        if let (Some(mx), Some(path), false) = (self.max_frames, self.screenshot.clone(), self.shot_taken) {
            if self.frame_count >= mx {
                self.rl.take_screenshot(&self.thread, &path);
                self.shot_taken = true;
            }
        }
    }
}

/// Spielt 3D-Befehle (begin_mode3D) + 2D-Layer (mit Scissor-Clip-Stack) auf ein
/// beliebiges Draw-Ziel ab -- den Screen ODER eine RenderTexture (beide impl
/// `RaylibDraw`). So laeuft derselbe Replay-Code mit und ohne Post-Shader.
fn render_scene<D: RaylibDraw>(
    d: &mut D, s: i32, clear: Color,
    layers: &[Layer], order: &[usize], textures: &[Tex], fonts: &[Font],
    cmds3d: &[Cmd3D], cam3d: Camera3D, models: &[Model],
    mut light_shader: Option<&mut Shader>, mat_locs: (i32, i32, i32),
    normal_mapped: &std::collections::HashSet<usize>,
    pbr_params: &std::collections::HashMap<usize, (f32, f32)>,
) {
    let (loc_use_normal, loc_metalness, loc_roughness) = mat_locs;
    // Per-Modell-Material-Uniforms (useNormalMap + metalness/roughness) vor dem Draw.
    let mut set_material = |ls: &mut Option<&mut Shader>, idx: usize| {
        if let Some(sh) = ls.as_mut() {
            if loc_use_normal >= 0 {
                sh.set_shader_value(loc_use_normal, if normal_mapped.contains(&idx) { 1i32 } else { 0i32 });
            }
            let (m, r) = pbr_params.get(&idx).copied().unwrap_or((0.0, 0.6));
            if loc_metalness >= 0 { sh.set_shader_value(loc_metalness, m); }
            if loc_roughness >= 0 { sh.set_shader_value(loc_roughness, r); }
        }
    };
    let mut clip_stack: Vec<(i32, i32, i32, i32)> = Vec::new();
    d.clear_background(clear);
            // 3D-Pass zuerst (in einem begin_mode3D-Block), 2D-HUD danach obenauf.
            if !cmds3d.is_empty() {
                let mut d3 = d.begin_mode3D(cam3d);
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
                    Cmd::TextureFlipped(i, x, y, fh, fv) => {
                        let t = &textures[*i].tex;
                        let sw = if *fh { -(t.width as f32) } else { t.width as f32 };
                        let sh = if *fv { -(t.height as f32) } else { t.height as f32 };
                        let src = Rectangle::new(0.0, 0.0, sw, sh);
                        let dst = Rectangle::new((x * s) as f32, (y * s) as f32, (t.width * s) as f32, (t.height * s) as f32);
                        d.draw_texture_pro(t, src, dst, Vector2::zero(), 0.0, Color::WHITE);
                    }
                    Cmd::AtlasDraw(i, sx, sy, sw, sh, dx, dy, fh) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32, if *fh { -(*sw as f32) } else { *sw as f32 }, *sh as f32);
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (sw * s) as f32, (sh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0,Color::WHITE);
                    }
                    Cmd::SpriteDraw(i, sx, sy, sw, sh, dx, dy, dw, dh, fx, fy, tint) => {
                        let src = Rectangle::new(*sx as f32, *sy as f32,
                            if *fx { -(*sw as f32) } else { *sw as f32 },
                            if *fy { -(*sh as f32) } else { *sh as f32 });
                        let dst = Rectangle::new((dx * s) as f32, (dy * s) as f32, (dw * s) as f32, (dh * s) as f32);
                        d.draw_texture_pro(&textures[*i].tex, src, dst, Vector2::zero(), 0.0,*tint);
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
            // Sicherheit: unbalancierte Clips nicht in den naechsten Frame lecken.
            if !clip_stack.is_empty() { unsafe { raylib::ffi::EndScissorMode(); } }
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
