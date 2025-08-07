/// Action execution module - handles all pad action types

use crate::core::{Action, DataRepository};
use crate::app::config::KeyboardLayout;
use crate::input::script;
use anyhow::Result;
use open;
use std::sync::{Arc, Mutex};

/// Execute a list of actions sequentially with optional repository access
pub fn execute_actions(
    actions: &[Action],
    keyboard_layout: &KeyboardLayout,
    repository: Option<Arc<Mutex<dyn DataRepository>>>,
    profile: Option<&str>
) -> Result<()> {
    log::info!("Executing {} actions", actions.len());

    for action in actions {
        match execute_action(action, keyboard_layout, repository.as_ref(), profile) {
            Err(e) => {
                log::error!("Failed to execute action {:?}: {}", action, e);
                return Err(e);
            },
            _ => {}
        }
    }

    log::info!("All actions executed successfully");
    Ok(())
}


/// Execute a single action
fn execute_action(
    action: &Action,
    keyboard_layout: &KeyboardLayout,
    repository: Option<&Arc<Mutex<dyn DataRepository>>>,
    profile: Option<&str>
) -> Result<()> {
    let keyboard_layout_mapping = keyboard_layout.mappings.clone();

    match action {
        Action::Shortcut(shortcut_text) => {
            log::info!("Executing shortcut: {}", shortcut_text);
            script::for_shortcut(shortcut_text.clone()).play()
        },
        Action::Text(text) => {
            log::info!("Executing text input: {}", text);
            script::for_text(text.clone(), keyboard_layout_mapping).play()
        },
        Action::Line(line_text) => {
            log::info!("Executing line input: {}", line_text);
            script::for_line(line_text.clone(), keyboard_layout_mapping).play()
        },
        Action::Pause(milliseconds) => {
            log::info!("Executing pause: {} ms", milliseconds);
            script::for_pause((*milliseconds).min(u16::MAX as u64) as u16).play()
        },
        Action::OpenUrl(url) => {
            log::info!("Executing OpenUrl: {}", url);
            open_url(url)
        },
        Action::CustomHomeAction => {
            log::info!("Executing CustomHomeAction");
            execute_custom_home_action(repository, profile)
        },
        Action::Command(command) => {
            log::info!("Executing command: {}", command);
            execute_command(command)
        }
    }
}

/// Execute the custom home action - updates timestamp in repository
fn execute_custom_home_action(
    repository: Option<&Arc<Mutex<dyn DataRepository>>>,
    profile: Option<&str>
) -> Result<()> {
    if let (Some(repo), Some(profile_name)) = (repository, profile) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Failed to get current time: {}", e))?;

        // Format timestamp as human-readable string
        let timestamp = chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // Store the timestamp in the repository
        let mut repo_guard = repo.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire repository lock: {}", e))?;
        repo_guard.set_board_data(profile_name, "home", "last_action_time", &timestamp)?;
        repo_guard.flush()?;

        log::info!("Updated home board last action timestamp to: {}", timestamp);
        Ok(())
    } else {
        log::warn!("CustomHomeAction called but no repository or profile provided");
        Ok(())
    }
}

/// Open a URL in the default web browser
fn open_url(url: &str) -> Result<()> {
    open::that(url).map_err(|e| anyhow::anyhow!("Failed to open URL {}: {}", url, e))
}

/// Execute a shell command asynchronously without waiting for completion
fn execute_command(command: &str) -> Result<()> {
    use std::process::{Command, Stdio};

    let mut cmd = Command::new("sh");
    cmd.args(["-c", command]);

    // Redirect stdout and stderr to /dev/null to ignore output
    cmd.stdout(Stdio::null())
       .stderr(Stdio::null())
       .stdin(Stdio::null());

    // Spawn the process without waiting for completion
    match cmd.spawn() {
        Ok(_) => {
            log::info!("Successfully spawned command: {}", command);
            Ok(())
        },
        Err(e) => {
            let error_msg = format!("Failed to spawn command '{}': {}", command, e);
            log::error!("{}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}