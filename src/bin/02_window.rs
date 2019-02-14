#![cfg(windows)]
// Let's put this so that it won't open the console
#![windows_subsystem = "windows"]

/// Example 2: Simple window that responds to a click event
///
/// Clicking on the window shows a message dialog
/// Example build up from these examples and documents:
///      https://drywa.me/2017/07/02/simple-win32-window-with-rust/
///      https://gist.github.com/TheSatoshiChiba/6dd94713669efd1636efe4ee026b67af
///      http://www.winprog.org/tutorial/
use std::error::Error;
use std::ptr::null_mut;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::{GetModuleFileNameW, GetModuleHandleW};
use winapi::um::winuser::*;

// Get a win32 lpstr from a &str, converting u8 to u16 and appending '\0'
fn to_wstring(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// Handle leftbuttonclick
unsafe fn on_lbuttondown(hwnd: HWND) {
    let hinstance = GetModuleHandleW(null_mut());
    let mut name: Vec<u16> = Vec::with_capacity(MAX_PATH as usize);
    let read_len = GetModuleFileNameW(hinstance, name.as_mut_ptr(), MAX_PATH as u32);
    name.set_len(read_len as usize);
    // We could convert name to String using String::from_utf16_lossy(&name)
    MessageBoxW(
        hwnd,
        name.as_ptr(),
        to_wstring("This program is:").as_ptr(),
        MB_OK | MB_ICONINFORMATION,
    );
}

// Window procedure function to handle events
pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        WM_LBUTTONDOWN => {
            on_lbuttondown(hwnd);
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
    return 0;
}

// Declare class and instantiate window
fn create_main_window(name: &str, title: &str) -> Result<HWND, Box<dyn Error>> {
    let name = to_wstring(name);
    let title = to_wstring(title);

    unsafe {
        // Get handle to the file used to create the calling process
        let hinstance = GetModuleHandleW(null_mut());

        // Create and register window class
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance, // Handle to the instance that contains the window procedure for the class
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: COLOR_WINDOWFRAME as HBRUSH,
            lpszMenuName: null_mut(),
            lpszClassName: name.as_ptr(),
            hIconSm: LoadIconW(null_mut(), IDI_APPLICATION),
        };

        // Register window class
        if RegisterClassExW(&wnd_class) == 0 {
            MessageBoxW(
                null_mut(),
                to_wstring("Window Registration Failed!").as_ptr(),
                to_wstring("Error").as_ptr(),
                MB_ICONEXCLAMATION | MB_OK,
            );
            return Err("Window Registration Failed".into());
        };

        // Create a window based on registered class
        let handle = CreateWindowExW(
            0,                                // dwExStyle
            name.as_ptr(),                    // lpClassName
            title.as_ptr(),                   // lpWindowName
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, // dwStyle
            CW_USEDEFAULT,                    // Int x
            CW_USEDEFAULT,                    // Int y
            CW_USEDEFAULT,                    // Int nWidth
            CW_USEDEFAULT,                    // Int nHeight
            null_mut(),                       // hWndParent
            null_mut(),                       // hMenu
            hinstance,                        // hInstance
            null_mut(),                       // lpParam
        );

        if handle.is_null() {
            MessageBoxW(
                null_mut(),
                to_wstring("Window Creation Failed!").as_ptr(),
                to_wstring("Error!").as_ptr(),
                MB_ICONEXCLAMATION | MB_OK,
            );
            return Err("Window Creation Failed!".into());
        }

        Ok(handle)
    }
}

// Message handling loop
fn run_message_loop(hwnd: HWND) -> WPARAM {
    unsafe {
        let mut msg: MSG = std::mem::uninitialized();
        loop {
            // Get message from message queue
            if GetMessageW(&mut msg, hwnd, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            } else {
                // Return on error (<0) or exit (=0) cases
                return msg.wParam;
            }
        }
    }
}

fn main() {
    let hwnd = create_main_window("my_window", "Example window creation")
        .expect("Window creation failed!");
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }
    run_message_loop(hwnd);
}
