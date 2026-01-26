// Resources:
// - https://www.steveblackburn.org/pubs/papers/immix-pldi-2008.pdf
// - https://rust-hosted-langs.github.io/book/chapter-simple-bump.html -- Section 3.X
use crate::block::*;

// Constants from Immix Paper
const BLOCK_SIZE_BITS: usize = 15;
const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS;

const LINE_SIZE_BITS: usize = 7;
const LINE_SIZE: usize = 1 << LINE_SIZE_BITS;

const ALLOC_ALIGN_MASK: usize = !(size_of::<usize>() - 1); 
const LINES_COUNT: usize = BLOCK_SIZE / LINE_SIZE;
const BLOCK_CAPACITY: usize = BLOCK_SIZE - LINES_COUNT;

#[derive(PartialEq)]
pub enum SizeClass {
    Small, // exactly one line
    Medium, // more than one line
    Large,  // more than block
}

impl SizeClass {
    pub fn new(size: usize) -> Self {
        match (size + LINE_SIZE - 1) / LINE_SIZE {
            0..=1 => Self::Small,
            2..=LINES_COUNT => Self::Medium,
            _ => Self::Large,
        }
    }
}

// current free region bounds = { bump_block.limit, ... , bump_block.cursor - 1 }
pub struct ThreadLocalAllocator {
    block: Block,
    meta: BlockMeta,
    limit: *const u8,
    cursor: *const u8,
}

pub struct BlockMeta {
    lines: *const u8,
}

impl ThreadLocalAllocator {
    pub fn build() -> Result<ThreadLocalAllocator, BlockError> {
        let block = Block::build(BLOCK_SIZE)?;
        let limit = block.as_ptr();
        let cursor = unsafe { block.as_ptr().add(BLOCK_CAPACITY) };
        let lines = unsafe { block.as_ptr().add(BLOCK_CAPACITY) };
        Ok(ThreadLocalAllocator{ block, limit, cursor, meta: BlockMeta{ lines }, })
    }

    pub fn current_hole_size(&self) -> usize {
        let cursor = self.cursor as usize;
        let limit = self.limit as usize;
        cursor.checked_sub(limit).expect("TLA: cursor underflew limit")
    }

    // returns the biggest sequance of free lines containning-
    // "size" bytes starting from "starting_line", in the form-
    // of a region (cursor: usize, limit: usize)
    fn find_free_hole(&mut self, starting_at: usize, size: usize) -> Option<(usize, usize)> {
        let starting_line = starting_at / LINES_COUNT;
        let occupied_lines = (size + LINES_COUNT - 1) / LINES_COUNT;
        let mut end = starting_line;
        let mut count = 0;

        for line_index in (0..starting_line).rev() {
            let is_marked = unsafe { *self.meta.lines.add(line_index) };
            if is_marked == 0 {
                count += 1;
                if line_index == 0 && count >= occupied_lines {
                    let cursor = end * LINE_SIZE;
                    let limit = line_index * LINE_SIZE;
                    return Some((cursor, limit));
                }
            } else {
                if count > occupied_lines {
                    let cursor = end * LINE_SIZE; 
                    let limit = (line_index + 2) * LINE_SIZE; // Conservatively mark one line
                    return Some((cursor, limit));
                }
                count = 0;
                end = line_index;
            }
        }

        None
    }

    pub fn inner_alloc(&mut self, size: usize) -> Option<*const u8> {
        let cursor = self.cursor as usize;
        let limit = self.limit as usize;
        let target = cursor.checked_sub(size)? & ALLOC_ALIGN_MASK;

        if target >= limit {
            self.cursor = target as *const u8;
            return Some(self.cursor);
        }

        if target >= self.block.as_ptr() as usize {
            if let Some((cursor, limit)) = self.find_free_hole(cursor, size) {
                self.cursor = cursor as *const u8;
                self.limit = limit as *const u8;
                return self.inner_alloc(size);
            }
        }

        None
    }
}
