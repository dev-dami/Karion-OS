use core::arch::asm;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

#[repr(C, packed)]
struct GdtPtr {
    limit: u16,
    base: u32,
}

const GDT_ENTRIES: usize = 5;
static mut GDT: [GdtEntry; GDT_ENTRIES] = [GdtEntry {
    limit_low: 0,
    base_low: 0,
    base_mid: 0,
    access: 0,
    granularity: 0,
    base_high: 0,
}; GDT_ENTRIES];
static mut GDT_PTR: GdtPtr = GdtPtr { limit: 0, base: 0 };

fn set_gate(idx: usize, base: u32, limit: u32, access: u8, gran: u8) {
    unsafe {
        let gdt_ptr = core::ptr::addr_of_mut!(GDT);
        (*gdt_ptr)[idx] = GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | (gran & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        };
    }
}

pub fn init() {
    set_gate(0, 0, 0, 0, 0);                       // Null segment
    set_gate(1, 0, 0xFFFFFFFF, 0x9A, 0xCF);        // Kernel code: ring 0, executable+readable
    set_gate(2, 0, 0xFFFFFFFF, 0x92, 0xCF);        // Kernel data: ring 0, writable
    set_gate(3, 0, 0xFFFFFFFF, 0xFA, 0xCF);        // User code: ring 3, executable+readable
    set_gate(4, 0, 0xFFFFFFFF, 0xF2, 0xCF);        // User data: ring 3, writable

    unsafe {
        let gdt_ptr = core::ptr::addr_of_mut!(GDT_PTR);
        let gdt_base = core::ptr::addr_of!(GDT) as u32;
        (*gdt_ptr) = GdtPtr {
            limit: (core::mem::size_of::<[GdtEntry; GDT_ENTRIES]>() - 1) as u16,
            base: gdt_base,
        };

        let gdt_ptr_addr = core::ptr::addr_of!(GDT_PTR);
        asm!(
            "lgdt ({ptr})",
            // Far jump to reload CS with kernel code segment (0x08)
            "ljmp $0x08, $2f",
            "2:",
            // Reload data segments with kernel data segment (0x10)
            "mov $0x10, %ax",
            "mov %ax, %ds",
            "mov %ax, %es",
            "mov %ax, %fs",
            "mov %ax, %gs",
            "mov %ax, %ss",
            ptr = in(reg) gdt_ptr_addr,
            options(att_syntax)
        );
    }
}
