#!/bin/bash

# Karion-OS Build Script
set -euo pipefail

echo "Building Karion-OS..."

for tool in nasm cargo ld; do
  if ! [ -x "$(which $tool)" ]; then
    echo "Error: $tool is not installed." >&2
    exit 1
  fi
done

mkdir -p build

RUST_TARGET="i686-unknown-linux-gnu"

if ! rustup target list --installed | grep -q "^${RUST_TARGET}$"; then
  echo "Installing Rust target: ${RUST_TARGET}"
  rustup target add "${RUST_TARGET}"
fi

# Build Rust kernel
cargo build --release --target "${RUST_TARGET}"

# Assemble boot stubs
nasm -f elf32 asm/boot.asm -o build/boot.o
nasm -f elf32 asm/isr.asm -o build/isr.o

# Link kernel binary
ld -m elf_i386 -T boot/linker.ld -o build/kernel.bin \
  build/boot.o \
  build/isr.o \
  "target/${RUST_TARGET}/release/libkarion_kernel.a"

# Create bootable ISO
if [ -x "$(which grub-mkrescue)" ]; then
    mkdir -p build/isodir/boot/grub
    cp build/kernel.bin build/isodir/boot/kernel
    cp boot/grub.cfg build/isodir/boot/grub/
    grub-mkrescue -o build/Karion-OS.iso build/isodir/
    rm -rf build/isodir

    echo ""
    echo "Build complete!"
    echo "  Kernel: build/kernel.bin"
    echo "  ISO:    build/Karion-OS.iso"
else
    echo ""
    echo "Build complete! (ISO skipped — GRUB not found)"
    echo "  Kernel: build/kernel.bin"
fi

echo "Done."
