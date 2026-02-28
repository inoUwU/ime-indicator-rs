use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::color_picker::{ColorPicker, ColorPickerState};
use gpui_component::label::Label;
use gpui_component::select::{Select, SelectEvent, SelectItem, SelectState};
use gpui_component::{Theme, ThemeRegistry};
use gpui_component::{button::*, *};
use shared::{AppConfig, DisplayPosition, load_config, save_config};

/// DisplayPosition のラッパー型（SelectItem トレイト実装用）
#[derive(Clone, Debug, PartialEq)]
struct PositionItem(DisplayPosition);

impl SelectItem for PositionItem {
    type Value = PositionItem;

    fn title(&self) -> SharedString {
        match self.0 {
            DisplayPosition::TopLeft => "TopLeft".into(),
            DisplayPosition::TopRight => "TopRight".into(),
            DisplayPosition::Center => "Center".into(),
            DisplayPosition::BottomLeft => "BottomLeft".into(),
            DisplayPosition::BottomRight => "BottomRight".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

/// DisplayPosition の選択肢
fn create_position_items() -> Vec<PositionItem> {
    vec![
        PositionItem(DisplayPosition::TopLeft),
        PositionItem(DisplayPosition::TopRight),
        PositionItem(DisplayPosition::Center),
        PositionItem(DisplayPosition::BottomLeft),
        PositionItem(DisplayPosition::BottomRight),
    ]
}

fn position_to_index(pos: &DisplayPosition) -> usize {
    match pos {
        DisplayPosition::TopLeft => 0,
        DisplayPosition::TopRight => 1,
        DisplayPosition::Center => 2,
        DisplayPosition::BottomLeft => 3,
        DisplayPosition::BottomRight => 4,
    }
}

/// 設定画面のビューモデル
struct SettingsView {
    config: Arc<Mutex<AppConfig>>,
    position_select: Entity<SelectState<Vec<PositionItem>>>,
    color_enable_select: Entity<ColorPickerState>,
    color_disable_select: Entity<ColorPickerState>,
}

impl SettingsView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let loaded_config = load_config();

        // 設定を共有状態でラップ
        let config = Arc::new(Mutex::new(loaded_config.clone()));

        // DisplayPosition の選択肢を Vec として作成
        let items = create_position_items();

        // 現在の設定値に対応するインデックスを取得
        let selected_index = position_to_index(&loaded_config.overlay.display_pos);

        let position_select =
            cx.new(|cx| SelectState::new(items, Some(IndexPath::new(selected_index)), window, cx));

        let color_enable_select = cx.new(|cx| ColorPickerState::new(window, cx));
        let color_disable_select = cx.new(|cx| ColorPickerState::new(window, cx));

        // 選択変更時のイベントを購読
        let config_clone = Arc::clone(&config);
        cx.subscribe_in(
            &position_select,
            window,
            move |_view, _state, event, _window, _cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    println!("Selected position: {:?}", value);
                    if let Ok(mut cfg) = config_clone.lock() {
                        cfg.overlay.display_pos = value.0.clone();
                    }
                }
            },
        )
        .detach();

        Self {
            config,
            position_select,
            color_enable_select,
            color_disable_select,
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let config_clone = Arc::clone(&self.config);

        div()
            .v_flex()
            .p_4()
            .gap_4()
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child(Label::new("IME Indicator Settings").text_2xl()),
            )
            // 現在の設定値を表示
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(Label::new("Current Settings").text_xl()),
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .child(Label::new("Display Position:"))
                            .child(Select::new(&self.position_select).w(px(150.0))),
                    )
                    // TODO：色を選択する
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .child(Label::new("Color for IME enable:"))
                            .child(ColorPicker::new(&self.color_enable_select)),
                    )
                    // TODO：色を選択する
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .child(Label::new("Color for IME disable:"))
                            .child(ColorPicker::new(&self.color_disable_select)),
                    ),
            )
            // Buttons
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .mt_4()
                    .child(Button::new("save").primary().label("Save").on_click(
                        move |_, _window, _cx| {
                            // 現在の設定を保存
                            if let Ok(cfg) = config_clone.lock() {
                                if let Err(e) = save_config(&cfg) {
                                    eprintln!("Failed to save config: {}", e);
                                } else {
                                    println!("Config saved successfully: {:?}", cfg);
                                }
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
            .child(
                // オーバーレイ要素
                div().absolute().top_10().left(px(10.0)).child(
                    // TODO: オーバーレイのアニメーション表示
                    Alert::success("success-alert", "Your operation completed successfully.")
                        .title("Success!"),
                ),
            )
    }
}

pub fn show() {
    let theme_name = SharedString::from("Ayu Light");
    let app = Application::new();

    app.run(move |cx| {
        gpui_component::init(cx);
        if let Err(err) = ThemeRegistry::watch_dir(
            PathBuf::from("./crates/settings-gui/themes"),
            cx,
            move |cx| {
                if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
                    Theme::global_mut(cx).apply_config(&theme);
                }
            },
        ) {
            eprintln!("Failed to watch themes directory: {}", err);
        }

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(500.0), px(500.0)),
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
