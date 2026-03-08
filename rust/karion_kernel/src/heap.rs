use core::alloc::{GlobalAlloc, Layout};

const HEAP_START: usize = 0x1000000; // 16MB
const HEAP_SIZE: usize = 0x400000; // 4MB

#[repr(C)]
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

pub struct HeapAllocator {
    head: *mut FreeBlock,
}

unsafe impl Send for HeapAllocator {}
unsafe impl Sync for HeapAllocator {}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
        }
    }
}

#[global_allocator]
static mut ALLOCATOR: HeapAllocator = HeapAllocator::new();

pub fn init() {
    unsafe {
        let alloc = core::ptr::addr_of_mut!(ALLOCATOR);
        let head = HEAP_START as *mut FreeBlock;
        (*head).size = HEAP_SIZE;
        (*head).next = core::ptr::null_mut();
        (*alloc).head = head;
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let min_size = core::mem::size_of::<FreeBlock>();
        let size = layout.size().max(min_size);
        let align = layout.align().max(core::mem::align_of::<FreeBlock>());
        let size = (size + align - 1) & !(align - 1);

        unsafe {
            let alloc = core::ptr::addr_of_mut!(ALLOCATOR);
            let mut prev: *mut FreeBlock = core::ptr::null_mut();
            let mut current = (*alloc).head;

            while !current.is_null() {
                let block_start = current as usize;
                let block_size = (*current).size;
                let aligned_addr = (block_start + align - 1) & !(align - 1);
                let offset = aligned_addr - block_start;

                if block_size >= size + offset {
                    let remaining = block_size - size - offset;

                    if offset >= min_size {
                        // Pre-alignment gap is large enough to keep as a free block
                        (*current).size = offset;
                        if remaining >= min_size {
                            let new_block = (aligned_addr + size) as *mut FreeBlock;
                            (*new_block).size = remaining;
                            (*new_block).next = (*current).next;
                            (*current).next = new_block;
                        }
                    } else {
                        // Absorb small alignment gap into the allocation
                        let total_alloc = size + offset;
                        let remaining = block_size - total_alloc;

                        if remaining >= min_size {
                            let new_block = (block_start + total_alloc) as *mut FreeBlock;
                            (*new_block).size = remaining;
                            (*new_block).next = (*current).next;

                            if prev.is_null() {
                                (*alloc).head = new_block;
                            } else {
                                (*prev).next = new_block;
                            }
                        } else {
                            if prev.is_null() {
                                (*alloc).head = (*current).next;
                            } else {
                                (*prev).next = (*current).next;
                            }
                        }
                    }

                    return aligned_addr as *mut u8;
                }

                prev = current;
                current = (*current).next;
            }
        }

        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let min_size = core::mem::size_of::<FreeBlock>();
        let size = layout.size().max(min_size);
        let align = layout.align().max(core::mem::align_of::<FreeBlock>());
        let size = (size + align - 1) & !(align - 1);

        unsafe {
            let alloc = core::ptr::addr_of_mut!(ALLOCATOR);
            let addr = ptr as usize;

            // Insert into address-sorted free list for coalescing
            let mut prev: *mut FreeBlock = core::ptr::null_mut();
            let mut current = (*alloc).head;

            while !current.is_null() && (current as usize) < addr {
                prev = current;
                current = (*current).next;
            }

            let block = ptr as *mut FreeBlock;
            (*block).size = size;
            (*block).next = current;

            if prev.is_null() {
                (*alloc).head = block;
            } else {
                (*prev).next = block;
            }

            // Coalesce with next block
            if !current.is_null() {
                let block_end = addr + size;
                if block_end == current as usize {
                    (*block).size += (*current).size;
                    (*block).next = (*current).next;
                }
            }

            // Coalesce with previous block
            if !prev.is_null() {
                let prev_end = prev as usize + (*prev).size;
                if prev_end == addr {
                    (*prev).size += (*block).size;
                    (*prev).next = (*block).next;
                }
            }
        }
    }
}
