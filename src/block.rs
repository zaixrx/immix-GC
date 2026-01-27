use std::ptr::NonNull;
use std::alloc::Layout;

use crate::api_alloc::AllocError;

pub struct Block {
    ptr: BlockPtr,
    size: BlockSize,
}

pub type BlockPtr = NonNull<u8>;
pub type BlockSize = usize;

impl Block {
    pub fn build(size: BlockSize) -> Result<Self, AllocError> {
        if !size.is_power_of_two() {
            return Err(AllocError::BadRequest);
        }
        Ok(Block {
            ptr: internal::alloc_block(size)?,
            size 
        })
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        println!("Block::drop");
        internal::dealloc(self);
    }
}

mod internal {
    use super::*;

    pub fn alloc_block(size: BlockSize) -> Result<BlockPtr, AllocError> {
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, size);
            let ptr = std::alloc::alloc(layout);
            if ptr.is_null() {
                return Err(AllocError::OOM);
            } else {
                return Ok(NonNull::new_unchecked(ptr));
            }
        }
    }

    pub fn dealloc(block: &mut Block) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(block.size, block.size);
            std::alloc::dealloc(block.as_ptr(), layout);
        }
    }
}
