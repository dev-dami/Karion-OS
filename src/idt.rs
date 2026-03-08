use core::arch::asm;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    base_low: u16,
    sel: u16,
    always0: u8,
    flags: u8,
    base_high: u16,
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u32,
}

const IDT_ENTRIES: usize = 256;

static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry {
    base_low: 0,
    sel: 0,
    always0: 0,
    flags: 0,
    base_high: 0,
}; IDT_ENTRIES];

static mut IDT_PTR: IdtPtr = IdtPtr { limit: 0, base: 0 };

fn set_gate(idx: usize, base: u32, sel: u16, flags: u8) {
    unsafe {
        let idt = core::ptr::addr_of_mut!(IDT);
        (*idt)[idx] = IdtEntry {
            base_low: (base & 0xFFFF) as u16,
            sel,
            always0: 0,
            flags,
            base_high: ((base >> 16) & 0xFFFF) as u16,
        };
    }
}

unsafe extern "C" {
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
    fn isr32();
    fn isr33();
    fn isr34();
    fn isr35();
    fn isr36();
    fn isr37();
    fn isr38();
    fn isr39();
    fn isr40();
    fn isr41();
    fn isr42();
    fn isr43();
    fn isr44();
    fn isr45();
    fn isr46();
    fn isr47();
    fn isr128();
}

pub fn init() {
    let isrs: [unsafe extern "C" fn(); 48] = [
        isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7, isr8, isr9, isr10, isr11, isr12, isr13,
        isr14, isr15, isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23, isr24, isr25,
        isr26, isr27, isr28, isr29, isr30, isr31, isr32, isr33, isr34, isr35, isr36, isr37,
        isr38, isr39, isr40, isr41, isr42, isr43, isr44, isr45, isr46, isr47,
    ];

    // 0x08 = kernel code segment, 0x8E = present | ring 0 | 32-bit interrupt gate
    for (i, isr) in isrs.iter().enumerate() {
        set_gate(i, *isr as u32, 0x08, 0x8E);
    }

    // 0xEE = present | DPL 3 | 32-bit interrupt gate (callable from userspace)
    set_gate(0x80, isr128 as unsafe extern "C" fn() as u32, 0x08, 0xEE);

    unsafe {
        let idt = core::ptr::addr_of!(IDT);
        let idt_ptr = core::ptr::addr_of_mut!(IDT_PTR);
        (*idt_ptr) = IdtPtr {
            limit: (core::mem::size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
            base: idt as *const _ as u32,
        };
        asm!("lidt [{}]", in(reg) idt_ptr, options(nostack));
    }
}
