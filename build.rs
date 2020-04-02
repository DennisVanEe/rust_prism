use std::env;
use std::path::PathBuf;

fn main() {
    // Check for EMBREE_DIR first and see if it's defined:
    if let Ok(e) = env::var("EMBREE_DIR") {
        let mut path = PathBuf::from(e);
        path.push("lib");
        println!("cargo:rustc-link-search=native={}", path.display());
        println!("cargo:rerun-if-env-changed=EMBREE_DIR");
    }
    println!("cargo:rustc-link-lib=embree3");
}
