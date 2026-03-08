use crate::drivers::timer;
use crate::keyboard::{self, KeyEvent};
use crate::vga;

fn write_u32(mut val: u32, color: u8) {
    if val == 0 {
        vga::put_char(b'0', color);
        return;
    }
    let mut buf = [0u8; 10];
    let mut pos = 10;
    while val > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    for &b in &buf[pos..] {
        vga::put_char(b, color);
    }
}

pub fn run() {
    let mut seed = timer::get_ticks() as u32;

    loop {
        vga::clear_screen();
        vga::write_centered(1, "NUMBER GUESSING GAME", vga::YELLOW);
        vga::write_centered(3, "Guess a number between 1 and 100  -  ESC to quit", vga::CYAN);
        vga::newline();

        // Generate target 1-100
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let target = ((seed >> 16) & 0x7fff) % 100 + 1;

        let mut guesses: u32 = 0;
        let mut input_buf = [0u8; 3];
        let mut input_len: usize = 0;

        vga::set_cursor(5, 2);
        vga::write_str("Enter your guess: ", vga::WHITE);

        loop {
            let key = keyboard::poll_key();
            match key {
                KeyEvent::Esc => return,
                KeyEvent::Char(c) if c >= b'0' && c <= b'9' => {
                    if input_len < 3 {
                        input_buf[input_len] = c;
                        input_len += 1;
                        vga::put_char(c, vga::WHITE);
                    }
                }
                KeyEvent::Backspace => {
                    if input_len > 0 {
                        input_len -= 1;
                        vga::backspace();
                    }
                }
                KeyEvent::Enter => {
                    if input_len == 0 {
                        continue;
                    }
                    // Parse number
                    let mut val: u32 = 0;
                    for i in 0..input_len {
                        val = val * 10 + (input_buf[i] - b'0') as u32;
                    }
                    input_len = 0;
                    guesses += 1;

                    vga::newline();
                    vga::set_cursor(vga::cursor_row(), 2);

                    if val < 1 || val > 100 {
                        vga::write_str("Please enter 1-100. Try again: ", vga::BROWN);
                        continue;
                    }

                    if val == target {
                        vga::newline();
                        vga::set_cursor(vga::cursor_row(), 2);
                        vga::write_str("Correct! You got it in ", vga::LIGHT_GREEN);
                        write_u32(guesses, vga::LIGHT_GREEN);
                        vga::write_str(" guess(es)!", vga::LIGHT_GREEN);
                        vga::newline();
                        vga::newline();
                        vga::set_cursor(vga::cursor_row(), 2);
                        vga::write_str("ENTER to play again  -  ESC to quit", vga::LIGHT_GRAY);

                        loop {
                            match keyboard::poll_key() {
                                KeyEvent::Esc => return,
                                KeyEvent::Enter => {
                                    seed = timer::get_ticks() as u32;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        break;
                    } else if val < target {
                        vga::write_str("Too low! Guess again: ", vga::LIGHT_BLUE);
                    } else {
                        vga::write_str("Too high! Guess again: ", vga::LIGHT_RED);
                    }
                }
                _ => {}
            }
        }
    }
}
