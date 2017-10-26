extern crate cc;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // use cool variety if debug mode enabled
    let variety = if env::var("DEBUG").map(|val| val=="true").unwrap_or(false) {
        "CONFIG_VAR_COOL"
    } else {
        "CONFIG_VAR_HOT"
    };

    cc::Build::new()
        .file("mps-kit/code/mps.c")
        .file("src/ffi/stub.c")
        .define(variety, None)
        .flag("-std=c11")
        .flag_if_supported("-Wimplicit-fallthrough=2")
        .include("mps-kit/code")
        .compile("librustmps.a");

    let bindings = bindgen::Builder::default()
        .header("mps-kit/code/mps.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
