#include "output.h"
#include "source.h"

void print_directory_entry(char* type, char* name, unsigned char color)
{
    print_formatted_string(type, color);
    print_formatted_string(name, WHITE_COLOR);
    print_newline();
}
