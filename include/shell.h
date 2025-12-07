#ifndef SHELL_DOT_H
#define SHELL_DOT_H

#include "source.h"

// Maximum length of a command line input
#define MAX_COMMAND_LENGTH 256
// Maximum number of arguments in a command
#define MAX_ARGS 10
// Maximum length of a single argument
#define MAX_ARG_LENGTH 64

// Structure to represent dir entry
typedef struct {
    char name[MAX_ARG_LENGTH];
    int is_directory;  // 1 if directory, 0 if file
    int size;          // size in bytes (for files)
} directory_entry_t;

// Structure to represent current directory
typedef struct {
    char path[MAX_COMMAND_LENGTH];
    int entry_count; // Maximum 50 entries
    directory_entry_t entries[50];
} directory_t;

// shell state
typedef struct {
    char current_input[MAX_COMMAND_LENGTH];
    int input_index;
    char current_path[MAX_COMMAND_LENGTH];
    directory_t current_directory;
} shell_state_t;

// initialize the shell
void shell_init();

// main shell loop
void shell_run();

// parse command line input into command and arguments
int parse_command(char* input, char* command, char** args);

// execute command with its arguments
int execute_command(char* command, char** args);

// command handlers funcs
int cmd_help(char** args);
int cmd_clear(char** args);
int cmd_echo(char** args);
int cmd_mkdir(char** args);
int cmd_ls(char** args);
int cmd_pwd(char** args);
int cmd_cd(char** args);

// Utility funcs
void print_prompt();
void add_directory_entry(directory_t* dir, char* name, int is_directory, int size);
void clear_directory(directory_t* dir);

#endif 