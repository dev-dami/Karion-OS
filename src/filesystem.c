#include "filesystem.h"
#include "source.h"
#include "inode.h"
#include "malloc.h"

// Global file system state
char current_path[MAX_COMMAND_LENGTH];
static unsigned int root_inum = 0;  // Root directory inode number (set during init)

/**
 * Resolve a path to an inode number
 * Handles absolute and relative paths, . and ..
 * 
 * @param path: Path to resolve
 * @return: Inode number, or 0 on error
 */
static unsigned int path_to_inum(char* path)
{
    if (path == NULL || strlen(path) == 0) {
        return 0;
    }

    // Start from root or current directory
    unsigned int current_inum = root_inum;
    if (path[0] != '/') {
        // Relative path - need to get current directory inode
        // For now, we'll use root as current (can be improved)
        current_inum = root_inum;
    }

    // Handle root path
    if (strcmp(path, "/") == 0) {
        return root_inum;
    }

    // Split path into components
    char path_copy[MAX_COMMAND_LENGTH];
    strcpy(path_copy, path);
    
    // Remove leading slash if present
    if (path_copy[0] == '/') {
        current_inum = root_inum;
        // Move pointer past first slash
        char* p = path_copy;
        while (*p == '/') p++;
        if (*p == '\0') return root_inum;
        strcpy(path_copy, p);
    }

    // Tokenize path from left to right
    char* token = path_copy;
    char* next_token;
    
    while (token != NULL && *token != '\0') {
        // Find next slash
        next_token = strchr(token, '/');
        if (next_token) {
            *next_token = '\0';
            next_token++;
        }

        // Skip empty tokens (multiple slashes)
        if (strlen(token) == 0) {
            token = next_token;
            continue;
        }

        // Handle . and ..
        if (strcmp(token, ".") == 0) {
            // Stay in current directory
            token = next_token;
            continue;
        } else if (strcmp(token, "..") == 0) {
            // Go to parent - for now, just go to root
            // In a full implementation, we'd read parent from directory
            current_inum = root_inum;
            token = next_token;
            continue;
        }

        // Look up directory entry
        inode_t dir_ip;
        if (iget(current_inum, &dir_ip) != 0) {
            return 0;
        }

        if (dir_ip.dinode.type != T_DIR) {
            return 0;  // Not a directory
        }

        unsigned int found_inum = dirlookup(&dir_ip, token);
        if (found_inum == 0) {
            return 0;  // Not found
        }

        current_inum = found_inum;
        token = next_token;
    }

    return current_inum;
}

/**
 * Get parent directory inode and filename from path
 * 
 * @param path: Full path
 * @param parent_inum: Output - parent directory inode number
 * @param name: Output - filename/dirname
 * @return: 0 on success, -1 on error
 */
static int split_path(char* path, unsigned int* parent_inum, char* name)
{
    if (path == NULL || parent_inum == NULL || name == NULL) {
        return -1;
    }

    // Find last slash
    char* last_slash = strrchr(path, '/');
    if (last_slash == NULL) {
        // No slash - current directory is parent
        *parent_inum = root_inum;
        strcpy(name, path);
        return 0;
    }

    // Extract name
    strcpy(name, last_slash + 1);
    if (strlen(name) == 0) {
        return -1;
    }

    // Get parent path
    if (last_slash == path) {
        // Path is like "/filename"
        *parent_inum = root_inum;
    } else {
        // Get parent directory path
        char parent_path[MAX_COMMAND_LENGTH];
        int len = last_slash - path;
        strncpy(parent_path, path, len);
        parent_path[len] = '\0';
        *parent_inum = path_to_inum(parent_path);
        if (*parent_inum == 0) {
            return -1;
        }
    }

    return 0;
}

// Initialize the file system
void filesystem_init()
{
    strcpy(current_path, "/");
    
    // Initialize Xv6-style file system
    if (fs_xv6_init() != 0) {
        // Failed to initialize - this is a problem
        return;
    }

    // Root directory is inode 1 (created by fs_xv6_init)
    root_inum = 1;
}

// Initialize file system
int fs_init()
{
    filesystem_init();
    return 1;
}

// Create a directory
int fs_create_directory(char* path)
{
    if (path == NULL || strlen(path) == 0) {
        return 0;
    }

    // Check if already exists
    unsigned int existing = path_to_inum(path);
    if (existing != 0) {
        return 0;  // Already exists
    }

    // Get parent directory and name
    unsigned int parent_inum;
    char name[MAX_ARG_LENGTH];
    if (split_path(path, &parent_inum, name) != 0) {
        return 0;
    }

    // Get parent directory inode
    inode_t parent_ip;
    if (iget(parent_inum, &parent_ip) != 0) {
        return 0;
    }

    if (parent_ip.dinode.type != T_DIR) {
        return 0;  // Parent is not a directory
    }

    // Allocate new directory inode
    unsigned int dir_inum = ialloc(T_DIR);
    if (dir_inum == 0) {
        return 0;  // Out of inodes
    }

    // Get new directory inode
    inode_t dir_ip;
    if (iget(dir_inum, &dir_ip) != 0) {
        ifree(dir_inum);
        return 0;
    }

    // Initialize directory with . and ..
    dirent_t dot_entry, dotdot_entry;
    dot_entry.inum = dir_inum;
    strncpy(dot_entry.name, ".", DIRSIZ);
    dotdot_entry.inum = parent_inum;
    strncpy(dotdot_entry.name, "..", DIRSIZ);

    char dir_data[BLOCK_SIZE];
    for (int i = 0; i < BLOCK_SIZE; i++) {
        dir_data[i] = 0;
    }
    
    dirent_t* entries = (dirent_t*)dir_data;
    entries[0] = dot_entry;
    entries[1] = dotdot_entry;

    if (writei(&dir_ip, dir_data, 0, 2 * sizeof(dirent_t)) != 2 * sizeof(dirent_t)) {
        ifree(dir_inum);
        return 0;
    }

    dir_ip.dinode.nlink = 2;
    if (iput(&dir_ip) != 0) {
        ifree(dir_inum);
        return 0;
    }

    // Link into parent directory
    if (dirlink(&parent_ip, name, dir_inum) != 0) {
        ifree(dir_inum);
        return 0;
    }

    // Update parent link count
    parent_ip.dinode.nlink++;
    if (iput(&parent_ip) != 0) {
        return 0;
    }

    return 1;
}

// Create a file
int fs_create_file(char* path, char* content)
{
    if (path == NULL || strlen(path) == 0) {
        return 0;
    }

    // Check if already exists
    unsigned int existing = path_to_inum(path);
    if (existing != 0) {
        return 0;  // Already exists
    }

    // Get parent directory and name
    unsigned int parent_inum;
    char name[MAX_ARG_LENGTH];
    if (split_path(path, &parent_inum, name) != 0) {
        return 0;
    }

    // Get parent directory inode
    inode_t parent_ip;
    if (iget(parent_inum, &parent_ip) != 0) {
        return 0;
    }

    if (parent_ip.dinode.type != T_DIR) {
        return 0;  // Parent is not a directory
    }

    // Allocate new file inode
    unsigned int file_inum = ialloc(T_FILE);
    if (file_inum == 0) {
        return 0;  // Out of inodes
    }

    // Get new file inode
    inode_t file_ip;
    if (iget(file_inum, &file_ip) != 0) {
        ifree(file_inum);
        return 0;
    }

    // Write content if provided
    if (content != NULL && strlen(content) > 0) {
        int written = writei(&file_ip, content, 0, strlen(content));
        if (written < 0) {
            ifree(file_inum);
            return 0;
        }
    }

    if (iput(&file_ip) != 0) {
        ifree(file_inum);
        return 0;
    }

    // Link into parent directory
    if (dirlink(&parent_ip, name, file_inum) != 0) {
        ifree(file_inum);
        return 0;
    }

    if (iput(&parent_ip) != 0) {
        return 0;
    }

    return 1;
}

// Delete a directory
int fs_delete_directory(char* path)
{
    if (path == NULL || strlen(path) == 0) {
        return 0;
    }

    // Can't delete root
    if (strcmp(path, "/") == 0) {
        return 0;
    }

    unsigned int dir_inum = path_to_inum(path);
    if (dir_inum == 0) {
        return 0;  // Not found
    }

    // Get directory inode
    inode_t dir_ip;
    if (iget(dir_inum, &dir_ip) != 0) {
        return 0;
    }

    if (dir_ip.dinode.type != T_DIR) {
        return 0;  // Not a directory
    }

    // Check if directory is empty (only . and .. should be present)
    char dir_data[BLOCK_SIZE * 12];
    int bytes_read = readi(&dir_ip, dir_data, 0, dir_ip.dinode.size);
    if (bytes_read < 0) {
        return 0;
    }

    dirent_t* entries = (dirent_t*)dir_data;
    int num_entries = bytes_read / sizeof(dirent_t);
    
    // Should only have . and ..
    if (num_entries > 2) {
        return 0;  // Directory not empty
    }

    // Get parent directory
    unsigned int parent_inum;
    char name[MAX_ARG_LENGTH];
    if (split_path(path, &parent_inum, name) != 0) {
        return 0;
    }

    // Remove from parent (simplified - would need to read and rewrite parent dir)
    // For now, just free the inode
    ifree(dir_inum);

    return 1;
}

// Delete a file
int fs_delete_file(char* path)
{
    if (path == NULL || strlen(path) == 0) {
        return 0;
    }

    unsigned int file_inum = path_to_inum(path);
    if (file_inum == 0) {
        return 0;  // Not found
    }

    // Get file inode
    inode_t file_ip;
    if (iget(file_inum, &file_ip) != 0) {
        return 0;
    }

    if (file_ip.dinode.type != T_FILE) {
        return 0;  // Not a file
    }

    // Free all data blocks
    for (int i = 0; i < 12; i++) {
        if (file_ip.dinode.addrs[i] != 0) {
            bfree(file_ip.dinode.addrs[i]);
        }
    }

    // Free the inode
    ifree(file_inum);

    return 1;
}

// Write content to a file
int fs_write_file(char* path, char* content)
{
    if (path == NULL || content == NULL) {
        return 0;
    }

    unsigned int file_inum = path_to_inum(path);
    if (file_inum == 0) {
        return 0;  // File not found
    }

    // Get file inode
    inode_t file_ip;
    if (iget(file_inum, &file_ip) != 0) {
        return 0;
    }

    if (file_ip.dinode.type != T_FILE) {
        return 0;  // Not a file
    }

    // Truncate file before writing (free blocks beyond new content size)
    unsigned int new_size = strlen(content);
    unsigned int old_size = file_ip.dinode.size;
    
    // Free blocks that won't be needed anymore
    if (new_size < old_size) {
        // Calculate which blocks to free
        unsigned int old_blocks = (old_size + BLOCK_SIZE - 1) / BLOCK_SIZE;
        unsigned int new_blocks = (new_size + BLOCK_SIZE - 1) / BLOCK_SIZE;
        
        // Free blocks beyond the new size
        for (unsigned int i = new_blocks; i < old_blocks && i < 12; i++) {
            if (file_ip.dinode.addrs[i] != 0) {
                bfree(file_ip.dinode.addrs[i]);
                file_ip.dinode.addrs[i] = 0;
            }
        }
    }
    
    // Truncate file size to 0 before writing (ensures clean overwrite)
    file_ip.dinode.size = 0;

    // Write content (overwrite existing)
    int written = writei(&file_ip, content, 0, strlen(content));
    if (written < 0) {
        return 0;
    }

    if (iput(&file_ip) != 0) {
        return 0;
    }

    return 1;
}

// Read content from a file
char* fs_read_file(char* path)
{
    if (path == NULL) {
        return NULL;
    }

    unsigned int file_inum = path_to_inum(path);
    if (file_inum == 0) {
        return NULL;  // File not found
    }

    // Get file inode
    inode_t file_ip;
    if (iget(file_inum, &file_ip) != 0) {
        return NULL;
    }

    if (file_ip.dinode.type != T_FILE) {
        return NULL;  // Not a file
    }

    // Allocate buffer for file content
    unsigned int file_size = file_ip.dinode.size;
    if (file_size == 0) {
        return NULL;
    }

    char* buffer = (char*)kmalloc(file_size + 1);
    if (buffer == NULL) {
        return NULL;  // Out of memory
    }

    // Read file content
    int bytes_read = readi(&file_ip, buffer, 0, file_size);
    if (bytes_read < 0) {
        kfree(buffer);
        return NULL;
    }

    buffer[bytes_read] = '\0';  // Null terminate
    return buffer;
}

// List directory contents
int fs_list_directory(char* path, directory_t* result)
{
    if (path == NULL || result == NULL) {
        return 0;
    }

    unsigned int dir_inum = path_to_inum(path);
    if (dir_inum == 0) {
        return 0;  // Directory not found
    }

    // Get directory inode
    inode_t dir_ip;
    if (iget(dir_inum, &dir_ip) != 0) {
        return 0;
    }

    if (dir_ip.dinode.type != T_DIR) {
        return 0;  // Not a directory
    }

    // Read directory entries
    char dir_data[BLOCK_SIZE * 12];
    int bytes_read = readi(&dir_ip, dir_data, 0, dir_ip.dinode.size);
    if (bytes_read < 0) {
        return 0;
    }

    // Copy path
    strcpy(result->path, path);
    result->entry_count = 0;

    // Parse directory entries
    dirent_t* entries = (dirent_t*)dir_data;
    int num_entries = bytes_read / sizeof(dirent_t);

    for (int i = 0; i < num_entries && result->entry_count < 50; i++) {
        // Skip . and ..
        if (strcmp(entries[i].name, ".") == 0 || strcmp(entries[i].name, "..") == 0) {
            continue;
        }

        // Get entry inode to determine type
        inode_t entry_ip;
        if (iget(entries[i].inum, &entry_ip) == 0) {
            strcpy(result->entries[result->entry_count].name, entries[i].name);
            result->entries[result->entry_count].is_directory = (entry_ip.dinode.type == T_DIR);
            result->entries[result->entry_count].size = entry_ip.dinode.size;
            result->entry_count++;
        }
    }

    return 1;
}

// Change directory
int fs_change_directory(char* path)
{
    if (path == NULL) {
        return 0;
    }

    unsigned int dir_inum = path_to_inum(path);
    if (dir_inum == 0) {
        return 0;  // Directory not found
    }

    // Get directory inode
    inode_t dir_ip;
    if (iget(dir_inum, &dir_ip) != 0) {
        return 0;
    }

    if (dir_ip.dinode.type != T_DIR) {
        return 0;  // Not a directory
    }

    // Update current path
    strcpy(current_path, path);
    return 1;
}

// Get current path
char* fs_get_current_path()
{
    return current_path;
}

// Path utilities (kept for compatibility)
int parse_path(char* full_path, char* parent_path, char* name)
{
    return split_path(full_path, NULL, name) == 0 ? 1 : 0;
}

int get_entry_by_path(char* path, fs_entry_t* entry)
{
    // This function is not used with inode-based FS
    // Kept for compatibility
    return 0;
}

int get_parent_directory_index(char* path)
{
    // This function is not used with inode-based FS
    // Kept for compatibility
    return -1;
}

// Format the file system
int fs_format()
{
    // Reinitialize file system
    filesystem_init();
    return 1;
}

// Save to memory
int fs_save_to_memory()
{
    // With inode-based FS, data is already in RAM disk
    return 1;
}

// Load from memory
int fs_load_from_memory()
{
    // Reinitialize
    filesystem_init();
    return 1;
}
