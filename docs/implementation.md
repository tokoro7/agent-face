# Implementation Status

## プロジェクト構成

```
agent-face/
├── Cargo.toml
├── characters/
│   └── cat.toml              # 組み込みキャラクター定義
├── src/
│   ├── main.rs               # エントリポイント、コマンドディスパッチ
│   ├── cli.rs                # clap によるCLI定義
│   ├── state.rs              # FaceState enum、StateMachine
│   ├── character.rs          # キャラクター TOML のロード・バリデーション
│   ├── watcher.rs            # notify による state file 監視
│   ├── renderer.rs           # crossterm によるターミナル描画
│   └── setup.rs              # agent-face setup コマンドの実装
├── docs/
│   ├── about.md              # プロジェクトのビジョンと UX 設計
│   ├── spec.md               # 技術仕様
│   ├── design-decisions.md   # 設計判断の記録
│   ├── dependencies.md       # 依存クレートの説明
│   └── implementation.md     # 本ファイル
└── _ref/                     # 参照実装（Python版）
```

## 実装済み機能

### CLI (`cli.rs`)

| コマンド | 説明 |
|---|---|
| `agent-face` | レンダラーを起動 |
| `agent-face set <STATE>` | state file に状態を書き込む |
| `agent-face setup <AGENT>` | エージェント連携をセットアップ |

オプション: `--character`, `--state-file`, `--characters-dir`

### State Machine (`state.rs`)

- 6 状態: idle, thinking, writing, error, success, listening
- `FromStr` / `Display` 実装で文字列と相互変換
- `StateMachine::tick()` で success → idle の 3 秒タイムアウト遷移

### Character Loader (`character.rs`)

- TOML ファイルからキャラクター定義を読み込み
- バリデーション: 全 6 state 定義の存在、フレーム数、speed_ms、カラー名
- `include_str!` で `characters/cat.toml` を組み込みフォールバックとして持つ
- 外部ディレクトリ (`~/.config/agent-face/characters/`) からの読み込みを優先

### State Watcher (`watcher.rs`)

- `notify` クレートで state file の親ディレクトリを監視
- ファイル変更 → `FaceState` にパースして通知
- ファイル削除 → `FileDeleted` イベントを通知
- ノンブロッキング (`try_recv`)

### Renderer (`renderer.rs`)

- crossterm の alternate screen + raw mode で描画
- 50ms tick でキーボード入力・state 変更・アニメーションを処理
- レイアウト: ヘッダー（キャラクター名）、ASCII フレーム（中央配置）、状態バッジ、フッター
- キーボード: `1`-`6` で状態切り替え、`c` でキャラクター切り替え、`q` / Ctrl+C で終了

### Setup (`setup.rs`)

- `agent-face setup claude-code` で Claude Code フック連携を一括セットアップ
- `~/.config/agent-face/adapters/claude-code/face-state.sh` を配置（chmod 755）
- `~/.claude/settings.json` の `hooks` に 9 イベントを**追記**（既存設定を保持）
- 同じフックが登録済みならスキップ（重複防止）
- フックスクリプトは `agent-face set <state>` を呼び出す（Python 非依存）

### State File 書き込み (`main.rs` 内 `cmd_set`)

- アトミック書き込み（temp file + rename）
- 親ディレクトリの自動作成
- 不正な state 名はエラー（exit code 1）

### パニック時のターミナル復元 (`main.rs`)

- `std::panic::set_hook` で raw mode 解除、カーソル復元、alternate screen 離脱

## 未実装

- `agent-face setup` 以外のエージェント対応（codex 等）
- config ファイル (`~/.config/agent-face/config.toml`) の読み込み
- 自動起動モード（`auto_launch` 設定）
- カラーリング（文字種別ごとの着色。現状は状態カラー単色）
- パーティクルアニメーション（泡、星）
- まばたき・呼吸アニメーション
- ターミナルリサイズ対応
- box キャラクター（cat のみ実装）
