use bindgen::Builder;
use cc;

use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new().file("extern/rply.c").compile("rply");

    // Create the interface to rply here:
    let rply_bindings = Builder::default()
        .header("extern/wrapper.h")
        .generate()
        .expect("Unable to generate bindings for rply from header file \"wrapper.h\".");

    // Define a place to output the bindings:
    let rply_rs_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    rply_bindings
        .write_to_file(rply_rs_path.join("wrapper.rs"))
        .expect("Unbale to write bindings for rply to rust file \"wrapper.rs\"");
}
