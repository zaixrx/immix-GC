mod block;
mod tl_alloc;

use tl_alloc::ThreadLocalAllocator;
use std::ffi::c_char;

unsafe extern "C" {
    fn printf(fmt: *const c_char, ...);
}

fn main() {
    let buf = c"Hello, World";
    let buf_size = buf.count_bytes();

    let mut block = ThreadLocalAllocator::build().expect("failed to request TLA");
    let ptr = block.inner_alloc(1 << 16).expect("exhausted TLA") as *mut c_char;
    // SAFETY: who gives a fuck
    unsafe {
        std::ptr::copy(buf.as_ptr(), ptr, buf_size);
        printf(c"libc_printf: %s\n".as_ptr() as *const c_char, ptr);
    }
}
