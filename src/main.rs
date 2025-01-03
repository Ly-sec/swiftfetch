use serde::Deserialize;
use std::{fs, process::Command};
use dirs::config_dir;
mod sys_info;
use sys_info::get_system_info;
use toml;

fn execute_command(command: &str) -> String {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string()
        })
        .unwrap_or_else(|_| "Command failed".to_string())
}

#[derive(Deserialize, Debug)]
struct Config {
    display: DisplayConfig,
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

    let config_data =
        fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: Config = toml::de::from_str(&config_data).expect("Failed to parse config file");

    let separator = config.display.separator.unwrap_or_else(|| ": ".to_string());

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

    let memory_used_gb_str = format!("{:.2}", memory_used_gb);
    let total_memory_gb_str = format!("{:.2}", total_memory_gb);
    let pacman_pkg_count_str = pacman_pkg_count.to_string();
    let flatpak_pkg_count_str = flatpak_pkg_count.to_string();
    let os_age_str = os_age.to_string();
    let hostname_trimmed = hostname.trim().to_string();
    let memory = format!("{} GB / {} GB", memory_used_gb_str, total_memory_gb_str);
    let user_info = format!("\x1b[1m{}@{}\x1b[0m", username, hostname_trimmed);

    let uptime_hours = uptime_seconds / 3600;
    let uptime_minutes = (uptime_seconds % 3600) / 60;
    let uptime_formatted = if uptime_hours > 0 {
        format!("{}h {:02}m", uptime_hours, uptime_minutes)
    } else {
        format!("{}m", uptime_minutes)
    };

    for entry in config.display.items {
        if entry.value.is_empty() {
            println!();
            continue;
        }
    
        let output = match entry.r#type.as_str() {
            "default" => match entry.value.as_str() {
                "kernel" => &kernel_version,
                "os" => &os_name,
                "cpu" => &cpu_brand,
                "wm" => &wm_de,
                "editor" => &editor,
                "shell" => &shell,
                "terminal" => &terminal,
                "username" => &username,
                "hostname" => &hostname_trimmed,
                "memory" => &memory,
                "pacman_pkg_count" => &pacman_pkg_count_str,
                "flatpak_pkg_count" => &flatpak_pkg_count_str,
                "uptime_seconds" => &uptime_formatted,
                "os_age" => &os_age_str,
                "user_info" => &user_info,
                _ => "Unknown default value",
            }
            .to_string(),
    
            "text" => entry.value.clone(),
    
            "command" => execute_command(&entry.value),
    
            _ => "Invalid type".to_string(),
        };
    
        if entry.key.is_empty() && entry.r#type == "text" {
            println!("{}", output);
        } else if entry.key == "user_info" {
            println!("{}", output);
        } else {
            println!("\x1b[1m{}\x1b[0m{}{}", entry.key, separator, output);
        }
    }
}
