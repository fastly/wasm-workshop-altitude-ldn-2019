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

    // put the message into the AssemblyScript heap
    let msg_ptr = inst.put_byte_slice(&msg);

    // run the hash function and get a pointer to the string with the hex digits
    let hash_str_ptr = inst
        .run("sha512", &[Val::GuestPtr(msg_ptr)])
        .unwrap()
        .into();

    // get the string out of the AssemblyScript heap
    let hash_str = inst.get_string(hash_str_ptr);

    println!("{}", hash_str);

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
