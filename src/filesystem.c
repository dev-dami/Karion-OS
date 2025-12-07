#include "filesystem.h"
#include "source.h"

#define MAX_ENTRIES 100  // Max directories/files

directory_t global_directories[MAX_ENTRIES];
int global_dir_count = 0;
char current_path[MAX_COMMAND_LENGTH]; // Current path

void filesystem_init()
{
    strcpy(current_path, "/");  // Initialize root
    
    if (global_dir_count < MAX_ENTRIES) {
        strcpy(global_directories[global_dir_count].path, "/");
        global_directories[global_dir_count].entry_count = 0;
        global_dir_count++;
    }
    
    // Create default directories
    fs_create_directory("/home");
    fs_create_directory("/usr");
    fs_create_directory("/bin");
}

int fs_init()
{
    global_dir_count = 0;
    filesystem_init();
    return 1;
}

int fs_create_directory(char* path)
{
    // Skip if directory exists
    for (int i = 0; i < global_dir_count; i++) {
        if (strcmp(global_directories[i].path, path) == 0) return 0;
    }
    
    if (global_dir_count < MAX_ENTRIES) {
        strcpy(global_directories[global_dir_count].path, path);
        global_directories[global_dir_count].entry_count = 0;
        global_dir_count++;
        
        // Determine parent directory
        char parent_path[MAX_COMMAND_LENGTH];
        strcpy(parent_path, path);
        int last_slash = -1;
        for (int i = strlen(parent_path) - 1; i >= 0; i--) {
            if (parent_path[i] == '/') { last_slash = i; break; }
        }
        
        if (last_slash > 0) parent_path[last_slash] = '\0';
        else if (last_slash == 0 && strlen(parent_path) > 1) parent_path[1] = '\0';
        else parent_path[0] = '\0';
        
        // Add new directory as parent entry
        if (strlen(parent_path) > 0) {
            for (int i = 0; i < global_dir_count - 1; i++) {
                if (strcmp(global_directories[i].path, parent_path) == 0) {
                    if (global_directories[i].entry_count < 50) {
                        int entry_idx = global_directories[i].entry_count;
                        char* dir_name = strrchr(path, '/');
                        if (dir_name != NULL) {
                            dir_name++;
                            strcpy(global_directories[i].entries[entry_idx].name, dir_name);
                            global_directories[i].entries[entry_idx].is_directory = 1;
                            global_directories[i].entries[entry_idx].size = 0;
                            global_directories[i].entry_count++;
                        }
                    }
                    break;
                }
            }
        }
        return 1;
    }
    
    return 0;
}

int fs_create_file(char* path, char* content)
{
    // feat: file creation not implemented
    return 1;
}

int fs_list_directory(char* path, directory_t* result)
{
    for (int i = 0; i < global_dir_count; i++) {
        if (strcmp(global_directories[i].path, path) == 0) {
            strcpy(result->path, global_directories[i].path);
            result->entry_count = global_directories[i].entry_count;
            for (int j = 0; j < global_directories[i].entry_count; j++) {
                strcpy(result->entries[j].name, global_directories[i].entries[j].name);
                result->entries[j].is_directory = global_directories[i].entries[j].is_directory;
                result->entries[j].size = global_directories[i].entries[j].size;
            }
            return 1;
        }
    }
    
    strcpy(result->path, path);
    result->entry_count = 0;
    return 0;
}

int fs_change_directory(char* path)
{
    for (int i = 0; i < global_dir_count; i++) {
        if (strcmp(global_directories[i].path, path) == 0) {
            strcpy(current_path, path);
            return 1;
        }
    }
    return 0;
}

char* fs_get_current_path()
{
    return current_path;
}

directory_t* get_directory_by_path(char* path)
{
    for (int i = 0; i < global_dir_count; i++) {
        if (strcmp(global_directories[i].path, path) == 0) return &global_directories[i];
    }
    return NULL;
}

int add_entry_to_directory(char* path, char* name, int is_directory, int size)
{
    directory_t* dir = get_directory_by_path(path);
    if (dir != NULL && dir->entry_count < 50) {
        strcpy(dir->entries[dir->entry_count].name, name);
        dir->entries[dir->entry_count].is_directory = is_directory;
        dir->entries[dir->entry_count].size = size;
        dir->entry_count++;
        return 1;
    }
    return 0;
}
