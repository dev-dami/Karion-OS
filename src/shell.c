#include "shell.h"
#include "filesystem.h"
#include "source.h"
#include "output.h"
#include "malloc.h"

shell_state_t shell_state;  // Current shell state

void shell_init()
{
    shell_state.input_index = 0;
    shell_state.current_input[0] = '\0';
    strcpy(shell_state.current_path, "/");
    strcpy(shell_state.current_directory.path, "/");
    shell_state.current_directory.entry_count = 0;

    filesystem_init();  // Initialize file system

    // Welcome message
    print_formatted_string("Karion-OS Shell v1.0", WHITE_COLOR);
    print_newline();
    print_formatted_string("Type 'help' for available commands", YELLOW);
    print_newline();
    print_prompt();
}

void print_prompt()
{
    print_newline();
    print_formatted_string("KARION-OS:", YELLOW);
    print_formatted_string(shell_state.current_path, GREEN);
    print_formatted_string("> ", YELLOW);
}

// Parse input into command and arguments
int parse_command(char* input, char* command, char** args)
{
    int arg_count = 0, in_token = 0;
    char* token_start = NULL;
    
    for (int i = 0; input[i] != '\0'; i++) {
        if (input[i] == ' ' || input[i] == '\t') {
            if (in_token) { input[i] = '\0'; args[arg_count++] = token_start; in_token = 0; }
        } else if (!in_token) { token_start = &input[i]; in_token = 1; }
    }
    if (in_token) args[arg_count++] = token_start;

    if (arg_count > 0) {
        strcpy(command, args[0]);
        for (int i = 0; i < arg_count - 1; i++) args[i] = args[i + 1];
        args[arg_count - 1] = NULL;
        return arg_count;
    }
    return 0;
}

// Execute a command
int execute_command(char* command, char** args)
{
    if (strcmp(command, "help") == 0) return cmd_help(args);
    if (strcmp(command, "clear") == 0) return cmd_clear(args);
    if (strcmp(command, "echo") == 0) return cmd_echo(args);
    if (strcmp(command, "mkdir") == 0) return cmd_mkdir(args);
    if (strcmp(command, "ls") == 0) return cmd_ls(args);
    if (strcmp(command, "pwd") == 0) return cmd_pwd(args);
    if (strcmp(command, "cd") == 0) return cmd_cd(args);
    if (strcmp(command, "touch") == 0) return cmd_touch(args);
    if (strcmp(command, "del") == 0) return cmd_del(args);
    if (strcmp(command, "cat") == 0) return cmd_cat(args);
    if (strcmp(command, "") == 0) return 0;

    print_string("\nCommand not found: ", RED);
    print_string(command, RED);
    print_string("\nType 'help' for available commands", RED);
    return -1;
}

// Built-in commands
int cmd_help(char** args)
{
    print_newline();
    print_formatted_string("Available commands:", WHITE_COLOR);
    print_newline();
    print_formatted_string("  help     - Show this help message", WHITE_COLOR);
    print_newline();
    print_formatted_string("  clear    - Clear the screen", WHITE_COLOR);
    print_newline();
    print_formatted_string("  echo     - Print text to the screen", WHITE_COLOR);
    print_newline();
    print_formatted_string("  mkdir    - Create a new directory", WHITE_COLOR);
    print_newline();
    print_formatted_string("  touch    - Create a new file", WHITE_COLOR);
    print_newline();
    print_formatted_string("  del      - Delete a file or directory", WHITE_COLOR);
    print_newline();
    print_formatted_string("  ls       - List directory contents", WHITE_COLOR);
    print_newline();
    print_formatted_string("  pwd      - Print working directory", WHITE_COLOR);
    print_newline();
    print_formatted_string("  cd       - Change directory", WHITE_COLOR);
    print_newline();
    print_formatted_string("  cat      - Read and display file contents", WHITE_COLOR);
    print_newline();
    print_formatted_string("  echo >   - Write text to file (e.g., echo hello > file.txt)", WHITE_COLOR);
    print_newline();
    return 0;
}

int cmd_clear(char** args)
{
    clear_screen();
    vga_index = 0;
    return 0;
}

int cmd_echo(char** args)
{
    if (!args[0]) { print_newline(); return 0; }

    // Check for redirection (echo text > file)
    int redirect_index = -1;
    for (int i = 0; args[i]; i++) {
        if (strcmp(args[i], ">") == 0) {
            redirect_index = i;
            break;
        }
    }

    if (redirect_index >= 0) {
        // File redirection mode
        if (!args[redirect_index + 1]) {
            print_formatted_string("Usage: echo <text> > <filename>", RED);
            print_newline();
            return -1;
        }

        // Build the text to write (everything before >)
        char text[MAX_COMMAND_LENGTH];
        text[0] = '\0';
        for (int i = 0; i < redirect_index; i++) {
            if (i > 0) strcat(text, " ");
            strcat(text, args[i]);
        }

        // Get filename
        char* filename = args[redirect_index + 1];
        
        // Build full path
        char full_path[MAX_COMMAND_LENGTH];
        if (strcmp(shell_state.current_path, "/") == 0) {
            strcpy(full_path, "/");
            strcat(full_path, filename);
        } else {
            strcpy(full_path, shell_state.current_path);
            strcat(full_path, "/");
            strcat(full_path, filename);
        }

        // Create file if it doesn't exist, or write to existing file
        if (fs_write_file(full_path, text)) {
            // Success - no output
            return 0;
        } else {
            // Try creating new file
            if (fs_create_file(full_path, text)) {
                return 0;
            }
            print_formatted_string("Error writing to file", RED);
            print_newline();
            return -1;
        }
    } else {
        // Normal echo mode - print to screen
        print_newline();
        for (int i = 0; args[i]; i++) {
            print_formatted_string(args[i], WHITE_COLOR);
            if (args[i + 1]) print_formatted_string(" ", WHITE_COLOR);
        }
        return 0;
    }
}

int cmd_mkdir(char** args)
{
    if (!args[0]) { print_formatted_string("Usage: mkdir <directory_name>", RED); print_newline(); return -1; }

    char full_path[MAX_COMMAND_LENGTH];
    if (strcmp(shell_state.current_path, "/") == 0) { strcpy(full_path, "/"); strcat(full_path, args[0]); }
    else { strcpy(full_path, shell_state.current_path); strcat(full_path, "/"); strcat(full_path, args[0]); }

    if (fs_create_directory(full_path)) {
        print_formatted_string("Directory created: ", GREEN);
        print_newline();
        print_formatted_string(full_path, GREEN);
        print_newline();
        return 0;
    }
    print_formatted_string("Error creating directory", RED);
    print_newline();
    return -1;
}

int cmd_ls(char** args)
{
    directory_t dir;
    if (fs_list_directory(shell_state.current_path, &dir)) {
        for (int i = 0; i < dir.entry_count; i++) {
            print_formatted_string(dir.entries[i].is_directory ? "[DIR] " : "[FILE] ", dir.entries[i].is_directory ? YELLOW : WHITE_COLOR);
            print_formatted_string(dir.entries[i].name, WHITE_COLOR);
            print_newline();
        }
        return 0;
    }
    print_formatted_string("\nError listing directory", RED);
    return -1;
}

int cmd_pwd(char** args)
{
    print_newline();
    print_formatted_string(shell_state.current_path, GREEN);
    print_newline();
    return 0;
}

int cmd_cd(char** args)
{
    if (!args[0]) { print_formatted_string("Usage: cd <directory_name>", RED); print_newline(); return -1; }

    char new_path[MAX_COMMAND_LENGTH];

    if (strcmp(args[0], "..") == 0) {
        if (strcmp(shell_state.current_path, "/") == 0) return 0;
        int len = strlen(shell_state.current_path);
        if (len > 1) {
            for (int i = len - 2; i >= 0; i--) {
                if (shell_state.current_path[i] == '/') {
                    strncpy(new_path, shell_state.current_path, i + 1);
                    new_path[i + 1] = '\0';
                    break;
                }
            }
        } else strcpy(new_path, "/");
    } else if (strcmp(args[0], ".") == 0) return 0;
    else if (args[0][0] == '/') strcpy(new_path, args[0]);
    else {
        if (strcmp(shell_state.current_path, "/") == 0) { strcpy(new_path, "/"); strcat(new_path, args[0]); }
        else { strcpy(new_path, shell_state.current_path); strcat(new_path, "/"); strcat(new_path, args[0]); }
    }

    if (fs_change_directory(new_path)) {
        strcpy(shell_state.current_path, new_path);
        fs_list_directory(shell_state.current_path, &shell_state.current_directory);
        return 0;
    }
    print_formatted_string("Directory not found: ", RED);
    print_newline();
    print_formatted_string(new_path, RED);
    print_newline();
    return -1;
}

int cmd_touch(char** args)
{
    if (!args[0]) { print_formatted_string("Usage: touch <file_name>", RED); print_newline(); return -1; }

    char full_path[MAX_COMMAND_LENGTH];
    if (strcmp(shell_state.current_path, "/") == 0) { strcpy(full_path, "/"); strcat(full_path, args[0]); }
    else { strcpy(full_path, shell_state.current_path); strcat(full_path, "/"); strcat(full_path, args[0]); }

    if (fs_create_file(full_path, NULL)) {
        print_formatted_string("File created: ", GREEN);
        print_newline();
        print_formatted_string(full_path, GREEN);
        print_newline();
        return 0;
    }
    print_formatted_string("Error creating file", RED);
    print_newline();
    return -1;
}

int cmd_del(char** args)
{
    if (!args[0]) { print_formatted_string("Usage: del <file_or_directory_name>", RED); print_newline(); return -1; }

    char full_path[MAX_COMMAND_LENGTH];
    if (strcmp(shell_state.current_path, "/") == 0) { strcpy(full_path, "/"); strcat(full_path, args[0]); }
    else { strcpy(full_path, shell_state.current_path); strcat(full_path, "/"); strcat(full_path, args[0]); }

    if (fs_delete_file(full_path) || fs_delete_directory(full_path)) {
        print_formatted_string("Deleted: ", GREEN);
        print_newline();
        print_formatted_string(full_path, GREEN);
        print_newline();
        return 0;
    }

    print_formatted_string("File or directory not found: ", RED);
    print_newline();
    print_formatted_string(full_path, RED);
    print_newline();
    return -1;
}

int cmd_cat(char** args)
{
    if (!args[0]) {
        print_formatted_string("Usage: cat <filename>", RED);
        print_newline();
        return -1;
    }

    // Build full path
    char full_path[MAX_COMMAND_LENGTH];
    if (strcmp(shell_state.current_path, "/") == 0) {
        strcpy(full_path, "/");
        strcat(full_path, args[0]);
    } else {
        strcpy(full_path, shell_state.current_path);
        strcat(full_path, "/");
        strcat(full_path, args[0]);
    }

    // Read file
    char* content = fs_read_file(full_path);
    if (content == NULL) {
        print_formatted_string("File not found or error reading: ", RED);
        print_newline();
        print_formatted_string(full_path, RED);
        print_newline();
        return -1;
    }

    // Display file content
    print_newline();
    print_formatted_string(content, WHITE_COLOR);
    print_newline();

    // Free the buffer (allocated by fs_read_file)
    kfree(content);

    return 0;
}
