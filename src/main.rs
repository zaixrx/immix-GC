mod block;
mod bump;

use std::ffi::c_char;

unsafe extern "C" {
    fn printf(fmt: *const c_char, ...);
}

fn main() {
    let buf = c"Hello, World";
    let buf_size = buf.count_bytes();
    unsafe {
        let mut block = BumpBlock::build().expect("failed to build block");
        let ptr = block.inner_alloc(buf_size).expect("failed to alloc") as *mut c_char;
        std::ptr::copy(buf.as_ptr(), ptr, buf_size);
        printf(c"libc_printf: %s\n".as_ptr() as *const c_char, ptr);
    }
}
