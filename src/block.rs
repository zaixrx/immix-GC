use std::ptr::NonNull;
use std::alloc::Layout;

pub(crate) struct Block {
    ptr: BlockPtr,
    size: BlockSize,
}

pub(crate) type BlockPtr = NonNull<u8>;
pub(crate) type BlockSize = usize;

#[derive(Debug)]
pub(crate) enum BlockError {
    BadRequest,
    OOM, 
}

impl Block {
    pub(crate) fn build(size: BlockSize) -> Result<Self, BlockError> {
        if !size.is_power_of_two() {
            return Err(BlockError::BadRequest);
        }
        Ok(Block {
            ptr: internal::alloc_block(size)?,
            size 
        })
    }

    pub(crate) fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        internal::dealloc(self);
    }
}

mod internal {
    use super::*;

    pub(crate) fn alloc_block(size: BlockSize) -> Result<BlockPtr, BlockError> {
        let ptr = unsafe {
            // blocks are aligned to their size
            let layout = Layout::from_size_align_unchecked(size, size);
            std::alloc::alloc(layout)
        };

        if ptr.is_null() {
             Err(BlockError::OOM)
        } else {
            unsafe {
                Ok(NonNull::new_unchecked(ptr))
            }
        }
    }

    pub(crate) fn dealloc(block: &mut Block) -> () {
        unsafe {
            let layout = Layout::from_size_align_unchecked(block.size, block.size);
            std::alloc::dealloc(block.as_ptr(), layout)
        }
    }
}
