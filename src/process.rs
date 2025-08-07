/// Linux process detection for HotKeys application
/// Detects the currently active window and extracts process information

use anyhow::{Result, anyhow};
use std::fs;
use std::process::Command;

#[derive(Debug)]
pub struct ProcessInfo {
    pub name: String,        // Process executable name (e.g., "sublime_text")
    pub pid: u32,            // Process ID
    pub window_id: Option<u64>, // X11 Window ID (if available)
    pub window_class: Option<String>, // X11 WM_CLASS (if available)
}

impl ProcessInfo {
    pub fn new(name: String, pid: u32) -> Self {
        Self {
            name,
            pid,
            window_id: None,
            window_class: None,
        }
    }

    pub fn with_window_info(mut self, window_id: u64, window_class: Option<String>) -> Self {
        self.window_id = Some(window_id);
        self.window_class = window_class;
        self
    }
}

/// Detect the currently active window and return process information
pub fn get_active_process_info() -> Result<ProcessInfo> {
    let start_time = std::time::Instant::now();

    // Try X11 approach first (most common on Linux)
    let result = match get_active_process_x11() {
        Ok(info) => Ok(info),
        Err(e) => {
            log::debug!("X11 detection failed: {}", e);
            Err(anyhow!("Could not detect active process - no supported display server found"))
        }
    };

    let duration = start_time.elapsed();
    log::debug!("get_active_process_info() took {}ms", duration.as_millis());

    result
}

/// Get active process info using X11 tools
fn get_active_process_x11() -> Result<ProcessInfo> {
    // Get the active window ID using xprop
    let output = Command::new("xprop")
        .args(&["-root", "_NET_ACTIVE_WINDOW"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get active window with xprop"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let window_id = parse_window_id_from_xprop(&output_str)?;

    // Get the process ID for this window
    let output = Command::new("xprop")
        .args(&["-id", &window_id.to_string(), "_NET_WM_PID"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get PID for window {}", window_id));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let pid = parse_pid_from_xprop(&output_str)?;

    // Get process name from /proc filesystem
    let process_name = get_process_name(pid)?;

    // Get window class for better matching
    let window_class = get_window_class(window_id).ok();

    let process_info = ProcessInfo::new(process_name, pid)
        .with_window_info(window_id, window_class);

    log::debug!("Detected: {:?}", process_info);
    Ok(process_info)
}

/// Parse window ID from xprop output
/// Example input: "_NET_ACTIVE_WINDOW(WINDOW): window id # 0x1e00007"
fn parse_window_id_from_xprop(output: &str) -> Result<u64> {
    for line in output.lines() {
        if line.contains("_NET_ACTIVE_WINDOW") {
            if let Some(hex_part) = line.split('#').nth(1) {
                let hex_str = hex_part.trim().trim_start_matches("0x");
                let window_id = u64::from_str_radix(hex_str, 16)
                    .map_err(|e| anyhow!("Failed to parse window ID '{}': {}", hex_str, e))?;
                return Ok(window_id);
            }
        }
    }
    Err(anyhow!("Could not find window ID in xprop output: {}", output))
}

/// Parse PID from xprop output
/// Example input: "_NET_WM_PID(CARDINAL) = 12345"
fn parse_pid_from_xprop(output: &str) -> Result<u32> {
    for line in output.lines() {
        if line.contains("_NET_WM_PID") {
            if let Some(pid_part) = line.split('=').nth(1) {
                let pid_str = pid_part.trim();
                let pid = pid_str.parse::<u32>()
                    .map_err(|e| anyhow!("Failed to parse PID '{}': {}", pid_str, e))?;
                return Ok(pid);
            }
        }
    }
    Err(anyhow!("Could not find PID in xprop output: {}", output))
}

/// Get process name from /proc filesystem
fn get_process_name(pid: u32) -> Result<String> {
    let comm_path = format!("/proc/{}/comm", pid);
    let comm = fs::read_to_string(&comm_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", comm_path, e))?;

    // comm contains the process name with a trailing newline
    Ok(comm.trim().to_string())
}

/// Get X11 window class for better application matching
fn get_window_class(window_id: u64) -> Result<String> {
    let output = Command::new("xprop")
        .args(&["-id", &window_id.to_string(), "WM_CLASS"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get window class"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_window_class_from_xprop(&output_str)
}

/// Parse window class from xprop output
/// Example input: 'WM_CLASS(STRING) = "sublime_text", "Sublime_text"'
fn parse_window_class_from_xprop(output: &str) -> Result<String> {
    for line in output.lines() {
        if line.contains("WM_CLASS") {
            if let Some(class_part) = line.split('=').nth(1) {
                let class_str = class_part.trim();
                // Extract the second class name (usually the application name)
                if let Some(captures) = class_str.split(',').nth(1) {
                    let class_name = captures.trim().trim_matches('"').trim();
                    return Ok(class_name.to_string());
                }
            }
        }
    }
    Err(anyhow!("Could not parse window class from: {}", output))
}

/// Get a list of all running processes using ps -aux
/// Returns ProcessInfo objects with only PID and name populated
pub fn get_all_processes() -> Result<Vec<ProcessInfo>> {
    let start_time = std::time::Instant::now();

    let output = Command::new("ps")
        .args(&["-aux"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to execute ps -aux command"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let result = parse_ps_output(&output_str);

    let duration = start_time.elapsed();
    match &result {
        Ok(processes) => log::debug!("get_all_processes() took {}ms, found {} processes",
                                   duration.as_millis(), processes.len()),
        Err(_) => log::debug!("get_all_processes() took {}ms, failed", duration.as_millis()),
    }

    result
}

/// Parse ps -aux output and extract process information
/// ps -aux format: USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND
fn parse_ps_output(output: &str) -> Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();

    // Skip the header line
    for line in output.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        // Split by whitespace, but be careful with command column which may contain spaces
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 11 {
            // Malformed line, skip it
            continue;
        }

        // Parse PID (second column)
        let pid_str = parts[1];
        let pid = match pid_str.parse::<u32>() {
            Ok(p) => p,
            Err(_) => continue, // Skip if PID parsing fails
        };

        // Extract process name from COMMAND (11th column onwards)
        // For most processes, we want just the executable name, not full path/args
        let command = parts[10..].join(" ");
        let process_name = extract_process_name(&command);

        let process_info = ProcessInfo::new(process_name, pid);
        processes.push(process_info);
    }

    Ok(processes)
}

/// Extract just the process name from the full command string
/// Examples:
/// "/usr/bin/firefox" -> "firefox"
/// "python3 script.py" -> "python3"
/// "[kthreadd]" -> "kthreadd"
fn extract_process_name(command: &str) -> String {
    let command = command.trim();

    // Handle kernel threads in brackets
    if command.starts_with('[') && command.contains(']') {
        if let Some(end) = command.find(']') {
            return command[1..end].to_string();
        }
    }

    // For regular processes, take the first part (executable)
    let first_part = command.split_whitespace().next().unwrap_or(command);

    // Extract just the filename from full path
    if let Some(last_slash) = first_part.rfind('/') {
        first_part[last_slash + 1..].to_string()
    } else {
        first_part.to_string()
    }
}

/// Check if X11 is available on the system
pub fn is_x11_available() -> bool {
    std::env::var("DISPLAY").is_ok() &&
    Command::new("xprop").arg("-version").output().map_or(false, |o| o.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_window_id() {
        let input = "_NET_ACTIVE_WINDOW(WINDOW): window id # 0x1e00007";
        let result = parse_window_id_from_xprop(input).unwrap();
        assert_eq!(result, 0x1e00007);
    }

    #[test]
    fn test_parse_pid() {
        let input = "_NET_WM_PID(CARDINAL) = 12345";
        let result = parse_pid_from_xprop(input).unwrap();
        assert_eq!(result, 12345);
    }

    #[test]
    fn test_parse_window_class() {
        let input = r#"WM_CLASS(STRING) = "sublime_text", "Sublime_text""#;
        let result = parse_window_class_from_xprop(input).unwrap();
        assert_eq!(result, "Sublime_text");
    }

    #[test]
    fn test_extract_process_name() {
        // Test full path
        assert_eq!(extract_process_name("/usr/bin/firefox"), "firefox");

        // Test simple command
        assert_eq!(extract_process_name("python3"), "python3");

        // Test command with arguments
        assert_eq!(extract_process_name("python3 script.py --arg"), "python3");

        // Test kernel thread
        assert_eq!(extract_process_name("[kthreadd]"), "kthreadd");

        // Test complex path with arguments
        assert_eq!(extract_process_name("/usr/lib/firefox/firefox --board /tmp"), "firefox");
    }

    #[test]
    fn test_parse_ps_output() {
        let sample_output = r#"USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
root           1  0.0  0.1 169804 11456 ?        Ss   Aug07   0:01 /sbin/init splash
root           2  0.0  0.0      0     0 ?        S    Aug07   0:00 [kthreadd]
user        1234  1.2  2.5 123456 98765 ?        Sl   10:30   0:05 /usr/bin/firefox
user        5678  0.1  0.8  45678 12345 pts/0    S+   11:00   0:00 python3 script.py"#;

        let result = parse_ps_output(sample_output).unwrap();
        assert_eq!(result.len(), 4);

        assert_eq!(result[0].pid, 1);
        assert_eq!(result[0].name, "init");

        assert_eq!(result[1].pid, 2);
        assert_eq!(result[1].name, "kthreadd");

        assert_eq!(result[2].pid, 1234);
        assert_eq!(result[2].name, "firefox");

        assert_eq!(result[3].pid, 5678);
        assert_eq!(result[3].name, "python3");
    }
}