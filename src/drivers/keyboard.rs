use crate::io;
use crate::isr::{self, Registers};
use crate::pic;
use core::sync::atomic::{AtomicUsize, Ordering};

const BUFFER_SIZE: usize = 64;

static mut KEY_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static BUF_HEAD: AtomicUsize = AtomicUsize::new(0);
static BUF_TAIL: AtomicUsize = AtomicUsize::new(0);

fn keyboard_irq_handler(_regs: &Registers) {
    let scancode = io::inb(0x60);
    let head = BUF_HEAD.load(Ordering::Relaxed);
    let next_head = (head + 1) % BUFFER_SIZE;
    if next_head != BUF_TAIL.load(Ordering::Acquire) {
        unsafe {
            let buf = core::ptr::addr_of_mut!(KEY_BUFFER);
            (*buf)[head] = scancode;
        }
        BUF_HEAD.store(next_head, Ordering::Release);
    }
}

pub fn pop_scancode() -> u8 {
    let tail = BUF_TAIL.load(Ordering::Relaxed);
    if tail == BUF_HEAD.load(Ordering::Acquire) {
        return 0;
    }
    let code = unsafe {
        let buf = core::ptr::addr_of!(KEY_BUFFER);
        (*buf)[tail]
    };
    BUF_TAIL.store((tail + 1) % BUFFER_SIZE, Ordering::Release);
    code
}

pub fn init() {
    isr::register_irq_handler(1, keyboard_irq_handler);
    pic::clear_mask(1);
}
