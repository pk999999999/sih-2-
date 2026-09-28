fn main() {
    println!("cargo:rustc-check-cfg=cfg(jocky_has_object)");
    println!("cargo:rerun-if-env-changed=JOCKY_OBJECT_PATH");
    if let Some(path) = std::env::var_os("JOCKY_OBJECT_PATH") {
        println!("cargo:rerun-if-changed={}", std::path::Path::new(&path).display());
        println!("cargo:rustc-link-arg={}", std::path::Path::new(&path).display());
        println!("cargo:rustc-cfg=jocky_has_object");
    }
}
