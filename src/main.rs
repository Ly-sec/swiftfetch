use serde::Deserialize;
use std::{collections::HashMap, fs};
use dirs::config_dir;
mod sys_info;
use sys_info::get_system_info;
use toml;

/// Converts a hex color string (e.g., "#FF4500") to an ANSI escape sequence
fn hex_to_ansi(hex: &str) -> String {
    if hex.starts_with('#') && hex.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[1..3], 16),
            u8::from_str_radix(&hex[3..5], 16),
            u8::from_str_radix(&hex[5..7], 16),
        ) {
            return format!("\x1b[38;2;{};{};{}m", r, g, b);
        }
    }
    "\x1b[0m".to_string() // Default color reset if invalid
}

/// Structs for parsing TOML config
#[derive(Deserialize, Debug)]
struct Config {
    display: DisplayConfig,
    colors: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct DisplayConfig {
    items: Vec<ConfigEntry>,
    separator: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ConfigEntry {
    key: String,
    #[serde(rename = "type")]
    r#type: String,
    value: String,
    color: Option<String>,
}

fn main() {
    let user_config_path = config_dir()
        .map(|p| p.join("swiftfetch/config.toml"))
        .unwrap_or_else(|| "config.toml".into());

    let config_path = if user_config_path.exists() {
        user_config_path
    } else {
        "/usr/share/swiftfetch/config.toml".into()
    };

    let config_data = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: Config = toml::de::from_str(&config_data).expect("Failed to parse config file");

    let separator = config.display.separator.unwrap_or_else(|| ": ".to_string());

    // Get system info
    let (
        os_name,
        kernel_version,
        cpu_brand,
        username,
        hostname,
        wm_de,
        memory_used_gb,
        total_memory_gb,
        pacman_pkg_count,
        flatpak_pkg_count,
        uptime_seconds,
        os_age,
        editor,
        shell,
        terminal,
    ) = get_system_info();

    let memory = format!("{:.2} GB / {:.2} GB", memory_used_gb, total_memory_gb);
    let user_info = format!("\x1b[1m{}@{}\x1b[0m", username, hostname.trim());

    let uptime_hours = uptime_seconds / 3600;
    let uptime_minutes = (uptime_seconds % 3600) / 60;
    let uptime_formatted = if uptime_hours > 0 {
        format!("{}h {:02}m", uptime_hours, uptime_minutes)
    } else {
        format!("{}m", uptime_minutes)
    };

    for entry in &config.display.items {
        if entry.value.is_empty() {
            println!();
            continue;
        }

        // Get hex color from config and convert it to ANSI
        let color_code = entry
            .color
            .as_ref()
            .and_then(|c| config.colors.get(c))
            .map(|hex| hex_to_ansi(hex))
            .unwrap_or_else(|| "\x1b[0m".to_string()); // Default reset if no color found

        let output_value = match entry.r#type.as_str() {
            "default" => {
                let value = match entry.value.as_str() {
                    "kernel" => kernel_version.to_string(),
                    "os" => os_name.to_string(),
                    "cpu" => cpu_brand.to_string(),
                    "wm" => wm_de.to_string(),
                    "editor" => editor.to_string(),
                    "shell" => shell.to_string(),
                    "terminal" => terminal.to_string(),
                    "username" => username.to_string(),
                    "hostname" => hostname.trim().to_string(),
                    "memory" => memory.clone(),
                    "pacman_pkg_count" => pacman_pkg_count.to_string(),
                    "flatpak_pkg_count" => flatpak_pkg_count.to_string(),
                    "uptime_seconds" => uptime_formatted.clone(),
                    "os_age" => os_age.to_string(),
                    "user_info" => user_info.clone(),
                    _ => "Unknown default value".to_string(),
                };
                value
            }
            "text" => entry.value.clone(),
            "command" => {
                use std::process::Command;
                Command::new("sh")
                    .arg("-c")
                    .arg(&entry.value)
                    .output()
                    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                    .unwrap_or_else(|_| "Command failed".to_string())
            }
            _ => "Invalid type".to_string(),
        };

        if entry.key.is_empty() || entry.key == "user_info" {
            println!("{}{}", color_code, output_value);
        } else if entry.key == "user_info" {
            println!("{}{}", color_code, output_value); // Ensure 'user_info' is colored too
        } else {
            println!("{}{}{}\x1b[0m{}", color_code, entry.key, separator, output_value);
        }
    }
}
