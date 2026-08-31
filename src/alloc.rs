use crate::{alloc_api::*, block::*, bump::*};
use std::{cell::UnsafeCell, marker::PhantomData, mem::replace, ptr::NonNull};

const WORD_SIZE: usize = size_of::<usize>();

pub struct UnsafeRawAllocator {
    head: Option<BumpAllocator>,
    overflow: Option<BumpAllocator>,
    freed: Vec<BumpAllocator>,
    large: Vec<Block>,
}

pub struct UnsafeAllocator<H: AllocHeader> {
    inner: UnsafeCell<UnsafeRawAllocator>,
    _header_type: PhantomData<*const H>,
}

pub struct ScopedGlobalAllocator<'memory, H: AllocHeader> {
    inner: &'memory UnsafeAllocator<H>,
}

impl UnsafeRawAllocator {
    pub fn new() -> Self {
        Self {
            head: None,
            overflow: None,
            freed: Vec::new(),
            large: Vec::new(),
        }
    }

    pub fn alloc(&mut self, size: usize) -> Result<*const u8, AllocError> {
        let class = SizeClass::new(size);
        if  class == SizeClass::Large {
            todo!("UnsafeRawAllocator: SizeClass::Large")
        }

        match self.head {
            Some(ref mut head) => {
                match class {
                    SizeClass::Medium if size > head.current_hole_size() => {
                        self.overflow_alloc(size)
                    },
                    _ => {
                        match head.inner_alloc(size) {
                            Ok(space) => Ok(space),
                            Err(BlockError::OOM) => {
                                let old = replace(head, BumpAllocator::build()
                                    .expect("UnsafeRawAllocator: out of memory"));
                                self.freed.push(old);
                                head.inner_alloc(size).map_err(Into::into)
                            },
                            Err(BlockError::BadRequest) => Err(AllocError::BadRequest)
                        }
                    }
                }
            },
            None => {
                let mut bump = BumpAllocator::build()
                    .expect("UnsafeRawAllocator: out of memory");
                let space = bump.inner_alloc(size).map_err(Into::into);
                self.head = Some(bump);
                space
            }
        }
    }

    // TODO: preallocate free blocks as appoesd of allocating on demand
    // expects a medium classed `size`, fails otherwise
    fn overflow_alloc(&mut self, size: usize) -> Result<*const u8, AllocError> {
        match self.overflow {
            Some(ref mut overflow) => {
                match overflow.inner_alloc(size) {
                    Ok(space) => Ok(space),
                    Err(BlockError::OOM) => {
                        let old = replace(overflow, BumpAllocator::build()
                                    .expect("UnsafeRawAllocator: out of memory"));
                        self.freed.push(old);
                        overflow.inner_alloc(size).map_err(Into::into)
                    },
                    Err(BlockError::BadRequest) => Err(AllocError::BadRequest)
                }
            },
            None => {
                let mut bump = BumpAllocator::build()
                    .expect("UnsafeRawAllocator: out of memory");
                let space = bump.inner_alloc(size).map_err(Into::into);
                self.overflow = Some(bump);
                space
            }
        }
    }
}

impl<H: AllocHeader> UnsafeAllocator<H> {
    pub fn new() -> Self {
        let inner = UnsafeRawAllocator::new();
        UnsafeAllocator {
            inner: UnsafeCell::new(inner),
            _header_type: PhantomData
        }
    }

    fn inner_alloc(&self, size: usize) -> Result<*const u8, AllocError> {
        let inner: &mut UnsafeRawAllocator = unsafe {
            &mut *self.inner.get()
        };
        inner.alloc(size)
    }
}

impl<H: AllocHeader> AllocRaw for UnsafeAllocator<H> {
    type Header = H;

    /// Allocates an data object of `Sized` type `T`, and copies `object` into
    /// the freshly allocated memory
    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, AllocError>
    where
        T: AllocObject<<Self::Header as AllocHeader>::TypeId> {
        let header_size = size_of::<Self::Header>();
        let object_size = size_of::<T>();
        let total_size = header_size + object_size;

        let size = (total_size + WORD_SIZE - 1) / WORD_SIZE;
        let memory = self.inner_alloc(size)?;

        // here for debugging purposes
        let mask = ALLOC_ALIGNMENT - 1;
        // println!("{:016X}", memory as usize);
        assert_eq!((memory as usize & mask) ^ mask, mask);

        unsafe {
            let header = Self::Header::new::<T>(object_size, Mark::Live);

            let memory = memory as *mut Self::Header;
            std::ptr::write(memory , header);

            let memory = memory.offset(1) as *mut T;
            std::ptr::write(memory, object);

            Ok(RawPtr::new(memory))
        }
    }

    /// Allocates an array of `arr_size` bytes, the allocated memory
    /// get's zero initialized
    fn alloc_array(&self, arr_size: usize) -> Result<RawPtr<u8>, AllocError> {
        let header_size = size_of::<Self::Header>();
        let total_size = header_size + arr_size;

        let size = (total_size + WORD_SIZE - 1) / WORD_SIZE;
        let memory = self.inner_alloc(size)?;

        // here for debugging purposes
        let mask = WORD_SIZE - 1;
        assert_eq!((memory as usize & mask) ^ mask, mask);

        unsafe {
            let header = Self::Header::new_array(size, Mark::Live);

            let memory = memory as *mut Self::Header;
            std::ptr::write(memory, header);

            let memory = memory.offset(1) as *mut u8;
            let slice = std::slice::from_raw_parts_mut(memory, size);
            for byte in slice {
                *byte = 0;
            }

            Ok(RawPtr::new(memory))
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
    pub fn alloc<T>(&self, object: T) -> Result<ScopedPtr<'_, T>, AllocError>
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


impl <'memory, H: AllocHeader> ScopedGlobalAllocator<'static, H> {
    pub fn new_static() -> ScopedGlobalAllocator<'static, H> {
        Self {
            inner: Box::leak(Box::new(UnsafeAllocator::<H>::new()))
        }
    }
}