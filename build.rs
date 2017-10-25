extern crate cc;

use std::env::var;

fn main() {
    // use cool variety if debug mode enabled
    let variety = if var("DEBUG").map(|val| val=="true").unwrap_or(false) {
        "CONFIG_VAR_COOL"
    } else {
        "CONFIG_VAR_HOT"
    };

    cc::Build::new()
        .file("mps-kit/code/mps.c")
        .file("src/ffi/glue.c")
        .define(variety, None)
        .flag("-std=c11")
        .include("mps-kit/code")
        .compile("librustmps.a");
}
