use crate::io;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

pub fn init() {
    let _mask1 = io::inb(PIC1_DATA);
    let _mask2 = io::inb(PIC2_DATA);

    // ICW1: begin init sequence in cascade mode
    io::outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io::io_wait();
    io::outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io::io_wait();

    // ICW2: remap IRQs away from CPU exceptions (0-31)
    io::outb(PIC1_DATA, 0x20); // IRQ 0-7 -> vectors 32-39
    io::io_wait();
    io::outb(PIC2_DATA, 0x28); // IRQ 8-15 -> vectors 40-47
    io::io_wait();

    // ICW3: wiring between master and slave
    io::outb(PIC1_DATA, 0x04); // Slave on IRQ2
    io::io_wait();
    io::outb(PIC2_DATA, 0x02); // Slave cascade identity
    io::io_wait();

    // ICW4: 8086 mode
    io::outb(PIC1_DATA, ICW4_8086);
    io::io_wait();
    io::outb(PIC2_DATA, ICW4_8086);
    io::io_wait();

    // Mask all IRQs until explicitly enabled
    io::outb(PIC1_DATA, 0xFF);
    io::outb(PIC2_DATA, 0xFF);
}

pub fn send_eoi(irq: u8) {
    if irq >= 8 {
        io::outb(PIC2_COMMAND, 0x20);
    }
    io::outb(PIC1_COMMAND, 0x20);
}

pub fn clear_mask(irq: u8) {
    if irq >= 8 {
        // Slave PIC IRQs also need cascade line (IRQ2) unmasked on master
        let master_val = io::inb(PIC1_DATA) & !(1 << 2);
        io::outb(PIC1_DATA, master_val);
        let value = io::inb(PIC2_DATA) & !(1 << (irq - 8));
        io::outb(PIC2_DATA, value);
    } else {
        let value = io::inb(PIC1_DATA) & !(1 << irq);
        io::outb(PIC1_DATA, value);
    }
}

pub fn set_mask(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_line = if irq < 8 { irq } else { irq - 8 };
    let value = io::inb(port) | (1 << irq_line);
    io::outb(port, value);
}
