use core::str;

use crate::blockfs;

const MAX_PATH_LEN: usize = 256;
const MAX_NAME_LEN: usize = 28; // matches blockfs MAX_NAME_BYTES

pub struct FileSystem {
    cwd_inode: u32,
    initialized: bool,
}

impl FileSystem {
    pub const fn new() -> Self {
        Self {
            cwd_inode: 0,
            initialized: false,
        }
    }

    pub fn cwd_inode(&self) -> u32 {
        self.cwd_inode
    }

    pub fn init(&mut self) {
        blockfs::format();
        self.cwd_inode = 0;
        self.initialized = true;
    }

    pub fn create_dir(&mut self, path: &str) -> bool {
        let (parent_ino, name) = match self.parent_and_name(path) {
            Some(v) => v,
            None => return false,
        };
        blockfs::bfs_create_dir(parent_ino, name).is_some()
    }

    pub fn create_file(&mut self, path: &str, content: &str) -> bool {
        let (parent_ino, name) = match self.parent_and_name(path) {
            Some(v) => v,
            None => return false,
        };
        let ino = match blockfs::bfs_create_file(parent_ino, name) {
            Some(i) => i,
            None => return false,
        };
        if !content.is_empty() {
            if !blockfs::bfs_write(ino, content.as_bytes()) {
                // Rollback: delete the created file
                blockfs::bfs_delete(parent_ino, name);
                return false;
            }
        }
        true
    }

    pub fn write_file(&mut self, path: &str, content: &str) -> bool {
        let ino = match self.resolve_path(path) {
            Some(v) => v,
            None => return false,
        };
        if blockfs::bfs_inode_type(ino) != 1 {
            return false;
        }
        blockfs::bfs_write(ino, content.as_bytes())
    }

    pub fn read_file<'a>(&'a self, path: &str, out: &'a mut [u8]) -> Option<&'a str> {
        let ino = self.resolve_path(path)?;
        if blockfs::bfs_inode_type(ino) != 1 {
            return None;
        }
        let n = blockfs::bfs_read(ino, out);
        str::from_utf8(&out[..n]).ok()
    }

    pub fn delete(&mut self, path: &str) -> bool {
        let ino = match self.resolve_path(path) {
            Some(v) if v != 0 => v,
            _ => return false,
        };

        // Find the parent inode by resolving parent path
        let (parent_ino, name) = match self.parent_and_name(path) {
            Some(v) => v,
            None => return false,
        };

        // Verify the resolved inode matches the name in parent
        let _ = ino; // we already confirmed it exists
        blockfs::bfs_delete(parent_ino, name)
    }

    pub fn change_dir(&mut self, path: &str) -> bool {
        let ino = match self.resolve_path(path) {
            Some(v) => v,
            None => return false,
        };
        if blockfs::bfs_inode_type(ino) != 2 {
            return false;
        }
        self.cwd_inode = ino;
        true
    }

    pub fn cwd_path<'a>(&self, out: &'a mut [u8; MAX_PATH_LEN]) -> &'a str {
        out.fill(0);
        if self.cwd_inode == 0 {
            out[0] = b'/';
            return "/";
        }

        // Walk up to root collecting inode numbers and names.
        // We need to find the name of each inode by looking it up in its parent.
        // Since we don't store parent pointers in blockfs, we do a reverse lookup.
        let mut chain_inos = [0u32; 64];
        let mut chain_names: [[u8; MAX_NAME_LEN]; 64] = [[0; MAX_NAME_LEN]; 64];
        let mut chain_name_lens = [0usize; 64];
        let mut chain_len = 0usize;

        let mut cur = self.cwd_inode;
        while cur != 0 && chain_len < 64 {
            chain_inos[chain_len] = cur;
            // Find cur's name by searching all directories for an entry pointing to cur
            let found = self.find_name_of_inode(cur);
            if let Some((name_bytes, nlen, parent)) = found {
                chain_names[chain_len][..nlen].copy_from_slice(&name_bytes[..nlen]);
                chain_name_lens[chain_len] = nlen;
                chain_len += 1;
                cur = parent;
            } else {
                chain_len += 1;
                break;
            }
        }

        let mut pos = 0usize;
        out[pos] = b'/';
        pos += 1;

        for i in (0..chain_len).rev() {
            let nlen = chain_name_lens[i];
            if nlen == 0 {
                continue;
            }
            if pos + nlen + 1 >= out.len() {
                break;
            }
            out[pos..pos + nlen].copy_from_slice(&chain_names[i][..nlen]);
            pos += nlen;
            if i > 0 {
                out[pos] = b'/';
                pos += 1;
            }
        }

        str::from_utf8(&out[..pos]).unwrap_or("/")
    }

    pub fn list_dir<F>(&self, path: &str, mut on_entry: F) -> bool
    where
        F: FnMut(bool, &str),
    {
        let ino = match self.resolve_path(path) {
            Some(v) => v,
            None => return false,
        };
        if blockfs::bfs_inode_type(ino) != 2 {
            return false;
        }

        blockfs::bfs_list(ino, &mut |_child_ino, name, is_dir| {
            on_entry(is_dir, name);
        });
        true
    }

    // -- Private helpers --

    /// Find the name of an inode and its parent by searching all directories.
    /// Returns (name_bytes, name_len, parent_inode).
    fn find_name_of_inode(&self, target: u32) -> Option<([u8; MAX_NAME_LEN], usize, u32)> {
        // Search all inodes that are directories
        for dir_ino in 0..128u32 {
            if blockfs::bfs_inode_type(dir_ino) != 2 {
                continue;
            }
            let mut result: Option<([u8; MAX_NAME_LEN], usize, u32)> = None;
            blockfs::bfs_list(dir_ino, &mut |child_ino, name, _is_dir| {
                if child_ino == target && result.is_none() {
                    let mut buf = [0u8; MAX_NAME_LEN];
                    let bytes = name.as_bytes();
                    let len = bytes.len().min(MAX_NAME_LEN);
                    buf[..len].copy_from_slice(&bytes[..len]);
                    result = Some((buf, len, dir_ino));
                }
            });
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn parent_and_name<'a>(&self, path: &'a str) -> Option<(u32, &'a str)> {
        let trimmed = path.trim_end_matches('/');
        if trimmed.is_empty() || trimmed == "/" {
            return None;
        }

        let (parent_path, name) = match trimmed.rsplit_once('/') {
            Some((p, n)) if !p.is_empty() => (p, n),
            Some((_p, n)) => ("/", n),
            None => (".", trimmed),
        };

        if name.is_empty() {
            return None;
        }

        let parent_ino = self.resolve_path(parent_path)?;
        Some((parent_ino, name))
    }

    fn resolve_path(&self, path: &str) -> Option<u32> {
        if path.is_empty() {
            return None;
        }

        let bytes = path.as_bytes();
        let mut current = if bytes[0] == b'/' {
            0u32
        } else {
            self.cwd_inode
        };

        let mut i = 0usize;

        while i < bytes.len() {
            while i < bytes.len() && bytes[i] == b'/' {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }

            let start = i;
            while i < bytes.len() && bytes[i] != b'/' {
                i += 1;
            }

            let segment = &bytes[start..i];
            if segment.len() == 1 && segment[0] == b'.' {
                continue;
            }
            if segment.len() == 2 && segment[0] == b'.' && segment[1] == b'.' {
                // Go to parent: find the parent of current inode
                if current == 0 {
                    // Already at root
                    continue;
                }
                // Find current inode's parent
                match self.find_name_of_inode(current) {
                    Some((_name, _nlen, parent)) => current = parent,
                    None => return None,
                }
                continue;
            }

            let seg_str = str::from_utf8(segment).ok()?;
            current = blockfs::bfs_lookup(current, seg_str)?;
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::FileSystem;
    use crate::blockfs::tests::TEST_LOCK;

    #[test]
    fn relative_paths_work() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/docs"));
        assert!(fs.change_dir("/docs"));
        assert!(fs.create_file("a.txt", "ok"));

        let mut out = [0u8; 64];
        let text = fs.read_file("/docs/a.txt", &mut out).expect("read");
        assert_eq!(text, "ok");
    }

    #[test]
    fn cwd_path_shows_root() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        let mut buf = [0u8; 256];
        assert_eq!(fs.cwd_path(&mut buf), "/");
    }

    #[test]
    fn cwd_path_shows_subdir() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/mydir"));
        assert!(fs.change_dir("/mydir"));
        let mut buf = [0u8; 256];
        assert_eq!(fs.cwd_path(&mut buf), "/mydir");
    }

    #[test]
    fn list_dir_works() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/stuff"));
        assert!(fs.create_file("/stuff/hello.txt", "hi"));

        let mut entries = std::vec::Vec::new();
        fs.list_dir("/stuff", |is_dir, name| {
            entries.push((is_dir, std::string::String::from(name)));
        });
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].1, "hello.txt");
        assert!(!entries[0].0);
    }

    #[test]
    fn delete_works() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_file("/del.txt", "bye"));
        assert!(fs.delete("/del.txt"));
        let mut out = [0u8; 64];
        assert!(fs.read_file("/del.txt", &mut out).is_none());
    }

    #[test]
    fn write_existing_file() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_file("/f.txt", "old"));
        assert!(fs.write_file("/f.txt", "new"));
        let mut out = [0u8; 64];
        let text = fs.read_file("/f.txt", &mut out).expect("read");
        assert_eq!(text, "new");
    }

    #[test]
    fn dotdot_navigation() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/a"));
        assert!(fs.create_dir("/a/b"));
        assert!(fs.change_dir("/a/b"));
        assert!(fs.change_dir(".."));
        let mut buf = [0u8; 256];
        assert_eq!(fs.cwd_path(&mut buf), "/a");
    }

    #[test]
    fn cwd_inode_getter() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert_eq!(fs.cwd_inode(), 0);
        assert!(fs.create_dir("/test"));
        assert!(fs.change_dir("/test"));
        assert_ne!(fs.cwd_inode(), 0);
    }

    #[test]
    fn change_dir_to_file_fails() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_file("/f.txt", "data"));
        assert!(!fs.change_dir("/f.txt"));
    }

    #[test]
    fn change_dir_nonexistent_fails() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(!fs.change_dir("/nope"));
    }

    #[test]
    fn delete_root_fails() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(!fs.delete("/"));
    }

    #[test]
    fn create_file_in_subdir() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/sub"));
        assert!(fs.create_file("/sub/f.txt", "hello"));
        let mut out = [0u8; 64];
        let text = fs.read_file("/sub/f.txt", &mut out).expect("read");
        assert_eq!(text, "hello");
    }

    #[test]
    fn read_nonexistent_returns_none() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        let mut out = [0u8; 64];
        assert!(fs.read_file("/ghost.txt", &mut out).is_none());
    }

    #[test]
    fn read_dir_returns_none() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/d"));
        let mut out = [0u8; 64];
        assert!(fs.read_file("/d", &mut out).is_none());
    }

    #[test]
    fn write_to_dir_fails() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/d"));
        assert!(!fs.write_file("/d", "data"));
    }

    #[test]
    fn list_nonexistent_fails() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(!fs.list_dir("/nope", |_, _| {}));
    }

    #[test]
    fn dot_path_resolves() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_file("./f.txt", "ok"));
        let mut out = [0u8; 64];
        assert_eq!(fs.read_file("./f.txt", &mut out).unwrap(), "ok");
    }

    #[test]
    fn dotdot_at_root_stays() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.change_dir(".."));
        let mut buf = [0u8; 256];
        assert_eq!(fs.cwd_path(&mut buf), "/");
    }

    #[test]
    fn nested_cwd_path() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(fs.create_dir("/a"));
        assert!(fs.create_dir("/a/b"));
        assert!(fs.create_dir("/a/b/c"));
        assert!(fs.change_dir("/a/b/c"));
        let mut buf = [0u8; 256];
        assert_eq!(fs.cwd_path(&mut buf), "/a/b/c");
    }

    #[test]
    fn empty_path_returns_none() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        let mut out = [0u8; 64];
        assert!(fs.read_file("", &mut out).is_none());
    }

    #[test]
    fn create_in_nonexistent_parent_fails() {
        let _lock = TEST_LOCK.lock().unwrap();
        let mut fs = FileSystem::new();
        fs.init();
        assert!(!fs.create_file("/no/such/path.txt", "data"));
    }
}
