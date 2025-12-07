#ifndef BUFFER_DOT_H
#define BUFFER_DOT_H

#include "block.h"

// Buffer cache for disk blocks
// Caches recently used blocks in memory to reduce disk I/O
// Simplified version inspired by Xv6

#define NBUF 16  // Number of buffers in cache

// Buffer structure
typedef struct buf {
    int valid;           // Has data been read from disk?
    int disk;            // Does disk "own" buffer? (dirty flag)
    unsigned int blockno; // Block number
    unsigned char data[512];  // Block data
    struct buf* next;    // Next buffer in hash chain
} buf_t;

// Initialize buffer cache
// Sets up the buffer pool
void buffer_init(void);

// Get buffer for a block
// Returns cached buffer or reads from disk
//
// @param blockno: Block number
// @return: Pointer to buffer, or NULL on error
buf_t* bread(unsigned int blockno);

// Write buffer to disk
// Marks buffer as dirty and writes to disk
//
// @param b: Pointer to buffer
void bwrite(buf_t* b);

// Release buffer
// Marks buffer as available (doesn't write to disk)
//
// @param b: Pointer to buffer
void brelse(buf_t* b);

#endif /* BUFFER_DOT_H */

