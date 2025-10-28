//! System information collection (OS, kernel, uptime, etc.)

use crate::error::Result;
use crate::utils::{file::*, command::*};
use crate::data::{OsInfo, SystemStatus, UserInfo};

/// Collect OS-related information
pub fn collect_os_info() -> Result<OsInfo> {
    Ok(OsInfo {
        name: read_os_name()?,
        kernel_version: read_kernel_version()?,
        age: get_os_age()?,
    })
}

/// Collect user and environment information
pub fn collect_user_info() -> Result<UserInfo> {
    let username = get_username();
    let hostname = get_hostname()?;
    let formatted_user_info = format!("\x1b[1m{}@{}\x1b[0m", username, hostname.trim());
    
    Ok(UserInfo {
        username,
        hostname,
        formatted_user_info,
    })
}

/// Collect system status information
pub fn collect_system_status() -> Result<SystemStatus> {
    let uptime_seconds = read_uptime()?;
    let uptime_formatted = crate::utils::parsing::format_uptime(uptime_seconds);
    
    Ok(SystemStatus {
        uptime_seconds,
        uptime_formatted,
        init_system: detect_init_system(),
        battery_info: get_battery_info().unwrap_or_else(|_| "No battery".to_string()),
        desktop_environment: crate::collectors::desktop::detect_wm_or_de(),
        editor: get_editor(),
        shell: get_shell(),
        terminal: get_terminal(),
    })
}

// Individual functions
fn read_os_name() -> Result<String> {
    let os_release = read_file_safe("/etc/os-release")?;
    for line in os_release.lines() {
        if line.starts_with("PRETTY_NAME") {
            return Ok(line.split('=').nth(1).unwrap_or("Unknown").trim_matches('"').to_string());
        }
    }
    Err(crate::error::SwiftfetchError::Detection("OS name not found".to_string()))
}

fn read_kernel_version() -> Result<String> {
    let version_info = read_file_safe("/proc/version")?;
    version_info.split_whitespace().nth(2)
        .map(|v| v.to_string())
        .ok_or_else(|| crate::error::SwiftfetchError::Detection("Kernel version not found".to_string()))
}

fn read_uptime() -> Result<u64> {
    let uptime_str = read_file_safe("/proc/uptime")?;
    let secs = uptime_str.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0.0);
    Ok(secs as u64)
}

fn get_os_age() -> Result<String> {
    let output = run_command("sh", &[
        "-c",
        "birth=$(stat -c %W / 2>/dev/null || echo 0); \
         if [ \"$birth\" -gt 0 ]; then \
           current=$(date +%s); \
           age=$(( (current - birth) / 86400 )); \
           if [ \"$age\" -eq 1 ]; then \
             echo \"$age day\"; \
           else \
             echo \"$age days\"; \
           fi; \
         else \
           echo \"Unsupported\"; \
         fi"
    ])?;
    Ok(output)
}

fn get_hostname() -> Result<String> {
    read_first_line("/proc/sys/kernel/hostname")
}

fn get_username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "Unknown".to_string())
}

fn get_editor() -> String {
    std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string())
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "Unknown".to_string())
        .replace("/usr/bin/", "")
        .replace("/bin/", "")
        .replace("/run/current-system/sw", "")
}

fn get_terminal() -> String {
    std::env::var("TERM")
        .unwrap_or_else(|_| "Unknown".to_string())
        .replace("xterm-", "")
}

fn detect_init_system() -> String {
    // Check for systemd first (most common)
    if command_succeeds("systemctl", &["--version"]) {
        return "systemd".to_string();
    }
    
    // Check for OpenRC
    if file_exists("/sbin/openrc") || file_exists("/usr/sbin/openrc") {
        return "OpenRC".to_string();
    }
    
    // Check for runit
    if file_exists("/etc/runit") {
        return "runit".to_string();
    }
    
    // Check for SysV init
    if file_exists("/etc/init.d") {
        return "SysV".to_string();
    }
    
    // Check for s6
    if file_exists("/etc/s6") {
        return "s6".to_string();
    }
    
    "Unknown".to_string()
}

fn get_battery_info() -> Result<String> {
    use std::fs;
    
    // Try to find battery information in /sys/class/power_supply/
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                // Look for BAT* entries (typical battery naming)
                if name_str.starts_with("BAT") {
                    let battery_path = entry.path();
                    
                    // Read capacity percentage
                    let capacity_path = battery_path.join("capacity");
                    let status_path = battery_path.join("status");
                    
                    if let (Ok(capacity), Ok(status)) = (
                        fs::read_to_string(&capacity_path),
                        fs::read_to_string(&status_path)
                    ) {
                        let capacity = capacity.trim();
                        let status = status.trim();
                        
                        let status_icon = match status {
                            "Charging" => "⚡",
                            "Discharging" => "🔋",
                            "Full" => "🔋",
                            "Not charging" => "🔌",
                            _ => "🔋",
                        };
                        
                        return Ok(format!("{}% {}", capacity, status_icon));
                    }
                }
            }
        }
    }
    
    Err(crate::error::SwiftfetchError::Detection("No battery found".to_string()))
}
