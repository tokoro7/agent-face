# Dependencies

| クレート | バージョン | 用途 |
|----------|-----------|------|
| crossterm | 0.28 | クロスプラットフォームのターミナル制御。raw mode、カーソル操作、キーボード入力、ANSI カラー出力を提供する。Windows/macOS/Linux で同一 API が使える |
| notify | 7 | ファイルシステムイベントの監視。OS ネイティブの仕組み（Linux: inotify、macOS: kqueue、Windows: ReadDirectoryChangesW）を抽象化する。state file の変更検知に使用 |
| clap | 4 | CLI 引数パーサー。`derive` feature でstructに属性マクロを付けるだけでサブコマンドやオプションを定義できる |
| serde | 1 | シリアライズ/デシリアライズフレームワーク。`derive` feature で `#[derive(Deserialize)]` が使える。toml クレートと組み合わせて設定・キャラクターファイルを読み込む |
| toml | 0.8 | TOML ファイルのパーサー。serde と統合されており、TOML → Rust 構造体への変換を行う |
