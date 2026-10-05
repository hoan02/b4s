fn main() {
    // Embed every profile deterministically; adding a model needs no Rust registration.
    let directory = std::path::Path::new("catalog/models");
    println!("cargo:rerun-if-changed={}", directory.display());
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .expect("model catalog directory")
        .map(|entry| entry.expect("model catalog entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    let mut source = String::from("pub const PROFILE_SOURCES: &[&str] = &[\n");
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let absolute = path.canonicalize().expect("profile path");
        source.push_str(&format!(
            "include_str!({:?}),\n",
            absolute.to_string_lossy()
        ));
    }
    source.push_str("];\n");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(output.join("model_profiles.rs"), source).expect("write embedded profiles");
    tauri_build::build()
}
