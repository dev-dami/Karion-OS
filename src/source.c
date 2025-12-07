#include "source.h"

// Initialize VGA buffer to point to VGA memory
unsigned short* terminal_buffer = (unsigned short*)0xB8000;  // VGA buffer
unsigned int vga_index = 0;                                  // Current position

void clear_screen(void)
{
    // Clear entire screen (25x80 characters)
    int index = 0;
    while (index < 80 * 25 * 2) {
        terminal_buffer[index] = ' ';
        index += 2;
    }
}

void print_string(char* str, unsigned char color)
{
    int index = 0;
    while (str[index]) {
        // Check to wrap to next line
        if (vga_index % 80 == 79) {
            print_newline();
        }

        terminal_buffer[vga_index] = (unsigned short)str[index] | (unsigned short)color << 8;
        index++;
        vga_index++;

        // Check to scroll the screen
        if (vga_index >= 80 * 25) {
            scroll_screen_up();
            vga_index = (24 * 80); // Position at the last line
        }
    }
}

void print_char(char str, unsigned char color)
{
    // Check to wrap to next line
    if (vga_index % 80 == 79) {
        print_newline();
    }

    terminal_buffer[vga_index] = str | (unsigned short)color << 8;
    vga_index++;

    // Check to scroll the screen
    if (vga_index >= 80 * 25) {
        scroll_screen_up();
        vga_index = (24 * 80); // Position at the last line
    }
}

void backspace_char(void)
{
    if (vga_index > 0) {
        vga_index--;
        terminal_buffer[vga_index] = ' ' | (unsigned short)WHITE_COLOR << 8;  // Clear char
    }
}

void scroll_screen_up(void)
{
    // Move lines up by one
    for (int i = 0; i < 24 * 80; i++) {
        terminal_buffer[i] = terminal_buffer[i + 80];
    }

    // Clear last line
    for (int i = 24 * 80; i < 25 * 80; i++) {
        terminal_buffer[i] = ' ' | (unsigned short)WHITE_COLOR << 8;
    }
}

int strlen(const char* str)
{
    int len = 0;
    while (str[len] != '\0') len++;
    return len;
}

int strcmp(const char* s1, const char* s2)
{
    int i = 0;
    while (s1[i] != '\0' && s2[i] != '\0') {
        if (s1[i] != s2[i]) return s1[i] - s2[i];
        i++;
    }
    return s1[i] - s2[i];
}

char* strcpy(char* dest, const char* src)
{
    int i = 0;
    while (src[i] != '\0') { dest[i] = src[i]; i++; }
    dest[i] = '\0';
    return dest;
}

char* strcat(char* dest, const char* src)
{
    int dest_len = strlen(dest), i = 0;
    while (src[i] != '\0') { dest[dest_len + i] = src[i]; i++; }
    dest[dest_len + i] = '\0';
    return dest;
}

char* strncpy(char* dest, const char* src, int n)
{
    int i = 0;
    while (i < n && src[i] != '\0') { dest[i] = src[i]; i++; }
    while (i < n) { dest[i] = '\0'; i++; }
    return dest;
}

char* strrchr(const char* str, int c)
{
    char* last = NULL;
    int i = 0;
    while (str[i] != '\0') {
        if (str[i] == c) last = (char*)&str[i];
        i++;
    }
    if (c == '\0') return (char*)&str[i];  // NULL terminate
    return last;
}

void print_newline(void)
{
    // Advance to next line
    int current_line = vga_index / 80;
    vga_index = (current_line + 1) * 80;

    // If reached bottom of screen, scroll
    if (vga_index >= 80 * 25) {
        // Scroll screen up by one line
        scroll_screen_up();
        // Position cursor at last line
        vga_index = (24 * 80);
    }
}

void print_formatted_string(char* str, unsigned char color)
{
    int index = 0;
    while (str[index] != '\0') {
        if (str[index] == '\n') {
            print_newline();
        } else {
            // Check to wrap to next line
            if (vga_index % 80 == 79) {
                print_newline();
            }

            terminal_buffer[vga_index] = str[index] | (unsigned short)color << 8;
            vga_index++;

            // Check to scroll the screen
            if (vga_index >= 80 * 25) {
                scroll_screen_up();
                vga_index = (24 * 80); // Position at the last line
            }
        }
        index++;
    }
}
