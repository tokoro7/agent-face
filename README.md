# agent-face

AI coding agent の動作状態に連動して表情が変わる、ターミナル向け ASCII アニメーションツール。

![agent-face writing state](docs/images/agent-face_readme_image_writing.png)

## Features

- エージェントの状態（思考中、書き込み中、エラー等）に応じて表情がリアルタイムに変化
- TOML ファイルでキャラクターを自由に定義可能
- エージェント非依存の設計（現在は Claude Code に対応）

## How to Install

Rust toolchain が必要です。[rustup](https://rustup.rs/) でインストールできます。

```sh
cargo install --git https://github.com/tokoro7/agent-face
```

## Setup

インストール後、エージェントとの連携をセットアップします。

### Claude Code

```sh
agent-face setup claude-code
```

これにより以下が行われます:

- `~/.config/agent-face/adapters/claude-code/face-state.sh` にフックスクリプトを配置
- `~/.claude/settings.json` にフック設定を追記

## Usage

別のターミナルで agent-face を起動しておき、もう一方のターミナルで Claude Code を使います。

```sh
agent-face
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `1`-`6` | 状態を手動切り替え (idle/thinking/writing/error/success/listening) |
| `c` | キャラクター切り替え |
| `q` | 終了 |

### States

| State | Description |
|-------|-------------|
| `idle` | 待機中 |
| `thinking` | 思考中 |
| `writing` | 書き込み中 |
| `error` | エラー発生 |
| `success` | タスク完了 |
| `listening` | ユーザー入力待ち |

## How to Uninstall

```sh
cargo uninstall agent-face
```
