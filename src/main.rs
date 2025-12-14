use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*, core::*,
};

// use tray_icon::{
//     Icon, TrayIconBuilder, TrayIconEvent,
//     menu::{Menu, MenuEvent},
// };

fn main() -> windows::core::Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        let window_class = s!("window");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("This is a sample window"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            None,
            None,
        )?;

        let mut message = MSG::default();

        while GetMessageA(&mut message, None, 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    } // unsafe {
    //     MessageBoxW(None, w!("hello, world!"), w!("nanka kaku"), MB_OK);k
    // }
    //

    // TODO: add icon from icon file.
    // TODO: add menu items to tray menu. quit , positon etc.
    // TODO: handle menu item click events.
    // TODO: finally make a ime indicator. e.g. show current ime mode use windows api and windows hooks.
    /*
        let tray_menu = Menu::new();
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("ime-indeicator")
            .with_icon(Icon::from_rgba(vec![255u8; 32 * 32 * 4], 32, 32).unwrap())
            .build()
            .unwrap();

        tray_icon.set_visible(true).unwrap();
        loop {
            if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                println!("tray event: {:?}", event);
            }

            if let Ok(event) = MenuEvent::receiver().try_recv() {
                println!("menu event: {:?}", event);
            }
        }
    */
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                println!("WM_PAINT");
                _ = ValidateRect(Some(window), None);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
