#include "ramdisk.h"
#include "source.h"

// Global RAM disk instance
static ramdisk_t g_ramdisk;

/**
 * Initialize RAM disk
 * Allocates memory for the disk and initializes the structure
 * 
 * @return: 0 on success, -1 on failure
 */
int ramdisk_init(void)
{
    if (g_ramdisk.initialized) {
        return 0;  // Already initialized
    }

    // Allocate memory for disk
    g_ramdisk.data = (unsigned char*)kmalloc(RAMDISK_SIZE);
    if (g_ramdisk.data == NULL) {
        return -1;  // Out of memory
    }

    // Initialize disk structure
    g_ramdisk.size = RAMDISK_SIZE;
    g_ramdisk.block_count = RAMDISK_BLOCKS;
    g_ramdisk.initialized = 1;

    // Zero out the disk (simulate fresh disk)
    for (unsigned int i = 0; i < RAMDISK_SIZE; i++) {
        g_ramdisk.data[i] = 0;
    }

    return 0;
}

/**
 * Read a single block from RAM disk
 * 
 * @param block_num: Block number (0-indexed)
 * @param buffer: Buffer to store data (must be at least BLOCK_SIZE bytes)
 * @return: 0 on success, -1 on error
 */
int ramdisk_read_block(unsigned int block_num, unsigned char* buffer)
{
    if (!g_ramdisk.initialized) {
        return -1;
    }

    if (block_num >= g_ramdisk.block_count) {
        return -1;  // Block out of range
    }

    if (buffer == NULL) {
        return -1;  // Invalid buffer
    }

    // Calculate offset in disk
    unsigned int offset = block_num * BLOCK_SIZE;

    // Copy block data to buffer
    for (unsigned int i = 0; i < BLOCK_SIZE; i++) {
        buffer[i] = g_ramdisk.data[offset + i];
    }

    return 0;
}

/**
 * Write a single block to RAM disk
 * 
 * @param block_num: Block number (0-indexed)
 * @param buffer: Data to write (must be BLOCK_SIZE bytes)
 * @return: 0 on success, -1 on error
 */
int ramdisk_write_block(unsigned int block_num, unsigned char* buffer)
{
    if (!g_ramdisk.initialized) {
        return -1;
    }

    if (block_num >= g_ramdisk.block_count) {
        return -1;  // Block out of range
    }

    if (buffer == NULL) {
        return -1;  // Invalid buffer
    }

    // Calculate offset in disk
    unsigned int offset = block_num * BLOCK_SIZE;

    // Copy buffer data to disk
    for (unsigned int i = 0; i < BLOCK_SIZE; i++) {
        g_ramdisk.data[offset + i] = buffer[i];
    }

    return 0;
}

/**
 * Read multiple blocks from RAM disk
 * 
 * @param start_block: Starting block number
 * @param count: Number of blocks to read
 * @param buffer: Buffer to store data (must be count * BLOCK_SIZE bytes)
 * @return: 0 on success, -1 on error
 */
int ramdisk_read_blocks(unsigned int start_block, unsigned int count, unsigned char* buffer)
{
    if (!g_ramdisk.initialized || buffer == NULL) {
        return -1;
    }

    if (start_block + count > g_ramdisk.block_count) {
        return -1;  // Blocks out of range
    }

    // Read each block
    for (unsigned int i = 0; i < count; i++) {
        if (ramdisk_read_block(start_block + i, buffer + (i * BLOCK_SIZE)) != 0) {
            return -1;
        }
    }

    return 0;
}

/**
 * Write multiple blocks to RAM disk
 * 
 * @param start_block: Starting block number
 * @param count: Number of blocks to write
 * @param buffer: Data to write (must be count * BLOCK_SIZE bytes)
 * @return: 0 on success, -1 on error
 */
int ramdisk_write_blocks(unsigned int start_block, unsigned int count, unsigned char* buffer)
{
    if (!g_ramdisk.initialized || buffer == NULL) {
        return -1;
    }

    if (start_block + count > g_ramdisk.block_count) {
        return -1;  // Blocks out of range
    }

    // Write each block
    for (unsigned int i = 0; i < count; i++) {
        if (ramdisk_write_block(start_block + i, buffer + (i * BLOCK_SIZE)) != 0) {
            return -1;
        }
    }

    return 0;
}

/**
 * Get RAM disk information
 * 
 * @param size: Pointer to store total size (can be NULL)
 * @param blocks: Pointer to store block count (can be NULL)
 */
void ramdisk_get_info(unsigned int* size, unsigned int* blocks)
{
    if (size != NULL) {
        *size = g_ramdisk.size;
    }
    if (blocks != NULL) {
        *blocks = g_ramdisk.block_count;
    }
}

