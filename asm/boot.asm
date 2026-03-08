bits 32
section .note.GNU-stack noalloc noexec nowrite progbits

section .multiboot
        dd 0x1BADB002            ; multiboot magic
        dd 0x0
        dd - (0x1BADB002 + 0x0)  ; checksum

section .text
global start
extern main

start:
        cli

        ; Set up stack (16-byte aligned for SSE)
        mov esp, stack_space
        and esp, 0xFFFFFFF0

        ; Enable SSE (required by Rust i686 ABI)
        mov eax, cr0
        and ax, 0xFFFB           ; clear CR0.EM (bit 2)
        or ax, 0x2               ; set CR0.MP (bit 1)
        mov cr0, eax
        mov eax, cr4
        or ax, 3 << 9            ; set CR4.OSFXSR and CR4.OSXMMEXCPT
        mov cr4, eax

        call main
        hlt

section .bss
align 16
resb 16384                       ; 16KB stack
stack_space:
