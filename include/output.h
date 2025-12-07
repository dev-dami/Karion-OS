#ifndef OUTPUT_DOT_H
#define OUTPUT_DOT_H

#include "source.h"

// Function to print a string with automatic line wrapping and proper newlines
void print_formatted_string(char* str, unsigned char color);
void print_newline(void);
void print_directory_entry(char* type, char* name, unsigned char color);

#endif /* OUTPUT_DOT_H */