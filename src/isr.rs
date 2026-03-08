use crate::pic;
use crate::syscall;
use crate::vga;

#[repr(C)]
pub struct Registers {
    pub ds: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub int_no: u32,
    pub err_code: u32,
    // CPU-pushed interrupt frame
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    // Only valid on ring 3 -> ring 0 transitions
    _useresp: u32,
    _ss: u32,
}

static EXCEPTION_MESSAGES: [&str; 32] = [
    "Division By Zero",
    "Debug",
    "Non Maskable Interrupt",
    "Breakpoint",
    "Into Detected Overflow",
    "Out of Bounds",
    "Invalid Opcode",
    "No Coprocessor",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Bad TSS",
    "Segment Not Present",
    "Stack Fault",
    "General Protection Fault",
    "Page Fault",
    "Unknown Interrupt",
    "Coprocessor Fault",
    "Alignment Check",
    "Machine Check",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
];

// One handler slot per IRQ line (0-15)
static mut IRQ_HANDLERS: [Option<fn(&Registers)>; 16] = [None; 16];

pub fn register_irq_handler(irq: usize, handler: fn(&Registers)) {
    if irq < 16 {
        unsafe {
            let handlers = core::ptr::addr_of_mut!(IRQ_HANDLERS);
            (*handlers)[irq] = Some(handler);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn isr_handler(regs: &Registers) {
    let int_no = regs.int_no;

    if int_no < 32 {
        let msg = if (int_no as usize) < EXCEPTION_MESSAGES.len() {
            EXCEPTION_MESSAGES[int_no as usize]
        } else {
            "Unknown Exception"
        };
        vga::write_str("EXCEPTION: ", vga::RED);
        vga::write_line(msg, vga::RED);

        // Page fault: read faulting address from CR2
        if int_no == 14 {
            let cr2: u32;
            unsafe {
                core::arch::asm!("mov {}, cr2", out(reg) cr2);
            }
            vga::write_str("Faulting address: 0x", vga::RED);
            let mut buf = [b'0'; 8];
            let mut val = cr2;
            for i in (0..8).rev() {
                let nibble = (val & 0xF) as u8;
                buf[i] = if nibble < 10 {
                    b'0' + nibble
                } else {
                    b'a' + nibble - 10
                };
                val >>= 4;
            }
            if let Ok(s) = core::str::from_utf8(&buf) {
                vga::write_line(s, vga::RED);
            }
        }

        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    } else if int_no >= 32 && int_no < 48 {
        let irq = (int_no - 32) as usize;

        unsafe {
            let handlers = core::ptr::addr_of!(IRQ_HANDLERS);
            if let Some(handler) = (*handlers)[irq] {
                handler(regs);
            }
        }

        pic::send_eoi(irq as u8);
    } else if int_no == 0x80 {
        syscall::syscall_handler(regs);
    }
}
