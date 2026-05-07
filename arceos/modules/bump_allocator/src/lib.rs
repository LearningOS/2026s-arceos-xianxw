#![no_std]

use allocator::{AllocError, BaseAllocator, ByteAllocator, PageAllocator};
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {

        if self.start == 0 && self.end == 0 {
            self.init(start, size);
            Ok(())
        } else {
            Err(allocator::AllocError::MemoryOverlap)
        }
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let ali_pos = (self.b_pos + layout.align() - 1) & !(layout.align() - 1);
        let next_pos = ali_pos + layout.size();
        if next_pos > self.end {
            return Err(AllocError::NotAllocated);
        }
        self.b_pos = next_pos;
        self.count += 1;
        Ok(NonNull::new(ali_pos as *mut u8).unwrap())
    }

    fn dealloc(&mut self, _pos: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.p_pos - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        let total_size = num_pages * SIZE;
        let align = align_pow2 * SIZE;
        let new_pos = (self.p_pos - total_size) & !(align - 1);
        if new_pos < self.b_pos {
            return Err(AllocError::NoMemory);
        }
        self.p_pos = new_pos;
        Ok(new_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        //页不会被释放
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / SIZE
    }
}