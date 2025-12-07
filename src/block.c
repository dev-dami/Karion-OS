#include "block.h"
#include "source.h"

// Global block device instance
static block_device_t g_block_device;

/**
 * Initialize block device
 * Sets up the underlying storage (currently RAM disk)
 * 
 * @return: 0 on success, -1 on error
 */
int block_device_init(void)
{
    if (g_block_device.initialized) {
        return 0;  // Already initialized
    }

    // Initialize RAM disk
    if (ramdisk_init() != 0) {
        return -1;
    }

    // Set up block device structure
    g_block_device.type = BLOCK_DEVICE_RAMDISK;
    g_block_device.block_size = BLOCK_SIZE;
    g_block_device.initialized = 1;

    // Get disk information
    ramdisk_get_info(NULL, &g_block_device.total_blocks);

    return 0;
}

/**
 * Read a block from the block device
 * 
 * @param block_num: Block number to read
 * @param buffer: Buffer to store data (must be at least block_size bytes)
 * @return: 0 on success, -1 on error
 */
int block_read(unsigned int block_num, unsigned char* buffer)
{
    if (!g_block_device.initialized) {
        return -1;
    }

    switch (g_block_device.type) {
        case BLOCK_DEVICE_RAMDISK:
            return ramdisk_read_block(block_num, buffer);
        default:
            return -1;
    }
}

/**
 * Write a block to the block device
 * 
 * @param block_num: Block number to write
 * @param buffer: Data to write (must be block_size bytes)
 * @return: 0 on success, -1 on error
 */
int block_write(unsigned int block_num, unsigned char* buffer)
{
    if (!g_block_device.initialized) {
        return -1;
    }

    switch (g_block_device.type) {
        case BLOCK_DEVICE_RAMDISK:
            return ramdisk_write_block(block_num, buffer);
        default:
            return -1;
    }
}

/**
 * Read multiple blocks from the block device
 * 
 * @param start_block: Starting block number
 * @param count: Number of blocks to read
 * @param buffer: Buffer to store data
 * @return: 0 on success, -1 on error
 */
int block_read_multiple(unsigned int start_block, unsigned int count, unsigned char* buffer)
{
    if (!g_block_device.initialized) {
        return -1;
    }

    switch (g_block_device.type) {
        case BLOCK_DEVICE_RAMDISK:
            return ramdisk_read_blocks(start_block, count, buffer);
        default:
            return -1;
    }
}

/**
 * Write multiple blocks to the block device
 * 
 * @param start_block: Starting block number
 * @param count: Number of blocks to write
 * @param buffer: Data to write
 * @return: 0 on success, -1 on error
 */
int block_write_multiple(unsigned int start_block, unsigned int count, unsigned char* buffer)
{
    if (!g_block_device.initialized) {
        return -1;
    }

    switch (g_block_device.type) {
        case BLOCK_DEVICE_RAMDISK:
            return ramdisk_write_blocks(start_block, count, buffer);
        default:
            return -1;
    }
}

/**
 * Get block device information
 * 
 * @param block_size: Pointer to store block size (can be NULL)
 * @param total_blocks: Pointer to store total blocks (can be NULL)
 */
void block_get_info(unsigned int* block_size, unsigned int* total_blocks)
{
    if (block_size != NULL) {
        *block_size = g_block_device.block_size;
    }
    if (total_blocks != NULL) {
        *total_blocks = g_block_device.total_blocks;
    }
}

