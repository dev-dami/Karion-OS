#include "source.h"
#include "keyboard.h"
#include "shell.h"

void main(void)
{
    clear_screen();

    shell_init();  // initialize the shell

    while (1) {
        keyboard_handler();  // proc keyboard input
    }

    return;
}
