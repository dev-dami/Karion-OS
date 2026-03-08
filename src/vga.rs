// VGA text-mode framebuffer (CGA-compatible, color mode)
const VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const CELLS: usize = WIDTH * HEIGHT;

pub const BLACK: u8 = 0;
pub const BLUE: u8 = 1;
pub const GREEN: u8 = 2;
pub const CYAN: u8 = 3;
pub const RED: u8 = 4;
pub const MAGENTA: u8 = 5;
pub const BROWN: u8 = 6;
pub const LIGHT_GRAY: u8 = 7;
pub const DARK_GRAY: u8 = 8;
pub const LIGHT_BLUE: u8 = 9;
pub const LIGHT_GREEN: u8 = 10;
pub const LIGHT_CYAN: u8 = 11;
pub const LIGHT_RED: u8 = 12;
pub const LIGHT_MAGENTA: u8 = 13;
pub const YELLOW: u8 = 14;
pub const WHITE: u8 = 15;

static mut INDEX: usize = 0;

#[inline]
fn cell(ch: u8, color: u8) -> u16 {
    (ch as u16) | ((color as u16) << 8)
}

pub fn clear_screen() {
    unsafe {
        for i in 0..CELLS {
            core::ptr::write_volatile(VGA_BUFFER.add(i), cell(b' ', WHITE));
        }
        INDEX = 0;
    }
}

pub fn clear_screen_color(bg: u8) {
    let attr = (bg << 4) | WHITE;
    unsafe {
        for i in 0..CELLS {
            core::ptr::write_volatile(VGA_BUFFER.add(i), cell(b' ', attr));
        }
        INDEX = 0;
    }
}

fn scroll_up() {
    unsafe {
        for i in 0..(WIDTH * (HEIGHT - 1)) {
            let next = core::ptr::read_volatile(VGA_BUFFER.add(i + WIDTH));
            core::ptr::write_volatile(VGA_BUFFER.add(i), next);
        }

        for i in (WIDTH * (HEIGHT - 1))..CELLS {
            core::ptr::write_volatile(VGA_BUFFER.add(i), cell(b' ', WHITE));
        }

        INDEX = WIDTH * (HEIGHT - 1);
    }
}

pub fn newline() {
    unsafe {
        let line = INDEX / WIDTH;
        INDEX = (line + 1) * WIDTH;
        if INDEX >= CELLS {
            scroll_up();
        }
    }
}

pub fn backspace() {
    unsafe {
        if INDEX == 0 {
            return;
        }
        INDEX -= 1;
        core::ptr::write_volatile(VGA_BUFFER.add(INDEX), cell(b' ', WHITE));
    }
}

pub fn put_char(ch: u8, color: u8) {
    if ch == b'\n' {
        newline();
        return;
    }

    unsafe {
        if INDEX >= CELLS {
            scroll_up();
        }
        core::ptr::write_volatile(VGA_BUFFER.add(INDEX), cell(ch, color));
        INDEX += 1;
        if INDEX >= CELLS {
            scroll_up();
        }
    }
}

pub fn put_char_at(row: usize, col: usize, ch: u8, color: u8) {
    if row >= HEIGHT || col >= WIDTH {
        return;
    }
    let pos = row * WIDTH + col;
    unsafe {
        core::ptr::write_volatile(VGA_BUFFER.add(pos), cell(ch, color));
    }
}

pub fn set_cursor(row: usize, col: usize) {
    if row < HEIGHT && col < WIDTH {
        unsafe {
            INDEX = row * WIDTH + col;
        }
    }
}

pub fn cursor_row() -> usize {
    unsafe { INDEX / WIDTH }
}

pub fn cursor_col() -> usize {
    unsafe { INDEX % WIDTH }
}

pub fn write_str(text: &str, color: u8) {
    for b in text.bytes() {
        put_char(b, color);
    }
}

pub fn write_line(text: &str, color: u8) {
    write_str(text, color);
    newline();
}

pub fn write_centered(row: usize, text: &str, color: u8) {
    let col = if text.len() < WIDTH {
        (WIDTH - text.len()) / 2
    } else {
        0
    };
    set_cursor(row, col);
    write_str(text, color);
}

pub fn fill_row(row: usize, ch: u8, color: u8) {
    if row >= HEIGHT {
        return;
    }
    for col in 0..WIDTH {
        put_char_at(row, col, ch, color);
    }
}
