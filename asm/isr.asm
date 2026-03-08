bits 32
section .note.GNU-stack noalloc noexec nowrite progbits

extern isr_handler

section .text

%macro ISR_NOERRCODE 1
global isr%1
isr%1:
    push dword 0        ; dummy error code
    push dword %1
    jmp isr_common_stub
%endmacro

%macro ISR_ERRCODE 1
global isr%1
isr%1:
    push dword %1       ; CPU already pushed error code
    jmp isr_common_stub
%endmacro
ISR_NOERRCODE 0
ISR_NOERRCODE 1
ISR_NOERRCODE 2
ISR_NOERRCODE 3
ISR_NOERRCODE 4
ISR_NOERRCODE 5
ISR_NOERRCODE 6
ISR_NOERRCODE 7
ISR_ERRCODE 8
ISR_NOERRCODE 9
ISR_ERRCODE 10
ISR_ERRCODE 11
ISR_ERRCODE 12
ISR_ERRCODE 13
ISR_ERRCODE 14
ISR_NOERRCODE 15
ISR_NOERRCODE 16
ISR_ERRCODE 17
ISR_NOERRCODE 18
ISR_NOERRCODE 19
ISR_NOERRCODE 20
ISR_NOERRCODE 21
ISR_NOERRCODE 22
ISR_NOERRCODE 23
ISR_NOERRCODE 24
ISR_NOERRCODE 25
ISR_NOERRCODE 26
ISR_NOERRCODE 27
ISR_NOERRCODE 28
ISR_ERRCODE 29
ISR_ERRCODE 30
ISR_NOERRCODE 31

; IRQs 0-15 -> ISR 32-47
%assign i 32
%rep 16
ISR_NOERRCODE i
%assign i i+1
%endrep

ISR_NOERRCODE 128               ; syscall (int 0x80)

isr_common_stub:
    pushad

    mov ax, ds
    push eax                        ; save ds

    mov ax, 0x10                    ; kernel data segment selector
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Call isr_handler with pointer to Registers struct on stack.
    ; Align stack to 16 bytes for Rust ABI, preserving original esp.
    mov ebp, esp                    ; save stack (points to Registers)
    sub esp, 4                      ; space for arg
    and esp, 0xFFFFFFF0             ; align to 16 bytes
    mov [esp], ebp                  ; arg = pointer to Registers
    call isr_handler
    mov esp, ebp                    ; restore stack

    pop eax                         ; restore ds
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    popad
    add esp, 8                      ; pop error code + ISR number
    iret
