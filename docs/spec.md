# agent-face Specification

## Overview

agent-face は、AI コーディングエージェント（Claude Code、Codex 等）と並行して別ターミナルで動作する ASCII アニメーション顔表示ツール。エージェントの動作状態に連動して表情がリアルタイムに変化し、AI の内部状態を視覚的にフィードバックする。

**実装言語**: Rust
**参照実装**: `_ref/agent-face-python/`

### 参照実装からの主な改善点

| 問題 | 参照実装 | 本実装 |
|------|---------|--------|
| エージェント依存 | Claude Code のフック機構専用 | エージェント非依存の state protocol |
| ASCII アートのハードコーディング | Python ファイル内に直書き | 外部 TOML ファイルで定義 |
| OS 依存 | POSIX 専用（`termios`/`tty`） | `crossterm` によるクロスプラットフォーム対応 |
| ファイルポーリング | 50ms 間隔でファイルを読み込み | `notify` クレートによるファイル変更検知 |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Agent Layer（各エージェント側が責任を持つ）                   │
│                                                             │
│  claude-code hook ──▶ adapters/claude-code/face-state.sh   │
│  any tool         ──▶ agent-face set <state>               │
│  codex hook       ──▶ adapters/codex/ (将来)               │
└────────────────────────────┬────────────────────────────────┘
                             │ write (plain text)
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  State File（エージェント非依存プロトコル）                    │
│  ~/.local/share/agent-face/state                           │
│  フォーマット: プレーンテキスト ("thinking\n" 等)             │
└────────────────────────────┬────────────────────────────────┘
                             │ watch (inotify / kqueue / ReadDirectoryChangesW)
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  agent-face（Rust バイナリ）                                 │
│  StateWatcher ──▶ StateMachine ──▶ Renderer                │
│                        ▲                                    │
│                  Keyboard Input                             │
└─────────────────────────────────────────────────────────────┘
```

### コンポーネント

| コンポーネント | 役割 |
|---|---|
| **Renderer** | ターミナル上に ASCII 顔を描画・アニメーション |
| **StateMachine** | 状態遷移を管理。タイムアウト処理（success → idle 等）を内包 |
| **StateWatcher** | state file の変更を検知し StateMachine へ通知 |
| **CharacterLoader** | TOML ファイルからキャラクター定義を読み込む |
| **Config** | `~/.config/agent-face/config.toml` から設定を読み込む |
| **Adapter Scripts** | 各エージェントのフックを state file への書き込みに変換 |

---

## State Protocol

### State File

- **パス（デフォルト）**: `~/.local/share/agent-face/state`
- **フォーマット**: プレーンテキスト。改行を含む state 名のみ
- **例**: `thinking\n`

state file のパスは設定ファイルまたは `--state-file` オプションで上書き可能。

### States

6 つの状態を持つ。

| State | 説明 | アニメーション速度 | カラー |
|---|---|---|---|
| `idle` | 待機中 | 800ms | Peach |
| `thinking` | 思考中 | 350ms | Amber |
| `writing` | 書き込み中 | 250ms | Cyan |
| `error` | エラー発生 | 500ms | Red |
| `success` | タスク完了 | 300ms | Green |
| `listening` | ユーザー入力待ち | 400ms | Lavender |

### 状態遷移

state file への書き込みによる遷移に加え、StateMachine 内でタイムアウト遷移を定義する。

```
(state file) "idle"      ──▶ idle
(state file) "thinking"  ──▶ thinking
(state file) "writing"   ──▶ writing
(state file) "error"     ──▶ error
(state file) "success"   ──▶ success ──(3s 後)──▶ idle
(state file) "listening" ──▶ listening
```

不明な state 名が書き込まれた場合は無視する（現在の状態を維持）。

---

## CLI Interface

```
agent-face [OPTIONS] [COMMAND]
```

### サブコマンド

| コマンド | 説明 |
|---|---|
| `agent-face` | (サブコマンドなし) レンダラーを起動する（セットアップ未実行時はエラーメッセージを表示） |
| `agent-face set <STATE>` | state file に状態を書き込む（エージェント統合用） |
| `agent-face setup <AGENT>` | エージェント連携をセットアップする（例: `claude-code`） |

### オプション

| オプション | デフォルト | 説明 |
|---|---|---|
| `--character <NAME>` | config で指定した値、なければ `cat` | 起動時のキャラクター |
| `--state-file <PATH>` | `~/.local/share/agent-face/state` | state file のパスを上書き |
| `--characters-dir <PATH>` | `~/.config/agent-face/characters/` | キャラクター TOML の検索ディレクトリ |

### `set` サブコマンド

```bash
agent-face set thinking
```

- state file に指定した状態名を書き込む
- state file の親ディレクトリが存在しない場合は作成する
- 無効な state 名を指定した場合はエラーを返す（exit code 1）

---

## Character File Format

キャラクターは TOML ファイルで定義する。

### 検索順序

1. `--characters-dir` で指定したディレクトリ
2. `~/.config/agent-face/characters/`
3. バイナリに組み込まれた組み込みキャラクター（フォールバック）

同名のキャラクターが複数の場所に存在する場合、上位のものが優先される。

### フォーマット

```toml
name = "cat"
display_name = "Neko"

[states.idle]
color = "peach"
speed_ms = 800
frames = [
    """
   /\_/\
  ( ●  ● )
  (  ω   )
  """,
    """
   /\_/\
  ( -  - )
  (  ω   )
  """,
]

[states.thinking]
color = "amber"
speed_ms = 350
frames = [
    # ...
]

# states.writing, states.error, states.success, states.listening も同様
```

### カラー定義

`color` フィールドには以下の名前が使用できる。

| 名前 | ANSI 256 |
|---|---|
| `peach` | 216 |
| `amber` | 214 |
| `cyan` | 116 |
| `red` | 174 |
| `green` | 114 |
| `lavender` | 183 |

### バリデーション

ロード時に以下を検証する。エラーの場合は起動を中断してメッセージを表示する。

- 全 6 state が定義されているか
- 各 state に 1 フレーム以上あるか
- `speed_ms` が 0 より大きいか
- `color` が定義済みの名前か

---

## Rendering

### レイアウト

```
┌─────────────────────────────────────────┐
│  ┤ agent-face ─ {display_name} ├        │  ← ヘッダー
│                                         │
│         {ASCII フレーム}                 │  ← キャラクター
│                                         │
│         ▸▸▸ {STATE} ◂◂◂               │  ← 状態バッジ（状態カラー）
│                                         │
│  [1]idle [2]think [c]char [q]quit       │  ← フッター（コントロール）
│                                    HH:MM│  ← 時計
└─────────────────────────────────────────┘
```

- キャラクターはターミナル中央に配置（動的計算）
- ターミナルリサイズ時に再計算・再描画

### アニメーション

- **フレームサイクル**: 各 state のフレーム配列をループ
- **まばたき**: idle 状態のみ。2.5〜5.0 秒のランダム間隔で差し込む
- **呼吸**: idle 状態で `sin(t × 1.5)` による微細な垂直オフセット
- **パーティクル**: thinking（泡: `·∙°⋅`）と success（星: `✦✧⋆`）で発生

### カラーリング

文字種別ごとに着色ルールを適用する。ルールはキャラクターファイルで上書き可能（将来拡張）。

| 文字種別 | 文字例 | スタイル |
|---|---|---|
| 目 | `●◉◕▪×^-` | 太字 白 |
| 構造線 | `╭╮╰╯│─` | 状態カラー |
| 記号 | `✦✧✓⚠✎` | 太字 + 状態カラー |
| 口 | `ωΩ△▽` | 太字 Peach |
| ヒゲ・装飾 | `＝～` | 暗め状態カラー |

---

## Keyboard Controls

| キー | アクション |
|---|---|
| `1` | idle 状態に切り替え |
| `2` | thinking 状態に切り替え |
| `3` | writing 状態に切り替え |
| `4` | error 状態に切り替え |
| `5` | success 状態に切り替え |
| `6` | listening 状態に切り替え |
| `c` | キャラクター切り替え（読み込まれたキャラクターをサイクル） |
| `q` | 終了 |

キーボード入力は state file の状態より優先される（手動オーバーライド）。
AUTO モード中にキーボードで切り替えた場合、次に state file が更新されると上書きされる。

---

## Operating Modes

### AUTO モード

- state file が起動時に存在する場合、または起動後に作成された場合に有効
- `notify` クレートによるファイル変更イベントで状態を更新
- キーボード操作も同時に使用可能

### MANUAL モード

- state file が存在しない場合に有効
- キーボード `[1]`-`[6]` で状態を手動切り替え
- `agent-face set <state>` を受け取ると AUTO モードへ移行

---

## Adapter Scripts

`adapters/` ディレクトリにエージェント別のフックスクリプトを配置する。
各アダプターの責務は「エージェント固有のイベント → `agent-face set <state>`（または state file への直接書き込み）」への変換のみ。

```
adapters/
├── claude-code/
│   ├── README.md          # セットアップ手順
│   ├── settings.json      # Claude Code フック設定
│   └── hooks/
│       └── face-state.sh  # フックハンドラ
└── (codex/ 等は将来追加)
```

### 新しいエージェントのアダプターを追加する手順

1. `adapters/<agent-name>/` ディレクトリを作成
2. エージェントのイベントを受け取り、`agent-face set <state>` を呼ぶスクリプトを実装
3. セットアップ手順を `README.md` に記載

agent-face バイナリ本体はアダプターを一切知らない。

---

## Configuration

`~/.config/agent-face/config.toml` から設定を読み込む。ファイルが存在しない場合はデフォルト値を使用する。

```toml
# Claude Code セットアップ済みフラグ（agent-face setup claude-code で自動設定）
claude_code_setup = false

# デフォルトキャラクター名（将来実装）
# default_character = "cat"

# state file のパス（省略時はデフォルトパス）
# state_file = "/custom/path/to/state"

# キャラクター TOML の追加検索ディレクトリ
# characters_dir = "~/.config/agent-face/characters/"
```

`claude_code_setup` が `false`（またはファイルが存在しない）場合、サブコマンドなしでの起動時にセットアップを促すメッセージを表示して終了する。

---

## Dependencies

| クレート | 用途 |
|---|---|
| `crossterm` | ターミナル制御（クロスプラットフォーム） |
| `notify` | ファイル変更検知 |
| `clap` | CLI 引数・サブコマンド |
| `serde` + `toml` | キャラクター・設定ファイルのデシリアライズ |

非同期ランタイム（`tokio`）は使用しない。スレッドベースの並行処理で実装する。

---

## Constraints & Non-Goals

- **Windows 対応**: `crossterm` により技術的に可能だが、初期リリースでは未検証
- **ネットワーク IPC**: 将来の拡張候補だが現バージョンはファイルベースのみ
- **GUI**: ターミナル専用
- **複数エージェントの同時表示**: 1 つの state file = 1 つの状態のみ管理
