#ifndef RAMDISK_DOT_H
#define RAMDISK_DOT_H

#include "malloc.h"

// RAM Disk Configuration
// Simulates a disk drive using memory
#define RAMDISK_SIZE        (1024 * 1024)  // 1MB disk
#define BLOCK_SIZE          512            // Standard disk sector size
#define RAMDISK_BLOCKS      (RAMDISK_SIZE / BLOCK_SIZE)  // 2048 blocks

// RAM Disk structure
typedef struct {
    unsigned char* data;      // Memory backing the disk
    unsigned int size;        // Total size in bytes
    unsigned int block_count; // Number of blocks
    int initialized;          // Initialization flag
} ramdisk_t;

// Initialize RAM disk
// Allocates memory and sets up the disk structure
int ramdisk_init(void);

// Read a block from RAM disk
// 
// @param block_num: Block number to read (0-indexed)
// @param buffer: Buffer to store read data (must be at least BLOCK_SIZE bytes)
// @return: 0 on success, -1 on error
int ramdisk_read_block(unsigned int block_num, unsigned char* buffer);

// Write a block to RAM disk
//
// @param block_num: Block number to write (0-indexed)
// @param buffer: Data to write (must be BLOCK_SIZE bytes)
// @return: 0 on success, -1 on error
int ramdisk_write_block(unsigned int block_num, unsigned char* buffer);

// Read multiple blocks
//
// @param start_block: Starting block number
// @param count: Number of blocks to read
// @param buffer: Buffer to store data (must be count * BLOCK_SIZE bytes)
// @return: 0 on success, -1 on error
int ramdisk_read_blocks(unsigned int start_block, unsigned int count, unsigned char* buffer);

// Write multiple blocks
//
// @param start_block: Starting block number
// @param count: Number of blocks to write
// @param buffer: Data to write (must be count * BLOCK_SIZE bytes)
// @return: 0 on success, -1 on error
int ramdisk_write_blocks(unsigned int start_block, unsigned int count, unsigned char* buffer);

// Get RAM disk information
//
// @param size: Pointer to store total size (can be NULL)
// @param blocks: Pointer to store block count (can be NULL)
void ramdisk_get_info(unsigned int* size, unsigned int* blocks);

#endif /* RAMDISK_DOT_H */

