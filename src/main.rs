use serde::Deserialize;
use std::{collections::HashMap, fs, io::{self, BufRead}};
use dirs::config_dir;
mod sys_info;
use sys_info::get_system_info;
use shellexpand;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

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
    ascii_color: String,
}

#[derive(Deserialize, Debug)]
struct ConfigEntry {
    key: String,
    #[serde(rename = "type")]
    r#type: String,
    value: String,
    color: Option<String>,
    value_color: Option<String>,
}

fn ensure_user_config_exists() {
    let user_config_path = config_dir()
        .map(|p| p.join("swiftfetch/config.toml"))
        .expect("Could not determine config dir");

    if !user_config_path.exists() {
        let default_config_path = "/usr/share/swiftfetch/config.toml";

        if let Some(parent) = user_config_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        if Path::new(default_config_path).exists() {
            fs::copy(default_config_path, &user_config_path).unwrap();
            println!("Created config at {}", user_config_path.display());
        } else {
            eprintln!("Missing default config at {}", default_config_path);
        }
    }
}

fn main() {
    ensure_user_config_exists();

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
            vec![]
        }
    } else {
        vec![]
    };

    let max_ascii_length = ascii_lines.iter().map(|line| UnicodeWidthStr::width(line.as_str())).max().unwrap_or(0);
    let ascii_color_code = config
        .colors
        .get(&config.display.ascii_color)
        .map(|hex| hex_to_ansi(hex))
        .unwrap_or_else(|| "\x1b[0m".to_string());

    for (i, entry) in config.display.items.iter().enumerate() {
        let ascii_line = if i < ascii_lines.len() {
            &ascii_lines[i]
        } else {
            ""
        };

        let padded_ascii = format!("{:<width$}", ascii_line, width = max_ascii_length);
        let colored_ascii = format!("{}{}", ascii_color_code, padded_ascii);

        if entry.value.is_empty() {
            println!("{}", colored_ascii);
            continue;
        }

        let key_color_code = entry
            .color
            .as_ref()
            .and_then(|c| config.colors.get(c))
            .map(|hex| hex_to_ansi(hex))
            .unwrap_or_else(|| "\x1b[0m".to_string());

        let value_color_code = entry
            .value_color
            .as_ref()
            .and_then(|c| config.colors.get(c))
            .map(|hex| hex_to_ansi(hex))
            .unwrap_or_else(|| key_color_code.clone());

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

        if entry.key.is_empty() || entry.key == "user_info" {
            println!("{}  {}{}\x1b[0m", colored_ascii, value_color_code, output_value);
        } else {
            println!("{}  {}{}\x1b[0m{}{}{}\x1b[0m", 
                colored_ascii, 
                key_color_code, 
                entry.key, 
                separator, 
                value_color_code,
                output_value
            );
        }
    }

    for i in config.display.items.len()..ascii_lines.len() {
        let padded_ascii = format!("{:<width$}", ascii_lines[i], width = max_ascii_length);
        println!("{}{}", ascii_color_code, padded_ascii);
    }

    // Reset colors at the end
    print!("\x1b[0m");
}
