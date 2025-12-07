#include "keyboard.h"
#include "source.h"
#include "shell.h"

int clicked = 0;
int canSend = 0;
extern shell_state_t shell_state; 

unsigned char get_scancode()
{
    unsigned char status;
    __asm__ volatile ("inb $0x64, %0" : "=a" (status));

    if (status & 0x01) {
        unsigned char inputdata;
        __asm__ volatile ("inb $0x60, %0" : "=a" (inputdata));
        return inputdata;
    }

    return 0;
}

void keyboard_handler()
{
    unsigned char scancode;
    unsigned int shift_key = 0;

    canSend = 0;
    scancode = get_scancode();

    if (scancode == 0) return;  // No key pressed

    if (scancode == 0x2A) shift_key = 1;        // Shift pressed
    else if (scancode & 0x80)                   // Key released
    {
        if ((scancode & 0x7F) == 0x2A) shift_key = 0; // Shift released
    }

    if (!(scancode & 0x80))  // Key press
    {
        char character = 0;
        int is_special = 0;

        switch(scancode) {
            case 0x01:  // ESC - reset shell
                clear_screen();
                vga_index = 0;
                shell_init();
                is_special = 1;
                break;

            case 0x1C:  // Enter - execute command
                if (shell_state.input_index > 0) {
                    shell_state.current_input[shell_state.input_index] = '\0';
                    char command[MAX_ARG_LENGTH];
                    char* args[MAX_ARGS];
                    parse_command(shell_state.current_input, command, args);
                    execute_command(command, args);
                    shell_state.input_index = 0;
                    for (int i = 0; i < MAX_COMMAND_LENGTH; i++)
                        shell_state.current_input[i] = '\0';
                    print_prompt();
                } else {
                    print_prompt();
                }
                is_special = 1;
                break;

            case 0x0E:  // Backspace
                if (shell_state.input_index > 0) {
                    shell_state.input_index--;
                    shell_state.current_input[shell_state.input_index] = '\0';
                    backspace_char();
                }
                is_special = 1;
                break;

            // Alphabet keys
            case 0x1E: character = (shift_key) ? 'A' : 'a'; break;
            case 0x30: character = (shift_key) ? 'B' : 'b'; break;
            case 0x2E: character = (shift_key) ? 'C' : 'c'; break;
            case 0x20: character = (shift_key) ? 'D' : 'd'; break;
            case 0x12: character = (shift_key) ? 'E' : 'e'; break;
            case 0x21: character = (shift_key) ? 'F' : 'f'; break;
            case 0x22: character = (shift_key) ? 'G' : 'g'; break;
            case 0x23: character = (shift_key) ? 'H' : 'h'; break;
            case 0x17: character = (shift_key) ? 'I' : 'i'; break;
            case 0x24: character = (shift_key) ? 'J' : 'j'; break;
            case 0x25: character = (shift_key) ? 'K' : 'k'; break;
            case 0x26: character = (shift_key) ? 'L' : 'l'; break;
            case 0x32: character = (shift_key) ? 'M' : 'm'; break;
            case 0x31: character = (shift_key) ? 'N' : 'n'; break;
            case 0x18: character = (shift_key) ? 'O' : 'o'; break;
            case 0x19: character = (shift_key) ? 'P' : 'p'; break;
            case 0x10: character = (shift_key) ? 'Q' : 'q'; break;
            case 0x13: character = (shift_key) ? 'R' : 'r'; break;
            case 0x1F: character = (shift_key) ? 'S' : 's'; break;
            case 0x14: character = (shift_key) ? 'T' : 't'; break;
            case 0x16: character = (shift_key) ? 'U' : 'u'; break;
            case 0x2F: character = (shift_key) ? 'V' : 'v'; break;
            case 0x11: character = (shift_key) ? 'W' : 'w'; break;
            case 0x2D: character = (shift_key) ? 'X' : 'x'; break;
            case 0x15: character = (shift_key) ? 'Y' : 'y'; break;
            case 0x2C: character = (shift_key) ? 'Z' : 'z'; break;

            // Number keys
            case 0x0B: character = '0'; break;
            case 0x02: character = '1'; break;
            case 0x03: character = '2'; break;
            case 0x04: character = '3'; break;
            case 0x05: character = '4'; break;
            case 0x06: character = '5'; break;
            case 0x07: character = '6'; break;
            case 0x08: character = '7'; break;
            case 0x09: character = '8'; break;
            case 0x0A: character = '9'; break;

            // Special characters
            case 0x29: character = '`'; break;
            case 0x0C: character = '-'; break;
            case 0x0D: character = '='; break;
            case 0x2B: character = '\\'; break;
            case 0x33: character = ','; break;
            case 0x34: character = '.'; break;
            case 0x35: character = '/'; break;
            case 0x1A: character = '['; break;
            case 0x1B: character = ']'; break;
            case 0x27: character = ';'; break;
            case 0x28: character = '\''; break;

            case 0x39: character = ' '; break; // Space
        }

        if (character && !is_special && shell_state.input_index < MAX_COMMAND_LENGTH - 1) {
            shell_state.current_input[shell_state.input_index++] = character;
            print_char(character, WHITE_COLOR);
        }
    }
}
