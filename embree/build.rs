use std::env;
use std::path::PathBuf;

use bindgen::Builder;

fn main() {
    // Find the embree install and bind to it:
    if let Ok(e) = env::var("EMBREE_DIR") {
        let mut embree_lib_dir = PathBuf::from(e);
        embree_lib_dir.push("lib");
        println!(
            "cargo:rustc-link-search=native={}",
            embree_lib_dir.display()
        );
        println!("cargo:rerun-if-env-changed=EMBREE_DIR");
    } else {
        panic!("Unable to find embree install location in environment EMBREE_DIR")
    }
    println!("cargo:rustc-link-lib=embree3");

    // Construct the header file:
    let embree_include_file = match env::var("EMBREE_DIR") {
        Ok(e) => {
            let mut dir = PathBuf::from(e);
            dir.push("include");
            dir.push("embree3");
            dir.push("rtcore.h");
            dir
        }
        _ => panic!("Unable to find embree install location in environment EMBREE_DIR"),
    };

    // Create the interface to embree here:
    let embree_bindings = Builder::default()
        .header(
            embree_include_file
                .to_str()
                .expect("Problem when converting EMBREE_DIR environment variable to string."),
        )
        .generate()
        .expect("Unable to generate bindings for embree3 from header file \"rtcore.h\".");

    // Define a place to output the bindings:
    let rtcore_rs_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    embree_bindings
        .write_to_file(rtcore_rs_path.join("rtcore.rs"))
        .expect("Unbale to write bindings for embree3 to rust file \"rtcore.rs\"");
}
