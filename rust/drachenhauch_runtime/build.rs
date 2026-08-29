//! Stackgroesse des Hauptthreads setzen -- und zwar NUR fuer das eigene Binary.
//!
//! Die VM rekursiert fuer jeden GB-Funktionsaufruf ueber den nativen Stack
//! (`exec` -> `run_frame` -> `dispatch` -> `exec`); ein Aufrufrahmen kostet
//! gemessen ~6,6 KB. Windows gibt einem Programm per Vorgabe 1 MB, das trug
//! ganze 146 Ebenen -- `MAX_CALL_DEPTH` in `vm.rs` wurde nie erreicht, der
//! Prozess stuerzte vorher ab. 64 MB kosten nichts: das ist reservierter
//! ADRESSRAUM, belegt wird nur, was wirklich benutzt wird.
//!
//! WARUM HIER UND NICHT IN `.cargo/config.toml`
//! --------------------------------------------
//! Dort stand es zuerst, als `rustflags` -- und die gelten fuer ALLE Artefakte,
//! auch fuer Proc-Macros und Build-Skripte der Abhaengigkeiten. Auf macOS ist
//! das ein harter Fehler, weil ld64 die Option nur fuer Programme annimmt:
//!
//!     ld: -stack_size option can only be used when linking a main executable
//!     ... -o .../libpaste-*.dylib ... "-Wl,-stack_size,0x4000000"
//!
//! `cargo:rustc-link-arg-bins` gilt dagegen ausschliesslich fuer die Binaries
//! dieses Crates. (Auf Windows fiel es nicht auf -- der MSVC-Linker nimmt
//! `/STACK:` auch fuer eine DLL entgegen und ignoriert es. Gruen auf der
//! eigenen Maschine hiess hier also nichts.)
//!
//! LINUX steht bewusst nicht dabei: dort bekommt der Hauptthread seinen Stack
//! aus RLIMIT_STACK (ueblich 8 MB), an das kein Linker-Flag heranreicht. Genau
//! deshalb liegt `MAX_CALL_DEPTH` bei 1000 und nicht hoeher -- die Grenze soll
//! auf JEDER Plattform vor dem Stack greifen, damit ueberall dieselbe
//! Fehlermeldung kommt statt mal einer Meldung und mal eines Absturzes.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    match (os.as_str(), env.as_str()) {
        ("windows", "msvc") => println!("cargo:rustc-link-arg-bins=/STACK:67108864"),
        ("windows", _) => println!("cargo:rustc-link-arg-bins=-Wl,--stack,67108864"),
        ("macos", _) => println!("cargo:rustc-link-arg-bins=-Wl,-stack_size,0x4000000"),
        _ => {}
    }
}
