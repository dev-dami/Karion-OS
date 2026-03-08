// Block-based filesystem backed by a RAM disk.
//
// Layout (1 MB = 2048 blocks of 512 bytes):
//   Block 0      : Superblock
//   Block 1-3    : Free-block bitmap (3 blocks = 1536 bytes, enough for 2048 bits)
//   Block 4-19   : Inode table (16 blocks = 8192 bytes = 128 inodes * 64 bytes)
//   Block 20+    : Data blocks

use core::str;

const DISK_SIZE: usize = 1_048_576; // 1 MB
const BLOCK_SIZE: usize = 512;
const TOTAL_BLOCKS: u32 = (DISK_SIZE / BLOCK_SIZE) as u32; // 2048

const BITMAP_START: u32 = 1;
const BITMAP_BLOCKS: u32 = 3;
const INODE_START: u32 = 4;
const INODE_BLOCKS: u32 = 16;
const DATA_START: u32 = 20;

const TOTAL_INODES: u32 = 128;
const INODE_SIZE: usize = 64;
const MAGIC: u32 = 0x4B52_4E46; // "KRNF"

const DIR_ENTRY_SIZE: usize = 32;
const DIR_ENTRIES_PER_BLOCK: usize = BLOCK_SIZE / DIR_ENTRY_SIZE; // 16
const MAX_NAME_BYTES: usize = 28;

const DIRECT_POINTERS: usize = 12;

// The RAM disk ---------------------------------------------------------------

static mut RAMDISK: [u8; DISK_SIZE] = [0u8; DISK_SIZE];

fn read_block(block: u32, buf: &mut [u8; BLOCK_SIZE]) {
    let off = block as usize * BLOCK_SIZE;
    unsafe {
        let ptr = core::ptr::addr_of!(RAMDISK);
        buf.copy_from_slice(&(&(*ptr))[off..off + BLOCK_SIZE]);
    }
}

fn write_block(block: u32, buf: &[u8; BLOCK_SIZE]) {
    let off = block as usize * BLOCK_SIZE;
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RAMDISK);
        (&mut (*ptr))[off..off + BLOCK_SIZE].copy_from_slice(buf);
    }
}

// Superblock -----------------------------------------------------------------

#[repr(C)]
struct Superblock {
    magic: u32,
    total_blocks: u32,
    total_inodes: u32,
    bitmap_start: u32,
    inode_start: u32,
    data_start: u32,
}

// Inode (64 bytes) -----------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct DiskInode {
    itype: u8,          // 0=free, 1=file, 2=directory
    size: u32,          // file/dir content size in bytes
    direct: [u32; 12],  // direct block pointers
    indirect: u32,      // single-indirect block pointer
    _pad: [u8; 3],      // padding to 64 bytes
}

const EMPTY_INODE: DiskInode = DiskInode {
    itype: 0,
    size: 0,
    direct: [0; 12],
    indirect: 0,
    _pad: [0; 3],
};

// DirEntry (32 bytes) --------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct DirEntry {
    inode: u32,
    name: [u8; MAX_NAME_BYTES],
}

const EMPTY_DIRENT: DirEntry = DirEntry {
    inode: 0,
    name: [0; MAX_NAME_BYTES],
};

// Little-endian helpers (raw byte arrays) ------------------------------------

fn read_u32_le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn write_u32_le(b: &mut [u8], off: usize, v: u32) {
    let le = v.to_le_bytes();
    b[off..off + 4].copy_from_slice(&le);
}

// Inode serialisation --------------------------------------------------------

fn inode_block_and_offset(ino: u32) -> (u32, usize) {
    let byte_off = ino as usize * INODE_SIZE;
    let blk = INODE_START + (byte_off / BLOCK_SIZE) as u32;
    let off = byte_off % BLOCK_SIZE;
    (blk, off)
}

fn load_inode(ino: u32) -> DiskInode {
    let (blk, off) = inode_block_and_offset(ino);
    let mut buf = [0u8; BLOCK_SIZE];
    read_block(blk, &mut buf);
    let b = &buf[off..off + INODE_SIZE];

    let mut di = EMPTY_INODE;
    di.itype = b[0];
    di.size = read_u32_le(b, 1);
    for i in 0..DIRECT_POINTERS {
        di.direct[i] = read_u32_le(b, 5 + i * 4);
    }
    di.indirect = read_u32_le(b, 5 + DIRECT_POINTERS * 4);
    di
}

fn store_inode(ino: u32, di: &DiskInode) {
    let (blk, off) = inode_block_and_offset(ino);
    let mut buf = [0u8; BLOCK_SIZE];
    read_block(blk, &mut buf);

    let b = &mut buf[off..off + INODE_SIZE];
    b[0] = di.itype;
    write_u32_le(b, 1, di.size);
    for i in 0..DIRECT_POINTERS {
        write_u32_le(b, 5 + i * 4, di.direct[i]);
    }
    write_u32_le(b, 5 + DIRECT_POINTERS * 4, di.indirect);
    // padding bytes stay zero
    write_block(blk, &buf);
}

// Bitmap helpers -------------------------------------------------------------

fn bitmap_is_set(block_num: u32) -> bool {
    let byte_idx = block_num as usize / 8;
    let bit_idx = block_num as usize % 8;
    let bmp_block = BITMAP_START + (byte_idx / BLOCK_SIZE) as u32;
    let bmp_off = byte_idx % BLOCK_SIZE;
    let mut buf = [0u8; BLOCK_SIZE];
    read_block(bmp_block, &mut buf);
    buf[bmp_off] & (1 << bit_idx) != 0
}

fn bitmap_set(block_num: u32) {
    let byte_idx = block_num as usize / 8;
    let bit_idx = block_num as usize % 8;
    let bmp_block = BITMAP_START + (byte_idx / BLOCK_SIZE) as u32;
    let bmp_off = byte_idx % BLOCK_SIZE;
    let mut buf = [0u8; BLOCK_SIZE];
    read_block(bmp_block, &mut buf);
    buf[bmp_off] |= 1 << bit_idx;
    write_block(bmp_block, &buf);
}

fn bitmap_clear(block_num: u32) {
    let byte_idx = block_num as usize / 8;
    let bit_idx = block_num as usize % 8;
    let bmp_block = BITMAP_START + (byte_idx / BLOCK_SIZE) as u32;
    let bmp_off = byte_idx % BLOCK_SIZE;
    let mut buf = [0u8; BLOCK_SIZE];
    read_block(bmp_block, &mut buf);
    buf[bmp_off] &= !(1 << bit_idx);
    write_block(bmp_block, &buf);
}

fn alloc_block() -> Option<u32> {
    for b in DATA_START..TOTAL_BLOCKS {
        if !bitmap_is_set(b) {
            bitmap_set(b);
            // Zero the block
            let zero = [0u8; BLOCK_SIZE];
            write_block(b, &zero);
            return Some(b);
        }
    }
    None
}

fn free_block(block_num: u32) {
    if block_num >= DATA_START && block_num < TOTAL_BLOCKS {
        bitmap_clear(block_num);
    }
}

// Inode allocator ------------------------------------------------------------

fn alloc_inode() -> Option<u32> {
    // inode 0 is root, skip it when searching for free inodes after format
    for i in 1..TOTAL_INODES {
        let di = load_inode(i);
        if di.itype == 0 {
            return Some(i);
        }
    }
    None
}

// Block pointers for an inode ------------------------------------------------

/// Get the block number for the `nth` logical block of an inode.
fn inode_get_block(di: &DiskInode, nth: usize) -> u32 {
    if nth < DIRECT_POINTERS {
        di.direct[nth]
    } else if di.indirect != 0 {
        let indirect_off = nth - DIRECT_POINTERS;
        if indirect_off >= BLOCK_SIZE / 4 {
            return 0;
        }
        let mut buf = [0u8; BLOCK_SIZE];
        read_block(di.indirect, &mut buf);
        read_u32_le(&buf, indirect_off * 4)
    } else {
        0
    }
}

/// Ensure that the `nth` logical block exists, allocating if necessary.
/// Returns the block number, or 0 on failure.
fn inode_ensure_block(ino: u32, di: &mut DiskInode, nth: usize) -> u32 {
    if nth < DIRECT_POINTERS {
        if di.direct[nth] == 0 {
            match alloc_block() {
                Some(b) => {
                    di.direct[nth] = b;
                    store_inode(ino, di);
                }
                None => return 0,
            }
        }
        di.direct[nth]
    } else {
        let indirect_off = nth - DIRECT_POINTERS;
        if indirect_off >= BLOCK_SIZE / 4 {
            return 0;
        }
        // Ensure the indirect block itself exists
        if di.indirect == 0 {
            match alloc_block() {
                Some(b) => {
                    di.indirect = b;
                    store_inode(ino, di);
                }
                None => return 0,
            }
        }
        let mut ibuf = [0u8; BLOCK_SIZE];
        read_block(di.indirect, &mut ibuf);
        let existing = read_u32_le(&ibuf, indirect_off * 4);
        if existing != 0 {
            return existing;
        }
        match alloc_block() {
            Some(b) => {
                write_u32_le(&mut ibuf, indirect_off * 4, b);
                write_block(di.indirect, &ibuf);
                b
            }
            None => 0,
        }
    }
}

// Directory helpers ----------------------------------------------------------

fn dir_add_entry(dir_ino: u32, child_ino: u32, name: &[u8]) -> bool {
    let mut di = load_inode(dir_ino);
    if di.itype != 2 {
        return false;
    }

    let num_entries = di.size as usize / DIR_ENTRY_SIZE;
    let total_slots_allocated = {
        let full_blocks = if di.size == 0 { 0 } else { (di.size as usize + BLOCK_SIZE - 1) / BLOCK_SIZE };
        full_blocks * DIR_ENTRIES_PER_BLOCK
    };

    // Look for a free slot in existing blocks
    for slot in 0..total_slots_allocated {
        let blk_idx = slot / DIR_ENTRIES_PER_BLOCK;
        let entry_off = (slot % DIR_ENTRIES_PER_BLOCK) * DIR_ENTRY_SIZE;
        let blk = inode_get_block(&di, blk_idx);
        if blk == 0 {
            continue;
        }
        let mut buf = [0u8; BLOCK_SIZE];
        read_block(blk, &mut buf);
        let ino_val = read_u32_le(&buf, entry_off);
        if ino_val == 0 {
            // Free slot - use it
            write_u32_le(&mut buf, entry_off, child_ino);
            let name_start = entry_off + 4;
            buf[name_start..name_start + MAX_NAME_BYTES].fill(0);
            let copy_len = name.len().min(MAX_NAME_BYTES - 1);
            buf[name_start..name_start + copy_len].copy_from_slice(&name[..copy_len]);
            write_block(blk, &buf);
            // Update size if this slot extends beyond current size
            let new_size = ((slot + 1) * DIR_ENTRY_SIZE) as u32;
            if new_size > di.size {
                di.size = new_size;
                store_inode(dir_ino, &di);
            }
            return true;
        }
    }

    // Need to append: figure out the next slot
    let next_slot = if total_slots_allocated > num_entries {
        // Shouldn't happen if we checked above, but just in case
        total_slots_allocated
    } else {
        total_slots_allocated
    };

    let blk_idx = next_slot / DIR_ENTRIES_PER_BLOCK;
    let entry_off = (next_slot % DIR_ENTRIES_PER_BLOCK) * DIR_ENTRY_SIZE;

    let blk = inode_ensure_block(dir_ino, &mut di, blk_idx);
    if blk == 0 {
        return false;
    }

    let mut buf = [0u8; BLOCK_SIZE];
    read_block(blk, &mut buf);
    write_u32_le(&mut buf, entry_off, child_ino);
    let name_start = entry_off + 4;
    buf[name_start..name_start + MAX_NAME_BYTES].fill(0);
    let copy_len = name.len().min(MAX_NAME_BYTES - 1);
    buf[name_start..name_start + copy_len].copy_from_slice(&name[..copy_len]);
    write_block(blk, &buf);

    di.size = ((next_slot + 1) * DIR_ENTRY_SIZE) as u32;
    store_inode(dir_ino, &di);
    true
}

fn dir_remove_entry(dir_ino: u32, name: &[u8]) -> bool {
    let di = load_inode(dir_ino);
    if di.itype != 2 {
        return false;
    }
    let num_entries = di.size as usize / DIR_ENTRY_SIZE;

    for slot in 0..num_entries {
        let blk_idx = slot / DIR_ENTRIES_PER_BLOCK;
        let entry_off = (slot % DIR_ENTRIES_PER_BLOCK) * DIR_ENTRY_SIZE;
        let blk = inode_get_block(&di, blk_idx);
        if blk == 0 {
            continue;
        }
        let mut buf = [0u8; BLOCK_SIZE];
        read_block(blk, &mut buf);
        let ino_val = read_u32_le(&buf, entry_off);
        if ino_val == 0 {
            continue;
        }
        let name_start = entry_off + 4;
        let stored = &buf[name_start..name_start + MAX_NAME_BYTES];
        if names_equal(stored, name) {
            // Clear the entry
            buf[entry_off..entry_off + DIR_ENTRY_SIZE].fill(0);
            write_block(blk, &buf);
            return true;
        }
    }
    false
}

fn names_equal(stored: &[u8], name: &[u8]) -> bool {
    // stored is a null-terminated name in a MAX_NAME_BYTES buffer
    let mut slen = 0;
    for &b in stored.iter() {
        if b == 0 {
            break;
        }
        slen += 1;
    }
    if slen != name.len() {
        return false;
    }
    // Manual byte-by-byte comparison (Rust's == on slices is broken on this
    // bare-metal i686 target with static relocation model)
    let mut i = 0;
    while i < slen {
        if stored[i] != name[i] {
            return false;
        }
        i += 1;
    }
    true
}

// Free all data blocks owned by an inode
fn free_inode_blocks(di: &DiskInode) {
    for i in 0..DIRECT_POINTERS {
        if di.direct[i] != 0 {
            free_block(di.direct[i]);
        }
    }
    if di.indirect != 0 {
        let mut ibuf = [0u8; BLOCK_SIZE];
        read_block(di.indirect, &mut ibuf);
        for i in 0..(BLOCK_SIZE / 4) {
            let b = read_u32_le(&ibuf, i * 4);
            if b != 0 {
                free_block(b);
            }
        }
        free_block(di.indirect);
    }
}

// Public API -----------------------------------------------------------------

/// Format the RAM disk, creating the superblock, bitmap, inode table, and root directory.
pub fn format() {
    // Zero entire disk
    unsafe {
        let ptr = core::ptr::addr_of_mut!(RAMDISK);
        (&mut (*ptr)).fill(0);
    }

    // Write superblock (block 0)
    let mut sb_buf = [0u8; BLOCK_SIZE];
    write_u32_le(&mut sb_buf, 0, MAGIC);
    write_u32_le(&mut sb_buf, 4, TOTAL_BLOCKS);
    write_u32_le(&mut sb_buf, 8, TOTAL_INODES);
    write_u32_le(&mut sb_buf, 12, BITMAP_START);
    write_u32_le(&mut sb_buf, 16, INODE_START);
    write_u32_le(&mut sb_buf, 20, DATA_START);
    write_block(0, &sb_buf);

    // Mark metadata blocks as used in bitmap (blocks 0..DATA_START)
    for b in 0..DATA_START {
        bitmap_set(b);
    }

    // Create root directory (inode 0)
    let root = DiskInode {
        itype: 2, // directory
        size: 0,
        direct: [0; 12],
        indirect: 0,
        _pad: [0; 3],
    };
    store_inode(0, &root);
}

/// Create a file in `parent_inode` directory. Returns the new inode number.
pub fn bfs_create_file(parent_inode: u32, name: &str) -> Option<u32> {
    let name_bytes = name.as_bytes();
    if name_bytes.is_empty() || name_bytes.len() >= MAX_NAME_BYTES {
        return None;
    }

    // Check parent is a directory
    let pdi = load_inode(parent_inode);
    if pdi.itype != 2 {
        return None;
    }

    // Check name doesn't already exist
    if bfs_lookup(parent_inode, name).is_some() {
        return None;
    }

    let ino = alloc_inode()?;
    let fi = DiskInode {
        itype: 1, // file
        size: 0,
        direct: [0; 12],
        indirect: 0,
        _pad: [0; 3],
    };
    store_inode(ino, &fi);

    if !dir_add_entry(parent_inode, ino, name_bytes) {
        // Roll back
        store_inode(ino, &EMPTY_INODE);
        return None;
    }

    Some(ino)
}

/// Create a subdirectory in `parent_inode`. Returns the new inode number.
pub fn bfs_create_dir(parent_inode: u32, name: &str) -> Option<u32> {
    let name_bytes = name.as_bytes();
    if name_bytes.is_empty() || name_bytes.len() >= MAX_NAME_BYTES {
        return None;
    }

    let pdi = load_inode(parent_inode);
    if pdi.itype != 2 {
        return None;
    }

    if bfs_lookup(parent_inode, name).is_some() {
        return None;
    }

    let ino = alloc_inode()?;
    let di = DiskInode {
        itype: 2, // directory
        size: 0,
        direct: [0; 12],
        indirect: 0,
        _pad: [0; 3],
    };
    store_inode(ino, &di);

    if !dir_add_entry(parent_inode, ino, name_bytes) {
        store_inode(ino, &EMPTY_INODE);
        return None;
    }

    Some(ino)
}

/// Write data to a file inode (replaces existing content).
pub fn bfs_write(inode: u32, data: &[u8]) -> bool {
    let mut di = load_inode(inode);
    if di.itype != 1 {
        return false;
    }

    // Free existing blocks first
    free_inode_blocks(&di);
    di.direct = [0; 12];
    di.indirect = 0;
    di.size = 0;
    store_inode(inode, &di);

    if data.is_empty() {
        return true;
    }

    let mut written = 0usize;
    let mut blk_idx = 0usize;

    while written < data.len() {
        let blk = inode_ensure_block(inode, &mut di, blk_idx);
        if blk == 0 {
            return false;
        }
        let mut buf = [0u8; BLOCK_SIZE];
        let chunk = (data.len() - written).min(BLOCK_SIZE);
        buf[..chunk].copy_from_slice(&data[written..written + chunk]);
        write_block(blk, &buf);
        written += chunk;
        blk_idx += 1;
    }

    di.size = data.len() as u32;
    store_inode(inode, &di);
    true
}

/// Read file content into `buf`. Returns number of bytes read.
pub fn bfs_read(inode: u32, buf: &mut [u8]) -> usize {
    let di = load_inode(inode);
    if di.itype != 1 {
        return 0;
    }

    let total = (di.size as usize).min(buf.len());
    let mut read_so_far = 0usize;
    let mut blk_idx = 0usize;

    while read_so_far < total {
        let blk = inode_get_block(&di, blk_idx);
        if blk == 0 {
            break;
        }
        let mut block_buf = [0u8; BLOCK_SIZE];
        read_block(blk, &mut block_buf);
        let chunk = (total - read_so_far).min(BLOCK_SIZE);
        buf[read_so_far..read_so_far + chunk].copy_from_slice(&block_buf[..chunk]);
        read_so_far += chunk;
        blk_idx += 1;
    }

    read_so_far
}

/// Delete a named entry from a parent directory, freeing its blocks and inode.
pub fn bfs_delete(parent_inode: u32, name: &str) -> bool {
    let child_ino = match bfs_lookup(parent_inode, name) {
        Some(i) => i,
        None => return false,
    };

    let di = load_inode(child_ino);

    // If directory, check it's empty
    if di.itype == 2 && di.size > 0 {
        // Check if any entries are non-zero
        let num_entries = di.size as usize / DIR_ENTRY_SIZE;
        let mut has_children = false;
        for slot in 0..num_entries {
            let blk_idx = slot / DIR_ENTRIES_PER_BLOCK;
            let entry_off = (slot % DIR_ENTRIES_PER_BLOCK) * DIR_ENTRY_SIZE;
            let blk = inode_get_block(&di, blk_idx);
            if blk == 0 {
                continue;
            }
            let mut buf = [0u8; BLOCK_SIZE];
            read_block(blk, &mut buf);
            let ino_val = read_u32_le(&buf, entry_off);
            if ino_val != 0 {
                has_children = true;
                break;
            }
        }
        if has_children {
            return false;
        }
    }

    // Remove from parent directory
    if !dir_remove_entry(parent_inode, name.as_bytes()) {
        return false;
    }

    // Free blocks and clear inode
    free_inode_blocks(&di);
    store_inode(child_ino, &EMPTY_INODE);
    true
}

/// Look up a child entry by name in a directory. Returns the child's inode number.
pub fn bfs_lookup(parent_inode: u32, name: &str) -> Option<u32> {
    let di = load_inode(parent_inode);
    if di.itype != 2 {
        return None;
    }
    let name_bytes = name.as_bytes();
    let num_entries = di.size as usize / DIR_ENTRY_SIZE;

    for slot in 0..num_entries {
        let blk_idx = slot / DIR_ENTRIES_PER_BLOCK;
        let entry_off = (slot % DIR_ENTRIES_PER_BLOCK) * DIR_ENTRY_SIZE;
        let blk = inode_get_block(&di, blk_idx);
        if blk == 0 {
            continue;
        }
        let mut buf = [0u8; BLOCK_SIZE];
        read_block(blk, &mut buf);
        let ino_val = read_u32_le(&buf, entry_off);
        if ino_val == 0 {
            continue;
        }
        let name_start = entry_off + 4;
        let stored = &buf[name_start..name_start + MAX_NAME_BYTES];
        if names_equal(stored, name_bytes) {
            return Some(ino_val);
        }
    }
    None
}

/// List directory entries, calling `callback(inode, name, is_dir)` for each.
pub fn bfs_list(inode: u32, callback: &mut dyn FnMut(u32, &str, bool)) {
    let di = load_inode(inode);
    if di.itype != 2 {
        return;
    }
    let num_entries = di.size as usize / DIR_ENTRY_SIZE;

    for slot in 0..num_entries {
        let blk_idx = slot / DIR_ENTRIES_PER_BLOCK;
        let entry_off = (slot % DIR_ENTRIES_PER_BLOCK) * DIR_ENTRY_SIZE;
        let blk = inode_get_block(&di, blk_idx);
        if blk == 0 {
            continue;
        }
        let mut buf = [0u8; BLOCK_SIZE];
        read_block(blk, &mut buf);
        let ino_val = read_u32_le(&buf, entry_off);
        if ino_val == 0 {
            continue;
        }
        let name_start = entry_off + 4;
        let stored = &buf[name_start..name_start + MAX_NAME_BYTES];
        let mut slen = 0;
        for &b in stored.iter() {
            if b == 0 {
                break;
            }
            slen += 1;
        }
        if let Ok(name) = str::from_utf8(&stored[..slen]) {
            let child_di = load_inode(ino_val);
            callback(ino_val, name, child_di.itype == 2);
        }
    }
}

/// Get the type of an inode (0=free, 1=file, 2=directory).
pub fn bfs_inode_type(inode: u32) -> u8 {
    load_inode(inode).itype
}

/// Get the size (in bytes) stored in an inode.
pub fn bfs_inode_size(inode: u32) -> u32 {
    load_inode(inode).size
}

// Tests ----------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn format_and_create_file() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let ino = bfs_create_file(0, "hello.txt").unwrap();
        assert!(bfs_write(ino, b"Hello World"));
        let mut buf = [0u8; 512];
        let n = bfs_read(ino, &mut buf);
        assert_eq!(&buf[..n], b"Hello World");
    }

    #[test]
    fn create_and_list_directory() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let dir_ino = bfs_create_dir(0, "docs").unwrap();
        let file_ino = bfs_create_file(dir_ino, "readme.txt").unwrap();
        assert!(bfs_write(file_ino, b"contents"));

        let mut found = false;
        bfs_list(0, &mut |_ino, name, is_dir| {
            if name == "docs" && is_dir {
                found = true;
            }
        });
        assert!(found);

        let mut found_file = false;
        bfs_list(dir_ino, &mut |_ino, name, is_dir| {
            if name == "readme.txt" && !is_dir {
                found_file = true;
            }
        });
        assert!(found_file);
    }

    #[test]
    fn lookup_works() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let ino = bfs_create_file(0, "test.txt").unwrap();
        assert_eq!(bfs_lookup(0, "test.txt"), Some(ino));
        assert_eq!(bfs_lookup(0, "nonexistent"), None);
    }

    #[test]
    fn delete_file() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let _ino = bfs_create_file(0, "todelete.txt").unwrap();
        assert!(bfs_delete(0, "todelete.txt"));
        assert_eq!(bfs_lookup(0, "todelete.txt"), None);
    }

    #[test]
    fn delete_nonempty_dir_fails() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let dir_ino = bfs_create_dir(0, "mydir").unwrap();
        let _file_ino = bfs_create_file(dir_ino, "child.txt").unwrap();
        assert!(!bfs_delete(0, "mydir"));
    }

    #[test]
    fn inode_type_and_size() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let ino = bfs_create_file(0, "sized.txt").unwrap();
        assert!(bfs_write(ino, b"12345"));
        assert_eq!(bfs_inode_type(ino), 1);
        assert_eq!(bfs_inode_size(ino), 5);
        assert_eq!(bfs_inode_type(0), 2); // root is dir
    }

    #[test]
    fn duplicate_name_rejected() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        assert!(bfs_create_file(0, "dup.txt").is_some());
        assert!(bfs_create_file(0, "dup.txt").is_none());
    }

    #[test]
    fn overwrite_file_content() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let ino = bfs_create_file(0, "over.txt").unwrap();
        assert!(bfs_write(ino, b"first"));
        assert!(bfs_write(ino, b"second"));
        let mut buf = [0u8; 64];
        let n = bfs_read(ino, &mut buf);
        assert_eq!(&buf[..n], b"second");
    }

    #[test]
    fn read_empty_file() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let ino = bfs_create_file(0, "empty.txt").unwrap();
        let mut buf = [0u8; 64];
        let n = bfs_read(ino, &mut buf);
        assert_eq!(n, 0);
    }

    #[test]
    fn delete_empty_dir() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let _dir = bfs_create_dir(0, "emptydir").unwrap();
        assert!(bfs_delete(0, "emptydir"));
        assert_eq!(bfs_lookup(0, "emptydir"), None);
    }

    #[test]
    fn nested_directories() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let d1 = bfs_create_dir(0, "a").unwrap();
        let d2 = bfs_create_dir(d1, "b").unwrap();
        let f = bfs_create_file(d2, "c.txt").unwrap();
        assert!(bfs_write(f, b"deep"));
        let mut buf = [0u8; 64];
        let n = bfs_read(f, &mut buf);
        assert_eq!(&buf[..n], b"deep");
    }

    #[test]
    fn multiple_files_in_dir() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        bfs_create_file(0, "a.txt").unwrap();
        bfs_create_file(0, "b.txt").unwrap();
        bfs_create_file(0, "c.txt").unwrap();
        let mut count = 0;
        bfs_list(0, &mut |_ino, _name, _is_dir| { count += 1; });
        assert_eq!(count, 3);
    }

    #[test]
    fn lookup_nonexistent_returns_none() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        assert_eq!(bfs_lookup(0, "ghost"), None);
    }

    #[test]
    fn write_larger_data() {
        let _lock = TEST_LOCK.lock().unwrap();
        format();
        let ino = bfs_create_file(0, "big.txt").unwrap();
        let data = [b'X'; 2048];
        assert!(bfs_write(ino, &data));
        let mut buf = [0u8; 2048];
        let n = bfs_read(ino, &mut buf);
        assert_eq!(n, 2048);
        assert!(buf.iter().all(|&b| b == b'X'));
    }
}
