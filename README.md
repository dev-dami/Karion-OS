# Karion-OS

> **This project is under active development.** A lot of things are buggy, incomplete, or straight up broken. Expect crashes, weird behavior, and missing features. If something doesn't work, it's probably a known issue. PRs and bug reports welcome.

A bare-metal x86 operating system kernel written in Rust, featuring a Unix-like shell, block filesystem, text editor, BASIC interpreter, and built-in games.

## Why Rust?

This project started in C. It worked, but I kept running into the same problems — buffer overflows, memory leaks that were impossible to track down, data races in interrupt handlers, and just general undefined behavior that would silently corrupt things. Debugging a kernel with no OS underneath to catch your mistakes is brutal.

At some point I was fighting with a heap allocator bug that was silently leaking memory and I just thought — why not try Rust? So I said f it and started porting everything over.

Turns out it was the right call:
- The borrow checker caught a data race in the keyboard ring buffer that I never would have found in C
- `cargo test` lets me run 90 unit tests on my actual kernel code without booting a VM
- The compiler literally won't let you forget to handle cases (every keyboard scancode, every shell command)
- `unsafe` blocks are only where they have to be (hardware registers, VGA framebuffer) — everything else is safe

Only `boot.asm` and `isr.asm` are still assembly because interrupt stubs need `pushad`/`iret`. Everything else is Rust.

## Features

> Most of these work but some are still buggy. The filesystem, shell commands, and games all have rough edges. Don't expect everything to work perfectly — this is a hobby OS, not production software.

- **Boot Animation** — ASCII art logo with animated progress bar
- **Unix-Like Shell** — Command history (arrow keys), path navigation, I/O redirection
- **Block Filesystem** — 1MB RAM disk with inodes, directories, file create/read/write/delete
- **Text Editor (nano)** — Full-screen editor with Ctrl+S save, Ctrl+X exit, line editing (buggy)
- **BASIC Interpreter** — Variables, if/else, while/for loops, print, interactive REPL (buggy)
- **Games** — Snake, Tic-Tac-Toe, number guessing (press Q or ESC to quit)
- **Memory Management** — Physical memory manager, paging, kernel heap with coalescing
- **Hardware Drivers** — PIT timer (100Hz), PS/2 keyboard with shift/ctrl, VGA text mode
- **Interrupt Handling** — GDT, IDT, PIC 8259, syscall interface (INT 0x80)

## Shell Commands

> Some commands might not work as expected — especially path-based ones like `cd`, `cat`, `mv`. Known issue with string comparison on this bare-metal target.

| Command | Description |
|---------|-------------|
| `help [cmd]` | Show help |
| `clear` | Clear screen |
| `echo [text] [> file]` | Print text or redirect to file |
| `ls [path]` | List directory |
| `cd <dir>` | Change directory |
| `pwd` | Print working directory |
| `cat <file>` | Read file |
| `touch <file>` | Create empty file |
| `mkdir <dir>` | Create directory |
| `rm <path>` | Remove file/directory |
| `mv <src> <dst>` | Move/rename |
| `stat <path>` | File info |
| `whoami` | Current user |
| `hostname` | System hostname |
| `uname [-a]` | System info |
| `uptime` | System uptime |
| `meminfo` | Memory usage |
| `history` | Command history |
| `nano [file]` | Text editor |
| `basic [file]` | BASIC interpreter/REPL |
| `snake` | Snake game |
| `tictactoe` | Tic-Tac-Toe |
| `guess` | Number guessing game |

## Build

```bash
./build.sh
```

Builds the Rust kernel, assembles boot/ISR stubs with NASM, links with LD, and creates a bootable GRUB ISO at `build/Karion-OS.iso`.

**Requirements:** `nasm`, `ld` (i686 cross-linker), `grub-mkrescue`, `xorriso`, Rust with `i686-unknown-linux-gnu` target.

Run tests:
```bash
cargo test
```

## Architecture

```
Cargo.toml            Rust crate config (no_std, staticlib)
build.sh              Build pipeline: cargo + nasm + ld + grub

asm/
  boot.asm            Multiboot entry, stack setup
  isr.asm             Interrupt/exception stubs (pushad/iret)

boot/
  linker.ld           ELF layout at 1MB
  grub.cfg            GRUB bootloader config

src/
  lib.rs              Kernel entry point, init sequence, main loop
  gdt.rs              Global Descriptor Table (flat model)
  idt.rs              Interrupt Descriptor Table (256 entries)
  isr.rs              Interrupt dispatcher, exception handlers
  pic.rs              PIC 8259 initialization and EOI
  pmm.rs              Physical memory manager (bitmap, 32MB)
  paging.rs           x86 paging (identity maps 20MB)
  heap.rs             Kernel heap allocator (linked-list, 4MB)
  vga.rs              VGA text-mode framebuffer (80x25, 16 colors)
  keyboard.rs         PS/2 scancode decoder (shift, ctrl, extended)
  shell.rs            Shell with history and command dispatch
  fs.rs               Filesystem interface (paths, cwd, CRUD)
  blockfs.rs          Block filesystem (inodes, bitmaps, RAM disk)
  editor.rs           Nano-like text editor
  basic.rs            BASIC interpreter with REPL
  games/              Snake, Tic-Tac-Toe, number guessing
  drivers/            PIT timer, PS/2 keyboard IRQ handler
  io.rs               Port I/O (inb, outb)
  syscall.rs          INT 0x80 syscall interface
  boot_anim.rs        Boot animation sequence
  intrinsics.rs       memcpy, memset, etc. for no_std
```

## Known Issues

- String comparison is broken on the bare-metal i686 target — Rust's `==` on `&str` generates bad code with static relocation. We use a manual byte-by-byte workaround but it doesn't cover every code path yet
- The text editor (nano) can be glitchy with long lines
- The BASIC interpreter is minimal — no functions, no arrays, single-char variable names only
- Filesystem is RAM-only — everything is lost on reboot
- No networking, no processes, no multitasking (yet)
- Games might leave visual artifacts when quitting
- A lot of things will probably crash in ways we haven't found yet

## License

See [LICENSE](LICENSE).
