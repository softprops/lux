extern crate serde_codegen;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/main.rs.in");
    let dst = Path::new(&out_dir).join("main.rs");

    serde_codegen::expand(&src, &dst).unwrap();
}
