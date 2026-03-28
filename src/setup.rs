use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const FACE_STATE_SH: &str = r#"#!/bin/bash
# agent-face adapter for Claude Code
# Reads hook event from stdin and maps it to a face state.

INPUT=$(cat)
EVENT=$(echo "$INPUT" | grep -o '"hook_event_name":"[^"]*"' | head -1 | cut -d'"' -f4)

case "$EVENT" in
  SessionStart)
    agent-face set idle
    ;;
  UserPromptSubmit)
    agent-face set thinking
    ;;
  PreToolUse)
    agent-face set writing
    ;;
  PostToolUse)
    agent-face set thinking
    ;;
  PostToolUseFailure|StopFailure)
    agent-face set error
    ;;
  Stop)
    agent-face set success
    ;;
  Notification)
    NTYPE=$(echo "$INPUT" | grep -o '"notification_type":"[^"]*"' | head -1 | cut -d'"' -f4)
    if [ "$NTYPE" = "permission_prompt" ] || [ "$NTYPE" = "elicitation_dialog" ]; then
      agent-face set listening
    fi
    ;;
  SessionEnd)
    agent-face set idle
    ;;
esac

exit 0
"#;

fn adapter_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home)
        .join(".config")
        .join("agent-face")
        .join("adapters")
        .join("claude-code")
}

fn claude_settings_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".claude").join("settings.json")
}

pub fn setup_claude_code() -> Result<(), String> {
    // 1. Install hook script.
    let adapter_dir = adapter_dir();
    fs::create_dir_all(&adapter_dir)
        .map_err(|e| format!("failed to create {}: {e}", adapter_dir.display()))?;

    let script_path = adapter_dir.join("face-state.sh");
    fs::write(&script_path, FACE_STATE_SH)
        .map_err(|e| format!("failed to write {}: {e}", script_path.display()))?;
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("failed to chmod {}: {e}", script_path.display()))?;

    println!("  Installed {}", script_path.display());

    // 2. Update ~/.claude/settings.json.
    let settings_path = claude_settings_path();
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("failed to read {}: {e}", settings_path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse {}: {e}", settings_path.display()))?
    } else {
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
        }
        serde_json::json!({})
    };

    let new_entry = serde_json::json!({
        "matcher": "",
        "hooks": [
            {
                "type": "command",
                "command": script_path.to_string_lossy(),
                "timeout": 5
            }
        ]
    });

    let events = [
        "SessionStart",
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "PostToolUseFailure",
        "Stop",
        "StopFailure",
        "Notification",
        "SessionEnd",
    ];

    let hooks = settings
        .as_object_mut()
        .ok_or("settings.json is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    let script_str = script_path.to_string_lossy().to_string();

    for event in events {
        let entries = hooks_obj
            .entry(event)
            .or_insert_with(|| serde_json::json!([]));

        let arr = entries
            .as_array_mut()
            .ok_or_else(|| format!("hooks.{event} is not an array"))?;

        // Skip if our hook is already registered.
        let already_exists = arr.iter().any(|entry| {
            entry["hooks"]
                .as_array()
                .is_some_and(|h| h.iter().any(|hook| hook["command"].as_str() == Some(&script_str)))
        });

        if !already_exists {
            arr.push(new_entry.clone());
        }
    }

    let output = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize settings: {e}"))?;
    fs::write(&settings_path, output)
        .map_err(|e| format!("failed to write {}: {e}", settings_path.display()))?;

    println!("  Updated  {}", settings_path.display());
    println!();
    println!("Setup complete! agent-face will now respond to Claude Code events.");

    Ok(())
}
