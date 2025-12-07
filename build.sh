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

# Create build artifacts directory
mkdir -p buildartifacts

# Compile the C source files with include path
gcc -m32 -c src/kernel.c -o buildartifacts/kernel.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/source.c -o buildartifacts/source.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/keyboard.c -o buildartifacts/keyboard.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/shell.c -o buildartifacts/shell.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/filesystem.c -o buildartifacts/filesystem.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/malloc.c -o buildartifacts/malloc.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/ramdisk.c -o buildartifacts/ramdisk.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/block.c -o buildartifacts/block.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/buffer.c -o buildartifacts/buffer.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include
gcc -m32 -c src/inode.c -o buildartifacts/inode.o -ffreestanding -fno-stack-protector -nostdlib -Wall -Wextra -I include

# Compile the assembly files using NASM
nasm -f elf32 src/boot.asm -o buildartifacts/boot.o

# Link everything together
ld -m elf_i386 -T src/linker.ld -o buildartifacts/kernel.bin buildartifacts/boot.o buildartifacts/kernel.o buildartifacts/source.o buildartifacts/keyboard.o buildartifacts/shell.o buildartifacts/filesystem.o buildartifacts/malloc.o buildartifacts/ramdisk.o buildartifacts/block.o buildartifacts/buffer.o buildartifacts/inode.o

# Create ISO if GRUB is available
if [ -x "$(which grub-mkrescue)" ]; then
    # Create boot directory structure
    mkdir -p iso/boot/grub

    # Copy kernel and rename it to "kernel" (without .bin extension) to match grub.cfg
    cp buildartifacts/kernel.bin iso/boot/kernel

    # Copy grub config
    cp src/grub.cfg iso/boot/grub/

    # Create ISO in iso folder
    grub-mkrescue -o iso/Karion-OS.iso iso/

    echo "Build complete!"
    echo "Kernel: buildartifacts/kernel.bin"
    echo "ISO: iso/Karion-OS.iso"
else
    echo "Build complete! (ISO creation skipped - GRUB not found)"
    echo "Kernel: buildartifacts/kernel.bin"
fi

# Cleanup temporary iso boot structure (keep the ISO file)
rm -rf iso/boot

echo "Done."