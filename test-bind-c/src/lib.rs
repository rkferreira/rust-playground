use std::os::raw::c_char;
use std::ffi::CStr;

// Exporting functions to C
//

#[no_mangle]
pub extern "C" fn echo_word(word: *const c_char) -> usize {
    unsafe {
        let c_str = CStr::from_ptr(word).to_str().unwrap();
        println!("Hello, world! {}", c_str);
    }
    0
}

#[no_mangle]
pub extern "C" fn hello() -> () {
    println!("Hello, world!");
}


// Tests
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_word() {
        let result = echo_word("test".as_ptr() as *const c_char);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_hello() {
        hello();
    }
}
