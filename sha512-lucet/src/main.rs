mod assemblyscript;

use crate::assemblyscript::{get_string, put_byte_slice, GuestPtr};
use lucet_runtime::{lucet_hostcalls, DlModule, Limits, MmapRegion, Region, Val};
use std::io::{self, Read};
use std::path::PathBuf;

fn main() {
    // read stdin until EOF and store the bytes in `message`
    let mut message: Vec<u8> = vec![];
    io::stdin().read_to_end(&mut message).unwrap();

    // load the Lucet module compiled with `build.rs`
    let module_path = PathBuf::from(env!("OUT_DIR")).join("module.so");
    let module = DlModule::load(&module_path).unwrap();

    // create a memory region to run the Lucet instance; we only need one slot, and the default
    // memory limits are fine for now
    let region = MmapRegion::create(1, &Limits::default()).unwrap();

    // create the instance in the memory region
    let mut inst = region.new_instance(module).unwrap();

    // put the message into the AssemblyScript heap
    let message_ptr = put_byte_slice(&mut inst, message.as_slice());

    // run the hash function and get a pointer to the string with the hex digits
    let hash_str_ptr = inst
        .run("sha512", &[Val::GuestPtr(message_ptr)])
        .unwrap()
        .into();

    // get the string out of the AssemblyScript heap
    let hash_str = get_string(&inst, hash_str_ptr);

    println!("{}", hash_str);
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
