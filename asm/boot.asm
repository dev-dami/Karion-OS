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
        mov esp, stack_space
        call main
        hlt

section .bss
resb 16384                       ; 16KB stack
stack_space:
