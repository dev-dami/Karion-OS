use crate::io;
use crate::isr::{self, Registers};
use crate::pic;

static mut TICKS: u64 = 0;

fn timer_handler(_regs: &Registers) {
    unsafe {
        let ticks = core::ptr::addr_of_mut!(TICKS);
        *ticks += 1;
    }
}

pub fn get_ticks() -> u64 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(TICKS)) }
}

pub fn init() {
    let divisor: u16 = 11932; // ~100Hz

    io::outb(0x43, 0x36);
    io::outb(0x40, (divisor & 0xFF) as u8);
    io::outb(0x40, ((divisor >> 8) & 0xFF) as u8);

    isr::register_irq_handler(0, timer_handler);
    pic::clear_mask(0);
}
