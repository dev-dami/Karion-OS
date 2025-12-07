#ifndef MALLOC_DOT_H
#define MALLOC_DOT_H

// Simple memory allocator for kernel
// Uses a simple linked list of free blocks

#define HEAP_START 0x1000000  // Start heap at 16MB (above kernel)
#define HEAP_SIZE  0x100000   // 1MB heap size
#define MIN_BLOCK_SIZE 16      // Minimum allocation size

// Block header structure
typedef struct block_header {
    unsigned int size;         // Size of block (including header)
    int free;                   // 1 if free, 0 if allocated
    struct block_header* next;  // Next block in list
} block_header_t;

// Initialize the heap
void heap_init(void);

// Allocate memory (similar to malloc)
void* kmalloc(unsigned int size);

// Free memory (similar to free)
void kfree(void* ptr);

// Get total allocated memory (for debugging)
unsigned int get_allocated_memory(void);

#endif /* MALLOC_DOT_H */

