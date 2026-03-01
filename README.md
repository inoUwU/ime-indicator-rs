<img src="./assets/icon.png" alt="IME Indicator" width="100"/>

# IME Indicator

WindowsのIME（インプットメソッドエディタ）状態を視覚的に表示するオーバーレイインジケーターアプリケーション。

## 機能

- IMEモード切り替え時にオーバーレイインジケーターを自動表示（3秒間）
- システムトレイに常駐
- リアルタイムIME状態検出（キーボードフック、ウィンドウイベントフック）
- 「あ」（日本語入力モード）/ 「A」（英語入力モード）の表示

## ビルド

```powershell
cargo build --release
```

## 実行

```powershell
cargo run --release
```

デバッグ時は `Taskfile.yml` のタスク実行を推奨します。

```powershell
task run
```

`task run` は `settings-gui` を先に debug ビルドしてから `ime-indicator` を起動するため、トレイメニューから開く設定画面の反映漏れを防げます。

設定GUIを単体で起動する場合：

```powershell
task run:settings
```

または、ビルド後の実行ファイルを直接起動：

```powershell
.\target\release\ime-indicator-rs.exe
```

## 使い方

1. アプリケーションを起動すると、システムトレイにアイコンが表示されます
2. IMEモードを切り替えると、画面右上にインジケーターが3秒間表示されます
3. 終了する場合は、システムトレイアイコンを右クリックして「Quit」を選択

## Windowsスタートアップに登録

Windowsの起動時に自動的にアプリケーションを起動するには、以下の方法があります。

1. `Win + R` を押して「ファイル名を指定して実行」を開く
2. `shell:startup` と入力してEnterキーを押す
3. スタートアップフォルダが開くので、`ime-indicator-rs.exe` のショートカットをこのフォルダに配置する

## 技術スタック

- Rust
- windows-rs（Windows API バインディング）
- tray-icon（システムトレイ統合）

## ライセンス

このプロジェクトのライセンス情報については、リポジトリを参照してください。
