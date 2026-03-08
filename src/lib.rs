#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

#[cfg(not(test))]
extern crate alloc;

mod basic;
mod blockfs;
#[cfg(not(test))]
mod boot_anim;
#[cfg(not(test))]
mod drivers;
#[cfg(not(test))]
mod editor;
mod fs;
#[cfg(not(test))]
mod games;
#[cfg(not(test))]
mod gdt;
#[cfg(not(test))]
mod heap;
#[cfg(not(test))]
mod idt;
mod intrinsics;
#[cfg(not(test))]
pub(crate) mod io;
#[cfg(not(test))]
mod isr;
mod keyboard;
#[cfg(not(test))]
mod paging;
#[cfg(not(test))]
pub(crate) mod pic;
#[cfg(not(test))]
mod pmm;
mod shell;
#[cfg(not(test))]
mod syscall;
mod vga;

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
use keyboard::KeyEvent;
use fs::FileSystem;
use shell::Shell;

struct Kernel {
    fs: FileSystem,
    shell: Shell,
}

impl Kernel {
    const fn new() -> Self {
        Self {
            fs: FileSystem::new(),
            shell: Shell::new(),
        }
    }

    #[cfg(not(test))]
    fn init(&mut self) {
        gdt::init();
        idt::init();
        pic::init();
        pmm::init();
        paging::init();
        heap::init();
        drivers::init_all();

        // sti required before boot_anim (uses timer ticks)
        unsafe { core::arch::asm!("sti"); }

        boot_anim::run();
        vga::enable_cursor();
        self.fs.init();
        self.shell.init();
    }

    #[cfg(not(test))]
    fn soft_reset(&mut self) {
        vga::clear_screen();
        self.fs.init();
        self.shell.init();
    }

    #[cfg(not(test))]
    fn tick(&mut self) {
        match keyboard::poll_key() {
            KeyEvent::None => {}
            KeyEvent::Esc => self.soft_reset(),
            KeyEvent::Enter => self.shell.submit(&mut self.fs),
            KeyEvent::Backspace => self.shell.backspace(),
            KeyEvent::ArrowUp => self.shell.history_up(),
            KeyEvent::ArrowDown => self.shell.history_down(),
            KeyEvent::Tab => {}
            KeyEvent::ArrowLeft | KeyEvent::ArrowRight => {}
            KeyEvent::CtrlChar(_) => {}
            KeyEvent::Char(c) => self.shell.push(c),
        }
    }
}

static mut KERNEL: Kernel = Kernel::new();

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        let kernel = core::ptr::addr_of_mut!(KERNEL);
        (*kernel).init();
        loop {
            (*kernel).tick();
            core::arch::asm!("hlt");
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    vga::write_str("KERNEL PANIC: ", vga::RED);
    if let Some(loc) = info.location() {
        vga::write_str(loc.file(), vga::RED);
        vga::write_str(":", vga::RED);
        let mut buf = [0u8; 10];
        let mut line = loc.line();
        if line == 0 {
            vga::write_str("0", vga::RED);
        } else {
            let mut pos = 10;
            while line > 0 && pos > 0 {
                pos -= 1;
                buf[pos] = b'0' + (line % 10) as u8;
                line /= 10;
            }
            if let Ok(s) = core::str::from_utf8(&buf[pos..]) {
                vga::write_str(s, vga::RED);
            }
        }
        vga::newline();
    }
    if let Some(msg) = info.message().as_str() {
        vga::write_line(msg, vga::RED);
    }
    loop {
        core::hint::spin_loop();
    }
}
