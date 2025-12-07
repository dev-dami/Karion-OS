# Karion-OS

A minimal educational operating system with a shell interface and a basic file system simulation.

## Why This Project

Karion-OS was developed as a hands-on learning project to explore the fundamentals of operating systems. The focus is on:

* **Understanding OS basics**: Booting, memory, and system initialization
* **Hardware interaction**: Keyboard and VGA communication
* **System design**: Shell interface, I/O handling, and file system simulation
* **C and Assembly integration**: Combining low-level hardware control with high-level programming

This project offers practical experience in systems programming and shows how OS components work together from boot to user interaction.

## Features

* Command-line shell with prompt
* Simulated file system: `mkdir`, `ls`, `cd`, `pwd`
* Colored text output
* Keyboard input handling

## Commands

* `help` — Show commands
* `clear` — Clear screen
* `echo` — Print text
* `ls` — List directory
* `pwd` — Show current directory
* `cd` — Change directory
* `mkdir` — Create directory

## Build

```bash
./build.sh
```

This generates `kernel.bin` and `Karion-OS.iso` for the the emulation.