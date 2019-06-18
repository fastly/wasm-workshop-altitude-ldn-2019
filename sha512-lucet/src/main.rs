#![allow(dead_code, unused_imports)]

mod assemblyscript;

use crate::assemblyscript::{AssemblyScript, GuestPtr};
use failure::Error;
use lucet_runtime::{lucet_hostcalls, DlModule, Limits, MmapRegion, Region, Val};
use std::io::{self, Read};
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    // read stdin until EOF and store the bytes in `message`
    let mut msg: Vec<u8> = vec![];
    io::stdin().read_to_end(&mut msg)?;

    // load the Lucet module compiled with `build.rs`
    let module_path = PathBuf::from(env!("OUT_DIR")).join("module.so");
    let module = DlModule::load(&module_path)?;

    // create a memory region to run the Lucet instance; we only need one slot, and the default
    // memory limits are fine for now
    let region = MmapRegion::create(1, &Limits::default())?;

    // create the instance in the memory region
    let mut inst = region.new_instance(module)?;

    // just run the hello function for now
    let hello: i32 = inst.run("hello", &[])?.into();
    let hello_char = std::char::from_u32(hello as u32).unwrap();

    println!("{}", hello_char);

    Ok(())
}

lucet_hostcalls! {
    #[no_mangle]
    pub unsafe extern "C" fn __as_abort(
        &mut _vmctx,
        _msg_ptr: GuestPtr,
        _file_ptr: GuestPtr,
        line: u32,
        col: u32,
    ) -> () {
        panic!("abort at {}:{}", line, col);
    }
}
