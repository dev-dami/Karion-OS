bits 32

section .multiboot               ; Multiboot header
        dd 0x1BADB002            ; Magic number for bootloader
        dd 0x0                   ; Flags
        dd - (0x1BADB002 + 0x0)  ; Checksum

section .text
global start
extern main                      ; main function in C

start:
        cli                      ; Disable interrupts
        mov esp, stack_space     ; Initialize stack pointer
        call main                ; Call C main
        hlt                      ; Halt CPU

section .bss
resb 8192                        ; 8KB stack space
stack_space:
