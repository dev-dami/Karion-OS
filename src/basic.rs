#[cfg(not(test))]
use crate::vga;
#[cfg(not(test))]
use crate::keyboard::{self, KeyEvent};

const MAX_SOURCE: usize = 2048;
const MAX_TOKENS: usize = 256;
const MAX_VARS: usize = 26;
const MAX_DEPTH: usize = 16;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Token {
    Number(i32),
    String { start: usize, len: usize },
    Ident(u8), // b'a'..=b'z'
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Assign,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Comma,
    KeywordInt,
    KeywordIf,
    KeywordElse,
    KeywordWhile,
    KeywordFor,
    KeywordPrint,
    Eof,
}

struct Tokenizer<'a> {
    src: &'a [u8],
    pos: usize,
    tokens: [Token; MAX_TOKENS],
    count: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &'a [u8]) -> Self {
        Self {
            src,
            pos: 0,
            tokens: [Token::Eof; MAX_TOKENS],
            count: 0,
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                self.pos += 1;
            } else if ch == b'/' && self.pos + 1 < self.src.len() && self.src[self.pos + 1] == b'/' {
                while self.pos < self.src.len() && self.src[self.pos] != b'\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn push(&mut self, tok: Token) -> bool {
        if self.count >= MAX_TOKENS {
            return false;
        }
        self.tokens[self.count] = tok;
        self.count += 1;
        true
    }

    fn tokenize(&mut self) -> Result<(), usize> {
        while self.pos < self.src.len() {
            self.skip_whitespace_and_comments();
            if self.pos >= self.src.len() {
                break;
            }

            let start = self.pos;
            let ch = self.src[self.pos];

            if ch.is_ascii_digit() {
                let mut val: i32 = 0;
                while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                    val = val.wrapping_mul(10).wrapping_add((self.src[self.pos] - b'0') as i32);
                    self.pos += 1;
                }
                if !self.push(Token::Number(val)) { return Err(start); }
            } else if ch == b'"' {
                self.pos += 1;
                let str_start = self.pos;
                while self.pos < self.src.len() && self.src[self.pos] != b'"' {
                    self.pos += 1;
                }
                if self.pos >= self.src.len() { return Err(str_start); }
                let str_len = self.pos - str_start;
                self.pos += 1; // skip closing quote
                if !self.push(Token::String { start: str_start, len: str_len }) { return Err(start); }
            } else if ch.is_ascii_alphabetic() || ch == b'_' {
                while self.pos < self.src.len()
                    && (self.src[self.pos].is_ascii_alphanumeric() || self.src[self.pos] == b'_')
                {
                    self.pos += 1;
                }
                let word = &self.src[start..self.pos];
                let tok = if word == b"int" {
                    Token::KeywordInt
                } else if word == b"if" {
                    Token::KeywordIf
                } else if word == b"else" {
                    Token::KeywordElse
                } else if word == b"while" {
                    Token::KeywordWhile
                } else if word == b"for" {
                    Token::KeywordFor
                } else if word == b"print" {
                    Token::KeywordPrint
                } else if word.len() == 1 && word[0] >= b'a' && word[0] <= b'z' {
                    Token::Ident(word[0])
                } else {
                    return Err(start);
                };
                if !self.push(tok) { return Err(start); }
            } else {
                self.pos += 1;
                let tok = match ch {
                    b'+' => Token::Plus,
                    b'-' => Token::Minus,
                    b'*' => Token::Star,
                    b'/' => Token::Slash,
                    b'%' => Token::Percent,
                    b'(' => Token::LParen,
                    b')' => Token::RParen,
                    b'{' => Token::LBrace,
                    b'}' => Token::RBrace,
                    b';' => Token::Semicolon,
                    b',' => Token::Comma,
                    b'=' => {
                        if self.pos < self.src.len() && self.src[self.pos] == b'=' {
                            self.pos += 1;
                            Token::Eq
                        } else {
                            Token::Assign
                        }
                    }
                    b'!' => {
                        if self.pos < self.src.len() && self.src[self.pos] == b'=' {
                            self.pos += 1;
                            Token::Neq
                        } else {
                            return Err(start);
                        }
                    }
                    b'<' => {
                        if self.pos < self.src.len() && self.src[self.pos] == b'=' {
                            self.pos += 1;
                            Token::Lte
                        } else {
                            Token::Lt
                        }
                    }
                    b'>' => {
                        if self.pos < self.src.len() && self.src[self.pos] == b'=' {
                            self.pos += 1;
                            Token::Gte
                        } else {
                            Token::Gt
                        }
                    }
                    _ => return Err(start),
                };
                if !self.push(tok) { return Err(start); }
            }
        }

        self.push(Token::Eof);
        Ok(())
    }
}

struct Interpreter<'a> {
    src: &'a [u8],
    tokens: [Token; MAX_TOKENS],
    count: usize,
    pos: usize,
    vars: [i32; MAX_VARS],
    defined: [bool; MAX_VARS],
    depth: usize,
    error: bool,
}

impl<'a> Interpreter<'a> {
    fn new(src: &'a [u8], tokens: [Token; MAX_TOKENS], count: usize) -> Self {
        Self {
            src,
            tokens,
            count,
            pos: 0,
            vars: [0; MAX_VARS],
            defined: [false; MAX_VARS],
            depth: 0,
            error: false,
        }
    }

    fn peek(&self) -> Token {
        if self.pos < self.count { self.tokens[self.pos] } else { Token::Eof }
    }

    fn advance(&mut self) -> Token {
        let t = self.peek();
        if self.pos < self.count { self.pos += 1; }
        t
    }

    fn expect(&mut self, expected: Token) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            self.err("unexpected token");
            false
        }
    }

    fn err(&mut self, _msg: &str) {
        if self.error { return; }
        self.error = true;
        #[cfg(not(test))]
        {
            vga::write_str("Error: ", vga::RED);
            vga::write_str(_msg, vga::RED);
            vga::write_str(" at token ", vga::RED);
            let mut buf = [0u8; 12];
            vga::write_line(fmt_usize(self.pos, &mut buf), vga::RED);
        }
    }

    fn var_idx(ch: u8) -> usize {
        (ch - b'a') as usize
    }

    fn run(&mut self) {
        while !self.error && self.peek() != Token::Eof {
            self.statement();
        }
    }

    fn statement(&mut self) {
        if self.error { return; }
        match self.peek() {
            Token::KeywordInt => self.decl_stmt(),
            Token::KeywordIf => self.if_stmt(),
            Token::KeywordWhile => self.while_stmt(),
            Token::KeywordFor => self.for_stmt(),
            Token::KeywordPrint => self.print_stmt(),
            Token::Ident(_) => self.assign_or_expr_stmt(),
            Token::LBrace => self.block(),
            Token::Semicolon => { self.advance(); }
            _ => {
                self.err("unexpected token at statement start");
            }
        }
    }

    fn decl_stmt(&mut self) {
        self.advance(); // consume 'int'
        let name = match self.advance() {
            Token::Ident(ch) => ch,
            _ => { self.err("expected variable name after 'int'"); return; }
        };
        let idx = Self::var_idx(name);
        self.defined[idx] = true;
        if self.peek() == Token::Assign {
            self.advance();
            let val = self.expr();
            self.vars[idx] = val;
        }
        self.expect(Token::Semicolon);
    }

    fn assign_or_expr_stmt(&mut self) {
        let name = match self.peek() {
            Token::Ident(ch) => ch,
            _ => { self.err("expected identifier"); return; }
        };
        self.advance();
        if self.peek() == Token::Assign {
            self.advance();
            let val = self.expr();
            let idx = Self::var_idx(name);
            if !self.defined[idx] {
                self.err("undefined variable");
                return;
            }
            self.vars[idx] = val;
            self.expect(Token::Semicolon);
        } else {
            // It was the start of an expression; we already consumed the ident.
            // We don't support expression statements that start with a variable
            // without assignment, so error.
            self.err("expected '=' after variable");
        }
    }

    fn print_stmt(&mut self) {
        self.advance(); // consume 'print'
        if !self.expect(Token::LParen) { return; }
        match self.peek() {
            Token::String { start: _start, len: _len } => {
                self.advance();
                #[cfg(not(test))]
                {
                    let bytes = &self.src[_start.._start + _len];
                    if let Ok(s) = core::str::from_utf8(bytes) {
                        vga::write_line(s, vga::WHITE);
                    }
                }
            }
            _ => {
                let _val = self.expr();
                #[cfg(not(test))]
                if !self.error {
                    print_i32(_val);
                    vga::newline();
                }
            }
        }
        if !self.expect(Token::RParen) { return; }
        self.expect(Token::Semicolon);
    }

    fn if_stmt(&mut self) {
        self.advance(); // consume 'if'
        if !self.expect(Token::LParen) { return; }
        let cond = self.expr();
        if !self.expect(Token::RParen) { return; }
        if cond != 0 {
            self.block_or_stmt();
            if self.peek() == Token::KeywordElse {
                self.advance();
                self.skip_block_or_stmt();
            }
        } else {
            self.skip_block_or_stmt();
            if self.peek() == Token::KeywordElse {
                self.advance();
                self.block_or_stmt();
            }
        }
    }

    fn while_stmt(&mut self) {
        self.advance(); // consume 'while'
        let loop_start = self.pos;
        if !self.expect(Token::LParen) { return; }
        let cond = self.expr();
        if !self.expect(Token::RParen) { return; }
        if cond == 0 {
            self.skip_block_or_stmt();
            return;
        }
        let body_start = self.pos;
        let mut iterations = 0u32;
        loop {
            if self.error { return; }
            self.pos = body_start;
            self.block_or_stmt();
            if self.error { return; }
            // Re-evaluate condition
            let after_body = self.pos;
            self.pos = loop_start;
            if !self.expect(Token::LParen) { return; }
            let cond = self.expr();
            if !self.expect(Token::RParen) { return; }
            if cond == 0 {
                self.pos = after_body;
                // Need to skip the body to move past it
                // But after_body is already past the body from last iteration
                return;
            }
            iterations += 1;
            if iterations > 100_000 {
                self.err("infinite loop detected (>100000 iterations)");
                return;
            }
        }
    }

    fn for_stmt(&mut self) {
        self.advance(); // consume 'for'
        if !self.expect(Token::LParen) { return; }
        // init
        if self.peek() == Token::KeywordInt {
            self.decl_stmt();
        } else if self.peek() != Token::Semicolon {
            self.assign_or_expr_stmt();
        } else {
            self.advance(); // empty init
        }
        let cond_start = self.pos;
        let cond = self.expr();
        if !self.expect(Token::Semicolon) { return; }
        let update_start = self.pos;
        // Skip the update expression to find body
        self.skip_assign_expr();
        if !self.expect(Token::RParen) { return; }
        if cond == 0 {
            self.skip_block_or_stmt();
            return;
        }
        let body_start = self.pos;
        let mut iterations = 0u32;
        loop {
            if self.error { return; }
            self.pos = body_start;
            self.block_or_stmt();
            if self.error { return; }
            let after_body = self.pos;
            // Execute update
            self.pos = update_start;
            self.exec_assign_expr();
            // Re-evaluate condition
            self.pos = cond_start;
            let cond = self.expr();
            if !self.expect(Token::Semicolon) { return; }
            if cond == 0 {
                self.pos = after_body;
                return;
            }
            iterations += 1;
            if iterations > 100_000 {
                self.err("infinite loop detected (>100000 iterations)");
                return;
            }
        }
    }

    // Skip over an assignment expression like "i = i + 1" without executing
    fn skip_assign_expr(&mut self) {
        // Consume tokens until we hit RParen (for 'for' update clause)
        let mut depth = 0u32;
        loop {
            match self.peek() {
                Token::Eof => return,
                Token::RParen if depth == 0 => return,
                Token::LParen => { depth += 1; self.advance(); }
                Token::RParen => { depth -= 1; self.advance(); }
                _ => { self.advance(); }
            }
        }
    }

    // Execute an assignment expression like "i = i + 1"
    fn exec_assign_expr(&mut self) {
        if let Token::Ident(ch) = self.peek() {
            self.advance();
            if self.peek() == Token::Assign {
                self.advance();
                let val = self.expr();
                let idx = Self::var_idx(ch);
                self.vars[idx] = val;
            }
        }
    }

    fn block_or_stmt(&mut self) {
        if self.peek() == Token::LBrace {
            self.block();
        } else {
            self.statement();
        }
    }

    fn block(&mut self) {
        if self.depth >= MAX_DEPTH {
            self.err("maximum nesting depth exceeded");
            return;
        }
        self.depth += 1;
        if !self.expect(Token::LBrace) { self.depth -= 1; return; }
        while !self.error && self.peek() != Token::RBrace && self.peek() != Token::Eof {
            self.statement();
        }
        self.expect(Token::RBrace);
        self.depth -= 1;
    }

    // Skip a block or single statement without executing
    fn skip_block_or_stmt(&mut self) {
        if self.peek() == Token::LBrace {
            self.skip_block();
        } else {
            self.skip_statement();
        }
    }

    fn skip_block(&mut self) {
        if self.peek() != Token::LBrace { return; }
        self.advance();
        let mut depth = 1u32;
        while depth > 0 && self.peek() != Token::Eof {
            match self.advance() {
                Token::LBrace => depth += 1,
                Token::RBrace => depth -= 1,
                _ => {}
            }
        }
    }

    fn skip_statement(&mut self) {
        // Skip tokens until we find a semicolon
        loop {
            match self.peek() {
                Token::Eof | Token::RBrace => return,
                Token::Semicolon => { self.advance(); return; }
                Token::LBrace => { self.skip_block(); return; }
                _ => { self.advance(); }
            }
        }
    }

    // Expression parsing: comparison -> additive -> multiplicative -> unary -> primary
    fn expr(&mut self) -> i32 {
        self.comparison()
    }

    fn comparison(&mut self) -> i32 {
        let mut left = self.additive();
        loop {
            if self.error { return 0; }
            match self.peek() {
                Token::Eq => { self.advance(); let r = self.additive(); left = if left == r { 1 } else { 0 }; }
                Token::Neq => { self.advance(); let r = self.additive(); left = if left != r { 1 } else { 0 }; }
                Token::Lt => { self.advance(); let r = self.additive(); left = if left < r { 1 } else { 0 }; }
                Token::Gt => { self.advance(); let r = self.additive(); left = if left > r { 1 } else { 0 }; }
                Token::Lte => { self.advance(); let r = self.additive(); left = if left <= r { 1 } else { 0 }; }
                Token::Gte => { self.advance(); let r = self.additive(); left = if left >= r { 1 } else { 0 }; }
                _ => break,
            }
        }
        left
    }

    fn additive(&mut self) -> i32 {
        let mut left = self.multiplicative();
        loop {
            if self.error { return 0; }
            match self.peek() {
                Token::Plus => { self.advance(); left = left.wrapping_add(self.multiplicative()); }
                Token::Minus => { self.advance(); left = left.wrapping_sub(self.multiplicative()); }
                _ => break,
            }
        }
        left
    }

    fn multiplicative(&mut self) -> i32 {
        let mut left = self.unary();
        loop {
            if self.error { return 0; }
            match self.peek() {
                Token::Star => { self.advance(); left = left.wrapping_mul(self.unary()); }
                Token::Slash => {
                    self.advance();
                    let r = self.unary();
                    if r == 0 { self.err("division by zero"); return 0; }
                    left = left.wrapping_div(r);
                }
                Token::Percent => {
                    self.advance();
                    let r = self.unary();
                    if r == 0 { self.err("division by zero"); return 0; }
                    left = left.wrapping_rem(r);
                }
                _ => break,
            }
        }
        left
    }

    fn unary(&mut self) -> i32 {
        if self.peek() == Token::Minus {
            self.advance();
            return self.primary().wrapping_neg();
        }
        self.primary()
    }

    fn primary(&mut self) -> i32 {
        match self.peek() {
            Token::Number(n) => { self.advance(); n }
            Token::Ident(ch) => {
                let idx = Self::var_idx(ch);
                if !self.defined[idx] {
                    self.err("undefined variable");
                    return 0;
                }
                self.advance();
                self.vars[idx]
            }
            Token::LParen => {
                self.advance();
                let val = self.expr();
                self.expect(Token::RParen);
                val
            }
            _ => {
                self.err("expected expression");
                0
            }
        }
    }
}

#[cfg(not(test))]
fn print_i32(val: i32) {
    if val == 0 {
        vga::put_char(b'0', vga::WHITE);
        return;
    }
    let mut n = val;
    if n < 0 {
        vga::put_char(b'-', vga::WHITE);
        // Handle i32::MIN by using wrapping
        if n == i32::MIN {
            vga::write_str("2147483648", vga::WHITE);
            return;
        }
        n = -n;
    }
    let mut buf = [0u8; 11];
    let mut pos = 11;
    while n > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    for i in pos..11 {
        vga::put_char(buf[i], vga::WHITE);
    }
}

fn fmt_usize(mut val: usize, buf: &mut [u8; 12]) -> &str {
    if val == 0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut pos = 12;
    while val > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    core::str::from_utf8(&buf[pos..]).unwrap_or("?")
}

#[cfg(not(test))]
pub fn run_script(source: &str) {
    let bytes = source.as_bytes();
    if bytes.len() > MAX_SOURCE {
        vga::write_line("Error: source too large (max 2048 bytes)", vga::RED);
        return;
    }

    let mut tokenizer = Tokenizer::new(bytes);
    if let Err(pos) = tokenizer.tokenize() {
        vga::write_str("Error: unexpected character at position ", vga::RED);
        let mut buf = [0u8; 12];
        vga::write_line(fmt_usize(pos, &mut buf), vga::RED);
        return;
    }

    let mut interp = Interpreter::new(bytes, tokenizer.tokens, tokenizer.count);
    interp.run();
}

#[cfg(not(test))]
pub fn repl() {
    vga::write_line("BASIC REPL (type 'exit' or press Esc to quit)", vga::LIGHT_CYAN);

    let mut vars = [0i32; MAX_VARS];
    let mut defined = [false; MAX_VARS];

    loop {
        vga::write_str("basic> ", vga::GREEN);
        let mut line = [0u8; 256];
        let mut len = 0usize;

        loop {
            let key = keyboard::poll_key();
            match key {
                KeyEvent::None => continue,
                KeyEvent::Esc => {
                    vga::newline();
                    return;
                }
                KeyEvent::Enter => {
                    vga::newline();
                    break;
                }
                KeyEvent::Backspace => {
                    if len > 0 {
                        len -= 1;
                        line[len] = 0;
                        vga::backspace();
                    }
                }
                KeyEvent::Char(c) => {
                    if len < 255 {
                        line[len] = c;
                        len += 1;
                        vga::put_char(c, vga::WHITE);
                    }
                }
                _ => {}
            }
        }

        if len == 0 { continue; }

        let text = match core::str::from_utf8(&line[..len]) {
            Ok(s) => s,
            Err(_) => continue,
        };

        if text == "exit" { return; }

        let bytes = text.as_bytes();
        let mut tokenizer = Tokenizer::new(bytes);
        if let Err(pos) = tokenizer.tokenize() {
            vga::write_str("Error: unexpected character at position ", vga::RED);
            let mut buf = [0u8; 12];
            vga::write_line(fmt_usize(pos, &mut buf), vga::RED);
            continue;
        }

        let mut interp = Interpreter::new(bytes, tokenizer.tokens, tokenizer.count);
        // Restore persistent variable state
        interp.vars = vars;
        interp.defined = defined;
        interp.run();
        // Save variable state for next iteration
        if !interp.error {
            vars = interp.vars;
            defined = interp.defined;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: tokenize source and return token count
    fn tokenize_ok(src: &str) -> usize {
        let mut t = Tokenizer::new(src.as_bytes());
        assert!(t.tokenize().is_ok(), "tokenize failed for: {}", src);
        t.count
    }

    fn tokenize_err(src: &str) {
        let mut t = Tokenizer::new(src.as_bytes());
        assert!(t.tokenize().is_err(), "expected tokenize error for: {}", src);
    }

    // Helper: run script and return interpreter state (vars, error flag)
    fn run_ok(src: &str) -> [i32; MAX_VARS] {
        let bytes = src.as_bytes();
        let mut t = Tokenizer::new(bytes);
        t.tokenize().expect("tokenize failed");
        let mut interp = Interpreter::new(bytes, t.tokens, t.count);
        interp.run();
        assert!(!interp.error, "interpreter error for: {}", src);
        interp.vars
    }

    fn run_err(src: &str) {
        let bytes = src.as_bytes();
        let mut t = Tokenizer::new(bytes);
        if t.tokenize().is_err() { return; } // tokenize error counts
        let mut interp = Interpreter::new(bytes, t.tokens, t.count);
        interp.run();
        assert!(interp.error, "expected interpreter error for: {}", src);
    }

    fn var(vars: &[i32; MAX_VARS], name: u8) -> i32 {
        vars[(name - b'a') as usize]
    }

    // --- Tokenizer tests ---

    #[test]
    fn tokenize_empty() {
        assert_eq!(tokenize_ok(""), 1); // just EOF
    }

    #[test]
    fn tokenize_number() {
        let mut t = Tokenizer::new(b"42");
        t.tokenize().unwrap();
        assert_eq!(t.tokens[0], Token::Number(42));
    }

    #[test]
    fn tokenize_string() {
        let mut t = Tokenizer::new(b"\"hello\"");
        t.tokenize().unwrap();
        assert!(matches!(t.tokens[0], Token::String { .. }));
    }

    #[test]
    fn tokenize_keywords() {
        let src = "int if else while for print";
        let mut t = Tokenizer::new(src.as_bytes());
        t.tokenize().unwrap();
        assert_eq!(t.tokens[0], Token::KeywordInt);
        assert_eq!(t.tokens[1], Token::KeywordIf);
        assert_eq!(t.tokens[2], Token::KeywordElse);
        assert_eq!(t.tokens[3], Token::KeywordWhile);
        assert_eq!(t.tokens[4], Token::KeywordFor);
        assert_eq!(t.tokens[5], Token::KeywordPrint);
    }

    #[test]
    fn tokenize_operators() {
        let src = "+ - * / % ( ) { } ; = == != < > <= >= ,";
        let mut t = Tokenizer::new(src.as_bytes());
        t.tokenize().unwrap();
        assert_eq!(t.tokens[0], Token::Plus);
        assert_eq!(t.tokens[1], Token::Minus);
        assert_eq!(t.tokens[2], Token::Star);
        assert_eq!(t.tokens[3], Token::Slash);
        assert_eq!(t.tokens[4], Token::Percent);
        assert_eq!(t.tokens[5], Token::LParen);
        assert_eq!(t.tokens[6], Token::RParen);
        assert_eq!(t.tokens[7], Token::LBrace);
        assert_eq!(t.tokens[8], Token::RBrace);
        assert_eq!(t.tokens[9], Token::Semicolon);
        assert_eq!(t.tokens[10], Token::Assign);
        assert_eq!(t.tokens[11], Token::Eq);
        assert_eq!(t.tokens[12], Token::Neq);
        assert_eq!(t.tokens[13], Token::Lt);
        assert_eq!(t.tokens[14], Token::Gt);
        assert_eq!(t.tokens[15], Token::Lte);
        assert_eq!(t.tokens[16], Token::Gte);
        assert_eq!(t.tokens[17], Token::Comma);
    }

    #[test]
    fn tokenize_identifiers() {
        let mut t = Tokenizer::new(b"a x z");
        t.tokenize().unwrap();
        assert_eq!(t.tokens[0], Token::Ident(b'a'));
        assert_eq!(t.tokens[1], Token::Ident(b'x'));
        assert_eq!(t.tokens[2], Token::Ident(b'z'));
    }

    #[test]
    fn tokenize_comments_skipped() {
        let src = "42 // this is a comment\n7";
        let mut t = Tokenizer::new(src.as_bytes());
        t.tokenize().unwrap();
        assert_eq!(t.tokens[0], Token::Number(42));
        assert_eq!(t.tokens[1], Token::Number(7));
    }

    #[test]
    fn tokenize_unterminated_string() {
        tokenize_err("\"hello");
    }

    #[test]
    fn tokenize_invalid_char() {
        tokenize_err("@");
    }

    #[test]
    fn tokenize_multi_char_name_rejected() {
        tokenize_err("foo"); // only single-letter vars allowed
    }

    // --- Interpreter: variable declaration and assignment ---

    #[test]
    fn decl_and_assign() {
        let vars = run_ok("int a = 5;");
        assert_eq!(var(&vars, b'a'), 5);
    }

    #[test]
    fn decl_no_init() {
        let vars = run_ok("int x;");
        assert_eq!(var(&vars, b'x'), 0);
    }

    #[test]
    fn reassign_variable() {
        let vars = run_ok("int a = 3; a = 10;");
        assert_eq!(var(&vars, b'a'), 10);
    }

    #[test]
    fn assign_undefined_errors() {
        run_err("a = 5;");
    }

    #[test]
    fn multiple_variables() {
        let vars = run_ok("int a = 1; int b = 2; int c = a + b;");
        assert_eq!(var(&vars, b'c'), 3);
    }

    // --- Arithmetic ---

    #[test]
    fn addition() {
        let vars = run_ok("int a = 3 + 4;");
        assert_eq!(var(&vars, b'a'), 7);
    }

    #[test]
    fn subtraction() {
        let vars = run_ok("int a = 10 - 3;");
        assert_eq!(var(&vars, b'a'), 7);
    }

    #[test]
    fn multiplication() {
        let vars = run_ok("int a = 6 * 7;");
        assert_eq!(var(&vars, b'a'), 42);
    }

    #[test]
    fn division() {
        let vars = run_ok("int a = 15 / 3;");
        assert_eq!(var(&vars, b'a'), 5);
    }

    #[test]
    fn modulo() {
        let vars = run_ok("int a = 17 % 5;");
        assert_eq!(var(&vars, b'a'), 2);
    }

    #[test]
    fn operator_precedence() {
        let vars = run_ok("int a = 2 + 3 * 4;");
        assert_eq!(var(&vars, b'a'), 14);
    }

    #[test]
    fn parentheses() {
        let vars = run_ok("int a = (2 + 3) * 4;");
        assert_eq!(var(&vars, b'a'), 20);
    }

    #[test]
    fn unary_minus() {
        let vars = run_ok("int a = -5;");
        assert_eq!(var(&vars, b'a'), -5);
    }

    #[test]
    fn division_by_zero() {
        run_err("int a = 1 / 0;");
    }

    #[test]
    fn modulo_by_zero() {
        run_err("int a = 1 % 0;");
    }

    #[test]
    fn nested_parens() {
        let vars = run_ok("int a = ((2 + 3) * (4 - 1));");
        assert_eq!(var(&vars, b'a'), 15);
    }

    // --- Comparisons ---

    #[test]
    fn comparison_eq() {
        let vars = run_ok("int a = 5 == 5;");
        assert_eq!(var(&vars, b'a'), 1);
    }

    #[test]
    fn comparison_neq() {
        let vars = run_ok("int a = 5 != 3;");
        assert_eq!(var(&vars, b'a'), 1);
    }

    #[test]
    fn comparison_lt() {
        let vars = run_ok("int a = 3 < 5; int b = 5 < 3;");
        assert_eq!(var(&vars, b'a'), 1);
        assert_eq!(var(&vars, b'b'), 0);
    }

    #[test]
    fn comparison_gt() {
        let vars = run_ok("int a = 5 > 3;");
        assert_eq!(var(&vars, b'a'), 1);
    }

    #[test]
    fn comparison_lte_gte() {
        let vars = run_ok("int a = 5 <= 5; int b = 5 >= 6;");
        assert_eq!(var(&vars, b'a'), 1);
        assert_eq!(var(&vars, b'b'), 0);
    }

    // --- Control flow ---

    #[test]
    fn if_true_branch() {
        let vars = run_ok("int a = 0; if (1) { a = 42; }");
        assert_eq!(var(&vars, b'a'), 42);
    }

    #[test]
    fn if_false_branch() {
        let vars = run_ok("int a = 0; if (0) { a = 42; }");
        assert_eq!(var(&vars, b'a'), 0);
    }

    #[test]
    fn if_else() {
        let vars = run_ok("int a = 0; if (0) { a = 1; } else { a = 2; }");
        assert_eq!(var(&vars, b'a'), 2);
    }

    #[test]
    fn while_loop() {
        let vars = run_ok("int a = 0; int i = 0; while (i < 5) { a = a + 1; i = i + 1; }");
        assert_eq!(var(&vars, b'a'), 5);
        assert_eq!(var(&vars, b'i'), 5);
    }

    #[test]
    fn while_never_enters() {
        let vars = run_ok("int a = 99; while (0) { a = 0; }");
        assert_eq!(var(&vars, b'a'), 99);
    }

    #[test]
    fn for_loop() {
        let vars = run_ok("int s = 0; for (int i = 1; i <= 10; i = i + 1) { s = s + i; }");
        assert_eq!(var(&vars, b's'), 55); // sum 1..10
    }

    #[test]
    fn nested_if() {
        let vars = run_ok("int a = 0; if (1) { if (1) { a = 99; } }");
        assert_eq!(var(&vars, b'a'), 99);
    }

    #[test]
    fn nested_loops() {
        let vars = run_ok(
            "int s = 0; \
             for (int i = 0; i < 3; i = i + 1) { \
                 for (int j = 0; j < 3; j = j + 1) { \
                     s = s + 1; \
                 } \
             }"
        );
        assert_eq!(var(&vars, b's'), 9);
    }

    // --- Print (just check no crash, output goes to VGA stub) ---

    #[test]
    fn print_number_no_crash() {
        run_ok("print(42);");
    }

    #[test]
    fn print_string_no_crash() {
        run_ok("print(\"hello\");");
    }

    #[test]
    fn print_expression_no_crash() {
        run_ok("int a = 5; print(a + 3);");
    }

    // --- Edge cases ---

    #[test]
    fn empty_program() {
        run_ok("");
    }

    #[test]
    fn semicolons_only() {
        run_ok(";;;");
    }

    #[test]
    fn complex_expression() {
        let vars = run_ok("int a = (10 + 20) * 3 - 5 / 1 + 7 % 3;");
        // (30) * 3 - 5 + 1 = 90 - 5 + 1 = 86
        assert_eq!(var(&vars, b'a'), 86);
    }

    #[test]
    fn all_26_variables() {
        let mut src = std::string::String::new();
        for c in b'a'..=b'z' {
            src.push_str(&std::format!("int {} = {}; ", c as char, (c - b'a') as i32));
        }
        let vars = run_ok(&src);
        for c in b'a'..=b'z' {
            assert_eq!(var(&vars, c), (c - b'a') as i32);
        }
    }
}
