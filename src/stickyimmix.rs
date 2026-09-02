
use crate::object::Mark;
use crate::error::{AllocError, BlockError};
use crate::rawptr::{AllocHeader, AllocObject, AllocRaw, RawPtr};
use crate::bump::{ALLOC_ALIGNMENT, BumpAllocator, SizeClass};

use std::cell::UnsafeCell;
use::std::mem::replace;
use::std::ptr::NonNull;
use::std::{marker::PhantomData};

const WORD_SIZE: usize = size_of::<usize>();

pub struct RawStickyImmixHeap {
    head: Option<BumpAllocator>,
    overflow: Option<BumpAllocator>,
    rest: Vec<BumpAllocator>,
}

pub struct StickyImmixHeap<H: AllocHeader> {
    inner: UnsafeCell<RawStickyImmixHeap>,
    _header_type: PhantomData<*const H>,
}

impl RawStickyImmixHeap {
    pub fn new() -> Self {
        Self {
            head: None,
            overflow: None,
            rest: Vec::new(),
        }
    }

    pub fn alloc(&mut self, size: usize) -> Result<*const u8, AllocError> {
        let class = SizeClass::new(size);
        if  class == SizeClass::Large {
            todo!("RawStickyImmixHeap: SizeClass::Large")
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
                                    .expect("RawStickyImmixHeap: out of memory"));

                                self.rest.push(old);

                                head.inner_alloc(size).map_err(Into::into)
                            },
                            Err(BlockError::BadRequest) => Err(AllocError::BadRequest)
                        }
                    }
                }
            },
            None => {
                let mut bump = BumpAllocator::build()
                    .expect("RawStickyImmixHeap: out of memory");

                let memory = bump.inner_alloc(size).map_err(Into::into);

                self.head = Some(bump);

                memory
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
                                    .expect("RawStickyImmixHeap: out of memory"));

                        self.rest.push(old);

                        overflow.inner_alloc(size).map_err(Into::into)
                    },
                    Err(BlockError::BadRequest) => Err(AllocError::BadRequest)
                }
            },
            None => {
                let mut bump = BumpAllocator::build()
                    .expect("RawStickyImmixHeap: out of memory");

                let space = bump.inner_alloc(size).map_err(Into::into);

                self.overflow = Some(bump);

                space
            }
        }
    }
}

impl<H: AllocHeader> StickyImmixHeap<H> {
    pub fn new() -> Self {
        let inner = RawStickyImmixHeap::new();
        StickyImmixHeap {
            inner: UnsafeCell::new(inner),
            _header_type: PhantomData
        }
    }

    /// Used to provide an immutable allocator interface, to comply with
    /// the internal mutability pattern
    fn inner_alloc(&self, size: usize) -> Result<*const u8, AllocError> {
        let inner: &mut RawStickyImmixHeap = unsafe {
            &mut *self.inner.get()
        };
        inner.alloc(size)
    }
}

impl<H: AllocHeader> AllocRaw for StickyImmixHeap<H> {
    type Header = H;

    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, AllocError>
    where
        T: AllocObject<<Self::Header as AllocHeader>::TypeId> {
        let header_size = size_of::<Self::Header>();
        let object_size = size_of::<T>();
        let total_size = header_size + object_size;

        let size = (total_size + WORD_SIZE - 1) / WORD_SIZE;
        let memory = self.inner_alloc(size)?;

        let mask = ALLOC_ALIGNMENT - 1;
        // println!("{:016X}", memory as usize);
        assert_eq!((memory as usize & mask) ^ mask, mask);

        let header = Self::Header::new::<T>(object_size, Mark::Live);

        unsafe {
            let memory = memory as *mut Self::Header;
            std::ptr::write(memory , header);

            let memory = memory.offset(1) as *mut T;
            std::ptr::write(memory, object);

            Ok(RawPtr::new(memory))
        }
    }

    /// get's zero initialized
    fn alloc_array(&self, arr_size: usize) -> Result<RawPtr<u8>, AllocError> {
        let header_size = size_of::<Self::Header>();
        let total_size = header_size + arr_size;

        let size = (total_size + WORD_SIZE - 1) / WORD_SIZE;
        let memory = self.inner_alloc(size)?;

        let mask = WORD_SIZE - 1;
        assert_eq!((memory as usize & mask) ^ mask, mask);

        let header = Self::Header::new_array(size, Mark::Live);

        unsafe {
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

#[cfg(test)]
mod raw_tests {
    use super::*;
    
    use crate::bump::{BLOCK_SIZE, BLOCK_CAPACITY, LINE_SIZE};

    #[test]
    fn test_alloc_head_none() -> Result<(), AllocError> {
        const ALLOC_SIZE: usize = LINE_SIZE;
        /*static*/ assert!(SizeClass::new(ALLOC_SIZE) == SizeClass::Small);

        let mut heap = RawStickyImmixHeap::new();

        // make sure head holds nothing
        assert!(heap.head.is_none());

        let _ = heap.alloc(ALLOC_SIZE)?;

        // make sure head holds an newly allocated block, then validate allocation
        assert!(heap.head.is_some());
        assert!(heap.head.unwrap().current_hole_size() == BLOCK_CAPACITY - ALLOC_SIZE);

        Ok(())
    }

    #[test]
    fn test_alloc_head_some() -> Result<(), AllocError> {
        const ALLOC_SIZE: usize = LINE_SIZE;
        /*static*/ assert!(SizeClass::new(ALLOC_SIZE) == SizeClass::Small);

        let mut heap = RawStickyImmixHeap::new();

        for _ in 0 .. BLOCK_CAPACITY / ALLOC_SIZE {
            let _ = heap.alloc(ALLOC_SIZE)?;
        }

        // make sure all allocations are small, and with the exact requested size
        assert!(heap.head.is_some());
        assert!(heap.head.unwrap().current_hole_size() == 0);

        Ok(())
    }

    #[test]
    fn test_alloc_head_some_oom() -> Result<(), AllocError> {
        const ALLOC_SIZE: usize = LINE_SIZE;
        /*static*/ assert!(SizeClass::new(ALLOC_SIZE) == SizeClass::Small);

        let mut heap = RawStickyImmixHeap::new();

        for _ in 0 .. BLOCK_CAPACITY / ALLOC_SIZE + 1 {
            let _ = heap.alloc(ALLOC_SIZE)?;
        }

        // make sure new block got assaigned to `head` with correct allocation size
        assert!(heap.head.is_some());
        assert!(heap.head.unwrap().current_hole_size() == BLOCK_CAPACITY - ALLOC_SIZE);

        // make sure old block entered `rest`, and that's it's filled as expected
        assert!(heap.rest.len() == 1);
        assert!(heap.rest[0].current_hole_size() == 0);

        Ok(())
    }

    #[test]
    fn test_alloc_overflow_none() -> Result<(), AllocError> {
        const ALLOC_SIZE: usize = BLOCK_SIZE / 2;
        /*static*/ assert!(SizeClass::new(ALLOC_SIZE) == SizeClass::Medium);

        let mut heap = RawStickyImmixHeap::new();

        assert!(heap.overflow.is_none());

        for _ in 0 .. 2 {
            let _ = heap.alloc(ALLOC_SIZE)?;
        }

        // make sure the old allocaiton is still inside `head` with the correct size
        assert!(heap.head.is_some());
        assert!(heap.head.unwrap().current_hole_size() == BLOCK_CAPACITY - ALLOC_SIZE);

        // validate second allocation fellback to overflow, and has the expected size
        assert!(heap.overflow.is_some());
        assert!(heap.overflow.unwrap().current_hole_size() == BLOCK_CAPACITY - ALLOC_SIZE);

        Ok(())
    }

    #[test]
    fn test_alloc_overflow_some() -> Result<(), AllocError> {
        const ALLOC_SIZE: usize = BLOCK_SIZE / 3 + 6; // must be multiple of 8
        /*static*/ assert!(SizeClass::new(ALLOC_SIZE) == SizeClass::Medium);

        let mut heap = RawStickyImmixHeap::new();

        for _ in 0 .. 4 {
            let _ = heap.alloc(ALLOC_SIZE)?;
            println!("{}", heap.head.as_ref().unwrap().current_hole_size());
        }

        // make sure `head` still holds old block with the expected size
        assert!(heap.head.is_some());
        assert!(heap.head.unwrap().current_hole_size() == BLOCK_CAPACITY - 2 * ALLOC_SIZE);

        // make sure `overflow` holds the new block with the expected size
        assert!(heap.overflow.is_some());
        assert!(heap.overflow.unwrap().current_hole_size() == BLOCK_CAPACITY - 2 * ALLOC_SIZE);

        Ok(())
    }

    #[test]
    fn test_alloc_overflow_some_oom() -> Result<(), AllocError> {
        const ALLOC_SIZE: usize = BLOCK_SIZE / 3 + 6; // must be multiple of 8
        /*static*/ assert!(SizeClass::new(ALLOC_SIZE) == SizeClass::Medium);

        let mut heap = RawStickyImmixHeap::new();

        for _ in 0 .. 5 {
            let _ = heap.alloc(ALLOC_SIZE)?;
            println!("{}", heap.head.as_ref().unwrap().current_hole_size());
        }

        // make sure `head` still holds old block with the expected size
        assert!(heap.head.is_some());
        assert!(heap.head.unwrap().current_hole_size() == BLOCK_CAPACITY - 2 * ALLOC_SIZE);

        // make sure `overflow` holds the new block with the expected size
        assert!(heap.overflow.is_some());
        assert!(heap.overflow.unwrap().current_hole_size() == BLOCK_CAPACITY - ALLOC_SIZE);

        // make sure `rest` holds the filled overflow block with the expected size
        assert!(heap.rest.len() == 1);
        assert!(heap.rest[0].current_hole_size() == BLOCK_CAPACITY - 2 * ALLOC_SIZE);

        Ok(())
    }

    // TODO: test_alloc_large
}