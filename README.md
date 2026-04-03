# agent-face

> Give your AI coding agent a face — ASCII art expressions that react to what it's doing, right in your terminal.

![agent-face writing state](docs/images/agent-face_readme_image_writing.png)

## Features

- **Live expressions** — the face reacts in real time as the agent thinks, writes, encounters errors, and more
- **Custom characters** — define your own characters with TOML configuration files
- **Agent-agnostic** — pluggable adapter architecture (currently supports [Claude Code](https://docs.anthropic.com/en/docs/claude-code))

## Installation

Requires the Rust toolchain. Install it via [rustup](https://rustup.rs/) if you haven't already.

```sh
cargo install --git https://github.com/tokoro7/agent-face
```

## Getting Started

### 1. Set up the agent integration

```sh
# For Claude Code
agent-face setup claude-code
```

This registers hook scripts so that agent events automatically update the face state:

- `~/.config/agent-face/adapters/claude-code/face-state.sh` — hook script
- `~/.claude/settings.json` — hook configuration entry

### 2. Launch

Open two terminals side by side — one for agent-face, one for your agent.

```sh
agent-face
```

That's it. The face will start reacting as soon as the agent begins working.

## Keyboard Controls

| Key | Action |
|-----|--------|
| `1`–`6` | Manually switch state (see table below) |
| `c` | Cycle through characters |
| `q` | Quit |

## States

| State | Trigger |
|-------|---------|
| `idle` | Agent is inactive |
| `thinking` | Agent is processing |
| `writing` | Agent is editing files |
| `error` | An error occurred |
| `success` | Task completed |
| `listening` | Waiting for user input |

## Uninstall

```sh
cargo uninstall agent-face
```

## License

MIT
