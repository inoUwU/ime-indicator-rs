use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DrawTextW, EndPaint,
        FillRect, PAINTSTRUCT, SetBkMode, TRANSPARENT,
    },
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::*,
    core::*,
};
mod utils;

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

        // 画面サイズを取得して中央に配置
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        let window_width = 300;
        let window_height = 100;
        let x = (screen_width - window_width) / 2;
        let y = (screen_height - window_height) / 2;

        let hwnd = CreateWindowExA(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            window_class,
            s!("IME Indicator Overlay"),
            WS_POPUP | WS_VISIBLE,
            x,             // x position (center)
            y,             // y position (center)
            window_width,  // width
            window_height, // height
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // 半透明設定（アルファ値128で50%透明）
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 128, LWA_ALPHA)?;

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
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut ps);

                // グレー色で背景を塗りつぶし（半透明になる）
                let brush = CreateSolidBrush(COLORREF(0x00808080)); // RGB(128, 128, 128)
                FillRect(hdc, &ps.rcPaint, brush);

                // テキストの背景を透明に設定
                SetBkMode(hdc, TRANSPARENT);

                let mut rect = RECT::default();
                let _ = GetClientRect(window, &mut rect);

                // 「こんにちは」を中央に表示
                let text = "こんにちは";
                let mut text_wide: Vec<u16> = text.encode_utf16().collect();
                DrawTextW(
                    hdc,
                    &mut text_wide,
                    &mut rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );

                let _ = EndPaint(window, &ps);
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
