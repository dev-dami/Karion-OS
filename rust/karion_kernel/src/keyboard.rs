#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyEvent {
    None,
    Esc,
    Enter,
    Backspace,
    Char(u8),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Tab,
    CtrlChar(u8),
}

static mut SHIFT: bool = false;
static mut CTRL: bool = false;
static mut EXTENDED: bool = false;

#[cfg(not(test))]
fn read_scancode() -> u8 {
    crate::drivers::keyboard::pop_scancode()
}

#[cfg(test)]
fn read_scancode() -> u8 {
    0
}

fn scancode_to_char(scancode: u8, shift: bool) -> Option<u8> {
    let c = match scancode {
        0x39 => b' ',
        0x02 => if shift { b'!' } else { b'1' },
        0x03 => if shift { b'@' } else { b'2' },
        0x04 => if shift { b'#' } else { b'3' },
        0x05 => if shift { b'$' } else { b'4' },
        0x06 => if shift { b'%' } else { b'5' },
        0x07 => if shift { b'^' } else { b'6' },
        0x08 => if shift { b'&' } else { b'7' },
        0x09 => if shift { b'*' } else { b'8' },
        0x0A => if shift { b'(' } else { b'9' },
        0x0B => if shift { b')' } else { b'0' },
        0x0C => if shift { b'_' } else { b'-' },
        0x0D => if shift { b'+' } else { b'=' },
        0x1A => if shift { b'{' } else { b'[' },
        0x1B => if shift { b'}' } else { b']' },
        0x2B => if shift { b'|' } else { b'\\' },
        0x27 => if shift { b':' } else { b';' },
        0x28 => if shift { b'"' } else { b'\'' },
        0x33 => if shift { b'<' } else { b',' },
        0x34 => if shift { b'>' } else { b'.' },
        0x35 => if shift { b'?' } else { b'/' },
        0x29 => if shift { b'~' } else { b'`' },
        0x10 => if shift { b'Q' } else { b'q' },
        0x11 => if shift { b'W' } else { b'w' },
        0x12 => if shift { b'E' } else { b'e' },
        0x13 => if shift { b'R' } else { b'r' },
        0x14 => if shift { b'T' } else { b't' },
        0x15 => if shift { b'Y' } else { b'y' },
        0x16 => if shift { b'U' } else { b'u' },
        0x17 => if shift { b'I' } else { b'i' },
        0x18 => if shift { b'O' } else { b'o' },
        0x19 => if shift { b'P' } else { b'p' },
        0x1E => if shift { b'A' } else { b'a' },
        0x1F => if shift { b'S' } else { b's' },
        0x20 => if shift { b'D' } else { b'd' },
        0x21 => if shift { b'F' } else { b'f' },
        0x22 => if shift { b'G' } else { b'g' },
        0x23 => if shift { b'H' } else { b'h' },
        0x24 => if shift { b'J' } else { b'j' },
        0x25 => if shift { b'K' } else { b'k' },
        0x26 => if shift { b'L' } else { b'l' },
        0x2C => if shift { b'Z' } else { b'z' },
        0x2D => if shift { b'X' } else { b'x' },
        0x2E => if shift { b'C' } else { b'c' },
        0x2F => if shift { b'V' } else { b'v' },
        0x30 => if shift { b'B' } else { b'b' },
        0x31 => if shift { b'N' } else { b'n' },
        0x32 => if shift { b'M' } else { b'm' },
        _ => return None,
    };
    Some(c)
}

pub fn poll_key() -> KeyEvent {
    let scancode = read_scancode();
    if scancode == 0 {
        return KeyEvent::None;
    }

    if scancode == 0xE0 {
        unsafe {
            let ext = core::ptr::addr_of_mut!(EXTENDED);
            *ext = true;
        }
        return KeyEvent::None;
    }

    let is_extended = unsafe {
        let ext = core::ptr::addr_of_mut!(EXTENDED);
        let v = *ext;
        *ext = false;
        v
    };

    let key_released = scancode & 0x80 != 0;
    let code = scancode & 0x7F;

    if is_extended {
        if key_released {
            return KeyEvent::None;
        }
        return match code {
            0x48 => KeyEvent::ArrowUp,
            0x50 => KeyEvent::ArrowDown,
            0x4B => KeyEvent::ArrowLeft,
            0x4D => KeyEvent::ArrowRight,
            _ => KeyEvent::None,
        };
    }

    unsafe {
        let shift = core::ptr::addr_of_mut!(SHIFT);
        if code == 0x2A || code == 0x36 {
            *shift = !key_released;
            return KeyEvent::None;
        }

        let ctrl = core::ptr::addr_of_mut!(CTRL);
        if code == 0x1D {
            *ctrl = !key_released;
            return KeyEvent::None;
        }

        if key_released {
            return KeyEvent::None;
        }

        match code {
            0x01 => KeyEvent::Esc,
            0x1C => KeyEvent::Enter,
            0x0E => KeyEvent::Backspace,
            0x0F => KeyEvent::Tab,
            _ => {
                if let Some(ascii) = scancode_to_char(code, *shift) {
                    if *ctrl && ascii.is_ascii_alphabetic() {
                        KeyEvent::CtrlChar(ascii.to_ascii_lowercase())
                    } else {
                        KeyEvent::Char(ascii)
                    }
                } else {
                    KeyEvent::None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scancode_letters_lowercase() {
        assert_eq!(scancode_to_char(0x10, false), Some(b'q'));
        assert_eq!(scancode_to_char(0x11, false), Some(b'w'));
        assert_eq!(scancode_to_char(0x12, false), Some(b'e'));
        assert_eq!(scancode_to_char(0x1E, false), Some(b'a'));
        assert_eq!(scancode_to_char(0x2C, false), Some(b'z'));
        assert_eq!(scancode_to_char(0x32, false), Some(b'm'));
    }

    #[test]
    fn scancode_letters_uppercase() {
        assert_eq!(scancode_to_char(0x10, true), Some(b'Q'));
        assert_eq!(scancode_to_char(0x1E, true), Some(b'A'));
        assert_eq!(scancode_to_char(0x2C, true), Some(b'Z'));
    }

    #[test]
    fn scancode_numbers() {
        assert_eq!(scancode_to_char(0x02, false), Some(b'1'));
        assert_eq!(scancode_to_char(0x03, false), Some(b'2'));
        assert_eq!(scancode_to_char(0x0B, false), Some(b'0'));
    }

    #[test]
    fn scancode_shift_symbols() {
        assert_eq!(scancode_to_char(0x02, true), Some(b'!'));
        assert_eq!(scancode_to_char(0x03, true), Some(b'@'));
        assert_eq!(scancode_to_char(0x04, true), Some(b'#'));
        assert_eq!(scancode_to_char(0x05, true), Some(b'$'));
        assert_eq!(scancode_to_char(0x0C, true), Some(b'_'));
        assert_eq!(scancode_to_char(0x0D, true), Some(b'+'));
    }

    #[test]
    fn scancode_punctuation() {
        assert_eq!(scancode_to_char(0x39, false), Some(b' '));
        assert_eq!(scancode_to_char(0x33, false), Some(b','));
        assert_eq!(scancode_to_char(0x34, false), Some(b'.'));
        assert_eq!(scancode_to_char(0x35, false), Some(b'/'));
        assert_eq!(scancode_to_char(0x27, false), Some(b';'));
        assert_eq!(scancode_to_char(0x1A, false), Some(b'['));
        assert_eq!(scancode_to_char(0x1B, false), Some(b']'));
    }

    #[test]
    fn scancode_unknown_returns_none() {
        assert_eq!(scancode_to_char(0x00, false), None);
        assert_eq!(scancode_to_char(0xFF, false), None);
        assert_eq!(scancode_to_char(0x40, false), None);
    }

    #[test]
    fn scancode_all_letters_covered() {
        let scancodes: [(u8, u8); 26] = [
            (0x1E, b'a'), (0x30, b'b'), (0x2E, b'c'), (0x20, b'd'),
            (0x12, b'e'), (0x21, b'f'), (0x22, b'g'), (0x23, b'h'),
            (0x17, b'i'), (0x24, b'j'), (0x25, b'k'), (0x26, b'l'),
            (0x32, b'm'), (0x31, b'n'), (0x18, b'o'), (0x19, b'p'),
            (0x10, b'q'), (0x13, b'r'), (0x1F, b's'), (0x14, b't'),
            (0x16, b'u'), (0x2F, b'v'), (0x11, b'w'), (0x2D, b'x'),
            (0x15, b'y'), (0x2C, b'z'),
        ];
        for (sc, expected) in scancodes {
            assert_eq!(scancode_to_char(sc, false), Some(expected),
                "scancode 0x{:02X} should map to '{}'", sc, expected as char);
        }
    }

    #[test]
    fn poll_key_returns_none_in_test() {
        // In test mode, read_scancode returns 0, so poll_key returns None
        assert_eq!(poll_key(), KeyEvent::None);
    }

    #[test]
    fn key_event_equality() {
        assert_eq!(KeyEvent::Char(b'a'), KeyEvent::Char(b'a'));
        assert_ne!(KeyEvent::Char(b'a'), KeyEvent::Char(b'b'));
        assert_ne!(KeyEvent::Enter, KeyEvent::Esc);
    }
}
