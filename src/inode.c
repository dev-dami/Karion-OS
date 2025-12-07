#include "inode.h"
#include "source.h"
#include "buffer.h"
#include "block.h"

// File system layout constants
#define SUPERBLOCK_BLOCK    0   // Superblock is at block 0
#define BITMAP_BLOCK        1   // Bitmap starts at block 1
#define INODE_TABLE_BLOCK   2   // Inode table starts at block 2
#define DATA_BLOCK_START    10  // Data blocks start here (after superblock, bitmap, inodes)

// Number of blocks for bitmap (1 bit per data block)
#define BITMAP_BLOCKS       1   // Simplified: 1 block = 4096 data blocks max

// Number of inodes (simplified)
#define NINODES             64  // 64 inodes should be enough for learning

// Global superblock (cached in memory)
static superblock_t g_superblock;
static int superblock_loaded = 0;

/**
 * Read superblock from disk
 * 
 * @param sb: Pointer to superblock structure to fill
 * @return: 0 on success, -1 on error
 */
int get_superblock(superblock_t* sb)
{
    if (sb == NULL) return -1;

    // Read superblock from block 0
    unsigned char block[BLOCK_SIZE];
    if (block_read(SUPERBLOCK_BLOCK, block) != 0) {
        return -1;
    }

    // Copy superblock data
    superblock_t* disk_sb = (superblock_t*)block;
    *sb = *disk_sb;

    // Cache it
    g_superblock = *sb;
    superblock_loaded = 1;

    return 0;
}

/**
 * Write superblock to disk
 * 
 * @param sb: Pointer to superblock structure to write
 * @return: 0 on success, -1 on error
 */
static int put_superblock(superblock_t* sb)
{
    if (sb == NULL) return -1;

    unsigned char block[BLOCK_SIZE];
    
    // Zero out block first
    for (int i = 0; i < BLOCK_SIZE; i++) {
        block[i] = 0;
    }

    // Copy superblock data
    superblock_t* disk_sb = (superblock_t*)block;
    *disk_sb = *sb;

    // Write to block 0
    if (block_write(SUPERBLOCK_BLOCK, block) != 0) {
        return -1;
    }

    g_superblock = *sb;
    return 0;
}

/**
 * Initialize the file system
 * Creates superblock and initializes structures
 * 
 * @return: 0 on success, -1 on error
 */
int fs_xv6_init(void)
{
    // Initialize block device
    if (block_device_init() != 0) {
        return -1;
    }

    // Check if file system already exists
    superblock_t sb;
    if (get_superblock(&sb) == 0 && sb.magic == FS_MAGIC) {
        // File system already exists
        return 0;
    }

    // Initialize buffer cache
    buffer_init();

    // Get block device info
    unsigned int total_blocks;
    block_get_info(NULL, &total_blocks);

    // Create new superblock
    superblock_t new_sb;
    new_sb.magic = FS_MAGIC;
    new_sb.size = total_blocks;
    new_sb.nblocks = total_blocks - DATA_BLOCK_START;  // Data blocks available
    new_sb.ninodes = NINODES;
    new_sb.inode_start = INODE_TABLE_BLOCK;
    new_sb.bitmap_start = BITMAP_BLOCK;
    new_sb.data_start = DATA_BLOCK_START;

    // Write superblock
    if (put_superblock(&new_sb) != 0) {
        return -1;
    }

    // Initialize bitmap (all blocks free initially)
    unsigned char bitmap_block[BLOCK_SIZE];
    for (int i = 0; i < BLOCK_SIZE; i++) {
        bitmap_block[i] = 0;  // All blocks free
    }
    if (block_write(BITMAP_BLOCK, bitmap_block) != 0) {
        return -1;
    }

    // Initialize inode table (all inodes free)
    unsigned char inode_block[BLOCK_SIZE];
    for (int i = 0; i < BLOCK_SIZE; i++) {
        inode_block[i] = 0;
    }
    
    // Write inode blocks (calculate how many we need)
    int inode_blocks = (NINODES * INODE_SIZE + BLOCK_SIZE - 1) / BLOCK_SIZE;
    for (int i = 0; i < inode_blocks; i++) {
        if (block_write(INODE_TABLE_BLOCK + i, inode_block) != 0) {
            return -1;
        }
    }

    // Allocate root directory inode
    unsigned int root_inum = ialloc(T_DIR);
    if (root_inum == 0) {
        return -1;
    }

    // Initialize root directory
    inode_t root_ip;
    if (iget(root_inum, &root_ip) != 0) {
        return -1;
    }

    // Add . and .. entries to root
    dirent_t dot_entry;
    dot_entry.inum = root_inum;
    strncpy(dot_entry.name, ".", DIRSIZ);
    
    dirent_t dotdot_entry;
    dotdot_entry.inum = root_inum;
    strncpy(dotdot_entry.name, "..", DIRSIZ);

    // Write directory entries
    char dir_data[BLOCK_SIZE];
    for (int i = 0; i < BLOCK_SIZE; i++) {
        dir_data[i] = 0;
    }
    
    dirent_t* entries = (dirent_t*)dir_data;
    entries[0] = dot_entry;
    entries[1] = dotdot_entry;

    if (writei(&root_ip, dir_data, 0, 2 * sizeof(dirent_t)) != 2 * sizeof(dirent_t)) {
        return -1;
    }

    root_ip.dinode.nlink = 2;  // . and .. links
    if (iput(&root_ip) != 0) {
        return -1;
    }

    return 0;
}

/**
 * Allocate a new inode
 * 
 * @param type: Type of inode (T_DIR, T_FILE, T_DEV)
 * @return: Inode number on success, 0 on error
 */
unsigned int ialloc(unsigned short type)
{
    if (!superblock_loaded) {
        if (get_superblock(&g_superblock) != 0) {
            return 0;
        }
    }

    // Read inode blocks and find free inode
    int inode_blocks = (g_superblock.ninodes * INODE_SIZE + BLOCK_SIZE - 1) / BLOCK_SIZE;
    
    for (int block = 0; block < inode_blocks; block++) {
        unsigned char inode_block[BLOCK_SIZE];
        if (block_read(g_superblock.inode_start + block, inode_block) != 0) {
            continue;
        }

        dinode_t* inodes = (dinode_t*)inode_block;
        int inodes_in_block = BLOCK_SIZE / INODE_SIZE;

        for (int i = 0; i < inodes_in_block; i++) {
            unsigned int inum = block * inodes_in_block + i + 1;  // Inodes start at 1
            
            if (inum > g_superblock.ninodes) {
                break;
            }

            // Check if inode is free (type == 0 means free)
            if (inodes[i].type == 0) {
                // Allocate this inode
                inodes[i].type = type;
                inodes[i].major = 0;
                inodes[i].minor = 0;
                inodes[i].nlink = 0;
                inodes[i].size = 0;
                for (int j = 0; j < 12; j++) {
                    inodes[i].addrs[j] = 0;
                }

                // Write back to disk
                if (block_write(g_superblock.inode_start + block, inode_block) != 0) {
                    return 0;
                }

                return inum;
            }
        }
    }

    return 0;  // No free inode found
}

/**
 * Free an inode
 * 
 * @param inum: Inode number to free
 */
void ifree(unsigned int inum)
{
    if (inum == 0 || inum > g_superblock.ninodes) {
        return;
    }

    // Calculate which block contains this inode
    int block_num = (inum - 1) / INODES_PER_BLOCK;
    int inode_offset = (inum - 1) % INODES_PER_BLOCK;

    unsigned char inode_block[BLOCK_SIZE];
    if (block_read(g_superblock.inode_start + block_num, inode_block) != 0) {
        return;
    }

    dinode_t* inodes = (dinode_t*)inode_block;
    inodes[inode_offset].type = 0;  // Mark as free

    block_write(g_superblock.inode_start + block_num, inode_block);
}

/**
 * Read inode from disk
 * 
 * @param inum: Inode number
 * @param ip: Pointer to inode structure to fill
 * @return: 0 on success, -1 on error
 */
int iget(unsigned int inum, inode_t* ip)
{
    if (ip == NULL || inum == 0) {
        return -1;
    }

    // Load superblock first if not already loaded
    // This must happen before validation that uses g_superblock.ninodes
    if (!superblock_loaded) {
        if (get_superblock(&g_superblock) != 0) {
            return -1;
        }
    }

    // Now validate inode number against loaded superblock
    if (inum > g_superblock.ninodes) {
        return -1;
    }

    // Calculate which block contains this inode
    int block_num = (inum - 1) / INODES_PER_BLOCK;
    int inode_offset = (inum - 1) % INODES_PER_BLOCK;

    unsigned char inode_block[BLOCK_SIZE];
    if (block_read(g_superblock.inode_start + block_num, inode_block) != 0) {
        return -1;
    }

    dinode_t* inodes = (dinode_t*)inode_block;
    ip->inum = inum;
    ip->ref = 1;
    ip->valid = 1;
    ip->dinode = inodes[inode_offset];

    return 0;
}

/**
 * Write inode to disk
 * 
 * @param ip: Pointer to inode structure to write
 * @return: 0 on success, -1 on error
 */
int iput(inode_t* ip)
{
    if (ip == NULL || !ip->valid) {
        return -1;
    }

    if (!superblock_loaded) {
        if (get_superblock(&g_superblock) != 0) {
            return -1;
        }
    }

    // Calculate which block contains this inode
    int block_num = (ip->inum - 1) / INODES_PER_BLOCK;
    int inode_offset = (ip->inum - 1) % INODES_PER_BLOCK;

    unsigned char inode_block[BLOCK_SIZE];
    if (block_read(g_superblock.inode_start + block_num, inode_block) != 0) {
        return -1;
    }

    dinode_t* inodes = (dinode_t*)inode_block;
    inodes[inode_offset] = ip->dinode;

    if (block_write(g_superblock.inode_start + block_num, inode_block) != 0) {
        return -1;
    }

    return 0;
}

/**
 * Allocate a data block
 * 
 * @return: Block number on success, 0 on error
 */
unsigned int balloc(void)
{
    if (!superblock_loaded) {
        if (get_superblock(&g_superblock) != 0) {
            return 0;
        }
    }

    // Read bitmap
    unsigned char bitmap[BLOCK_SIZE];
    if (block_read(g_superblock.bitmap_start, bitmap) != 0) {
        return 0;
    }

    // Find first free block (bit == 0)
    for (unsigned int i = 0; i < g_superblock.nblocks; i++) {
        int byte = i / 8;
        int bit = i % 8;

        if (byte >= BLOCK_SIZE) {
            break;  // Out of bitmap
        }

        if ((bitmap[byte] & (1 << bit)) == 0) {
            // Found free block - mark it as used
            bitmap[byte] |= (1 << bit);
            if (block_write(g_superblock.bitmap_start, bitmap) != 0) {
                return 0;
            }

            return g_superblock.data_start + i;
        }
    }

    return 0;  // No free blocks
}

/**
 * Free a data block
 * 
 * @param block_num: Block number to free
 */
void bfree(unsigned int block_num)
{
    if (block_num < g_superblock.data_start) {
        return;  // Can't free system blocks
    }

    unsigned int block_index = block_num - g_superblock.data_start;
    if (block_index >= g_superblock.nblocks) {
        return;  // Invalid block
    }

    // Read bitmap
    unsigned char bitmap[BLOCK_SIZE];
    if (block_read(g_superblock.bitmap_start, bitmap) != 0) {
        return;
    }

    // Clear bit
    int byte = block_index / 8;
    int bit = block_index % 8;
    bitmap[byte] &= ~(1 << bit);

    block_write(g_superblock.bitmap_start, bitmap);
}

/**
 * Map logical block number to physical block number
 * 
 * @param ip: Pointer to inode
 * @param bn: Logical block number within file
 * @return: Physical block number, or 0 on error
 */
unsigned int bmap(inode_t* ip, unsigned int bn)
{
    if (ip == NULL || !ip->valid) {
        return 0;
    }

    // Simplified: only direct blocks (no indirect blocks)
    if (bn >= 12) {
        return 0;  // Block number too large
    }

    // If block not allocated, allocate it
    if (ip->dinode.addrs[bn] == 0) {
        unsigned int block = balloc();
        if (block == 0) {
            return 0;  // Out of blocks
        }
        ip->dinode.addrs[bn] = block;
    }

    return ip->dinode.addrs[bn];
}

/**
 * Read data from inode
 * 
 * @param ip: Pointer to inode
 * @param dst: Destination buffer
 * @param offset: Byte offset in file
 * @param n: Number of bytes to read
 * @return: Number of bytes read, or -1 on error
 */
int readi(inode_t* ip, char* dst, unsigned int offset, unsigned int n)
{
    if (ip == NULL || !ip->valid || dst == NULL) {
        return -1;
    }

    if (offset >= ip->dinode.size) {
        return 0;  // Beyond end of file
    }

    if (offset + n > ip->dinode.size) {
        n = ip->dinode.size - offset;  // Don't read past end
    }

    unsigned int total_read = 0;
    unsigned int current_offset = offset;

    while (total_read < n) {
        // Calculate which block we're in
        unsigned int block_num = current_offset / BLOCK_SIZE;
        unsigned int block_offset = current_offset % BLOCK_SIZE;
        unsigned int to_read = BLOCK_SIZE - block_offset;
        if (to_read > n - total_read) {
            to_read = n - total_read;
        }

        // Get physical block
        unsigned int phys_block = bmap(ip, block_num);
        if (phys_block == 0) {
            break;  // Block not allocated
        }

        // Read block
        unsigned char block_data[BLOCK_SIZE];
        if (block_read(phys_block, block_data) != 0) {
            return -1;
        }

        // Copy data
        for (unsigned int i = 0; i < to_read; i++) {
            dst[total_read + i] = block_data[block_offset + i];
        }

        total_read += to_read;
        current_offset += to_read;
    }

    return total_read;
}

/**
 * Write data to inode
 * 
 * @param ip: Pointer to inode
 * @param src: Source buffer
 * @param offset: Byte offset in file
 * @param n: Number of bytes to write
 * @return: Number of bytes written, or -1 on error
 */
int writei(inode_t* ip, char* src, unsigned int offset, unsigned int n)
{
    if (ip == NULL || !ip->valid || src == NULL) {
        return -1;
    }

    unsigned int total_written = 0;
    unsigned int current_offset = offset;

    while (total_written < n) {
        // Calculate which block we're in
        unsigned int block_num = current_offset / BLOCK_SIZE;
        unsigned int block_offset = current_offset % BLOCK_SIZE;
        unsigned int to_write = BLOCK_SIZE - block_offset;
        if (to_write > n - total_written) {
            to_write = n - total_written;
        }

        // Get physical block (allocates if needed)
        unsigned int phys_block = bmap(ip, block_num);
        if (phys_block == 0) {
            return -1;  // Out of blocks
        }

        // Read existing block (if partial write)
        unsigned char block_data[BLOCK_SIZE];
        if (block_offset > 0 || to_write < BLOCK_SIZE) {
            if (block_read(phys_block, block_data) != 0) {
                return -1;
            }
        } else {
            // Full block write - zero it first
            for (int i = 0; i < BLOCK_SIZE; i++) {
                block_data[i] = 0;
            }
        }

        // Copy data into block
        for (unsigned int i = 0; i < to_write; i++) {
            block_data[block_offset + i] = src[total_written + i];
        }

        // Write block
        if (block_write(phys_block, block_data) != 0) {
            return -1;
        }

        total_written += to_write;
        current_offset += to_write;
    }

    // Update file size
    if (current_offset > ip->dinode.size) {
        ip->dinode.size = current_offset;
    }

    return total_written;
}

/**
 * Look up directory entry
 * 
 * @param dp: Pointer to directory inode
 * @param name: Filename to find
 * @return: Inode number if found, 0 if not found
 */
unsigned int dirlookup(inode_t* dp, char* name)
{
    if (dp == NULL || !dp->valid || dp->dinode.type != T_DIR || name == NULL) {
        return 0;
    }

    // Read directory entries
    char dir_data[BLOCK_SIZE * 12];  // Max 12 blocks for directory
    int bytes_read = readi(dp, dir_data, 0, dp->dinode.size);
    if (bytes_read <= 0) {
        return 0;
    }

    dirent_t* entries = (dirent_t*)dir_data;
    int num_entries = bytes_read / sizeof(dirent_t);

    for (int i = 0; i < num_entries; i++) {
        if (strcmp(entries[i].name, name) == 0) {
            return entries[i].inum;
        }
    }

    return 0;  // Not found
}

/**
 * Add directory entry
 * 
 * @param dp: Pointer to directory inode
 * @param name: Filename
 * @param inum: Inode number to link
 * @return: 0 on success, -1 on error
 */
int dirlink(inode_t* dp, char* name, unsigned int inum)
{
    if (dp == NULL || !dp->valid || dp->dinode.type != T_DIR || name == NULL) {
        return -1;
    }

    // Check if entry already exists
    if (dirlookup(dp, name) != 0) {
        return -1;  // Entry already exists
    }

    // Find end of directory
    unsigned int dir_size = dp->dinode.size;
    dirent_t new_entry;
    new_entry.inum = inum;
    strncpy(new_entry.name, name, DIRSIZ);
    new_entry.name[DIRSIZ - 1] = '\0';

    // Write new entry at end
    int written = writei(dp, (char*)&new_entry, dir_size, sizeof(dirent_t));
    if (written != sizeof(dirent_t)) {
        return -1;
    }

    return 0;
}

