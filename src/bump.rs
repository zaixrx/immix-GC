// Resources:
// - https://www.steveblackburn.org/pubs/papers/immix-pldi-2008.pdf
// - https://rust-hosted-langs.github.io/book/chapter-simple-bump.html

use crate::block::*;

// Constants from Immix Paper
const ALLOC_ALIGN_MASK: usize = !(size_of::<usize>() - 1); 

const BLOCK_SIZE_BITS: usize = 15;
const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS;

const LINE_SIZE_BITS: usize = 7;
const LINE_SIZE: usize = 1 << LINE_SIZE_BITS;

const LINES_COUNT: usize = BLOCK_SIZE / LINE_SIZE;
const BLOCK_CAPACITY: usize = BLOCK_SIZE - LINES_COUNT;

// free region bounds = { bump_block.limit, ... , bump_block.cursor - 1 }
pub struct BumpBlock {
    block: Block,
    limit: *const u8,
    cursor: *const u8,
    meta: BlockMeta,
}

pub struct BlockMeta {
    lines: *const u8,
}

impl BumpBlock {
    pub fn build() -> Result<BumpBlock, BlockError> {
        let block = Block::build(BLOCK_SIZE)?;
        let limit = block.as_ptr();
        let cursor = unsafe { block.as_ptr().add(BLOCK_CAPACITY) };
        let lines = unsafe { block.as_ptr().add(BLOCK_CAPACITY) };
        Ok(BumpBlock{ block, limit, cursor, meta: BlockMeta{ lines }, })
    }

    // returns the biggest sequance of free lines containning-
    // "size" bytes starting from "starting_line", in the form-
    // of a region (cursor: usize, limit: usize)
    fn find_free_region(&mut self, starting_at: usize, size: usize) -> Option<(usize, usize)> {
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
                    let limit = (line_index + 2) * LINE_SIZE;
                    return Some((cursor, limit));
                }
                count = 0;
                end = line_index;
            }
        }

        None
    }

    fn inner_alloc(&mut self, size: usize) -> Option<*const u8> {
        let cursor = self.cursor as usize;
        let limit = self.limit as usize;
        let target = cursor.checked_sub(size)? & ALLOC_ALIGN_MASK;

        if target >= limit {
            self.cursor = target as *const u8;
            return Some(self.cursor);
        }

        if let Some((cursor, limit)) = self.find_free_region(cursor, size) {
            self.cursor = cursor as *const u8;
            self.limit = limit as *const u8;
            return self.inner_alloc(size);
        }

        None
    }
}
