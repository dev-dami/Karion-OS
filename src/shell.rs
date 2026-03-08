use crate::fs::FileSystem;
use crate::vga;

const MAX_INPUT: usize = 256;
const MAX_TOKENS: usize = 16;
const HISTORY_SIZE: usize = 16;

pub struct Shell {
    input: [u8; MAX_INPUT],
    len: usize,
    history: [[u8; MAX_INPUT]; HISTORY_SIZE],
    history_lens: [usize; HISTORY_SIZE],
    history_count: usize,
    history_pos: usize,
    browsing_history: bool,
}

impl Shell {
    pub const fn new() -> Self {
        Self {
            input: [0; MAX_INPUT],
            len: 0,
            history: [[0; MAX_INPUT]; HISTORY_SIZE],
            history_lens: [0; HISTORY_SIZE],
            history_count: 0,
            history_pos: 0,
            browsing_history: false,
        }
    }

    pub fn init(&mut self) {
        self.len = 0;
        self.input = [0; MAX_INPUT];
        self.browsing_history = false;
        vga::newline();
        vga::write_line("Karion-OS Shell v3.1", vga::LIGHT_CYAN);
        vga::write_line("Type 'help' for available commands", vga::DARK_GRAY);
        vga::newline();
        self.print_prompt(&mut [0; 256], None);
    }

    pub fn push(&mut self, c: u8) {
        if self.len >= MAX_INPUT - 1 {
            return;
        }
        self.browsing_history = false;

        self.input[self.len] = c;
        self.len += 1;
        vga::put_char(c, vga::WHITE);
    }

    pub fn backspace(&mut self) {
        if self.len == 0 {
            return;
        }
        self.len -= 1;
        self.input[self.len] = 0;
        vga::backspace();
    }

    pub fn history_up(&mut self) {
        if self.history_count == 0 {
            return;
        }
        if !self.browsing_history {
            self.browsing_history = true;
            self.history_pos = self.history_count;
        }
        if self.history_pos == 0 {
            return;
        }
        self.history_pos -= 1;
        self.load_history_entry();
    }

    pub fn history_down(&mut self) {
        if !self.browsing_history {
            return;
        }
        if self.history_pos + 1 >= self.history_count {
            self.browsing_history = false;
            self.clear_input_line();
            self.len = 0;
            self.input = [0; MAX_INPUT];
            return;
        }
        self.history_pos += 1;
        self.load_history_entry();
    }

    fn load_history_entry(&mut self) {
        let idx = self.history_pos % HISTORY_SIZE;
        let entry_len = self.history_lens[idx];

        self.clear_input_line();
        self.input = self.history[idx];
        self.len = entry_len;

        for i in 0..self.len {
            vga::put_char(self.input[i], vga::WHITE);
        }
    }

    fn clear_input_line(&mut self) {
        for _ in 0..self.len {
            vga::backspace();
        }
    }

    fn save_to_history(&mut self) {
        if self.len == 0 {
            return;
        }
        if self.history_count > 0 {
            let last = (self.history_count - 1) % HISTORY_SIZE;
            if self.history_lens[last] == self.len
                && self.history[last][..self.len] == self.input[..self.len]
            {
                return;
            }
        }
        let idx = self.history_count % HISTORY_SIZE;
        self.history[idx] = self.input;
        self.history_lens[idx] = self.len;
        self.history_count += 1;
    }

    pub fn submit(&mut self, fs: &mut FileSystem) {
        vga::newline();
        self.browsing_history = false;

        let mut local = [0u8; MAX_INPUT];
        local[..self.len].copy_from_slice(&self.input[..self.len]);
        let line = match core::str::from_utf8(&local[..self.len]) {
            Ok(v) => v,
            Err(_) => {
                self.reset();
                self.print_prompt(&mut [0; 256], Some(fs));
                return;
            }
        };

        self.save_to_history();
        self.execute(line, fs);
        self.reset();
        self.print_prompt(&mut [0; 256], Some(fs));
    }

    fn reset(&mut self) {
        self.input = [0; MAX_INPUT];
        self.len = 0;
    }

    fn execute(&mut self, line: &str, fs: &mut FileSystem) {
        let mut tokens = [""; MAX_TOKENS];
        let mut argc = 0usize;

        for token in line.split_whitespace() {
            if argc >= MAX_TOKENS {
                vga::write_line("Too many arguments", vga::RED);
                return;
            }
            tokens[argc] = token;
            argc += 1;
        }

        if argc == 0 {
            return;
        }

        let cmd = tokens[0];
        if streq(cmd, "help") {
            self.help(&tokens, argc);
        } else if streq(cmd, "clear") {
            vga::clear_screen();
        } else if streq(cmd, "echo") {
            self.echo(&tokens, argc, fs);
        } else if streq(cmd, "mkdir") {
            self.mkdir(&tokens, argc, fs);
        } else if streq(cmd, "touch") {
            self.touch(&tokens, argc, fs);
        } else if streq(cmd, "rm") || streq(cmd, "del") {
            self.del(&tokens, argc, fs);
        } else if streq(cmd, "ls") {
            self.ls(&tokens, argc, fs);
        } else if streq(cmd, "pwd") {
            self.pwd(fs);
        } else if streq(cmd, "cd") {
            self.cd(&tokens, argc, fs);
        } else if streq(cmd, "cat") {
            self.cat(&tokens, argc, fs);
        } else if streq(cmd, "stat") {
            self.stat(&tokens, argc, fs);
        } else if streq(cmd, "mv") {
            self.mv(&tokens, argc, fs);
        } else if streq(cmd, "whoami") {
            vga::write_line("root", vga::GREEN);
        } else if streq(cmd, "hostname") {
            vga::write_line("karion", vga::GREEN);
        } else if streq(cmd, "uname") {
            self.uname(&tokens, argc);
        } else if streq(cmd, "history") {
            self.show_history();
        } else {
            #[cfg(not(test))]
            {
                if streq(cmd, "uptime") {
                    self.uptime();
                    return;
                } else if streq(cmd, "meminfo") {
                    self.meminfo();
                    return;
                } else if streq(cmd, "nano") {
                    self.nano(&tokens, argc, fs);
                    return;
                } else if streq(cmd, "snake") {
                    crate::games::snake::run();
                    return;
                } else if streq(cmd, "tictactoe") {
                    crate::games::tictactoe::run();
                    return;
                } else if streq(cmd, "guess") {
                    crate::games::guess::run();
                    return;
                } else if streq(cmd, "basic") {
                    self.basic(&tokens, argc, fs);
                    return;
                }
            }
            vga::write_str(cmd, vga::RED);
            vga::write_line(": command not found", vga::RED);
        }
    }

    fn help(&self, tokens: &[&str; MAX_TOKENS], argc: usize) {
        if argc >= 2 {
            match tokens[1] {
                "echo" => {
                    vga::write_line("echo [text] [> file]", vga::WHITE);
                    vga::write_line("  Print text or redirect to file", vga::DARK_GRAY);
                }
                "mkdir" => {
                    vga::write_line("mkdir <dir>", vga::WHITE);
                    vga::write_line("  Create a directory", vga::DARK_GRAY);
                }
                "rm" | "del" => {
                    vga::write_line("rm <path>", vga::WHITE);
                    vga::write_line("  Remove a file or empty directory", vga::DARK_GRAY);
                }
                "ls" => {
                    vga::write_line("ls [path]", vga::WHITE);
                    vga::write_line("  List directory contents", vga::DARK_GRAY);
                }
                "cat" => {
                    vga::write_line("cat <file>", vga::WHITE);
                    vga::write_line("  Display file contents", vga::DARK_GRAY);
                }
                "stat" => {
                    vga::write_line("stat <path>", vga::WHITE);
                    vga::write_line("  Show file/directory info", vga::DARK_GRAY);
                }
                "mv" => {
                    vga::write_line("mv <src> <dst>", vga::WHITE);
                    vga::write_line("  Move/rename a file or directory", vga::DARK_GRAY);
                }
                "uname" => {
                    vga::write_line("uname [-a]", vga::WHITE);
                    vga::write_line("  Show system information", vga::DARK_GRAY);
                }
                _ => {
                    vga::write_str("No help for: ", vga::YELLOW);
                    vga::write_line(tokens[1], vga::YELLOW);
                }
            }
            return;
        }

        vga::write_line("Karion-OS Built-in Commands", vga::LIGHT_CYAN);
        vga::newline();
        vga::write_str("  help [cmd]   ", vga::WHITE);
        vga::write_line("Show help", vga::DARK_GRAY);
        vga::write_str("  clear        ", vga::WHITE);
        vga::write_line("Clear screen", vga::DARK_GRAY);
        vga::write_str("  echo [text]  ", vga::WHITE);
        vga::write_line("Print text (supports > redirect)", vga::DARK_GRAY);
        vga::write_str("  ls [path]    ", vga::WHITE);
        vga::write_line("List directory", vga::DARK_GRAY);
        vga::write_str("  cd <dir>     ", vga::WHITE);
        vga::write_line("Change directory", vga::DARK_GRAY);
        vga::write_str("  pwd          ", vga::WHITE);
        vga::write_line("Print working directory", vga::DARK_GRAY);
        vga::write_str("  cat <file>   ", vga::WHITE);
        vga::write_line("Read file", vga::DARK_GRAY);
        vga::write_str("  touch <file> ", vga::WHITE);
        vga::write_line("Create empty file", vga::DARK_GRAY);
        vga::write_str("  mkdir <dir>  ", vga::WHITE);
        vga::write_line("Create directory", vga::DARK_GRAY);
        vga::write_str("  rm <path>    ", vga::WHITE);
        vga::write_line("Remove file/directory", vga::DARK_GRAY);
        vga::write_str("  mv <s> <d>   ", vga::WHITE);
        vga::write_line("Move/rename", vga::DARK_GRAY);
        vga::write_str("  stat <path>  ", vga::WHITE);
        vga::write_line("File info", vga::DARK_GRAY);
        vga::write_str("  whoami       ", vga::WHITE);
        vga::write_line("Current user", vga::DARK_GRAY);
        vga::write_str("  hostname     ", vga::WHITE);
        vga::write_line("System hostname", vga::DARK_GRAY);
        vga::write_str("  uname [-a]   ", vga::WHITE);
        vga::write_line("System info", vga::DARK_GRAY);
        vga::write_str("  uptime       ", vga::WHITE);
        vga::write_line("System uptime", vga::DARK_GRAY);
        vga::write_str("  meminfo      ", vga::WHITE);
        vga::write_line("Memory usage", vga::DARK_GRAY);
        vga::write_str("  history      ", vga::WHITE);
        vga::write_line("Command history", vga::DARK_GRAY);
        vga::newline();
        vga::write_line("Applications", vga::LIGHT_CYAN);
        vga::write_str("  nano [file]  ", vga::WHITE);
        vga::write_line("Text editor", vga::DARK_GRAY);
        vga::write_str("  basic [file] ", vga::WHITE);
        vga::write_line("BASIC interpreter/REPL", vga::DARK_GRAY);
        vga::write_str("  snake        ", vga::WHITE);
        vga::write_line("Snake game", vga::DARK_GRAY);
        vga::write_str("  tictactoe    ", vga::WHITE);
        vga::write_line("Tic-tac-toe game", vga::DARK_GRAY);
        vga::write_str("  guess        ", vga::WHITE);
        vga::write_line("Number guessing game", vga::DARK_GRAY);
    }

    fn echo(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &mut FileSystem) {
        if argc == 1 {
            vga::newline();
            return;
        }

        let mut redirect_pos = None;
        for (i, token) in tokens.iter().enumerate().take(argc).skip(1) {
            if *token == ">" {
                redirect_pos = Some(i);
                break;
            }
        }

        if let Some(pos) = redirect_pos {
            if pos + 1 >= argc {
                vga::write_line("Usage: echo <text> > <filename>", vga::RED);
                return;
            }

            let mut content = [0u8; MAX_INPUT];
            let mut len = 0usize;
            for (i, token) in tokens.iter().enumerate().take(pos).skip(1) {
                let bytes = token.as_bytes();
                if len + bytes.len() + 1 >= content.len() {
                    vga::write_line("Echo text too long", vga::RED);
                    return;
                }
                if i > 1 {
                    content[len] = b' ';
                    len += 1;
                }
                content[len..len + bytes.len()].copy_from_slice(bytes);
                len += bytes.len();
            }

            let text = core::str::from_utf8(&content[..len]).unwrap_or("");
            let file = tokens[pos + 1];
            if !fs.write_file(file, text) && !fs.create_file(file, text) {
                vga::write_line("Error writing to file", vga::RED);
            }
            return;
        }

        for (i, token) in tokens.iter().enumerate().take(argc).skip(1) {
            if i > 1 {
                vga::put_char(b' ', vga::WHITE);
            }
            vga::write_str(token, vga::WHITE);
        }
        vga::newline();
    }

    fn mkdir(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &mut FileSystem) {
        if argc < 2 {
            vga::write_line("Usage: mkdir <directory>", vga::RED);
            return;
        }
        if !fs.create_dir(tokens[1]) {
            vga::write_line("mkdir: cannot create directory", vga::RED);
        }
    }

    fn touch(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &mut FileSystem) {
        if argc < 2 {
            vga::write_line("Usage: touch <file>", vga::RED);
            return;
        }
        if !fs.create_file(tokens[1], "") {
            vga::write_line("touch: cannot create file", vga::RED);
        }
    }

    fn del(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &mut FileSystem) {
        if argc < 2 {
            vga::write_line("Usage: rm <path>", vga::RED);
            return;
        }
        if !fs.delete(tokens[1]) {
            vga::write_str("rm: cannot remove '", vga::RED);
            vga::write_str(tokens[1], vga::RED);
            vga::write_line("'", vga::RED);
        }
    }

    fn ls(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &FileSystem) {
        let path = if argc >= 2 { tokens[1] } else { "." };
        if !fs.list_dir(path, |is_dir, name| {
            if is_dir {
                vga::write_str(name, vga::LIGHT_CYAN);
                vga::write_line("/", vga::LIGHT_CYAN);
            } else {
                vga::write_line(name, vga::WHITE);
            }
        }) {
            vga::write_str("ls: cannot access '", vga::RED);
            vga::write_str(path, vga::RED);
            vga::write_line("'", vga::RED);
        }
    }

    fn pwd(&self, fs: &FileSystem) {
        let mut path = [0u8; 256];
        let cwd = fs.cwd_path(&mut path);
        vga::write_line(cwd, vga::WHITE);
    }

    fn cd(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &mut FileSystem) {
        if argc < 2 {
            fs.change_dir("/");
            return;
        }
        if !fs.change_dir(tokens[1]) {
            vga::write_str("cd: no such directory: ", vga::RED);
            vga::write_line(tokens[1], vga::RED);
        }
    }

    fn cat(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &FileSystem) {
        if argc < 2 {
            vga::write_line("Usage: cat <file>", vga::RED);
            return;
        }

        let mut out = [0u8; 512];
        match fs.read_file(tokens[1], &mut out) {
            Some(content) => {
                if content.is_empty() {
                    return;
                }
                vga::write_line(content, vga::WHITE);
            }
            None => {
                vga::write_str("cat: ", vga::RED);
                vga::write_str(tokens[1], vga::RED);
                vga::write_line(": No such file", vga::RED);
            }
        }
    }

    fn stat(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &FileSystem) {
        if argc < 2 {
            vga::write_line("Usage: stat <path>", vga::RED);
            return;
        }

        let path = tokens[1];
        let mut out = [0u8; 512];
        if let Some(content) = fs.read_file(path, &mut out) {
            vga::write_str("  File: ", vga::WHITE);
            vga::write_line(path, vga::GREEN);
            vga::write_str("  Type: regular file", vga::WHITE);
            vga::newline();
            vga::write_str("  Size: ", vga::WHITE);
            let mut buf = [0u8; 20];
            vga::write_str(usize_to_str(content.len(), &mut buf), vga::GREEN);
            vga::write_line(" bytes", vga::WHITE);
            return;
        }
        let mut found = false;
        let mut child_count = 0usize;
        if fs.list_dir(path, |_, _| {
            child_count += 1;
        }) {
            found = true;
        }
        if found {
            vga::write_str("  File: ", vga::WHITE);
            vga::write_line(path, vga::LIGHT_CYAN);
            vga::write_line("  Type: directory", vga::WHITE);
            vga::write_str("  Entries: ", vga::WHITE);
            let mut buf = [0u8; 20];
            vga::write_line(usize_to_str(child_count, &mut buf), vga::GREEN);
            return;
        }
        vga::write_str("stat: cannot stat '", vga::RED);
        vga::write_str(path, vga::RED);
        vga::write_line("': No such file or directory", vga::RED);
    }

    fn mv(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &mut FileSystem) {
        if argc < 3 {
            vga::write_line("Usage: mv <source> <dest>", vga::RED);
            return;
        }
        let src = tokens[1];
        let dst = tokens[2];

        let mut out = [0u8; 512];
        if let Some(content) = fs.read_file(src, &mut out) {
            let content_copy = content.len();
            let mut content_buf = [0u8; 512];
            content_buf[..content_copy].copy_from_slice(&out[..content_copy]);
            let text = core::str::from_utf8(&content_buf[..content_copy]).unwrap_or("");
            if fs.create_file(dst, text) {
                fs.delete(src);
            } else {
                vga::write_line("mv: cannot move file", vga::RED);
            }
        } else if fs.list_dir(src, |_, _| {}) {
            vga::write_line("mv: cannot move directories yet", vga::YELLOW);
        } else {
            vga::write_str("mv: cannot stat '", vga::RED);
            vga::write_str(src, vga::RED);
            vga::write_line("'", vga::RED);
        }
    }

    fn uname(&self, tokens: &[&str; MAX_TOKENS], argc: usize) {
        if argc >= 2 && tokens[1] == "-a" {
            vga::write_line("Karion-OS karion 3.1.0 i686 x86", vga::WHITE);
        } else {
            vga::write_line("Karion-OS", vga::WHITE);
        }
    }

    fn show_history(&self) {
        let start = if self.history_count > HISTORY_SIZE {
            self.history_count - HISTORY_SIZE
        } else {
            0
        };
        let mut buf = [0u8; 20];
        for i in start..self.history_count {
            let idx = i % HISTORY_SIZE;
            let entry_len = self.history_lens[idx];
            vga::write_str("  ", vga::DARK_GRAY);
            vga::write_str(usize_to_str(i + 1, &mut buf), vga::DARK_GRAY);
            vga::write_str("  ", vga::DARK_GRAY);
            if let Ok(s) = core::str::from_utf8(&self.history[idx][..entry_len]) {
                vga::write_line(s, vga::WHITE);
            }
        }
        if self.history_count == 0 {
            vga::write_line("No history", vga::DARK_GRAY);
        }
    }

    #[cfg(not(test))]
    fn uptime(&self) {
        let ticks = crate::drivers::timer::get_ticks();
        let total_secs = ticks / 100; // PIT fires at 100 Hz
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        let mut buf = [0u8; 20];

        vga::write_str("up ", vga::WHITE);
        if hours > 0 {
            vga::write_str(u64_to_str(hours, &mut buf), vga::GREEN);
            vga::write_str("h ", vga::WHITE);
        }
        if mins > 0 || hours > 0 {
            vga::write_str(u64_to_str(mins, &mut buf), vga::GREEN);
            vga::write_str("m ", vga::WHITE);
        }
        vga::write_str(u64_to_str(secs, &mut buf), vga::GREEN);
        vga::write_str("s (", vga::WHITE);
        vga::write_str(u64_to_str(ticks, &mut buf), vga::DARK_GRAY);
        vga::write_line(" ticks)", vga::DARK_GRAY);
    }

    #[cfg(not(test))]
    fn nano(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &FileSystem) {
        let filename = if argc >= 2 { Some(tokens[1]) } else { None };
        crate::editor::run(filename, fs.cwd_inode());
        vga::clear_screen();
        self.print_prompt(&mut [0; 256], Some(fs));
    }

    #[cfg(not(test))]
    fn basic(&self, tokens: &[&str; MAX_TOKENS], argc: usize, fs: &FileSystem) {
        if argc >= 2 {
            // Run a script file
            let mut out = [0u8; 4096];
            match fs.read_file(tokens[1], &mut out) {
                Some(content) => crate::basic::run_script(content),
                None => {
                    vga::write_str("basic: cannot read '", vga::RED);
                    vga::write_str(tokens[1], vga::RED);
                    vga::write_line("'", vga::RED);
                }
            }
        } else {
            crate::basic::repl();
        }
    }

    #[cfg(not(test))]
    fn meminfo(&self) {
        let free_frames = crate::pmm::free_frame_count();
        let free_kb = free_frames * 4; // 4 KB per frame
        let total_kb = 32 * 1024; // assumes 32 MB physical RAM
        let used_kb = total_kb - free_kb;
        let mut buf = [0u8; 20];

        vga::write_str("Total:  ", vga::WHITE);
        vga::write_str(usize_to_str(total_kb, &mut buf), vga::GREEN);
        vga::write_line(" KB", vga::WHITE);
        vga::write_str("Used:   ", vga::WHITE);
        vga::write_str(usize_to_str(used_kb, &mut buf), vga::YELLOW);
        vga::write_line(" KB", vga::WHITE);
        vga::write_str("Free:   ", vga::WHITE);
        vga::write_str(usize_to_str(free_kb, &mut buf), vga::GREEN);
        vga::write_line(" KB", vga::WHITE);
    }

    fn print_prompt(&self, path_buf: &mut [u8; 256], fs: Option<&FileSystem>) {
        vga::write_str("root@karion", vga::GREEN);
        vga::write_str(":", vga::DARK_GRAY);
        if let Some(filesystem) = fs {
            let path = filesystem.cwd_path(path_buf);
            vga::write_str(path, vga::LIGHT_CYAN);
        } else {
            vga::write_str("/", vga::LIGHT_CYAN);
        }
        vga::write_str("$ ", vga::WHITE);
    }
}

fn u64_to_str(mut val: u64, buf: &mut [u8; 20]) -> &str {
    if val == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut pos = 20;
    while val > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    core::str::from_utf8(&buf[pos..]).unwrap_or("?")
}

/// Manual byte-by-byte string comparison (avoids compiler-generated str match)
fn streq(a: &str, b: &str) -> bool {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    if ab.len() != bb.len() {
        return false;
    }
    let mut i = 0;
    while i < ab.len() {
        if ab[i] != bb[i] {
            return false;
        }
        i += 1;
    }
    true
}

fn usize_to_str(mut val: usize, buf: &mut [u8; 20]) -> &str {
    if val == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut pos = 20;
    while val > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    core::str::from_utf8(&buf[pos..]).unwrap_or("?")
}
