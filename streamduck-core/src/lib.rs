use std::ffi::{c_char, CStr};

#[no_mangle]
pub extern "C" fn print_test(str: *const c_char) {
    let cstr = unsafe { CStr::from_ptr(str) };
    println!("{}", cstr.to_str().unwrap());
}