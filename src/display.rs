use crate::config::{Config, ConfigEntry};
use std::{fs, io::{self, BufRead}, path::Path, process::Command};
use unicode_width::UnicodeWidthStr;
use shellexpand;

pub fn get_default_ascii(distro: &str) -> Option<&'static str> {
    match distro.to_lowercase().as_str() {
        "arch" => Some(include_str!("../ascii/arch.txt")),
        "ubuntu" => Some(include_str!("../ascii/ubuntu.txt")),
        "cachyos" => Some(include_str!("../ascii/cachyos.txt")),
        "debian" => Some(include_str!("../ascii/debian.txt")),
        "fedora" => Some(include_str!("../ascii/fedora.txt")),
        "gentoo" => Some(include_str!("../ascii/gentoo.txt")),
        "void" => Some(include_str!("../ascii/void.txt")),
        "nixos" => Some(include_str!("../ascii/nixos.txt")),
        "pika" => Some(include_str!("../ascii/pika.txt")),
        _ => None,
    }
}

pub fn hex_to_ansi(color: &str) -> String {
    // First try ANSI color names
    if let Some(ansi_code) = get_ansi_color_code(color) {
        return ansi_code;
    }
    
    // Fallback to hex color parsing for custom colors
    if color.starts_with('#') && color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color[1..3], 16),
            u8::from_str_radix(&color[3..5], 16),
            u8::from_str_radix(&color[5..7], 16),
        ) {
            return format!("\x1b[38;2;{};{};{}m", r, g, b);
        }
    }
    
    // Default fallback
    "\x1b[0m".to_string()
}

fn get_ansi_color_code(color_name: &str) -> Option<String> {
    match color_name.to_lowercase().as_str() {
        // Standard 8 colors (30-37)
        "black" => Some("\x1b[30m".to_string()),
        "red" => Some("\x1b[31m".to_string()),
        "green" => Some("\x1b[32m".to_string()),
        "yellow" => Some("\x1b[33m".to_string()),
        "blue" => Some("\x1b[34m".to_string()),
        "magenta" => Some("\x1b[35m".to_string()),
        "cyan" => Some("\x1b[36m".to_string()),
        "white" => Some("\x1b[37m".to_string()),
        
        // Bright colors (90-97)
        "bright_black" | "gray" | "grey" => Some("\x1b[90m".to_string()),
        "bright_red" => Some("\x1b[91m".to_string()),
        "bright_green" => Some("\x1b[92m".to_string()),
        "bright_yellow" => Some("\x1b[93m".to_string()),
        "bright_blue" => Some("\x1b[94m".to_string()),
        "bright_magenta" => Some("\x1b[95m".to_string()),
        "bright_cyan" => Some("\x1b[96m".to_string()),
        "bright_white" => Some("\x1b[97m".to_string()),
        
        // Additional aliases
        "orange" => Some("\x1b[91m".to_string()), // bright red
        "purple" => Some("\x1b[35m".to_string()), // magenta
        "violet" => Some("\x1b[95m".to_string()), // bright magenta
        
        // Reset
        "reset" | "default" => Some("\x1b[0m".to_string()),
        
        _ => {
            // Show available colors if an invalid color is used
            eprintln!("Error: Unknown color '{}'. Available colors:", color_name);
            eprintln!("  Standard: black, red, green, yellow, blue, magenta, cyan, white");
            eprintln!("  Bright:   bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white");
            eprintln!("  Aliases:  gray/grey, orange, purple, violet, reset/default");
            eprintln!("  Or use hex colors like #FF0000");
            None
        }
    }
}

pub fn load_ascii_lines(config: &Config, os_name: &str) -> Vec<String> {
    let use_default = config.display.use_default_ascii.unwrap_or(true);

    match config.display.ascii_path.as_ref().map(|p| p.trim()).filter(|p| !p.is_empty()) {
        Some(ascii_path) => {
            let expanded_path = shellexpand::tilde(ascii_path).to_string();
            if Path::new(&expanded_path).exists() {
                if let Ok(file) = fs::File::open(&expanded_path) {
                    return io::BufReader::new(file).lines().filter_map(Result::ok).collect();
                }
            }
            // If file does not exist or failed to open
            if use_default {
                if let Some(default_ascii) = get_default_ascii(os_name) {
                    return default_ascii.lines().map(String::from).collect();
                }
            }
            vec![]
        }
        None if use_default => {
            if let Some(default_ascii) = get_default_ascii(os_name) {
                return default_ascii.lines().map(String::from).collect();
            }
            vec![]
        }
        _ => vec![],
    }
}

pub struct SystemData {
    pub os_name: String,
    pub kernel_version: String,
    pub cpu_brand: String,
    pub gpu: String,
    pub all_gpus: Vec<String>,
    pub username: String,
    pub hostname: String,
    pub wm_de: String,
    pub memory: String,
    pub pkg_count: usize,
    pub flatpak_pkg_count: usize,
    pub uptime_formatted: String,
    pub os_age: String,
    pub editor: String,
    pub shell: String,
    pub terminal: String,
    pub user_info: String,
    pub disk_usage: String,
    pub init_system: String,
    pub battery_info: String,
}

pub fn render_output(config: &Config, system_data: &SystemData) {
    let separator = config.display.separator.clone().unwrap_or_else(|| ": ".to_string());
    let ascii_lines = load_ascii_lines(config, &system_data.os_name);
    let show_all_gpus = config.display.show_all_gpus.unwrap_or(false);

    let max_ascii_length = ascii_lines
        .iter()
        .map(|line| UnicodeWidthStr::width(line.as_str()))
        .max()
        .unwrap_or(0);

        let ascii_color_code = config
            .colors
            .get(&config.display.ascii_color)
            .map(|color| hex_to_ansi(color))
            .unwrap_or_else(|| {
                eprintln!("Warning: ascii_color '{}' not found in colors map", config.display.ascii_color);
                get_ansi_color_code(&config.display.ascii_color).unwrap_or_else(|| "\x1b[0m".to_string())
            });

    let mut rendered_items = Vec::new();

    for entry in config.display.items.iter() {
        // Handle GPU entries specially if show_all_gpus is enabled
        if entry.value == "gpu" && show_all_gpus {
            // For GPU entries, add multiple entries if we have multiple GPUs
            for (gpu_idx, _gpu_name) in system_data.all_gpus.iter().enumerate() {
                let modified_entry = ConfigEntry {
                    key: entry.key.clone(),
                    r#type: entry.r#type.clone(),
                    value: format!("gpu{}", gpu_idx + 1),
                    color: entry.color.clone(),
                    value_color: entry.value_color.clone(),
                };
                rendered_items.push(modified_entry);
            }
        } else if entry.value != "gpu" || !show_all_gpus {
            // For non-GPU entries, or GPU entries when show_all_gpus is false, add as normal
            rendered_items.push(entry.clone());
        }
    }

    for (i, entry) in rendered_items.iter().enumerate() {
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
            .map(|color| hex_to_ansi(color))
            .unwrap_or_else(|| {
                if let Some(ref color_key) = entry.color {
                    eprintln!("Warning: color key '{}' not found in colors map", color_key);
                    get_ansi_color_code(color_key).unwrap_or_else(|| "\x1b[0m".to_string())
                } else {
                    "\x1b[0m".to_string()
                }
            });

        let value_color_code = match &entry.value_color {
            Some(value_color_key) => match config.colors.get(value_color_key) {
                Some(color) => hex_to_ansi(color),
                None => {
                    eprintln!("Warning: value_color key '{}' not found in colors map", value_color_key);
                    get_ansi_color_code(value_color_key).unwrap_or_else(|| "\x1b[0m".to_string())
                }
            },
            None => "\x1b[0m".to_string(),
        };

        let output_value = get_output_value(entry, system_data);

        if entry.r#type == "text" {
            let text_color = entry
                .color
                .as_ref()
                .and_then(|c| config.colors.get(c))
                .map(|color| hex_to_ansi(color))
                .unwrap_or_else(|| {
                    if let Some(ref color_key) = entry.color {
                        get_ansi_color_code(color_key).unwrap_or_else(|| "\x1b[0m".to_string())
                    } else {
                        "\x1b[0m".to_string()
                    }
                });
            println!("{}  {}{}\x1b[0m", colored_ascii, text_color, output_value);
        } else if entry.key.is_empty() || entry.key == "user_info" {
            println!("{}  {}{}\x1b[0m", colored_ascii, value_color_code, output_value);
        } else {
            println!(
                "{}  {}{}{}\x1b[0m{}{}{}\x1b[0m",
                colored_ascii,
                key_color_code,
                entry.key,
                separator,
                value_color_code,
                output_value,
                "\x1b[0m"
            );
        }
    }

    // Print remaining ASCII lines
    for i in rendered_items.len()..ascii_lines.len() {
        let padded_ascii = format!("{:<width$}", ascii_lines[i], width = max_ascii_length);
        println!("{}{}", ascii_color_code, padded_ascii);
    }

    print!("\x1b[0m");
}

fn get_output_value(entry: &ConfigEntry, system_data: &SystemData) -> String {
    match entry.r#type.as_str() {
        "default" => match entry.value.as_str() {
            "kernel" => system_data.kernel_version.clone(),
            "os" => system_data.os_name.clone(),
            "cpu" => system_data.cpu_brand.clone(),
            "gpu" => system_data.gpu.clone(),
            "gpu1" => system_data.all_gpus.get(0).cloned().unwrap_or_else(|| "No GPU".to_string()),
            "gpu2" => system_data.all_gpus.get(1).cloned().unwrap_or_else(|| "No secondary GPU".to_string()),
            "gpu3" => system_data.all_gpus.get(2).cloned().unwrap_or_else(|| "No third GPU".to_string()),
            "wm" => system_data.wm_de.clone(),
            "editor" => system_data.editor.clone(),
            "shell" => system_data.shell.clone(),
            "terminal" => system_data.terminal.clone(),
            "username" => system_data.username.clone(),
            "hostname" => system_data.hostname.trim().to_string(),
            "memory" => system_data.memory.clone(),
            "pkg_count" => system_data.pkg_count.to_string(),
            "flatpak_pkg_count" => system_data.flatpak_pkg_count.to_string(),
            "uptime_seconds" => system_data.uptime_formatted.clone(),
            "os_age" => system_data.os_age.clone(),
            "user_info" => system_data.user_info.clone(),
            "disk_usage" => system_data.disk_usage.clone(),
            "init_system" => system_data.init_system.clone(),
            "battery" => system_data.battery_info.clone(),
            _ => "Unknown default value".to_string(),
        },
        "text" => entry.value.clone(),
        "command" => {
            Command::new("sh")
                .arg("-c")
                .arg(&entry.value)
                .output()
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                .unwrap_or_else(|_| "Command failed".to_string())
        }
        _ => "Invalid type".to_string(),
    }
}
