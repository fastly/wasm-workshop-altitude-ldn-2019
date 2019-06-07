use failure::Error;
use lucetc::{Bindings, Lucetc, LucetcOpts};
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    let cargo_manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let wasm_path = cargo_manifest_dir
        .parent()
        .unwrap()
        .join("sha512-web")
        .join("src")
        .join("assembly")
        .join("module.wasm");
    println!("cargo:rerun-if-changed={}", wasm_path.display());

    let bindings_path = cargo_manifest_dir.join("bindings.json");
    println!("cargo:rerun-if-changed={}", bindings_path.display());

    let bindings = Bindings::from_file(&bindings_path)?;

    Lucetc::new(&wasm_path)
        .with_bindings(bindings)
        .shared_object_file(out_dir.join("module.so"))?;

    Ok(())
}
