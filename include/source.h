#ifndef SOURCE_DOT_H
#define SOURCE_DOT_H

#define NULL ((void*)0)

#define BLACK 0
#define GREEN 2
#define RED 4
#define YELLOW 14
#define WHITE_COLOR 15

// Global VGA buffer vars
extern unsigned short* terminal_buffer;
extern unsigned int vga_index;

// Basic output funcs
void print_string(char* str, unsigned char color);
void print_char(char str, unsigned char color);
void clear_screen(void);
void backspace_char(void);
void scroll_screen_up(void);

// String util funcs
int strlen(const char* str);
int strcmp(const char* s1, const char* s2);
char* strcpy(char* dest, const char* src);
char* strcat(char* dest, const char* src);
char* strncpy(char* dest, const char* src, int n);
char* strchr(const char* str, int c);
char* strrchr(const char* str, int c);

// Formatted output funcs
void print_formatted_string(char* str, unsigned char color);
void print_newline(void);

#endif 
