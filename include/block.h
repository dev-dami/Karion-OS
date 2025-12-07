#ifndef BLOCK_DOT_H
#define BLOCK_DOT_H

#include "ramdisk.h"

// Block device abstraction layer
// Provides a unified interface for block-based storage

#define BLOCK_DEVICE_RAMDISK 0

// Block device structure
typedef struct {
    int type;              // Device type (RAMDISK, etc.)
    int initialized;       // Initialization flag
    unsigned int block_size;  // Size of each block
    unsigned int total_blocks; // Total number of blocks
} block_device_t;

// Initialize block device
// Sets up the underlying storage (RAM disk)
//
// @return: 0 on success, -1 on error
int block_device_init(void);

// Read a block from the block device
//
// @param block_num: Block number to read
// @param buffer: Buffer to store data (must be at least block_size bytes)
// @return: 0 on success, -1 on error
int block_read(unsigned int block_num, unsigned char* buffer);

// Write a block to the block device
//
// @param block_num: Block number to write
// @param buffer: Data to write (must be block_size bytes)
// @return: 0 on success, -1 on error
int block_write(unsigned int block_num, unsigned char* buffer);

// Read multiple blocks
//
// @param start_block: Starting block number
// @param count: Number of blocks to read
// @param buffer: Buffer to store data
// @return: 0 on success, -1 on error
int block_read_multiple(unsigned int start_block, unsigned int count, unsigned char* buffer);

// Write multiple blocks
//
// @param start_block: Starting block number
// @param count: Number of blocks to write
// @param buffer: Data to write
// @return: 0 on success, -1 on error
int block_write_multiple(unsigned int start_block, unsigned int count, unsigned char* buffer);

// Get block device information
//
// @param block_size: Pointer to store block size (can be NULL)
// @param total_blocks: Pointer to store total blocks (can be NULL)
void block_get_info(unsigned int* block_size, unsigned int* total_blocks);

#endif /* BLOCK_DOT_H */

