fn main() {
    // tauri-build embeds the icon into the exe but asks Cargo to rerun only for the config and the
    // capabilities, so a new icon would never reach the exe.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    // *About*'s build time: restamped whenever the app's sources, the crates or the workspace manifest
    // change, so it is the time the program was last compiled.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../../crates");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    let built = jiff::Zoned::now().strftime("%Y-%m-%d %H:%M").to_string();
    println!("cargo:rustc-env=PACKING_BUILT={built}");
    tauri_build::build()
}
