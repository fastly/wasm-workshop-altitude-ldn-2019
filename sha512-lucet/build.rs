use lucetc::{Bindings, Lucetc, LucetcOpts};
use std::env;
use std::path::PathBuf;

fn main() {
    let cargo_manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let wasm_path = cargo_manifest_dir
        .parent()
        .unwrap()
        .join("sha512-web")
        .join("src")
        .join("assembly")
        .join("module.wasm");

    Lucetc::new(&wasm_path)
        .with_bindings(Bindings::from_file(cargo_manifest_dir.join("bindings.json")).unwrap())
        .shared_object_file(out_dir.join("module.so"))
        .unwrap();
}
