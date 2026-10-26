fn main() {
    // tauri-build embeds the icon into the exe but asks Cargo to rerun only for the config and the
    // capabilities, so a new icon would never reach the exe.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    tauri_build::build()
}
