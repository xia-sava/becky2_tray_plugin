fn main() {
    // Becky! looks up callbacks by their undecorated names (e.g. BKC_OnStart),
    // while stdcall symbols on i686 are decorated (_BKC_OnStart@0).
    // The module definition file maps the plain names to the decorated symbols.
    let def_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("becky2_tray_plugin.def");
    println!("cargo:rerun-if-changed={}", def_path.display());
    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def_path.display());
}
