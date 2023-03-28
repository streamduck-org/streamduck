use std::ffi::{c_char, CStr, CString};
use streamduck_core::print_test;

#[no_mangle]
pub unsafe extern "C" fn print_plugin(str: *const c_char) {
    // Receive
    let cstr = CStr::from_ptr(str);
    println!("Written from plugin: {}", cstr.to_string_lossy());

    // Send
    let my_str = CString::new("print_test asked by plugin").unwrap();
    print_test(my_str.into_raw());
}