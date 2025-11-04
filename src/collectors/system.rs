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
    use std::io::{BufRead, BufReader};
    use std::fs::File;
    
    // Read line by line to find PRETTY_NAME early
    let file = File::open("/etc/os-release")?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("PRETTY_NAME") {
            return Ok(line.split('=').nth(1).unwrap_or("Unknown").trim_matches('"').to_string());
        }
    }
    Err(crate::error::SwiftfetchError::Detection("OS name not found".to_string()))
}

fn read_kernel_version() -> Result<String> {
    // /proc/version is a single line, so use optimized read
    let version_info = crate::utils::file::read_first_line("/proc/version")?;
    version_info.split_whitespace().nth(2)
        .map(|v| v.to_string())
        .ok_or_else(|| crate::error::SwiftfetchError::Detection("Kernel version not found".to_string()))
}

fn read_uptime() -> Result<u64> {
    // /proc/uptime is a single line
    let uptime_str = crate::utils::file::read_first_line("/proc/uptime")?;
    let secs = uptime_str.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0.0);
    Ok(secs as u64)
}

fn get_os_age() -> Result<String> {
    use std::ffi::CString;
    use std::time::{SystemTime, UNIX_EPOCH};
    use libc;
    
    // Use statx syscall to get birth time directly (faster than stat command)
    #[cfg(target_os = "linux")]
    {
        unsafe {
            let path = CString::new("/").unwrap();
            let mut statx_buf: libc::statx = std::mem::zeroed();
            let flags = libc::AT_FDCWD as libc::c_int;
            let mask = libc::STATX_BTIME as libc::c_uint;
            
            // statx syscall (Linux-specific, requires glibc 2.28+)
            let result = libc::syscall(
                libc::SYS_statx,
                flags,
                path.as_ptr() as *const libc::c_char,
                libc::AT_SYMLINK_NOFOLLOW as libc::c_int,
                mask,
                &mut statx_buf as *mut _ as *mut libc::c_void
            );
            
            if result == 0 && (statx_buf.stx_mask & mask) != 0 {
                let birth_sec = statx_buf.stx_btime.tv_sec as i64;
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                
                if birth_sec > 0 {
                    let age_days = (now - birth_sec) / 86400;
                    if age_days == 1 {
                        return Ok("1 day".to_string());
                    } else {
                        return Ok(format!("{} days", age_days));
                    }
                }
            }
        }
    }
    
    // Fallback to stat command if statx fails or not available
    let stat_output = run_command("stat", &["-c", "%W", "/"])?;
    let birth_timestamp: i64 = stat_output.trim().parse().unwrap_or(0);
    
    if birth_timestamp > 0 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let age_days = (now - birth_timestamp) / 86400;
        
        if age_days == 1 {
            Ok("1 day".to_string())
        } else {
            Ok(format!("{} days", age_days))
        }
    } else {
        Ok("Unsupported".to_string())
    }
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
    // Check for systemd first (most common) - check files before spawning process
    if file_exists("/run/systemd/system") || 
       file_exists("/usr/lib/systemd/systemd") ||
       file_exists("/etc/systemd/system") {
        return "systemd".to_string();
    }
    
    // Only run systemctl command as last resort
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
