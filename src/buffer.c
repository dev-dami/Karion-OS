#include "buffer.h"
#include "source.h"

// Buffer cache pool
static buf_t bufs[NBUF];
static int buffer_initialized = 0;

// Simple hash table for buffers (by block number)
#define HASH_SIZE 8
static buf_t* hash_table[HASH_SIZE];

/**
 * Hash function for block numbers
 * 
 * @param blockno: Block number
 * @return: Hash index
 */
static unsigned int hash(unsigned int blockno)
{
    return blockno % HASH_SIZE;
}

/**
 * Initialize buffer cache
 * Sets up the buffer pool and hash table
 */
void buffer_init(void)
{
    if (buffer_initialized) {
        return;
    }

    // Initialize all buffers
    for (int i = 0; i < NBUF; i++) {
        bufs[i].valid = 0;
        bufs[i].disk = 0;
        bufs[i].blockno = 0;
        bufs[i].next = NULL;
    }

    // Initialize hash table
    for (int i = 0; i < HASH_SIZE; i++) {
        hash_table[i] = NULL;
    }

    buffer_initialized = 1;
}

/**
 * Find buffer in cache by block number
 * 
 * @param blockno: Block number to find
 * @return: Pointer to buffer if found, NULL otherwise
 */
static buf_t* bget(unsigned int blockno)
{
    if (!buffer_initialized) {
        buffer_init();
    }

    // Search hash chain
    unsigned int h = hash(blockno);
    buf_t* b = hash_table[h];

    while (b != NULL) {
        if (b->blockno == blockno && b->valid) {
            return b;  // Found in cache
        }
        b = b->next;
    }

    // Not in cache - find a free buffer
    for (int i = 0; i < NBUF; i++) {
        if (!bufs[i].valid) {
            // Use this buffer
            bufs[i].valid = 1;
            bufs[i].disk = 0;
            bufs[i].blockno = blockno;
            
            // Add to hash chain
            bufs[i].next = hash_table[h];
            hash_table[h] = &bufs[i];
            
            return &bufs[i];
        }
    }

    // No free buffer - use first one (simple eviction)
    // In a real system, you'd use LRU or similar
    buf_t* victim = &bufs[0];
    
    // Remove from hash chain
    unsigned int victim_hash = hash(victim->blockno);
    if (hash_table[victim_hash] == victim) {
        hash_table[victim_hash] = victim->next;
    } else {
        buf_t* prev = hash_table[victim_hash];
        while (prev != NULL && prev->next != victim) {
            prev = prev->next;
        }
        if (prev != NULL) {
            prev->next = victim->next;
        }
    }

    // Write back if dirty
    if (victim->disk) {
        block_write(victim->blockno, victim->data);
    }

    // Reuse buffer
    victim->valid = 1;
    victim->disk = 0;
    victim->blockno = blockno;
    victim->next = hash_table[h];
    hash_table[h] = victim;

    return victim;
}

/**
 * Get buffer for a block (read from disk if not cached)
 * 
 * @param blockno: Block number
 * @return: Pointer to buffer, or NULL on error
 */
buf_t* bread(unsigned int blockno)
{
    buf_t* b = bget(blockno);
    if (b == NULL) {
        return NULL;
    }

    // If not already loaded, read from disk
    if (!b->disk) {
        if (block_read(blockno, b->data) != 0) {
            b->valid = 0;
            return NULL;
        }
        b->disk = 1;  // Mark as loaded from disk
    }

    return b;
}

/**
 * Write buffer to disk
 * 
 * @param b: Pointer to buffer
 */
void bwrite(buf_t* b)
{
    if (b == NULL || !b->valid) {
        return;
    }

    // Write to disk
    if (block_write(b->blockno, b->data) == 0) {
        b->disk = 1;  // Mark as synced with disk
    }
}

/**
 * Release buffer
 * Makes buffer available for reuse (doesn't write to disk)
 * 
 * @param b: Pointer to buffer
 */
void brelse(buf_t* b)
{
    // In this simplified version, we don't do anything
    // In a real system, this would update LRU, unlock, etc.
    // For now, buffers stay in cache until evicted
}

