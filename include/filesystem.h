#ifndef FILESYSTEM_DOT_H
#define FILESYSTEM_DOT_H

#include "shell.h"

// file system simulation funcs
int fs_init();
int fs_create_directory(char* path);
int fs_create_file(char* path, char* content);
int fs_list_directory(char* path, directory_t* result);
int fs_change_directory(char* path);
char* fs_get_current_path();

// Init file system
void filesystem_init();

// get directory by path
directory_t* get_directory_by_path(char* path);

// add entry to a directory
int add_entry_to_directory(char* path, char* name, int is_directory, int size);

#endif 