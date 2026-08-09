# Rapid Type - Copilot Instructions

## プロジェクト概要

**Rapid Type** は以下を含む Rust ワークスペースです：
- **typing-engine**: NFA ベースのタイピング入力エンジン（コアライブラリ）
- **app-egui**: egui フレームワークで構築されたGUIアプリケーション
- **app-tauri**: 将来の Tauri UI 実装（オプション）

ビジネスロジック（エンジン）とプレゼンテーション（UI）を分離したモジュール構造のタイピングアプリケーションです。

## ビルド、テスト、リント コマンド

### ビルド

```bash
# ワークスペース全体をビルド
cargo build

# 特定のパッケージをビルド
cargo build -p app-egui
cargo build -p typing-engine

# リリースビルド
cargo build --release
```

### テスト

```bash
# ワークスペース内のすべてのテストを実行
cargo test

# 特定のパッケージのテストを実行
cargo test -p typing-engine

# テスト名で単一テストを実行
cargo test test_name

# 出力付きでテストを実行
cargo test -- --nocapture
```

### チェック/リント

```bash
# ビルドなしでコードをチェック
cargo check

# コードをフォーマット
cargo fmt

# clippy（リンター）を実行
cargo clippy
```

## ワークスペース構造

### メンバー

- **typing-engine**: NFA ベースの入力処理エンジン

- **app-egui**: egui UI アプリケーション

### 主要な規約

- 共有リゾルバーバージョン 3 を使用した Cargo ワークスペース
- すべてのパッケージで Edition 2024 を使用
- `typing-engine` は `EngineInputResult` enum で結果を返す
- 複数の UI アプリは同じ `typing-engine` に依存（疎結合設計）
- UTF-8 文字数で計算（バイト数ではなく）

## typing-engine の使用方法

### 基本的な使用例

```rust
use typing_engine::TypingEngine;

let mut engine = TypingEngine::new("か");
match engine.input('k') {
    EngineInputResult::Accepted => println!("入力受け入れ"),
    EngineInputResult::Completed => println!("完了！"),
    EngineInputResult::Rejected => println!("不正な入力"),
    _ => {}
}
```

### 主な API

- `TypingEngine::new(reading: &str)` - 新規エンジン作成
- `engine.input(c: char) -> EngineInputResult` - 文字入力処理
- `engine.get_guide() -> String` - 次の入力候補を表示
- `engine.completed_char_count() -> usize` - 完了した文字数
- `engine.is_completed() -> bool` - 完了状態確認

### 対応している日本語

- 基本ひらがな：あいうえお ～ わをん
- 濁音・半濁音：がぎぐげご ～ ぽ
- 拗音：きゃきゅきょ ～ りょ
- 複合パターン：複数のローマ字オプション（ka/ca など）

## 開発ノート

- ワークスペースはルートに共有の `Cargo.lock` を使用
- ビルド出力は `target/` ディレクトリに出力（`.gitignore` に含まれている）
- NFA は build_from_tables() で全ローマ字パターンから初期化
- トライ木構造により効率的な状態遷移を実現
- Edition 2024 には最新の Rust ツールチェーンが必要

# Cargo.toml 更新
[workspace]
members = ["app-egui", "typing-engine"]

# app-tauri/Cargo.toml に依存を追加
[dependencies]
typing-engine = { path = "../typing-engine" }
```

各 UI アプリは独立したバイナリとして `target/debug/` に出力される。
コードを生成する前に、必ずcontext7やtavilyでライブラリのAPIを確認してください。

