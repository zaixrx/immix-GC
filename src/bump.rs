// Resources:
// - https://www.steveblackburn.org/pub(crate)s/papers/immix-pldi-2008.pdf
// - https://rust-hosted-langs.github.io/book/chapter-simple-bump.html -- Section 3.X

use crate::block::Block;
use crate::error::{AllocError, BlockError};

// Constants from Immix Paper
pub(crate) const BLOCK_SIZE_BITS: usize = 15;
pub(crate) const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS;

pub(crate) const LINE_SIZE_BITS: usize = 7;
pub(crate) const LINE_SIZE: usize = 1 << LINE_SIZE_BITS;

pub(crate) const LINES_COUNT: usize = BLOCK_SIZE / LINE_SIZE;
pub(crate) const BLOCK_CAPACITY: usize = BLOCK_SIZE - LINES_COUNT;

pub(crate) const ALLOC_ALIGNMENT: usize = 2 * size_of::<usize>();
pub(crate) const ALLOC_ALIGN_MASK: usize = !(ALLOC_ALIGNMENT - 1); 

#[derive(PartialEq)]
/// Small: zero or one line,
/// Medium: more than one line in a block,
/// Large: more than block
pub(crate) enum SizeClass {
    Small,
    Medium,
    Large,
}

impl SizeClass {
    pub(crate) fn new(size: usize) -> Self {
        match (size + LINE_SIZE - 1) / LINE_SIZE {
            0..=1 => Self::Small,
            2..=LINES_COUNT => Self::Medium,
            _ => Self::Large,
        }
    }
}

// current hole line bounds = { limit, ... , cursor - 1 }
pub(crate) struct BumpAllocator {
    block: Block,
    meta: BlockMeta,
    limit: *const u8,
    cursor: *const u8,
}

pub(crate) struct BlockMeta {
    lines: *const u8,
}

impl BlockMeta {
    /// Finds the first hole from the offset `start` that is larger than 
    /// or equal to `size` aligned up to `LINES_COUNT`
    ///
    /// Returns line start and end offsets respectively
    fn find_free_hole(&mut self, start: usize, size: usize) -> Option<(usize, usize)> {
        let starting_line = start / LINES_COUNT;
        let required_lines = (size + LINES_COUNT - 1) / LINES_COUNT;

        let mut count = 0;
        let mut top = starting_line;
        
        for index in (0..starting_line).rev() {
            let is_marked = unsafe { 
                *self.lines.add(index)
            };

            if is_marked == 0 {
                count += 1;
                if index == 0 && count >= required_lines {
                    let cursor = top * LINE_SIZE;
                    let limit = index * LINE_SIZE;
                    return Some((cursor, limit));
                }
            } else {
                if count + 1 >= required_lines {
                    let cursor = top * LINE_SIZE; 
                    let limit = (index + 2) * LINE_SIZE; // Conservatively mark one line
                    return Some((cursor, limit));
                }
                count = 0;
                top = index;
            }
        }

        None
    }
}

impl BumpAllocator {
    /// Builds a new bump allocator
    /// 
    /// Can only fail because of an `AllocError::OOM` error
    pub(crate) fn build() -> Result<BumpAllocator, AllocError> {
        let block = Block::build(BLOCK_SIZE).map_err(|err| {
            match err {
                BlockError::BadRequest => panic!("BumpAllocator: invalid `BLOCK_SIZE`"),
                BlockError::OOM => AllocError::OOM
            }
        })?;

        Ok(BumpAllocator{
            limit: block.as_ptr(),
            cursor: unsafe {
                block.as_ptr().add(BLOCK_CAPACITY)
            },
            meta: BlockMeta{
                lines: unsafe {
                    block.as_ptr().add(BLOCK_CAPACITY)
                }
            }, 
            block
        })
    }

    pub(crate) fn current_hole_size(&self) -> usize {
        let cursor = self.cursor as usize;
        let limit = self.limit as usize;
        cursor.checked_sub(limit).expect("BumpAllocator: cursor underflew limit")
    }

    pub(crate) fn inner_alloc(&mut self, size: usize) -> Result<*const u8, BlockError> {
        let ptr = self.cursor as usize;
        let limit = self.limit as usize;

        let next_ptr = ptr.checked_sub(size).ok_or(BlockError::BadRequest)? & ALLOC_ALIGN_MASK;
        if next_ptr >= limit {
            self.cursor = next_ptr as *const u8;
            return Ok(self.cursor);
        }

        let base = self.block.as_ptr() as usize;
        if next_ptr >= base {
            let start_offset = unsafe {
                next_ptr.unchecked_sub(base)
            };
            
            if let Some((cursor, limit)) = self.meta.find_free_hole(start_offset, size) {
                self.cursor = (cursor + base) as *const u8;
                self.limit = (limit + base) as *const u8;
                return self.inner_alloc(size);
            }
        }

        Err(BlockError::OOM)
    }
}
