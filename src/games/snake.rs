use crate::drivers::timer;
use crate::keyboard::{self, KeyEvent};
use crate::vga;

const PLAY_TOP: usize = 2;
const PLAY_BOTTOM: usize = 22;
const PLAY_LEFT: usize = 2;
const PLAY_RIGHT: usize = 77;
const MAX_LEN: usize = 200;
const TICK_INTERVAL: u64 = 13;

#[derive(Copy, Clone, PartialEq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

struct Snake {
    body: [(usize, usize); MAX_LEN],
    head: usize,
    len: usize,
    dir: Dir,
    score: u16,
    food_row: usize,
    food_col: usize,
    seed: u32,
    game_over: bool,
}

impl Snake {
    fn new(seed: u32) -> Self {
        let mid_row = (PLAY_TOP + PLAY_BOTTOM) / 2;
        let mid_col = (PLAY_LEFT + PLAY_RIGHT) / 2;
        let mut body = [(0usize, 0usize); MAX_LEN];
        body[0] = (mid_row, mid_col - 2);
        body[1] = (mid_row, mid_col - 1);
        body[2] = (mid_row, mid_col);
        Self {
            body,
            head: 2,
            len: 3,
            dir: Dir::Right,
            score: 0,
            food_row: 0,
            food_col: 0,
            seed,
            game_over: false,
        }
    }

    fn rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed >> 16) & 0x7fff
    }

    fn tail_index(&self) -> usize {
        if self.head >= self.len - 1 {
            self.head - (self.len - 1)
        } else {
            MAX_LEN - (self.len - 1 - self.head)
        }
    }

    fn spawn_food(&mut self) {
        loop {
            let r = PLAY_TOP + 1 + (self.rand() as usize % (PLAY_BOTTOM - PLAY_TOP - 1));
            let c = PLAY_LEFT + 1 + (self.rand() as usize % (PLAY_RIGHT - PLAY_LEFT - 1));
            if !self.occupies(r, c) {
                self.food_row = r;
                self.food_col = c;
                return;
            }
        }
    }

    fn occupies(&self, row: usize, col: usize) -> bool {
        let tail = self.tail_index();
        let mut i = tail;
        loop {
            if self.body[i] == (row, col) {
                return true;
            }
            if i == self.head {
                break;
            }
            i = (i + 1) % MAX_LEN;
        }
        false
    }

    fn step(&mut self) {
        let (hr, hc) = self.body[self.head];
        let (nr, nc) = match self.dir {
            Dir::Up => (hr.wrapping_sub(1), hc),
            Dir::Down => (hr + 1, hc),
            Dir::Left => (hr, hc.wrapping_sub(1)),
            Dir::Right => (hr, hc + 1),
        };

        // Wall collision
        if nr <= PLAY_TOP || nr >= PLAY_BOTTOM || nc <= PLAY_LEFT || nc >= PLAY_RIGHT {
            self.game_over = true;
            return;
        }

        // Self collision (check before adding new head)
        if self.occupies(nr, nc) {
            self.game_over = true;
            return;
        }

        let ate = nr == self.food_row && nc == self.food_col;

        if !ate {
            // Erase tail
            let tail = self.tail_index();
            let (tr, tc) = self.body[tail];
            vga::put_char_at(tr, tc, b' ', vga::BLACK);
        }

        // Advance head
        let new_head = (self.head + 1) % MAX_LEN;
        self.body[new_head] = (nr, nc);
        self.head = new_head;

        if ate {
            if self.len < MAX_LEN {
                self.len += 1;
            }
            self.score += 10;
            self.spawn_food();
            draw_food(self.food_row, self.food_col);
            draw_score(self.score);
        }

        // Draw new head
        vga::put_char_at(nr, nc, b'O', vga::LIGHT_GREEN);

        // Redraw segment behind head as body char
        let behind = if self.head == 0 { MAX_LEN - 1 } else { self.head - 1 };
        if self.len > 1 {
            let (br, bc) = self.body[behind];
            vga::put_char_at(br, bc, b'o', vga::GREEN);
        }
    }
}

fn draw_border() {
    for col in PLAY_LEFT..=PLAY_RIGHT {
        vga::put_char_at(PLAY_TOP, col, b'-', vga::LIGHT_GRAY);
        vga::put_char_at(PLAY_BOTTOM, col, b'-', vga::LIGHT_GRAY);
    }
    for row in PLAY_TOP..=PLAY_BOTTOM {
        vga::put_char_at(row, PLAY_LEFT, b'|', vga::LIGHT_GRAY);
        vga::put_char_at(row, PLAY_RIGHT, b'|', vga::LIGHT_GRAY);
    }
    vga::put_char_at(PLAY_TOP, PLAY_LEFT, b'+', vga::LIGHT_GRAY);
    vga::put_char_at(PLAY_TOP, PLAY_RIGHT, b'+', vga::LIGHT_GRAY);
    vga::put_char_at(PLAY_BOTTOM, PLAY_LEFT, b'+', vga::LIGHT_GRAY);
    vga::put_char_at(PLAY_BOTTOM, PLAY_RIGHT, b'+', vga::LIGHT_GRAY);
}

fn draw_food(row: usize, col: usize) {
    vga::put_char_at(row, col, b'*', vga::LIGHT_RED);
}

fn draw_score(score: u16) {
    vga::set_cursor(0, 2);
    vga::write_str("Score: ", vga::YELLOW);
    write_u16(score, vga::YELLOW);
    vga::write_str("   ", vga::BLACK);
}

fn write_u16(mut val: u16, color: u8) {
    if val == 0 {
        vga::put_char(b'0', color);
        return;
    }
    let mut buf = [0u8; 5];
    let mut pos = 5;
    while val > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    for &b in &buf[pos..] {
        vga::put_char(b, color);
    }
}

fn draw_snake_full(snake: &Snake) {
    let tail = snake.tail_index();
    let mut i = tail;
    loop {
        let (r, c) = snake.body[i];
        if i == snake.head {
            vga::put_char_at(r, c, b'O', vga::LIGHT_GREEN);
        } else {
            vga::put_char_at(r, c, b'o', vga::GREEN);
        }
        if i == snake.head {
            break;
        }
        i = (i + 1) % MAX_LEN;
    }
}

pub fn run() {
    vga::clear_screen();
    vga::write_centered(0, "SNAKE  -  WASD/Arrows to move  -  Q/ESC to quit", vga::CYAN);
    draw_border();

    let seed = timer::get_ticks() as u32;
    let mut snake = Snake::new(seed);
    snake.spawn_food();

    draw_snake_full(&snake);
    draw_food(snake.food_row, snake.food_col);
    draw_score(snake.score);

    let mut last_tick = timer::get_ticks();

    loop {
        match keyboard::poll_key() {
            KeyEvent::Esc | KeyEvent::Char(b'q') | KeyEvent::Char(b'Q') => return,
            KeyEvent::Char(b'w') | KeyEvent::Char(b'W') | KeyEvent::ArrowUp => {
                if snake.dir != Dir::Down {
                    snake.dir = Dir::Up;
                }
            }
            KeyEvent::Char(b's') | KeyEvent::Char(b'S') | KeyEvent::ArrowDown => {
                if snake.dir != Dir::Up {
                    snake.dir = Dir::Down;
                }
            }
            KeyEvent::Char(b'a') | KeyEvent::Char(b'A') | KeyEvent::ArrowLeft => {
                if snake.dir != Dir::Right {
                    snake.dir = Dir::Left;
                }
            }
            KeyEvent::Char(b'd') | KeyEvent::Char(b'D') | KeyEvent::ArrowRight => {
                if snake.dir != Dir::Left {
                    snake.dir = Dir::Right;
                }
            }
            _ => {}
        }

        let now = timer::get_ticks();
        if now.wrapping_sub(last_tick) >= TICK_INTERVAL {
            last_tick = now;
            snake.step();
            if snake.game_over {
                vga::write_centered(12, "GAME OVER!", vga::LIGHT_RED);
                vga::write_centered(14, "Press any key to exit...", vga::LIGHT_GRAY);
                loop {
                    let k = keyboard::poll_key();
                    if k != KeyEvent::None {
                        return;
                    }
                }
            }
        }
    }
}
