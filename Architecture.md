# Architecture

```mermaid
graph LR
  %% コンポーネント
  Installer["Installer / Hook Setter"]
  MainApp["IME Indicator (Main App)"]
  UI["UI / Tray Indicator"]

  subgraph Targets["Target Processes (例)"]
    direction TB
    ProcB["Process B"]
    ProcC["Process C"]
    ProcB --> HookB["Hook DLL (in Proc B)\nNamed Pipe Client"]
    ProcC --> HookC["Hook DLL (in Proc C)\nNamed Pipe Client"]
  end

  %% フロー: フック登録（ラベルはクォートして特殊文字を回避）
  Installer -->|"SetWindowsHookEx(WH_KEYBOARD/WH_KEYBOARD_LL, DLLPath)"| HookDLL["Hook DLL (DLL ファイル)"]
  Installer --> MainApp
  note right of Installer
    Installer は SetWindowsHookEx を呼び出し、
    フック DLL をターゲットにロードする
  end note

  %% フロー: イベント伝搬（名前付きパイプ経由）
  HookB -->|"キーボードイベント (CAPTURE)"| HookB_Client["Named Pipe Client"]
  HookC -->|"キーボードイベント (CAPTURE)"| HookC_Client["Named Pipe Client"]
  HookB_Client -->|"Write(\"\\\\.\\\\pipe\\\\ime_indicator_{user_or_sid}\")"| MainApp
  HookC_Client -->|"Write(\"\\\\.\\\\pipe\\\\ime_indicator_{user_or_sid}\")"| MainApp
  MainApp -->|"Ack/Command"| HookB_Client
  MainApp -->|"Ack/Command"| HookC_Client

  %% メインアプリ -> UI
  MainApp --> UI
  UI -->|表示更新| MainApp

  %% 補助情報
  classDef note fill:#f9f,stroke:#333,stroke-width:1px;
  class HookDLL,HookB_Client,HookC_Client note;

  %% 再接続/バッファリングの注記
  subgraph Notes["運用上の注意"]
    direction TB
    PipeName["パイプ名: \\\\.\\pipe\\ime_indicator_{user_or_sid}"]
    Security["セキュリティ: アクセス制御（ユーザースコープ推奨）"]
    Reconnect["クライアント側は未接続時にバッファし再接続を試行"]
  end
  MainApp --- PipeName
  MainApp --- Security
  HookB_Client --- Reconnect
  HookC_Client --- Reconnect
```
