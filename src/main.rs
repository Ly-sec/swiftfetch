use serde::Deserialize;
use std::{collections::HashMap, fs, io::{self, BufRead}};
use dirs::config_dir;
mod sys_info;
use sys_info::get_system_info;
use shellexpand;
use std::path::Path;

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
    "\x1b[0m".to_string()
}

#[derive(Deserialize, Debug)]
struct Config {
    display: DisplayConfig,
    colors: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct DisplayConfig {
    items: Vec<ConfigEntry>,
    separator: Option<String>,
    ascii_path: Option<String>,
    ascii_color: String, // Add the field for ASCII color
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

    let (
        os_name,
        kernel_version,
        cpu_brand,
        username,
        hostname,
        wm_de,
        memory_used_gb,
        total_memory_gb,
        pkg_count,
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

    let ascii_path = shellexpand::tilde(config.display.ascii_path.as_deref().unwrap_or("~/.config/swiftfetch/ascii.txt")).to_string();
    let ascii_lines: Vec<String> = if Path::new(&ascii_path).exists() {
        if let Ok(file) = fs::File::open(ascii_path) {
            io::BufReader::new(file).lines().filter_map(Result::ok).collect()
        } else {
            vec![] // If file opens but can't be read, return empty
        }
    } else {
        vec![] // If file doesn't exist, return empty
    };

    let mut ascii_iter = ascii_lines.iter();
    let max_ascii_length = ascii_lines.iter().map(|line| line.len()).max().unwrap_or(0);

    // Get the ASCII color from the config (default to white if not set)
    let ascii_color_code = config
        .colors
        .get(&config.display.ascii_color)
        .map(|hex| hex_to_ansi(hex))
        .unwrap_or_else(|| "\x1b[0m".to_string()); // Default to reset color if no color is specified

    for entry in &config.display.items {
        let empty_string = String::new();
        let ascii_part = ascii_iter.next().unwrap_or(&empty_string);
        
        // Apply the color to the ASCII part
        let colored_ascii_part = format!("{}{}", ascii_color_code, ascii_part);

        // If entry value is empty, just print the ASCII part
        if entry.value.is_empty() {
            println!("{}", colored_ascii_part);
            continue;
        }

        // Determine the color for this entry
        let color_code = entry
            .color
            .as_ref()
            .and_then(|c| config.colors.get(c))
            .map(|hex| hex_to_ansi(hex))
            .unwrap_or_else(|| "\x1b[0m".to_string());

        let output_value = match entry.r#type.as_str() {
            "default" => {
                match entry.value.as_str() {
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
                    "pkg_count" => pkg_count.to_string(),
                    "flatpak_pkg_count" => flatpak_pkg_count.to_string(),
                    "uptime_seconds" => uptime_formatted.clone(),
                    "os_age" => os_age.to_string(),
                    "user_info" => user_info.clone(),
                    _ => "Unknown default value".to_string(),
                }
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

        let padding = " ".repeat(max_ascii_length - ascii_part.len());

        if entry.key.is_empty() || entry.key == "user_info" {
            println!("{}{}{}  {}", colored_ascii_part, padding, color_code, output_value);
        } else {
            println!("\x1b[0m{}{}  {}{}{}\x1b[0m{}", colored_ascii_part, padding, color_code, entry.key, separator, output_value);
        }
        print!("\x1b[0m");
    }
}
