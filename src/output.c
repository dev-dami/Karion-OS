#include "output.h"
#include "source.h"

void print_newline(void)
{
    // Move to start of next line -- 80 chars per line
    int current_line = vga_index / 80;
    vga_index = (current_line + 1) * 80;
    
    // Wrap around if end of screen reached
    if (vga_index >= 80 * 25) {
        vga_index = (current_line % 25) * 80;
    }
}

void print_formatted_string(char* str, unsigned char color)
{
    int index = 0;
    while (str[index] != '\0') {
        if (str[index] == '\n') {
            print_newline();
        } else {
            // Wrap line if at end
            if (vga_index % 80 == 79) print_newline();
            
            terminal_buffer[vga_index] = str[index] | (unsigned short)color << 8;
            vga_index++;
        }
        index++;
    }
}

void print_directory_entry(char* type, char* name, unsigned char color)
{
    print_formatted_string(type, color);
    print_formatted_string(name, WHITE_COLOR);
    print_newline();
}
