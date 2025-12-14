// use std::ffi::OsStr;
// use std::iter::once;
// use std::os::windows::ffi::OsStrExt;
// use std::ptr::null_mut;

// use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
// use windows::Win32::Graphics::Gdi::{
//     BeginPaint, CreateSolidBrush, EndPaint, FillRect, PAINTSTRUCT,
// };
// use windows::Win32::UI::WindowsAndMessaging::{
//     CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW,
//     GetMessageW, LWA_COLORKEY, LoadCursorW, MSG, PostQuitMessage, SW_SHOW,
//     SetLayeredWindowAttributes, ShowWindow, TranslateMessage, WM_DESTROY, WM_PAINT, WNDCLASSW,
//     WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
// };
// use windows::core::PCWSTR;

// fn to_pcwstr(s: &str) -> PCWSTR {
//     let v: Vec<u16> = OsStr::new(s).encode_wide().chain(once(0)).collect();
//     PCWSTR(v.as_ptr())
// }

// extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
//     unsafe {
//         match msg {
//             WM_PAINT => {
//                 let mut ps = PAINTSTRUCT::default();
//                 let hdc = BeginPaint(hwnd, &mut ps);
//                 // Fill background with magenta (color key) so it becomes transparent
//                 let brush = CreateSolidBrush(RGB(255, 0, 255));
//                 FillRect(hdc, &ps.rcPaint, brush);
//                 EndPaint(hwnd, &ps);
//                 LRESULT(0)
//             }
//             WM_DESTROY => {
//                 PostQuitMessage(0);
//                 LRESULT(0)
//             }
//             _ => DefWindowProcW(hwnd, msg, wparam, lparam),
//         }
//     }
// }

// /// Spawn a simple transparent, borderless overlay window on a new thread.
// /// The window uses a magenta color-key to make its background fully transparent.
// pub fn spawn_overlay() {
//     std::thread::spawn(|| unsafe {
//         let class_name = to_pcwstr("ime_indicator_overlay");

//         let mut wc = WNDCLASSW::default();
//         wc.hCursor = LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW).unwrap();
//         wc.lpszClassName = class_name;
//         wc.style = CS_HREDRAW | CS_VREDRAW;
//         wc.lpfnWndProc = Some(window_proc);

//         RegisterClassW(&wc);

//         // Create a popup (no border/title) window
//         let hwnd = CreateWindowExW(
//             WS_EX_LAYERED.0 | WS_EX_TOOLWINDOW.0,
//             class_name,
//             PCWSTR::null(),
//             WS_POPUP | WS_VISIBLE,
//             CW_USEDEFAULT,
//             CW_USEDEFAULT,
//             300,
//             100,
//             HWND(0),
//             None,
//             None,
//             null_mut(),
//         );

//         // Make the magenta color (RGB(255,0,255)) transparent
//         // COLORREF for magenta is 0x00FF00FF
//         SetLayeredWindowAttributes(hwnd, 0x00FF00FF, 0, LWA_COLORKEY);

//         ShowWindow(hwnd, SW_SHOW);

//         let mut msg = MSG::default();
//         while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
//             TranslateMessage(&msg);
//             DispatchMessageW(&msg);
//         }
//     });
// }
