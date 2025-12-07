#include "malloc.h"
#include "source.h"

// Heap memory region
static char heap[HEAP_SIZE];
static block_header_t* free_list = NULL;
static int heap_initialized = 0;

/**
 * Initialize the heap allocator
 * Sets up the initial free block covering the entire heap
 */
void heap_init(void)
{
    if (heap_initialized) return;

    // Initialize first block as free, covering entire heap
    block_header_t* first_block = (block_header_t*)heap;
    first_block->size = HEAP_SIZE;
    first_block->free = 1;
    first_block->next = NULL;
    
    free_list = first_block;
    heap_initialized = 1;
}

/**
 * Allocate memory block
 * Uses first-fit algorithm: find first free block large enough
 * 
 * @param size: Number of bytes to allocate
 * @return: Pointer to allocated memory, or NULL if out of memory
 */
void* kmalloc(unsigned int size)
{
    if (!heap_initialized) {
        heap_init();
    }

    if (size == 0) return NULL;

    // Add header size and align to 4 bytes
    unsigned int total_size = size + sizeof(block_header_t);
    total_size = (total_size + 3) & ~3;  // Align to 4 bytes
    
    // Minimum block size
    if (total_size < MIN_BLOCK_SIZE) {
        total_size = MIN_BLOCK_SIZE;
    }

    block_header_t* current = free_list;
    block_header_t* prev = NULL;

    // First-fit: find first block large enough
    while (current != NULL) {
        if (current->free && current->size >= total_size) {
            // Found a suitable block
            
            // If block is much larger, split it
            if (current->size >= total_size + sizeof(block_header_t) + MIN_BLOCK_SIZE) {
                // Split the block
                block_header_t* new_block = (block_header_t*)((char*)current + total_size);
                new_block->size = current->size - total_size;
                new_block->free = 1;
                new_block->next = current->next;
                
                current->size = total_size;
                current->next = new_block;
            } else {
                // Remove from free list
                if (prev) {
                    prev->next = current->next;
                } else {
                    free_list = current->next;
                }
            }

            // Mark as allocated
            current->free = 0;
            current->next = NULL;

            // Return pointer to data (after header)
            return (void*)((char*)current + sizeof(block_header_t));
        }
        
        prev = current;
        current = current->next;
    }

    // No suitable block found
    return NULL;
}

/**
 * Free allocated memory block
 * Coalesces adjacent free blocks to reduce fragmentation
 * 
 * @param ptr: Pointer to memory to free (must be from kmalloc)
 */
void kfree(void* ptr)
{
    if (ptr == NULL) return;

    // Get block header
    block_header_t* block = (block_header_t*)((char*)ptr - sizeof(block_header_t));
    
    if (block->free) {
        // Already free - double free error
        return;
    }

    block->free = 1;

    // Coalesce with adjacent free blocks
    block_header_t* current = free_list;
    block_header_t* prev = NULL;

    // Find insertion point in free list (sorted by address)
    while (current != NULL && (char*)current < (char*)block) {
        prev = current;
        current = current->next;
    }

    // Check if we can merge with previous block
    if (prev != NULL) {
        block_header_t* prev_end = (block_header_t*)((char*)prev + prev->size);
        if (prev_end == block) {
            // Merge with previous block
            prev->size += block->size;
            block = prev;
        } else {
            // Insert after previous
            block->next = prev->next;
            prev->next = block;
        }
    } else {
        // Insert at beginning
        block->next = free_list;
        free_list = block;
    }

    // Check if we can merge with next block
    block_header_t* block_end = (block_header_t*)((char*)block + block->size);
    if (block->next != NULL && block_end == block->next) {
        // Merge with next block
        block->size += block->next->size;
        block->next = block->next->next;
    }
}

/**
 * Get total allocated memory (for debugging)
 * 
 * @return: Total bytes currently allocated
 */
unsigned int get_allocated_memory(void)
{
    unsigned int total = 0;
    block_header_t* current = (block_header_t*)heap;
    char* heap_end = heap + HEAP_SIZE;

    while ((char*)current < heap_end) {
        if (!current->free) {
            total += current->size - sizeof(block_header_t);
        }
        current = (block_header_t*)((char*)current + current->size);
    }

    return total;
}

