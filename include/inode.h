#ifndef INODE_DOT_H
#define INODE_DOT_H

#include "block.h"

// Inode-based file system (simplified Xv6-style)
// Inodes are the fundamental structure representing files and directories

// File types (similar to Xv6)
#define T_DIR  1   // Directory
#define T_FILE 2   // Regular file
#define T_DEV  3   // Device file

// Inode structure (on-disk format)
// This is what gets stored on the disk
typedef struct {
    unsigned short type;      // File type (T_DIR, T_FILE, T_DEV)
    unsigned short major;     // Major device number (for T_DEV)
    unsigned short minor;     // Minor device number (for T_DEV)
    unsigned short nlink;     // Number of links to this inode
    unsigned int size;        // Size of file in bytes
    unsigned int addrs[12];   // Data block addresses (direct blocks)
    // Note: Xv6 uses NDIRECT+1, we use 12 direct blocks for simplicity
} dinode_t;

// In-memory inode structure
// This is what we use while the inode is in use
typedef struct {
    unsigned int inum;        // Inode number
    int ref;                  // Reference count
    int valid;                // Is inode valid/loaded?
    dinode_t dinode;          // On-disk inode data
} inode_t;

// File system layout (simplified)
#define FS_MAGIC       0x12345678  // Magic number to identify file system
#define BLOCK_SIZE     512         // Block size in bytes
#define INODE_SIZE     sizeof(dinode_t)  // Size of one inode
#define INODES_PER_BLOCK (BLOCK_SIZE / INODE_SIZE)  // How many inodes per block

// Superblock structure (stored at block 0)
typedef struct {
    unsigned int magic;       // Magic number
    unsigned int size;        // Total size of file system in blocks
    unsigned int nblocks;     // Number of data blocks
    unsigned int ninodes;     // Number of inodes
    unsigned int inode_start; // Starting block of inode table
    unsigned int bitmap_start;// Starting block of free block bitmap
    unsigned int data_start;  // Starting block of data area
} superblock_t;

// Directory entry structure
// Directories are files containing directory entries
#define DIRSIZ 14  // Maximum filename length
typedef struct {
    unsigned short inum;      // Inode number
    char name[DIRSIZ];        // Filename
} dirent_t;

// File system functions

// Initialize the file system
// Creates superblock and initializes structures
//
// @return: 0 on success, -1 on error
int fs_xv6_init(void);

// Allocate a new inode
// Finds a free inode and marks it as used
//
// @param type: Type of inode (T_DIR, T_FILE, T_DEV)
// @return: Inode number on success, 0 on error
unsigned int ialloc(unsigned short type);

// Free an inode
// Marks the inode as free
//
// @param inum: Inode number to free
void ifree(unsigned int inum);

// Read inode from disk
// Loads inode data from disk into memory
//
// @param inum: Inode number
// @param ip: Pointer to inode structure to fill
// @return: 0 on success, -1 on error
int iget(unsigned int inum, inode_t* ip);

// Write inode to disk
// Saves inode data from memory to disk
//
// @param ip: Pointer to inode structure to write
// @return: 0 on success, -1 on error
int iput(inode_t* ip);

// Allocate a data block
// Finds a free block and marks it as used
//
// @return: Block number on success, 0 on error
unsigned int balloc(void);

// Free a data block
// Marks the block as free in the bitmap
//
// @param block_num: Block number to free
void bfree(unsigned int block_num);

// Map logical block number to physical block number
// Handles direct blocks (simplified - no indirect blocks)
//
// @param ip: Pointer to inode
// @param bn: Logical block number within file
// @return: Physical block number, or 0 on error
unsigned int bmap(inode_t* ip, unsigned int bn);

// Read data from inode
// Reads bytes from a file starting at offset
//
// @param ip: Pointer to inode
// @param dst: Destination buffer
// @param offset: Byte offset in file
// @param n: Number of bytes to read
// @return: Number of bytes read, or -1 on error
int readi(inode_t* ip, char* dst, unsigned int offset, unsigned int n);

// Write data to inode
// Writes bytes to a file starting at offset
//
// @param ip: Pointer to inode
// @param src: Source buffer
// @param offset: Byte offset in file
// @param n: Number of bytes to write
// @return: Number of bytes written, or -1 on error
int writei(inode_t* ip, char* src, unsigned int offset, unsigned int n);

// Look up directory entry
// Finds a directory entry by name
//
// @param dp: Pointer to directory inode
// @param name: Filename to find
// @return: Inode number if found, 0 if not found
unsigned int dirlookup(inode_t* dp, char* name);

// Add directory entry
// Adds a new entry to a directory
//
// @param dp: Pointer to directory inode
// @param name: Filename
// @param inum: Inode number to link
// @return: 0 on success, -1 on error
int dirlink(inode_t* dp, char* name, unsigned int inum);

// Get superblock
// Reads the superblock from disk
//
// @param sb: Pointer to superblock structure to fill
// @return: 0 on success, -1 on error
int get_superblock(superblock_t* sb);

#endif /* INODE_DOT_H */

