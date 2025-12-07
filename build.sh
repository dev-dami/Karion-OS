#!/bin/bash

# Karion-OS Build Script

echo "Building Karion-OS..."

# Check if required tools are available
if ! [ -x "$(which nasm)" ]; then
  echo "Error: nasm is not installed." >&2
  exit 1
fi

if ! [ -x "$(which gcc)" ]; then
  echo "Error: gcc is not installed." >&2
  exit 1
fi

if ! [ -x "$(which ld)" ]; then
  echo "Error: ld is not installed." >&2
  exit 1
fi

# Compile the C source files with include path
gcc -m32 -c src/kernel.c -o kernel.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/source.c -o source.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/keyboard.c -o keyboard.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/shell.c -o shell.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/filesystem.c -o filesystem.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include

# Compile the assembly files using NASM
nasm -f elf32 src/boot.asm -o boot.o

# Link everything together
ld -m elf_i386 -T src/linker.ld -o kernel.bin boot.o kernel.o source.o keyboard.o shell.o filesystem.o

# Create ISO if GRUB is available
if [ -x "$(which grub-mkrescue)" ]; then
    # Create boot directory structure
    mkdir -p iso/boot/grub

    # Copy kernel and rename it to "kernel" (without .bin extension) to match grub.cfg
    cp kernel.bin iso/boot/kernel

    # Copy grub config
    cp src/grub.cfg iso/boot/grub/

    # Create ISO
    grub-mkrescue -o Karion-OS.iso iso/

    echo "Build complete!"
    echo "Kernel: kernel.bin"
    echo "ISO: Karion-OS.iso"
else
    echo "Build complete! (ISO creation skipped - GRUB not found)"
    echo "Kernel: kernel.bin"
fi

# Cleanup object files
rm -f *.o
rm -rf iso/

echo "Done."