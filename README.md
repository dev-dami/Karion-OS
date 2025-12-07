# Karion-OS

A minimal educational operating system with a shell interface and an inode-based file system.

## Why This Project

Karion-OS was developed as a hands-on learning project to explore the fundamentals of operating systems. The focus is on:

* **Understanding OS basics**: Booting, memory, and system initialization
* **Hardware interaction**: Keyboard and VGA communication
* **System design**: Shell interface, I/O handling, and file system implementation
* **C and Assembly integration**: Combining low-level hardware control with high-level programming
* **File system architecture**: Inode-based storage, block allocation, and directory management

This project offers practical experience in systems programming and shows how OS components work together from boot to user interaction.

## Features

* Command-line shell with prompt
* Inode-based file system (Xv6-inspired): block allocation, directory entries, file operations
* Memory allocator: heap-based dynamic memory management
* RAM disk: simulated block device storage
* Colored text output
* Keyboard input handling

## Commands

* `help` — Show commands
* `clear` — Clear screen
* `echo` — Print text (or use `echo text > file` to write to file)
* `touch` — Create file
* `cat` — Read and display file contents
* `ls` — List directory
* `pwd` — Show current directory
* `cd` — Change directory
* `mkdir` — Create directory
* `del` — Delete file or directory

## Build

```bash
./build.sh
```

This generates `buildartifacts/kernel.bin` and `iso/Karion-OS.iso` for emulation.