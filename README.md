# PopWin (Windows版PopClip - egui版)

## 概要

テキスト選択時にフローティングツールバーを表示し、コピー・ペーストなどのアクションを即座に実行できるWindows専用アプリケーションです。
パフォーマンス最優先で設計されており、**Rust + egui + Win32 API** を使用してマシン負荷を極限まで低減しています。

## 動作環境

- Windows 10 または Windows 11
- 開発環境: Rust (stable) + MSVC

## ビルド手順

### 1. Rustのインストール (Windows)

[rustup.rs](https://rustup.rs/) からインストーラーをダウンロードし、実行してください。
インストール時、**MSVC** (Visual Studio C++ Build Tools) が必要になります。

### 2. リポジトリのクローン

このプロジェクトフォルダ (`popwin`) に移動します。

```powershell
cd popwin
```

### 3. ビルドと実行

#### 開発ビルド (デバッグ用)
```powershell
cargo run
```

#### リリースビルド (最適化済み・軽量)
```powershell
cargo build --release
```
生成された実行ファイルは `target/release/popwin.exe` にあります。

## Features
- **Floating Toolbar**: Automatically appears near the mouse cursor after text selection (drag).
- **Clipboard Actions**: Copy, Cut, and Paste buttons.
- **AI Search**: Quick search on Perplexity.ai.
- **In-App Translation**: Real-time English-to-Japanese translation using Google Translate API (Async).
- **Cross-Platform**: Full logic on Windows, TUI-based simulation mode on macOS/Linux.

## How to Run

### Windows (Production)
1. Ensure Rust is installed.
2. Run `cargo run --release`.
3. Select any text in any application by dragging with the left mouse button.
4. The floating toolbar will appear. Click buttons to perform actions.
5. Click "Quit App" to exit.

### macOS/Linux (Simulation)
1. Run `cargo run`.
2. A TUI simulation will start, showing how the app reacts to events and performs translations.

## Architecture
- **Hooks**: Windows low-level mouse hooks (`SetWindowsHookExW`).
- **Automation**: UI Automation API for text extraction without clipboard interference.
- **GUI**: `egui` with `eframe` (WGPU backend).
- **Actions**: Async HTTP requests for translation and clipboard manipulation.

## PoC (概念実証) の機能と制限

現在のバージョン (v0.1 PoC) は以下の機能が実装されています：

- **グローバルマウスフック**: どのアプリ上でもドラッグ操作を検知します。
- **テキスト選択取得**:
  - 優先: UI Automation API (対応アプリのみ)
  - フォールバック: `Ctrl+C` シミュレーション (全アプリ対応だがクリップボードを使用)
- **アクション**:
  - コピー
  - ペースト
- **UI**:
  - 選択範囲の左側に縦列で表示されるアイコンツールバー
  - 透明・枠なしウィンドウ
  - Excel、PowerPoint等のOfficeアプリで動作

### v0.1 スコープ
v0.1では基本的なコピー・ペースト機能のみを実装し、動作の安定性とパフォーマンスを優先します。

### 既知の課題
- **フォーカス奪取**: ツールバーをクリックした際、元のウィンドウからフォーカスが外れ、ごく一部のアプリで選択範囲が解除される可能性があります（`WS_EX_NOACTIVATE` 等の調整が必要になる場合があります）。
- **誤検知**: ドラッグ操作の判定閾値 (5px, 100ms) は調整が必要です。

## ライセンス

MIT License
