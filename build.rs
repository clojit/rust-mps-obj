#![feature(env)]

extern crate gcc;

use std::env::var;

fn main() {
    // use cool variety if debug mode enabled
    let variety = if var("DEBUG").map(|val| val=="true").unwrap_or(false) {
        "CONFIG_VAR_COOL"
    } else {
        "CONFIG_VAR_HOT"
    };

    gcc::Config::new()
                .file("mps-kit-1.114.0/code/mps.c")
                .file("src/rust-mps.c")
                .define(variety, None)
                .flag("-std=c11")
                .include("mps-kit-1.114.0/code")
                .compile("librustmps.a");
}
