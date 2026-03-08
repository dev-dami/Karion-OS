use crate::pmm;

const PAGE_PRESENT: u32 = 0x01;
const PAGE_WRITABLE: u32 = 0x02;
const PAGE_USER: u32 = 0x04;

const ENTRIES_PER_TABLE: usize = 1024;

#[repr(C, align(4096))]
struct PageDirectory {
    entries: [u32; ENTRIES_PER_TABLE],
}

#[repr(C, align(4096))]
#[derive(Copy, Clone)]
struct PageTable {
    entries: [u32; ENTRIES_PER_TABLE],
}

// 5 page tables identity-map the first 20MB: kernel (0-4MB) + heap (16-20MB)
static mut PAGE_DIR: PageDirectory = PageDirectory {
    entries: [0; ENTRIES_PER_TABLE],
};
static mut PAGE_TABLES: [PageTable; 5] = [PageTable {
    entries: [0; ENTRIES_PER_TABLE],
}; 5];

pub fn init() {
    unsafe {
        let dir = core::ptr::addr_of_mut!(PAGE_DIR);
        let tables = core::ptr::addr_of_mut!(PAGE_TABLES);

        // Identity map first 20MB (5 tables x 4MB each)
        for t in 0..5 {
            for i in 0..ENTRIES_PER_TABLE {
                let phys_addr = (t * ENTRIES_PER_TABLE + i) * 4096;
                (*tables)[t].entries[i] = (phys_addr as u32) | PAGE_PRESENT | PAGE_WRITABLE;
            }
            (*dir).entries[t] =
                (core::ptr::addr_of!((*tables)[t]) as u32) | PAGE_PRESENT | PAGE_WRITABLE;
        }

        // Load page directory into CR3 and set PG bit (bit 31) in CR0
        let dir_phys = dir as *const _ as u32;
        core::arch::asm!(
            "mov cr3, {dir}",
            "mov {tmp}, cr0",
            "or {tmp}, 0x80000000",
            "mov cr0, {tmp}",
            dir = in(reg) dir_phys,
            tmp = out(reg) _,
        );
    }
}

// Maps a 4KB page for addresses beyond the initial 20MB identity map.
// Allocated page tables must fall within the identity-mapped region (<20MB)
// so they're writable. PMM allocates low addresses first, so this holds
// as long as the first 5120 frames aren't exhausted.
pub fn map_page(virt: usize, phys: usize, flags: u32) {
    let dir_idx = virt >> 22;
    let table_idx = (virt >> 12) & 0x3FF;

    unsafe {
        let dir = core::ptr::addr_of_mut!(PAGE_DIR);

        if (*dir).entries[dir_idx] & PAGE_PRESENT == 0 {
            if let Some(table_phys) = pmm::alloc_frame() {
                // Page table must be in identity-mapped region to be writable
                if table_phys >= 20 * 1024 * 1024 {
                    pmm::free_frame(table_phys);
                    return;
                }
                let table_ptr = table_phys as *mut u32;
                for i in 0..ENTRIES_PER_TABLE {
                    core::ptr::write_volatile(table_ptr.add(i), 0);
                }
                (*dir).entries[dir_idx] = (table_phys as u32) | PAGE_PRESENT | PAGE_WRITABLE;
            } else {
                return;
            }
        }

        let table_phys_addr = (*dir).entries[dir_idx] & 0xFFFFF000;
        if (table_phys_addr as usize) >= 20 * 1024 * 1024 {
            return;
        }
        let table_ptr = table_phys_addr as *mut u32;
        core::ptr::write_volatile(
            table_ptr.add(table_idx),
            (phys as u32) | flags | PAGE_PRESENT,
        );

        core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
    }
}
