use std::io::Command;
use std::os;

fn invoke(cmd: &Command) {
    match cmd.output() {
        Err(e) => panic!("failed to execute compiler: {}", e),
        Ok(o)  => if !o.status.success() {
            let stdout = String::from_utf8_lossy(&*o.output);
            let stderr = String::from_utf8_lossy(&*o.error);
            panic!("\nstdout: {}\nstderr: {}", stdout, stderr)
        }
    }
}

fn main() {
    let out_dir = os::getenv("OUT_DIR").unwrap();
    let debug: bool = os::getenv("DEBUG").unwrap().parse().unwrap();

    // MPS itself
    let mut cc = Command::new("clang");
    cc.arg("-fPIC");
    cc.arg("-c").arg("mps-kit-1.114.0/code/mps.c");
    cc.arg("-o").arg(format!("{}/mps.o", out_dir));
    if debug {
        cc.args(&["-g", "-DCONFIG_VAR_COOL"]);
    } else {
        cc.arg("-O2");
    }
    invoke(&cc);

    // Rust MPS glue code
    let mut cc = Command::new("clang");
    cc.arg("-fPIC");
    cc.arg("-c").arg("src/rust-mps.c");
    cc.arg("-I").arg("mps-kit-1.114.0/code");
    cc.arg("-o").arg(format!("{}/rust-mps.o", out_dir));
    invoke(&cc);

    // build librustmps.a
    let mut ar = Command::new("ar");
    ar.arg("crus").arg("librustmps.a");
    ar.arg("mps.o").arg("rust-mps.o");
    ar.cwd(&Path::new(&out_dir));
    invoke(&ar);

    println!("cargo:rustc-flags=-L {} -l rustmps:static", out_dir);
}
