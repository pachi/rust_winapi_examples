// Example from https://github.com/retep998/winapi-rs

#[cfg(windows)]
use std::io::Error;

#[cfg(windows)]
// Get win32 lpstr from &str, converting u8 to u16 and appending '\0'
// See retep998's traits for a more general solution: https://users.rust-lang.org/t/tidy-pattern-to-work-with-lpstr-mutable-char-array/2976/2
fn to_wstring(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
fn print_message(msg: &str) -> Result<i32, Error> {
    use std::ptr::null_mut;
    use winapi::um::winuser::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

    let lp_text = to_wstring(msg);
    let lp_caption = to_wstring("Hello world window");
    let ret = unsafe {
        // https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-messageboxw
        MessageBoxW(
            null_mut(),          // hWnd
            lp_text.as_ptr(),    // text
            lp_caption.as_ptr(), // caption (dialog box title)
            MB_OK | MB_ICONINFORMATION,
        )
    };
    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(ret)
    }
}
#[cfg(windows)]
fn main() {
    print_message("Hello, world!").unwrap();
}


#[cfg(not(windows))]
fn main() {
    println!("Hello world only works on windows!");
}
