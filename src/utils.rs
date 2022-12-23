use std::ffi::OsString;
use std::os::windows::prelude::OsStringExt;

// Convert a UCS2 wide char string to a Rust String
pub fn wstr(slice: &[u16]) -> String {
    let len = slice.iter().position(|&c| c == 0).unwrap_or(0);
    OsString::from_wide(&slice[0..len])
        .to_string_lossy()
        .to_string()
}
