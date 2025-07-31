# System Tray

System Tray は、クロスプラットフォームのシステムトレイアイコン機能を Rust から提供するライブラリです。Qt フレームワークをバックエンドとして利用し、システムトレイアイコンの作成、メニューアイテムの追加、アイコンの設定、クリックイベントやメニューアイテム選択イベントのハンドリングを可能にします。

## 特徴

- **クロスプラットフォーム**: Windows, macOS, Linux (Qt がサポートする環境) で動作します。
- **シンプルな API**: 直感的で使いやすい Rust API を提供します。
- **メニュー機能**: システムトレイアイコンにコンテキストメニューを追加できます。
- **イベントハンドリング**: トレイのクリック、ダブルクリック、メニューアイテムのクリックイベントをサポートします。

## 使い方

### 依存関係

`Cargo.toml` に以下の依存関係を追加してください。Qt ライブラリがシステムにインストールされている必要があります。

```toml
[dependencies]
system-tray = { version = "0.1.0", git="https://github.com/The-Infinitysrust.system-tray" } # 最新のバージョンに合わせてください
```

### コード例

基本的な使用方法は以下の通りです。

```rust
use rust_qt_system_tray::{SystemTray, Menu, Event};
use std::thread;
use std::time::Duration;

// アプリケーションのアイコンデータ (例: 1x1ピクセルの透明なPNG)
// 実際のアプリケーションでは、適切なアイコンデータに置き換えてください。
const ICON_DATA: &[u8] = include_bytes!("icon.svg");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tray = SystemTray::new("MyOrg", "MyTrayApp")
        .icon(ICON_DATA, "svg")
        .menu(Menu::new("Open App".to_string(), "open_app".to_string()))
        .menu(Menu::new("Settings".to_string(), "settings".to_string()))
        .menu(Menu::new("Quit".to_string(), "quit".to_string()));

    tray.start(); // Qt イベントループを別スレッドで開始

    println!("System tray application started. Polling for events...");

    loop {
        match tray.poll_event()? {
            Event::None => {
                // イベントがない場合は少し待機してCPU使用率を抑える
                thread::sleep(Duration::from_millis(100));
            },
            Event::TrayClicked => {
                println!("System tray clicked!");
            },
            Event::TrayDoubleClicked => {
                println!("System tray double-clicked!");
            },
            Event::MenuItemClicked(id) => {
                println!("Menu item clicked: {}", id);
                if id == "quit" {
                    println!("Quit menu item clicked. Exiting...");
                    break;
                }
            },
        }
    }

    // `main`関数の終わりで `tray` がスコープを抜けると `drop` が呼ばれ、Qtアプリが停止します。
    // 明示的に停止したい場合は `tray.stop()` を呼び出すことも可能です。
    Ok(())
}
```

### ビルドと実行

プロジェクトをビルドして実行します。

```bash
cargo run
```

## 開発

このライブラリは、Rust と Qt C++ のバインディングを利用しています。Qt の開発環境が正しく設定されていることを確認してください。

### 必要なもの

- Rust toolchain (stable)
- Qt 6 開発ライブラリ

## 貢献

バグ報告、機能リクエスト、プルリクエストは大歓迎です。

## ライセンス

このプロジェクトは [MIT License](https://www.google.com/search?q=LICENSE) の下でライセンスされています。

