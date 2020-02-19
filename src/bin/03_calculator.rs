#![cfg(windows)]
// Let's put this so that it won't open the console
#![windows_subsystem = "windows"]

// Example from https://www.codeproject.com/Tips/1070559/Calculator-Interface-Design-In-Rust-Language

use std::error::Error;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi;
use winapi::um::winuser::*;

use std::ptr::null_mut;

// Global Model to keep state
struct Model {
    op1: i32,
    op2: i32,
    op: &'static str,
    hwnd_display: HWND,
}

static mut MODEL: Model = Model {
    op1: 0,
    op2: 0,
    op: "",
    hwnd_display: 0 as HWND,
};

// Get win32 lpstr from &str, converting u8 to u16 and appending '\0'
fn to_wstring(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// Handle click on clear button
unsafe fn on_clear_click() {
    MODEL.op1 = 0;
    MODEL.op2 = 0;
    MODEL.op = "";

    SetWindowTextW(MODEL.hwnd_display, to_wstring("0").as_ptr());
}

// Handle click on number buttons
unsafe fn on_numbers_click(id: u32) {
    // Here our numbers_click event codes
    let num: i32 = id as i32 - 101;
    let val = match MODEL.op {
        "" => {
            MODEL.op1 = MODEL.op1 * 10 + num;
            MODEL.op1
        }
        _ => {
            MODEL.op2 = MODEL.op2 * 10 + num;
            MODEL.op2
        }
    };
    SetWindowTextW(MODEL.hwnd_display, to_wstring(&val.to_string()).as_ptr());
}

// Handle click on operator buttons
unsafe fn on_operators_click(id: u32) {
    MODEL.op = match id {
        140 => "+",
        141 => "-",
        142 => "x",
        143 => "/",
        144 => "%",
        _ => panic!("Unexpected operator"),
    };
    SetWindowTextW(MODEL.hwnd_display, to_wstring(MODEL.op).as_ptr());
}

// Handle click on equal sign button
unsafe fn on_equal_click() {
    MODEL.op1 = match MODEL.op {
        "+" => MODEL.op1 + MODEL.op2,
        "-" => MODEL.op1 - MODEL.op2,
        "x" => MODEL.op1 * MODEL.op2,
        "/" => {
            if MODEL.op2 != 0 {
                (MODEL.op1 as f32 / MODEL.op2 as f32) as i32
            } else {
                on_clear_click();
                0
            }
        }
        "%" => (MODEL.op1 as f32 * (MODEL.op2 as f32 / 100.0)) as i32,
        "" => MODEL.op1,
        _ => panic!("Unexpected operator"),
    };
    MODEL.op2 = 0;
    MODEL.op = "";
    SetWindowTextW(
        MODEL.hwnd_display,
        to_wstring(&MODEL.op1.to_string()).as_ptr(),
    );
}

// Window procedure (main window)
pub unsafe extern "system" fn window_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(h_wnd);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        WM_CTLCOLORSTATIC => {
            if MODEL.hwnd_display == (l_param as HWND) {
                // Change display label control background color to white
                let h_brush = wingdi::CreateSolidBrush((255 | (255 << 8)) | (255 | (255 << 16)));
                return h_brush as LRESULT;
            };
        }
        WM_COMMAND => {
            // Detect buttons click event
            match w_param {
                101..=110 => on_numbers_click(w_param as u32), // Numbers 0-9
                120 => on_clear_click(),                       // Clear
                130 => on_equal_click(),                       // Equal
                140..=144 => on_operators_click(w_param as u32), // Operators
                _ => panic!("Unknown button"),
            }
        }
        _ => return DefWindowProcW(h_wnd, msg, w_param, l_param),
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
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: 16 as HBRUSH,
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
            0,
            name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            366,
            400,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );

        if handle.is_null() {
            MessageBoxW(
                null_mut(),
                to_wstring("Main Window Creation Failed!").as_ptr(),
                to_wstring("Error!").as_ptr(),
                MB_ICONEXCLAMATION | MB_OK,
            );
            return Err("Window Creation Failed!".into());
        }

        Ok(handle)
    }
}

// Build GUI elements inside main window
unsafe fn init_interface(h_wnd: HWND) {
    // Entry for Display
    MODEL.hwnd_display = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        to_wstring("static").as_ptr(),
        to_wstring("0").as_ptr(),
        WS_CHILD | WS_VISIBLE | 2, /*SS_RIGHT*/
        46,
        20,
        256,
        60,
        h_wnd,
        340 as HMENU,
        0 as HINSTANCE, // usa GetModuleHandle()
        null_mut(),
    );

    // Button for number 0
    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("0").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        46,  // x
        264, // y
        148,
        32,
        h_wnd,
        101 as HMENU, // ID = 101
        0 as HINSTANCE,
        null_mut(),
    );

    // Buttons for numbers from 1 to 9
    let mut x = 46;
    let mut y = 210;

    for btn_num_id in 102..=110 {
        let txt = (btn_num_id - 101).to_string();

        CreateWindowExW(
            0,
            to_wstring("button").as_ptr(),
            to_wstring(&txt).as_ptr(),
            WS_CHILD | WS_VISIBLE,
            x,
            y,
            40,
            32,
            h_wnd,
            btn_num_id as HMENU, // ID = 101 + i
            0 as HINSTANCE,
            null_mut(),
        );

        x = x + 54;

        if (btn_num_id - 101) % 3 == 0 {
            x = 46;
            y = y - 54;
        }
    }

    x = 208;
    y = 102;

    // Buttons for operators +, - , x, /, %
    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("+").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        40,
        32,
        h_wnd,
        140 as HMENU, // ID = 113
        0 as HINSTANCE,
        null_mut(),
    );

    y = y + 54;

    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("-").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        40,
        32,
        h_wnd,
        141 as HMENU, // ID
        0 as HINSTANCE,
        null_mut(),
    );

    y = y + 54;

    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("x").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        40,
        32,
        h_wnd,
        142 as HMENU,
        0 as HINSTANCE,
        null_mut(),
    );

    y = y + 54;

    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("/").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        40,
        32,
        h_wnd,
        143 as HMENU,
        0 as HINSTANCE,
        null_mut(),
    );

    x = 262;
    y = 102;

    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("C").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        40,
        32,
        h_wnd,
        120 as HMENU, // ID
        0 as HINSTANCE,
        null_mut(),
    );

    y = y + 54;

    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("%").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        40,
        32,
        h_wnd,
        144 as HMENU, // ID
        0 as HINSTANCE,
        null_mut(),
    );

    y = y + 54;

    // Equal sign button
    CreateWindowExW(
        0,
        to_wstring("Button").as_ptr(),
        to_wstring("=").as_ptr(),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        40,
        86,
        h_wnd,
        130 as HMENU, // ID
        0 as HINSTANCE,
        null_mut(),
    );
}

// Message handling loop
fn run_message_loop(hwnd: HWND) -> WPARAM {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();

        loop {
            // Get message from message queue
            if GetMessageW(&mut msg, hwnd, 0, 0) > 0 {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            } else {
                // Return on error (<0) or exit (=0) cases
                return msg.wParam;
            }
        }
    }
}

fn main() {
    let hwnd = create_main_window("my_window", "Simple Calculator Interface In Rust")
        .expect("Window creation failed!");
    unsafe {
        init_interface(hwnd);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }
    run_message_loop(hwnd);
}
