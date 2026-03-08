use crate::blockfs;
use crate::keyboard::{self, KeyEvent};
use crate::vga;

const MAX_LINES: usize = 256;
const MAX_LINE_LEN: usize = 80;
const EDITOR_ROWS: usize = 23;
const SCREEN_COLS: usize = 80;
const STATUS_ROW: usize = 23;
const HELP_ROW: usize = 24;
const STATUS_COLOR: u8 = (vga::LIGHT_GRAY << 4) | vga::BLACK;
const HELP_COLOR: u8 = (vga::BLACK << 4) | vga::DARK_GRAY;
const TEXT_COLOR: u8 = vga::WHITE;
const TILDE_COLOR: u8 = vga::DARK_GRAY;
const MSG_COLOR: u8 = vga::YELLOW;

struct Editor {
    lines: [[u8; MAX_LINE_LEN]; MAX_LINES],
    line_lens: [usize; MAX_LINES],
    num_lines: usize,
    cursor_row: usize,
    cursor_col: usize,
    scroll_offset: usize,
    filename: [u8; 32],
    filename_len: usize,
    modified: bool,
    parent_inode: u32,
    message: [u8; 64],
    message_len: usize,
}

impl Editor {
    const fn new() -> Self {
        Self {
            lines: [[0; MAX_LINE_LEN]; MAX_LINES],
            line_lens: [0; MAX_LINES],
            num_lines: 1,
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            filename: [0; 32],
            filename_len: 0,
            modified: false,
            parent_inode: 0,
            message: [0; 64],
            message_len: 0,
        }
    }

    fn set_filename(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(31);
        self.filename[..len].copy_from_slice(&bytes[..len]);
        self.filename_len = len;
    }

    fn filename_str(&self) -> &str {
        if self.filename_len == 0 {
            return "";
        }
        unsafe { core::str::from_utf8_unchecked(&self.filename[..self.filename_len]) }
    }

    fn set_message(&mut self, msg: &str) {
        let bytes = msg.as_bytes();
        let len = bytes.len().min(64);
        self.message[..len].copy_from_slice(&bytes[..len]);
        self.message_len = len;
    }

    fn load_file(&mut self) {
        if self.filename_len == 0 {
            return;
        }
        let name = self.filename_str();
        let inode = match blockfs::bfs_lookup(self.parent_inode, name) {
            Some(ino) => ino,
            None => {
                self.set_message("(New file)");
                return;
            }
        };

        let size = blockfs::bfs_inode_size(inode) as usize;
        if size == 0 {
            self.set_message("(Empty file)");
            return;
        }

        let mut buf = [0u8; MAX_LINES * MAX_LINE_LEN];
        let n = blockfs::bfs_read(inode, &mut buf);

        self.num_lines = 0;
        let mut col = 0usize;
        for i in 0..n {
            if buf[i] == b'\n' {
                if self.num_lines < MAX_LINES {
                    self.line_lens[self.num_lines] = col;
                    self.num_lines += 1;
                }
                col = 0;
            } else {
                if self.num_lines < MAX_LINES && col < MAX_LINE_LEN {
                    self.lines[self.num_lines][col] = buf[i];
                    col += 1;
                }
            }
        }
        // Handle last line (no trailing newline)
        if self.num_lines < MAX_LINES {
            self.line_lens[self.num_lines] = col;
            self.num_lines += 1;
        }

        if self.num_lines == 0 {
            self.num_lines = 1;
        }
    }

    fn save_file(&mut self) -> bool {
        if self.filename_len == 0 {
            self.set_message("No filename");
            return false;
        }

        let name = self.filename_str();
        let inode = match blockfs::bfs_lookup(self.parent_inode, name) {
            Some(ino) => ino,
            None => match blockfs::bfs_create_file(self.parent_inode, name) {
                Some(ino) => ino,
                None => {
                    self.set_message("Error: cannot create file");
                    return false;
                }
            },
        };

        let mut buf = [0u8; MAX_LINES * MAX_LINE_LEN + MAX_LINES];
        let mut pos = 0usize;
        for i in 0..self.num_lines {
            let len = self.line_lens[i];
            if pos + len + 1 >= buf.len() {
                break;
            }
            buf[pos..pos + len].copy_from_slice(&self.lines[i][..len]);
            pos += len;
            if i + 1 < self.num_lines {
                buf[pos] = b'\n';
                pos += 1;
            }
        }

        if blockfs::bfs_write(inode, &buf[..pos]) {
            self.modified = false;
            self.set_message("Saved");
            true
        } else {
            self.set_message("Error: write failed");
            false
        }
    }

    fn draw_screen(&self) {
        for screen_row in 0..EDITOR_ROWS {
            let doc_row = self.scroll_offset + screen_row;
            if doc_row < self.num_lines {
                let len = self.line_lens[doc_row];
                for col in 0..SCREEN_COLS {
                    if col < len {
                        vga::put_char_at(screen_row, col, self.lines[doc_row][col], TEXT_COLOR);
                    } else {
                        vga::put_char_at(screen_row, col, b' ', TEXT_COLOR);
                    }
                }
            } else {
                vga::put_char_at(screen_row, 0, b'~', TILDE_COLOR);
                for col in 1..SCREEN_COLS {
                    vga::put_char_at(screen_row, col, b' ', TEXT_COLOR);
                }
            }
        }

        self.draw_status_bar();
        self.draw_help_line();

        let screen_y = self.cursor_row - self.scroll_offset;
        vga::set_cursor(screen_y, self.cursor_col);
    }

    fn draw_status_bar(&self) {
        // Fill status bar
        for col in 0..SCREEN_COLS {
            vga::put_char_at(STATUS_ROW, col, b' ', STATUS_COLOR);
        }

        // Left side: filename + modified indicator
        let mut col = 1usize;
        if self.filename_len > 0 {
            for i in 0..self.filename_len {
                if col < SCREEN_COLS {
                    vga::put_char_at(STATUS_ROW, col, self.filename[i], STATUS_COLOR);
                    col += 1;
                }
            }
        } else {
            let label = b"[new file]";
            for &ch in label.iter() {
                if col < SCREEN_COLS {
                    vga::put_char_at(STATUS_ROW, col, ch, STATUS_COLOR);
                    col += 1;
                }
            }
        }

        if self.modified {
            if col < SCREEN_COLS {
                vga::put_char_at(STATUS_ROW, col, b'*', STATUS_COLOR);
            }
        }

        // Right side: "Ln X, Col Y"
        let mut right_buf = [0u8; 24];
        let mut rpos = 0usize;
        let ln_label = b"Ln ";
        for &b in ln_label.iter() {
            right_buf[rpos] = b;
            rpos += 1;
        }
        rpos += write_usize_to_buf(&mut right_buf[rpos..], self.cursor_row + 1);
        right_buf[rpos] = b',';
        rpos += 1;
        right_buf[rpos] = b' ';
        rpos += 1;
        let col_label = b"Col ";
        for &b in col_label.iter() {
            right_buf[rpos] = b;
            rpos += 1;
        }
        rpos += write_usize_to_buf(&mut right_buf[rpos..], self.cursor_col + 1);

        let start_col = if SCREEN_COLS > rpos + 1 { SCREEN_COLS - rpos - 1 } else { 0 };
        for i in 0..rpos {
            if start_col + i < SCREEN_COLS {
                vga::put_char_at(STATUS_ROW, start_col + i, right_buf[i], STATUS_COLOR);
            }
        }
    }

    fn draw_help_line(&self) {
        for col in 0..SCREEN_COLS {
            vga::put_char_at(HELP_ROW, col, b' ', HELP_COLOR);
        }

        if self.message_len > 0 {
            for i in 0..self.message_len {
                if i < SCREEN_COLS {
                    vga::put_char_at(HELP_ROW, i, self.message[i], MSG_COLOR);
                }
            }
        } else {
            let help = b"^X Exit  ^S Save  Esc Quit";
            let start = 1usize;
            for (i, &ch) in help.iter().enumerate() {
                if start + i < SCREEN_COLS {
                    vga::put_char_at(HELP_ROW, start + i, ch, HELP_COLOR);
                }
            }
        }
    }

    fn ensure_cursor_visible(&mut self) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_offset + EDITOR_ROWS {
            self.scroll_offset = self.cursor_row - EDITOR_ROWS + 1;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let len = self.line_lens[self.cursor_row];
            if self.cursor_col > len {
                self.cursor_col = len;
            }
            self.ensure_cursor_visible();
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.num_lines {
            self.cursor_row += 1;
            let len = self.line_lens[self.cursor_row];
            if self.cursor_col > len {
                self.cursor_col = len;
            }
            self.ensure_cursor_visible();
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_lens[self.cursor_row];
            self.ensure_cursor_visible();
        }
    }

    fn move_right(&mut self) {
        let len = self.line_lens[self.cursor_row];
        if self.cursor_col < len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.num_lines {
            self.cursor_row += 1;
            self.cursor_col = 0;
            self.ensure_cursor_visible();
        }
    }

    fn insert_char(&mut self, c: u8) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let len = self.line_lens[row];
        if len >= MAX_LINE_LEN - 1 {
            return;
        }

        // Shift characters right
        let mut i = len;
        while i > col {
            self.lines[row][i] = self.lines[row][i - 1];
            i -= 1;
        }
        self.lines[row][col] = c;
        self.line_lens[row] = len + 1;
        self.cursor_col += 1;
        self.modified = true;
        self.message_len = 0;
    }

    fn insert_newline(&mut self) {
        if self.num_lines >= MAX_LINES {
            return;
        }

        let row = self.cursor_row;
        let col = self.cursor_col;
        let old_len = self.line_lens[row];

        // Shift all lines below down by one
        let mut i = self.num_lines;
        while i > row + 1 {
            self.lines[i] = self.lines[i - 1];
            self.line_lens[i] = self.line_lens[i - 1];
            i -= 1;
        }
        self.num_lines += 1;

        // Split current line at cursor
        let new_line_idx = row + 1;
        let tail_len = old_len - col;
        self.lines[new_line_idx] = [0; MAX_LINE_LEN];
        // Copy tail to temp buffer to avoid borrow conflict
        let mut tmp = [0u8; MAX_LINE_LEN];
        tmp[..tail_len].copy_from_slice(&self.lines[row][col..col + tail_len]);
        self.lines[new_line_idx][..tail_len].copy_from_slice(&tmp[..tail_len]);
        self.line_lens[new_line_idx] = tail_len;

        // Truncate current line
        for i in col..old_len {
            self.lines[row][i] = 0;
        }
        self.line_lens[row] = col;

        self.cursor_row += 1;
        self.cursor_col = 0;
        self.modified = true;
        self.message_len = 0;
        self.ensure_cursor_visible();
    }

    fn backspace(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;

        if col > 0 {
            // Delete char before cursor
            let len = self.line_lens[row];
            let mut i = col - 1;
            while i + 1 < len {
                self.lines[row][i] = self.lines[row][i + 1];
                i += 1;
            }
            self.lines[row][len - 1] = 0;
            self.line_lens[row] = len - 1;
            self.cursor_col -= 1;
            self.modified = true;
            self.message_len = 0;
        } else if row > 0 {
            // Merge with previous line
            let prev = row - 1;
            let prev_len = self.line_lens[prev];
            let cur_len = self.line_lens[row];

            if prev_len + cur_len <= MAX_LINE_LEN {
                // Append current line to previous (temp buffer to avoid borrow conflict)
                let mut tmp = [0u8; MAX_LINE_LEN];
                tmp[..cur_len].copy_from_slice(&self.lines[row][..cur_len]);
                self.lines[prev][prev_len..prev_len + cur_len]
                    .copy_from_slice(&tmp[..cur_len]);
                self.line_lens[prev] = prev_len + cur_len;

                // Shift lines up
                let mut i = row;
                while i + 1 < self.num_lines {
                    self.lines[i] = self.lines[i + 1];
                    self.line_lens[i] = self.line_lens[i + 1];
                    i += 1;
                }
                self.num_lines -= 1;
                self.lines[self.num_lines] = [0; MAX_LINE_LEN];
                self.line_lens[self.num_lines] = 0;

                self.cursor_row = prev;
                self.cursor_col = prev_len;
                self.modified = true;
                self.message_len = 0;
                self.ensure_cursor_visible();
            }
        }
    }

    fn insert_tab(&mut self) {
        for _ in 0..4 {
            self.insert_char(b' ');
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key {
            KeyEvent::None => {}
            KeyEvent::Esc => return true,
            KeyEvent::CtrlChar(b'x') => {
                if self.modified {
                    self.set_message("Unsaved changes! ^S to save, Esc to discard");
                } else {
                    return true;
                }
            }
            KeyEvent::CtrlChar(b's') => {
                self.save_file();
            }
            KeyEvent::ArrowUp => self.move_up(),
            KeyEvent::ArrowDown => self.move_down(),
            KeyEvent::ArrowLeft => self.move_left(),
            KeyEvent::ArrowRight => self.move_right(),
            KeyEvent::Enter => self.insert_newline(),
            KeyEvent::Backspace => self.backspace(),
            KeyEvent::Tab => self.insert_tab(),
            KeyEvent::Char(c) => self.insert_char(c),
            KeyEvent::CtrlChar(_) => {}
        }
        false
    }
}

fn write_usize_to_buf(buf: &mut [u8], val: usize) -> usize {
    if val == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 10];
    let mut n = val;
    let mut pos = 10;
    while n > 0 && pos > 0 {
        pos -= 1;
        tmp[pos] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    let len = 10 - pos;
    buf[..len].copy_from_slice(&tmp[pos..]);
    len
}

pub fn run(filename: Option<&str>, parent_inode: u32) {
    let mut editor = Editor::new();
    editor.parent_inode = parent_inode;

    if let Some(name) = filename {
        editor.set_filename(name);
        editor.load_file();
    } else {
        editor.set_message("(New file)");
    }

    vga::clear_screen();
    editor.draw_screen();

    loop {
        let key = keyboard::poll_key();
        if key == KeyEvent::None {
            core::hint::spin_loop();
            continue;
        }
        if editor.handle_key(key) {
            break;
        }
        editor.draw_screen();
    }

    vga::clear_screen();
}
