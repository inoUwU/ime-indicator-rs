use gpui::*;
use gpui_component::{button::*, *};
use shared::{AppConfig, load_config, save_config};

/// 設定画面のビューモデル
struct SettingsView {
    config: AppConfig,
    // TODO: 入力フィールドを追加（gpui-component の Input API を学習後）
}

impl SettingsView {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            config: load_config(),
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let config = self.config.clone();

        div()
            .v_flex()
            .p_4()
            .gap_4()
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("IME Indicator Settings"),
            )
            // 現在の設定値を表示
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Current Settings"),
                    )
                    .child(format!("Offset X: {}", config.overlay.offset_x))
                    .child(format!("Offset Y: {}", config.overlay.offset_y))
                    .child(format!("Width: {}", config.overlay.width))
                    .child(format!("Height: {}", config.overlay.height))
                    .child(format!(
                        "Display Duration: {}ms",
                        config.overlay.display_duration_ms
                    )),
            )
            // Buttons
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .mt_4()
                    .child(Button::new("save").primary().label("Save (WIP)").on_click(
                        move |_, _window, _cx| {
                            // TODO: 入力値から設定を保存
                            if let Err(e) = save_config(&config) {
                                eprintln!("Failed to save config: {}", e);
                            } else {
                                println!("Config saved successfully!");
                            }
                        },
                    ))
                    .child(
                        Button::new("cancel")
                            .label("Close")
                            .on_click(|_, window, _cx| {
                                window.remove_window();
                            }),
                    ),
            )
            // TODOメッセージ
            .child(
                div()
                    .mt_4()
                    .p_2()
                    .bg(gpui::hsla(0.15, 0.8, 0.5, 0.2))
                    .rounded_md()
                    .child("TODO: Input fields will be added here. Feel free to practice GPUI!"),
            )
    }
}

pub fn show() {
    let app = Application::new();

    app.run(move |cx| {
        gpui_component::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(400.0), px(400.0)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("IME Indicator Settings".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| SettingsView::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
