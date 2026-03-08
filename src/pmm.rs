const FRAME_SIZE: usize = 4096;
const MAX_FRAMES: usize = 32768; // 128MB addressable (32768 * 4KB)
const BITMAP_SIZE: usize = MAX_FRAMES / 32;

// All frames marked used initially
static mut BITMAP: [u32; BITMAP_SIZE] = [0xFFFFFFFF; BITMAP_SIZE];
static mut TOTAL_FRAMES: usize = 0;
static mut USED_FRAMES: usize = 0;

fn set_frame(frame: usize) {
    unsafe {
        let bm = core::ptr::addr_of_mut!(BITMAP);
        (*bm)[frame / 32] |= 1 << (frame % 32);
    }
}

fn clear_frame(frame: usize) {
    unsafe {
        let bm = core::ptr::addr_of_mut!(BITMAP);
        (*bm)[frame / 32] &= !(1 << (frame % 32));
    }
}

fn test_frame(frame: usize) -> bool {
    unsafe {
        let bm = core::ptr::addr_of!(BITMAP);
        (*bm)[frame / 32] & (1 << (frame % 32)) != 0
    }
}

// Assumes 32MB RAM. First 4MB reserved for kernel/stack/early structures.
pub fn init() {
    let mem_size: usize = 32 * 1024 * 1024;
    let total = mem_size / FRAME_SIZE;

    unsafe {
        let tf = core::ptr::addr_of_mut!(TOTAL_FRAMES);
        *tf = total;
        let uf = core::ptr::addr_of_mut!(USED_FRAMES);
        *uf = total;
    }

    // Free frames from 4MB to 32MB (frames 1024..8192)
    let start_frame = 4 * 1024 * 1024 / FRAME_SIZE;
    let end_frame = total;
    for i in start_frame..end_frame {
        clear_frame(i);
        unsafe {
            let uf = core::ptr::addr_of_mut!(USED_FRAMES);
            *uf -= 1;
        }
    }
}

pub fn alloc_frame() -> Option<usize> {
    unsafe {
        let bm = core::ptr::addr_of!(BITMAP);
        for i in 0..BITMAP_SIZE {
            if (*bm)[i] != 0xFFFFFFFF {
                for bit in 0..32u32 {
                    let frame = i * 32 + bit as usize;
                    if !test_frame(frame) {
                        set_frame(frame);
                        let uf = core::ptr::addr_of_mut!(USED_FRAMES);
                        *uf += 1;
                        return Some(frame * FRAME_SIZE);
                    }
                }
            }
        }
    }
    None
}

pub fn free_frame(addr: usize) {
    let frame = addr / FRAME_SIZE;
    if test_frame(frame) {
        clear_frame(frame);
        unsafe {
            let uf = core::ptr::addr_of_mut!(USED_FRAMES);
            *uf -= 1;
        }
    }
}

pub fn free_frame_count() -> usize {
    unsafe {
        let tf = core::ptr::addr_of!(TOTAL_FRAMES);
        let uf = core::ptr::addr_of!(USED_FRAMES);
        *tf - *uf
    }
}
