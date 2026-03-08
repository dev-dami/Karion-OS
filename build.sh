#!/bin/bash

# Karion-OS Build Script
set -euo pipefail

echo "Building Karion-OS..."

# Check if required tools are available
if ! [ -x "$(which nasm)" ]; then
  echo "Error: nasm is not installed." >&2
  exit 1
fi

if ! [ -x "$(which cargo)" ]; then
  echo "Error: cargo is not installed." >&2
  exit 1
fi

if ! [ -x "$(which ld)" ]; then
  echo "Error: ld is not installed." >&2
  exit 1
fi

# Create build artifacts directory
mkdir -p buildartifacts

RUST_TARGET="i686-unknown-linux-gnu"
RUST_MANIFEST="rust/karion_kernel/Cargo.toml"
RUST_LIB="rust/karion_kernel/target/${RUST_TARGET}/release/libkarion_kernel.a"

# Ensure 32-bit Rust target is available
if ! rustup target list --installed | grep -q "^${RUST_TARGET}$"; then
  echo "Installing missing Rust target: ${RUST_TARGET}"
  rustup target add "${RUST_TARGET}"
fi

# Build Rust kernel core (replaces C kernel objects)
cargo build --manifest-path "${RUST_MANIFEST}" --release --target "${RUST_TARGET}"

# Compile the assembly files using NASM
nasm -f elf32 src/boot.asm -o buildartifacts/boot.o
nasm -f elf32 src/isr.asm -o buildartifacts/isr.o

# Link everything together
ld -m elf_i386 -T src/linker.ld -o buildartifacts/kernel.bin \
  buildartifacts/boot.o \
  buildartifacts/isr.o \
  "${RUST_LIB}"

# Create ISO if GRUB is available
if [ -x "$(which grub-mkrescue)" ]; then
    # Create boot directory structure in a separate staging folder
    mkdir -p isodir/boot/grub

    # Copy kernel and rename it to "kernel" (without .bin extension) to match grub.cfg
    cp buildartifacts/kernel.bin isodir/boot/kernel

    # Copy grub config
    cp src/grub.cfg isodir/boot/grub/

    # Ensure output directory exists
    mkdir -p iso

    # Create ISO in iso folder (reading from isodir)
    grub-mkrescue -o iso/Karion-OS.iso isodir/

    # Cleanup staging directory
    rm -rf isodir

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
