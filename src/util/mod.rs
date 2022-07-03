use libc::c_char;
use std::ffi::CString;

pub fn zascii(slice: &[c_char]) -> String {
    String::from_iter(slice.iter().take_while(|c| **c != 0).map(|c| *c as u8 as char))
}

pub fn str2dst(dst: *mut c_char, src: &str) -> *mut c_char {
    unsafe {
        let str2cstr = CString::new(src).unwrap();
        let src = str2cstr.as_ptr() as *const c_char;
        libc::strcpy(dst, src)
    }
}
