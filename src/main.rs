use std::{fs, process};
use serde::Deserialize;
use colored::*;
use dirs;

mod sys_info;
use sys_info::get_system_info;

#[derive(Deserialize)]
struct Config {
    display: DisplayConfig,
}

#[derive(Deserialize)]
struct DisplayConfig {
    user_host_format: Option<String>,
    separator: Option<String>,
    order: Option<Vec<String>>,
    os: Option<String>,
    kernel: Option<String>,
    cpu: Option<String>,
    wm: Option<String>,
    packages: Option<String>,
    flatpak: Option<String>,
    ram: Option<String>,
    uptime: Option<String>,
    os_age: Option<String>,
    editor: Option<String>,
    shell: Option<String>,
    terminal: Option<String>,
}

fn format_uptime(uptime_seconds: u64) -> String {
    let hours = uptime_seconds / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn main() {
    // Determine the configuration file path
    let config_path = dirs::config_dir()
        .map(|p| p.join("swiftfetch/config.toml"))
        .unwrap_or_else(|| "config.toml".into()); // Fallback to local config.toml

    // Read the configuration file
    let config_data = fs::read_to_string(&config_path).unwrap_or_else(|_| {
        eprintln!("Failed to read configuration file at {}", config_path.display());
        process::exit(1);
    });

    // Parse the configuration file
    let config: Config = toml::de::from_str(&config_data).unwrap_or_else(|_| {
        eprintln!("Failed to parse configuration file at {}", config_path.display());
        process::exit(1);
    });

    // Set defaults for missing configuration values
    let separator = config.display.separator.as_deref().unwrap_or(": ");
    let order = config.display.order.clone().unwrap_or_else(|| vec![
        "os".to_string(),
        "kernel".to_string(),
        "cpu".to_string(),
        "wm".to_string(),
        "packages".to_string(),
        "flatpak".to_string(),
        "ram".to_string(),
        "uptime".to_string(),
        "os_age".to_string(),
        "editor".to_string(),
        "shell".to_string(),
        "terminal".to_string(),
    ]);

    // Fetch system information.
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

    // Fetch shell and terminal from environment variables
    // Display the user-host format with a fallback
    let user_host = config
        .display
        .user_host_format
        .as_deref()
        .unwrap_or("{username}@{hostname}")
        .replace("{username}", &username)
        .replace("{hostname}", &hostname);
    println!("{}", user_host.bold());

    // Display information based on the order defined in the configuration
    for field in order.iter() {
        match field.as_str() {
            "os" => println!(
                "{}{}{}",
                config.display.os.as_deref().unwrap_or("OS").bold(),
                separator,
                os_name
            ),
            "kernel" => println!(
                "{}{}{}",
                config.display.kernel.as_deref().unwrap_or("Kernel").bold(),
                separator,
                kernel_version
            ),
            "cpu" => println!(
                "{}{}{}",
                config.display.cpu.as_deref().unwrap_or("CPU").bold(),
                separator,
                cpu_brand
            ),
            "wm" => println!(
                "{}{}{}",
                config.display.wm.as_deref().unwrap_or("WM").bold(),
                separator,
                wm_de
            ),
            "packages" => println!(
                "{}{}{}",
                config.display.packages.as_deref().unwrap_or("PKGS").bold(),
                separator,
                pacman_pkg_count
            ),
            "flatpak" => println!(
                "{}{}{}",
                config.display.flatpak.as_deref().unwrap_or("FLAT").bold(),
                separator,
                flatpak_pkg_count
            ),
            "ram" => println!(
                "{}{}{}",
                config.display.ram.as_deref().unwrap_or("RAM").bold(),
                separator,
                format!("{:.2} GB / {:.2} GB", memory_used_gb, total_memory_gb)
            ),
            "uptime" => println!(
                "{}{}{}",
                config.display.uptime.as_deref().unwrap_or("Uptime").bold(),
                separator,
                format_uptime(uptime_seconds)
            ),
            "os_age" => println!(
                "{}{}{}",
                config.display.os_age.as_deref().unwrap_or("Age").bold(),
                separator,
                os_age
            ),
            "editor" => println!(
                "{}{}{}",
                config.display.editor.as_deref().unwrap_or("Editor").bold(),
                separator,
                editor
            ),
            "shell" => println!(
                "{}{}{}",
                config.display.shell.as_deref().unwrap_or("Shell").bold(),
                separator,
                shell
            ),
            "terminal" => println!(
                "{}{}{}",
                config.display.terminal.as_deref().unwrap_or("Terminal").bold(),
                separator,
                terminal
            ),
            _ => continue, // Ignore invalid or unknown fields
        }
    }
    println!();
}
