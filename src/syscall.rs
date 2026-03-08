use crate::isr::Registers;
use crate::vga;

// Linux-compatible syscall numbers
const SYS_READ: u32 = 0;
const SYS_WRITE: u32 = 1;
const SYS_EXIT: u32 = 60;

pub fn syscall_handler(regs: &Registers) {
    match regs.eax {
        SYS_READ => sys_read(regs),
        SYS_WRITE => sys_write(regs),
        SYS_EXIT => sys_exit(regs),
        _ => {
            vga::write_str("Unknown syscall: ", vga::RED);
            let mut buf = [0u8; 10];
            let s = uint_to_str(regs.eax, &mut buf);
            vga::write_line(s, vga::RED);
        }
    }
}

fn sys_write(regs: &Registers) {
    let fd = regs.ebx;
    let buf_ptr = regs.ecx as *const u8;
    let len = regs.edx as usize;

    if fd == 1 || fd == 2 {
        // stdout or stderr
        for i in 0..len {
            let ch = unsafe { core::ptr::read_volatile(buf_ptr.add(i)) };
            vga::put_char(ch, vga::WHITE);
        }
    }
}

fn sys_read(regs: &Registers) {
    let _fd = regs.ebx;
    let _buf_ptr = regs.ecx as *mut u8;
    let _len = regs.edx as usize;
    // TODO: read from keyboard buffer when process model is ready
}

fn sys_exit(_regs: &Registers) {
    vga::write_line("Process exited", vga::YELLOW);
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

fn uint_to_str(mut val: u32, buf: &mut [u8; 10]) -> &str {
    if val == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut pos = 10;
    while val > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    core::str::from_utf8(&buf[pos..]).unwrap_or("?")
}
