#ifndef FILESYSTEM_DOT_H
#define FILESYSTEM_DOT_H

#include "shell.h"

// File system constants
#define MAX_FILESYSTEM_ENTRIES 100
#define MAX_FILE_SIZE 1024
#define SECTOR_SIZE 512

// File types
#define FILE_TYPE_REGULAR 0
#define FILE_TYPE_DIRECTORY 1
#define FILE_TYPE_SYMLINK 2

// File entry structure
typedef struct {
    char name[MAX_ARG_LENGTH];
    int type;           // 0=file, 1=dir
    int size;
    int start_sector;
    int sector_count;
    char content[MAX_FILE_SIZE];
    int parent_index;
} fs_entry_t;

// Directory structure
typedef struct {
    fs_entry_t entries[MAX_FILESYSTEM_ENTRIES];
    int entry_count;
    int parent_index;
} fs_directory_t;

// File system structure
typedef struct {
    fs_entry_t entries[MAX_FILESYSTEM_ENTRIES];
    int total_entries;
    int initialized;
} filesystem_t;

// File system functions
int fs_init();
int fs_create_directory(char* path);
int fs_create_file(char* path, char* content);
int fs_delete_directory(char* path);
int fs_delete_file(char* path);
int fs_write_file(char* path, char* content);
char* fs_read_file(char* path);
int fs_list_directory(char* path, directory_t* result);
int fs_change_directory(char* path);
char* fs_get_current_path();

// Initialize the simulated file system
void filesystem_init();

// Path utilities
int parse_path(char* full_path, char* parent_path, char* name);
int get_entry_by_path(char* path, fs_entry_t* entry);
int get_parent_directory_index(char* path);

// Real file system operations
int fs_format();
int fs_save_to_memory();
int fs_load_from_memory();

#endif
