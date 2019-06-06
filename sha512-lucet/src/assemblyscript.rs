/// Functions for getting values into and out of an AssemblyScript instance heap

use byteorder::{ByteOrder, LittleEndian};
use lucet_runtime::InstanceHandle;

pub type GuestPtr = u32;
pub type GuestSize = u32;

fn array_buffer_size(slice: &[u8]) -> GuestSize {
    const HEADER_SIZE: GuestSize = 8;
    1 << (32 - (slice.len() as GuestSize + HEADER_SIZE - 1).leading_zeros())
}

pub fn put_byte_slice(inst: &mut InstanceHandle, slice: &[u8]) -> GuestPtr {
    if slice.len() > std::u32::MAX as usize {
        panic!("byte slice too large for 32-bit address space");
    }

    // TypedArray
    let ptr: GuestPtr = inst.run("memory.allocate", &[12u32.into()]).unwrap().into();
    // ArrayBuffer
    let buf: GuestPtr = inst
        .run("memory.allocate", &[array_buffer_size(slice).into()])
        .unwrap()
        .into();

    let guest_heap = inst.heap_mut();

    // TypedArray.buffer
    LittleEndian::write_u32(&mut guest_heap[ptr as usize..], buf);
    // TypedArray.byteOffset
    LittleEndian::write_u32(&mut guest_heap[(ptr + 4) as usize..], 0);
    // TypedArray.byteLength
    LittleEndian::write_u32(
        &mut guest_heap[(ptr + 8) as usize..],
        slice.len() as GuestSize,
    );

    // ArrayBuffer.byteLength
    LittleEndian::write_u32(&mut guest_heap[buf as usize..], slice.len() as GuestSize);
    // ArrayBuffer.padding
    LittleEndian::write_u32(&mut guest_heap[(buf + 4) as usize..], 0);
    // ArrayBuffer data
    &mut guest_heap[(buf + 8) as usize..(buf + 8) as usize + slice.len()].copy_from_slice(slice);

    ptr
}

pub fn get_byte_slice(inst: &InstanceHandle, ptr: GuestPtr) -> Vec<u8> {
    let guest_heap = inst.heap();

    let ptr = ptr as usize;

    let buf = LittleEndian::read_u32(&guest_heap[ptr..]) as usize;
    let byte_offset = LittleEndian::read_u32(&guest_heap[ptr + 4..]) as usize;
    let byte_length = LittleEndian::read_u32(&guest_heap[ptr + 8..]) as usize;

    let buf_length = LittleEndian::read_u32(&guest_heap[buf..]) as usize;
    assert!(buf_length >= byte_offset + byte_length);

    guest_heap[buf + 8 + byte_offset..buf + 8 + byte_offset + byte_length].to_vec()
}

pub fn get_string(inst: &InstanceHandle, ptr: GuestPtr) -> String {
    let guest_heap = inst.heap();
    
    // read the length field
    let len = LittleEndian::read_u32(&guest_heap[ptr as usize..]);

    // followed by that number of UTF-16 characters (`len * 2` bytes)
    let u16s = guest_heap[(ptr + 4) as usize..(ptr + 4 + (len * 2)) as usize]
        .chunks(2)
        .map(LittleEndian::read_u16);

    // decode and replace invalid characters
    std::char::decode_utf16(u16s)
        .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
        .collect()
}
