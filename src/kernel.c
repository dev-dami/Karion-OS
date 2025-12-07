#include "source.h"
#include "keyboard.h"
#include "shell.h"
#include "malloc.h"
#include "inode.h"

void main(void)
{
    clear_screen();

    // Initialize memory allocator
    heap_init();

    shell_init();  // initialize shell

    while (1) {
        keyboard_handler();  // proc keyboard input
    }

    return;
}
