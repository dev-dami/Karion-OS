use crate::drivers::timer;
use crate::vga;

fn wait_ticks(count: u64) {
    let start = timer::get_ticks();
    while timer::get_ticks() - start < count {
        core::hint::spin_loop();
    }
}

fn draw_logo() {
    let logo: [&str; 7] = [
        r"  _  __          _              ___  ____  ",
        r" | |/ /__ _ _ __(_) ___  _ __  / _ \/ ___| ",
        r" | ' // _` | '__| |/ _ \| '_ \| | | \___ \ ",
        r" | . \ (_| | |  | | (_) | | | | |_| |___) |",
        r" |_|\_\__,_|_|  |_|\___/|_| |_|\___/|____/ ",
        r"                                            ",
        r"        Unix-Like Kernel v3.0               ",
    ];

    let start_row = 3;
    for (i, line) in logo.iter().enumerate() {
        vga::write_centered(start_row + i, line, vga::LIGHT_CYAN);
        wait_ticks(1);
    }
}

fn draw_progress_bar() {
    let row = 12;
    let bar_width = 40;
    let bar_start = (80 - bar_width - 2) / 2;

    vga::put_char_at(row, bar_start, b'[', vga::WHITE);
    vga::put_char_at(row, bar_start + bar_width + 1, b']', vga::WHITE);

    let steps: [(&str, u8); 6] = [
        ("Initializing GDT...", vga::DARK_GRAY),
        ("Loading IDT...", vga::DARK_GRAY),
        ("Configuring PIC...", vga::DARK_GRAY),
        ("Setting up memory...", vga::DARK_GRAY),
        ("Loading drivers...", vga::DARK_GRAY),
        ("Starting shell...", vga::DARK_GRAY),
    ];

    let step_width = bar_width / steps.len();

    for (step_idx, (msg, _)) in steps.iter().enumerate() {
        vga::set_cursor(14, 0);
        for c in 0..80 {
            vga::put_char_at(14, c, b' ', vga::WHITE);
        }
        vga::write_centered(14, msg, vga::YELLOW);

        let fill_start = bar_start + 1 + step_idx * step_width;
        let fill_end = if step_idx == steps.len() - 1 {
            bar_start + 1 + bar_width
        } else {
            fill_start + step_width
        };

        for col in fill_start..fill_end {
            let progress_color = match step_idx {
                0..=1 => vga::CYAN,
                2..=3 => vga::LIGHT_CYAN,
                _ => vga::GREEN,
            };
            vga::put_char_at(row, col, 0xDB, progress_color); // CP437 full block
        }

        wait_ticks(2);
    }
}

fn draw_system_info() {
    let row = 16;
    vga::write_centered(row, "Karion-OS loaded successfully", vga::LIGHT_GREEN);
    wait_ticks(3);

    vga::write_centered(row + 2, "[ GDT ] [ IDT ] [ PIC ] [ PMM ] [ Paging ] [ Heap ]", vga::GREEN);
    wait_ticks(3);

    vga::write_centered(row + 3, "[ Timer IRQ0 ] [ Keyboard IRQ1 ] [ Syscall INT 0x80 ]", vga::GREEN);
    wait_ticks(3);
}

pub fn run() {
    vga::clear_screen();
    draw_logo();
    wait_ticks(5);
    draw_progress_bar();
    draw_system_info();
    wait_ticks(10);
    vga::clear_screen();
}
