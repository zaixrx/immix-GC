use crate::{api_alloc::*, block::*, tl_alloc::*};
use std::{cell::UnsafeCell, marker::PhantomData, mem::replace, ptr::NonNull};

const WORD_SIZE: usize = size_of::<usize>();

#[derive(Debug, Clone, Copy)]
pub struct RawPtr<T: Sized> {
    ptr: NonNull<T>,
}

impl<T> ScopedRef<T> for RawPtr<T> {
    fn scoped_ref<'scope>(&self, _guard: &'scope dyn MutatorScope) -> &'scope T {
        unsafe { &*self.ptr.as_ptr() }
    }
}

pub struct UnsafeRawGlobalAllocator {
    head: Option<ThreadLocalAllocator>,
    overflow: Option<ThreadLocalAllocator>,
    rest: Vec<ThreadLocalAllocator>,
    large: Vec<Block>,
}

pub struct UnsafeGlobalAllocator<H: AllocHeader> {
    inner: UnsafeCell<UnsafeRawGlobalAllocator>,
    _header_type: PhantomData<*const H>,
}

pub struct ScopedGlobalAllocator<'memory, H: AllocHeader> {
    inner: &'memory UnsafeGlobalAllocator<H>,
}

impl UnsafeRawGlobalAllocator {
    fn new() -> Self {
        Self {
            head: None,
            overflow: None,
            rest: Vec::new(),
            large: Vec::new(),
        }
    }

    /// expects a medium classed `size`, fails otherwise
    /// TODO: preallocate free blocks as appoesd of allocating on demand
    fn overflow_alloc(&mut self, size: usize) -> Result<*const u8, AllocError> {
        match self.overflow {
            Some(ref mut overflow) => {
                let space = match overflow.inner_alloc(size) {
                    Some(space) => space,
                    None => {
                        let old = replace(overflow, ThreadLocalAllocator::build()?);
                        self.rest.push(old);
                        let space = overflow.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                        space
                    }
                };
                return Ok(space);
            },
            None => {
                let mut tlalloc = ThreadLocalAllocator::build()?;
                let space = tlalloc.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                self.overflow = Some(tlalloc);
                return Ok(space);
            }
        };
    }
}

impl<H: AllocHeader> UnsafeGlobalAllocator<H> {
    pub fn new() -> Self {
        let inner = UnsafeRawGlobalAllocator::new();
        UnsafeGlobalAllocator {
            inner: UnsafeCell::new(inner),
            _header_type: PhantomData
        }
    }

    fn find_space(&self, size: usize) -> Result<*const u8, AllocError> {
        let inner = unsafe { &mut *self.inner.get() };
        let size_class = SizeClass::new(size);
        if size_class == SizeClass::Large {
            let block = Block::build(size)?;
            let space = block.as_ptr();
            inner.large.push(block);
            return Ok(space);
        }
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
                                let old = replace(head, ThreadLocalAllocator::build()?);
                                inner.rest.push(old);
                                let space = head.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                                space
                            }
                        };
                        return Ok(space);
                    }
                };
            },
            None => {
                let mut tlalloc = ThreadLocalAllocator::build().expect("Failed to acquire head block");
                let space = tlalloc.inner_alloc(size).expect("UNEXPECTED_ERROR: `size` too big to fit inside an medium sized block");
                inner.head = Some(tlalloc);
                return Ok(space);
            }
        }
    }
}

impl<H: AllocHeader> AllocRaw for UnsafeGlobalAllocator<H> {
    type Header = H;

    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, AllocError>
    where
        T: AllocObject<<Self::Header as AllocHeader>::TypeId> {
            let header_size = size_of::<Self::Header>();
            let object_size = size_of::<T>();
            let total_size = header_size + object_size;

            let size = (total_size + WORD_SIZE - 1) / WORD_SIZE;
            let space = self.find_space(size)?;

            unsafe {
                let header = Self::Header::new::<T>(object_size, Mark::Live);
                std::ptr::write(space as *mut Self::Header, header);
                let space = space.offset(1) as *mut T;
                std::ptr::write(space, object);
                Ok(RawPtr{
                    ptr: NonNull::new_unchecked(space)
                })
            }
    }

    fn alloc_array(&self, size_bytes: usize) -> Result<RawPtr<u8>, AllocError> {
        let header_size = size_of::<Self::Header>();
        let total_size = header_size + size_bytes;

        let size = (total_size + WORD_SIZE - 1) / WORD_SIZE;
        let space = self.find_space(size)?;

        unsafe {
            let header = Self::Header::new_array(size_bytes, Mark::Live);
            std::ptr::write(space as *mut Self::Header, header);

            let space = space.offset(1) as *mut u8;
            let slice = std::slice::from_raw_parts_mut(space, size_bytes);
            for byte in slice { *byte = 0; }

            Ok(RawPtr{
                ptr: NonNull::new_unchecked(space)
            })
        }
    }

    fn get_header(object: NonNull<()>) -> NonNull<Self::Header> {
        unsafe {
            object.cast::<Self::Header>().offset(-1)
        }
    }

    fn get_object(header: NonNull<Self::Header>) -> NonNull<()> {
        unsafe {
            header.offset(1).cast::<()>()
        }
    }
}

impl<'memory, H: AllocHeader> MutatorScope for ScopedGlobalAllocator<'memory, H> {}

impl<'memory, H: AllocHeader> ScopedGlobalAllocator<'memory, H> {
    fn alloc<T>(&self, object: T) -> Result<ScopedPtr<'_, T>, AllocError>
    where
        T: AllocObject<H::TypeId>,
        RawPtr<T>: Copy,
    {
            Ok(ScopedPtr::new(
                self,
                self.inner.alloc(object)?
            ))
    }
}
