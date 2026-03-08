// Required by the compiler when building without libstd/libc.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0usize;
    while i < n {
        unsafe {
            *dst.add(i) = *src.add(i);
        }
        i += 1;
    }
    dst
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if (dst as usize) <= (src as usize) {
        unsafe { memcpy(dst, src, n) }
    } else {
        let mut i = n;
        while i > 0 {
            let j = i - 1;
            unsafe {
                *dst.add(j) = *src.add(j);
            }
            i -= 1;
        }
        dst
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dst: *mut u8, val: i32, n: usize) -> *mut u8 {
    let byte = val as u8;
    let mut i = 0usize;
    while i < n {
        unsafe {
            *dst.add(i) = byte;
        }
        i += 1;
    }
    dst
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let av = unsafe { *a.add(i) };
        let bv = unsafe { *b.add(i) };
        if av != bv {
            return av as i32 - bv as i32;
        }
        i += 1;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    if unsafe { memcmp(a, b, n) } == 0 {
        0
    } else {
        1
    }
}
