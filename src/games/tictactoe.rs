use crate::drivers::timer;
use crate::keyboard::{self, KeyEvent};
use crate::vga;

const EMPTY: u8 = 0;
const PLAYER_X: u8 = 1;
const PLAYER_O: u8 = 2;

// Grid drawn starting at row 6, col 30
const GRID_ROW: usize = 6;
const GRID_COL: usize = 30;
const CELL_W: usize = 6;
const CELL_H: usize = 3;

struct Game {
    board: [u8; 9],
    seed: u32,
}

impl Game {
    fn new(seed: u32) -> Self {
        Self {
            board: [EMPTY; 9],
            seed,
        }
    }

    fn rand(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed >> 16) & 0x7fff
    }

    fn check_winner(&self) -> u8 {
        const LINES: [[usize; 3]; 8] = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8],
            [0, 3, 6], [1, 4, 7], [2, 5, 8],
            [0, 4, 8], [2, 4, 6],
        ];
        for line in &LINES {
            let a = self.board[line[0]];
            if a != EMPTY && a == self.board[line[1]] && a == self.board[line[2]] {
                return a;
            }
        }
        EMPTY
    }

    fn is_full(&self) -> bool {
        self.board.iter().all(|&c| c != EMPTY)
    }

    fn can_win(&self, player: u8, pos: usize) -> bool {
        if self.board[pos] != EMPTY {
            return false;
        }
        let mut temp = self.board;
        temp[pos] = player;
        const LINES: [[usize; 3]; 8] = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8],
            [0, 3, 6], [1, 4, 7], [2, 5, 8],
            [0, 4, 8], [2, 4, 6],
        ];
        for line in &LINES {
            let a = temp[line[0]];
            if a == player && a == temp[line[1]] && a == temp[line[2]] {
                return true;
            }
        }
        false
    }

    fn ai_move(&mut self) -> usize {
        // Win if possible
        for i in 0..9 {
            if self.can_win(PLAYER_O, i) {
                return i;
            }
        }
        // Block player win
        for i in 0..9 {
            if self.can_win(PLAYER_X, i) {
                return i;
            }
        }
        // Take center
        if self.board[4] == EMPTY {
            return 4;
        }
        // Take a corner
        let corners = [0, 2, 6, 8];
        let start = self.rand() as usize % 4;
        for i in 0..4 {
            let c = corners[(start + i) % 4];
            if self.board[c] == EMPTY {
                return c;
            }
        }
        // Take any empty
        for i in 0..9 {
            if self.board[i] == EMPTY {
                return i;
            }
        }
        0
    }
}

fn draw_grid() {
    // Horizontal lines
    for offset in [CELL_H, CELL_H * 2 + 1] {
        let row = GRID_ROW + offset;
        for c in 0..(CELL_W * 3 + 2) {
            vga::put_char_at(row, GRID_COL + c, b'-', vga::LIGHT_GRAY);
        }
    }
    // Vertical lines
    for offset in [CELL_W, CELL_W * 2 + 1] {
        let col = GRID_COL + offset;
        for r in 0..(CELL_H * 3 + 2) {
            vga::put_char_at(GRID_ROW + r, col, b'|', vga::LIGHT_GRAY);
        }
    }
}

fn cell_center(pos: usize) -> (usize, usize) {
    let cr = pos / 3;
    let cc = pos % 3;
    let row = GRID_ROW + cr * (CELL_H + 1) + CELL_H / 2;
    let col = GRID_COL + cc * (CELL_W + 1) + CELL_W / 2;
    (row, col)
}

fn draw_mark(pos: usize, mark: u8) {
    let (row, col) = cell_center(pos);
    let (ch, color) = if mark == PLAYER_X {
        (b'X', vga::LIGHT_CYAN)
    } else {
        (b'O', vga::LIGHT_RED)
    };
    vga::put_char_at(row, col, ch, color);
}

fn draw_board(game: &Game) {
    for i in 0..9 {
        let (row, col) = cell_center(i);
        if game.board[i] == EMPTY {
            // Show position number hint
            vga::put_char_at(row, col, b'1' + i as u8, vga::DARK_GRAY);
        } else {
            draw_mark(i, game.board[i]);
        }
    }
}

pub fn run() {
    let mut seed = timer::get_ticks() as u32;

    loop {
        vga::clear_screen();
        vga::write_centered(1, "TIC-TAC-TOE", vga::YELLOW);
        vga::write_centered(3, "You are X  -  Keys 1-9 to place  -  Q/ESC to quit", vga::CYAN);

        let mut game = Game::new(seed);
        draw_grid();
        draw_board(&game);

        vga::set_cursor(20, 2);
        vga::write_str("Your move (1-9): ", vga::WHITE);

        let winner;
        loop {
            let key = keyboard::poll_key();
            match key {
                KeyEvent::Esc | KeyEvent::Char(b'q') | KeyEvent::Char(b'Q') => return,
                KeyEvent::Char(c) if c >= b'1' && c <= b'9' => {
                    let pos = (c - b'1') as usize;
                    if game.board[pos] != EMPTY {
                        continue;
                    }
                    game.board[pos] = PLAYER_X;
                    draw_mark(pos, PLAYER_X);

                    let w = game.check_winner();
                    if w != EMPTY || game.is_full() {
                        winner = w;
                        break;
                    }

                    // AI move
                    let ai = game.ai_move();
                    game.board[ai] = PLAYER_O;
                    draw_mark(ai, PLAYER_O);

                    let w = game.check_winner();
                    if w != EMPTY || game.is_full() {
                        winner = w;
                        break;
                    }
                }
                _ => {}
            }
        }

        // Show result
        let msg = match winner {
            PLAYER_X => "You win!",
            PLAYER_O => "Computer wins!",
            _ => "It's a draw!",
        };
        let color = match winner {
            PLAYER_X => vga::LIGHT_GREEN,
            PLAYER_O => vga::LIGHT_RED,
            _ => vga::YELLOW,
        };
        vga::write_centered(18, msg, color);
        vga::write_centered(20, "ENTER to play again  -  Q/ESC to quit", vga::LIGHT_GRAY);

        // Wait for Enter or Esc/Q
        loop {
            match keyboard::poll_key() {
                KeyEvent::Esc | KeyEvent::Char(b'q') | KeyEvent::Char(b'Q') => return,
                KeyEvent::Enter => {
                    seed = timer::get_ticks() as u32;
                    break;
                }
                _ => {}
            }
        }
    }
}
