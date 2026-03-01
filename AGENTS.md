
# AGENTS.md

**AIエージェント向けワークスペースガイドライン**

## プロジェクト概要
Windows用Rustアプリ。IME状態をオーバーレイで表示。`windows`クレートでWinAPI、`tray-icon`でトレイ連携、設定UIはGPUIで別プロセス。

## アーキテクチャ
- `ime-indicator`：メインプロセス。IME検出・オーバーレイ描画・トレイ制御。
- `settings-gui`：設定UI（GPUI、別exe）。
- `shared`：設定構造体・ファイルI/O。

### 主な設計方針
- 設定UIは別プロセスで起動（高速なGPUI開発のため）
- グローバル状態は`OnceLock`＋`Arc<Mutex<T>>`で管理（`ime/mod.rs`参照）
- IME状態取得は50msデバウンス
- 設定変更は`config.toml`を介して反映

## コードスタイル
- Rust 2024 edition（nightly）
- フォーマット：`cargo fmt --all`
- 命名：snake_case（関数/変数）、PascalCase（型）
- Windows API呼び出しは必ず`unsafe`＋安全性コメント（例：`overlay.rs`）
- エラーは`windows::core::Result<T>`または`anyhow::Result<T>`で伝播、`log`で記録
- ログ：`env_logger`で初期化、`debug!`/`info!`/`warn!`/`error!`を使い分け

## ビルド・テスト
- ビルド：`cargo build`、リリース：`cargo build --release`
- 実行：`cargo run -p ime-indicator`、設定UI：`cargo run -p settings-gui`
- フォーマット：`cargo fmt --all`
- 静的解析：`cargo clippy --all`、チェック：`cargo check`
- テスト：`cargo test --all`（各モジュールに`#[cfg(test)]`）

## プロジェクト固有の慣習
- 長寿命の静的状態は`ime/mod.rs`で`OnceLock`＋`Arc<Mutex<T>>`で初期化
- Windowsメッセージは定数で定義（例：`WM_IME_STATUS_CHANGED`）
- 設定ファイルは`serde`＋`toml`でシリアライズ。パスはexe横（デバッグ時は`target/debug/config.toml`）
- デフォルト値は`#[derive(Default)]`で実装
- コミットメッセージはConventional Commits（`feat(overlay): ...`等）

## 外部連携・依存
- Windows API（`windows`クレート経由）
- `tray-icon`（トレイアイコン）
- `serde`/`toml`（設定）
- `log`/`env_logger`（ログ）
- `anyhow`（エラー）
- `ctrlc`（Ctrl+C対応）

## セキュリティ・注意点
- Windows API呼び出しは`unsafe`必須。ハンドルや座標は必ず検証
- フック処理は最小限に（パフォーマンス劣化防止）
- マルチモニタ座標は境界検証（現状50pxマージン固定、要改善）
- 設定ファイル読み込みはエラー処理必須（`toml::from_str`はpanicの可能性）

---
不明点や追加すべき内容があればフィードバックしてください。
