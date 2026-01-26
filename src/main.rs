mod tl_alloc;
mod g_alloc;
mod block;

use std::ffi::c_char;
use g_alloc::GlobalAllocator;

unsafe extern "C" {
    fn printf(fmt: *const c_char, ...);
}

fn main() {
    let buf = c"Hello, World";
    let buf_size = buf.count_bytes();

    let allocator = GlobalAllocator::<c_char>::new();
    let space = allocator.alloc(buf_size);

    // SAFETY: who gives a fuck
    unsafe {
        std::ptr::copy(buf.as_ptr(), space as *mut c_char, buf_size);
        printf(c"libc_printf: %s\n".as_ptr() as *const c_char, space);
    }
}
