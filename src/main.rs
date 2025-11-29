mod utils;
use windows::{Win32::UI::WindowsAndMessaging::*, core::*};

fn main() {
    unsafe {
        MessageBoxW(None, w!("hello, world!"), w!("nanka kaku"), MB_OK);
    }
}
