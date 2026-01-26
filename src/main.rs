mod tla;
mod block;

use tla::{ThreadLocalAllocator, SizeClass};
use std::{cell::UnsafeCell, ffi::c_char, marker::PhantomData, mem::replace};

struct UnsafeGlobalAllocator {
    head: Option<ThreadLocalAllocator>,
    overflow: Option<ThreadLocalAllocator>,
    rest: Vec<ThreadLocalAllocator>,
}

impl UnsafeGlobalAllocator {
    // TODO: preallocate free blocks as appoesd of allocating on demand
    // expects a medium classed `size`, fails otherwise
    fn overflow_alloc(&mut self, size: usize) -> *const u8 {
        match self.overflow {
            Some(ref mut overflow) => {
                let space = match overflow.inner_alloc(size) {
                    Some(space) => space,
                    None => {
                        let old = replace(overflow, ThreadLocalAllocator::build().expect("Failed to acquire new block"));
                        self.rest.push(old);
                        let space = overflow.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                        space
                    }
                };
                return space;
            },
            None => {
                let mut tlalloc = ThreadLocalAllocator::build().expect("Failed to acquire overflow block");
                let space = tlalloc.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                self.overflow = Some(tlalloc);
                return space;
            }
        };
    }
}

pub struct GlobalAllocator<H> {
    inner: UnsafeCell<UnsafeGlobalAllocator>,
    _header_type: PhantomData<*const H>,
}

impl<H> GlobalAllocator<H> {
    pub fn alloc(&self, size: usize) -> *const u8 {
        let size_class = SizeClass::new(size);
        if size_class == SizeClass::Large {
            todo!("SizeClass::Large")
        }
        // SAFETY: my balls
        let inner = unsafe { &mut *self.inner.get() };
        match inner.head {
            Some(ref mut head) => {
                match size_class {
                    SizeClass::Medium if size > head.current_hole_size() => {
                        return inner.overflow_alloc(size);
                    },
                    _ => {
                        let space = match head.inner_alloc(size) {
                            Some(space) => space,
                            None => {
                                let old = replace(head, ThreadLocalAllocator::build().expect("Failed to acquire new block"));
                                inner.rest.push(old);
                                let space = head.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                                space
                            }
                        };
                        return space;
                    }
                };
            },
            None => {
                let mut tlalloc = ThreadLocalAllocator::build().expect("Failed to acquire head block");
                let space = tlalloc.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                inner.head = Some(tlalloc);
                return space;
            }
        }
    }
}

unsafe extern "C" {
    fn printf(fmt: *const c_char, ...);
}

fn main() {
    let buf = c"Hello, World";
    let buf_size = buf.count_bytes();

    let mut block = ThreadLocalAllocator::build().expect("failed to request block");
    let ptr = block.inner_alloc(1 << 16).expect("exhausted block") as *mut c_char;
    // SAFETY: who gives a fuck
    unsafe {
        std::ptr::copy(buf.as_ptr(), ptr, buf_size);
        printf(c"libc_printf: %s\n".as_ptr() as *const c_char, ptr);
    }
}
