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
    user_host_format: String,
    separator: String,
    order: Vec<String>,
    os: String,
    kernel: String,
    cpu: String,
    wm: String,
    packages: String,
    flatpak: String,
    ram: String,
    uptime: String,
    os_age: String,
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
    let config_path = dirs::config_dir()
        .map(|p| p.join("swiftfetch/config.toml"))
        .unwrap_or_else(|| "config.toml".into()); // Fallback to local config.toml

    let config_data = fs::read_to_string(&config_path).unwrap_or_else(|_| {
        eprintln!("Failed to read configuration file at {}", config_path.display());
        process::exit(1);
    });

    let config: Config = toml::de::from_str(&config_data).unwrap_or_else(|_| {
        eprintln!("Failed to parse configuration file at {}", config_path.display());
        process::exit(1);
    });

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
    ) = get_system_info();

    // Display the user-host format.
    let user_host = config
        .display
        .user_host_format
        .replace("{username}", &username)
        .replace("{hostname}", &hostname);
    println!("{}", user_host.bold());

    // Display information based on the order defined in the configuration.
    for field in config.display.order.iter() {
        match field.as_str() {
            "os" => println!(
                "{}{}{}",
                config.display.os.bold(),
                config.display.separator,
                os_name
            ),
            "kernel" => println!(
                "{}{}{}",
                config.display.kernel.bold(),
                config.display.separator,
                kernel_version
            ),
            "cpu" => println!(
                "{}{}{}",
                config.display.cpu.bold(),
                config.display.separator,
                cpu_brand
            ),
            "wm" => println!(
                "{}{}{}",
                config.display.wm.bold(),
                config.display.separator,
                wm_de
            ),
            "packages" => println!(
                "{}{}{}",
                config.display.packages.bold(),
                config.display.separator,
                pacman_pkg_count
            ),
            "flatpak" => println!(
                "{}{}{}",
                config.display.flatpak.bold(),
                config.display.separator,
                flatpak_pkg_count
            ),
            "ram" => println!(
                "{}{}{}",
                config.display.ram.bold(),
                config.display.separator,
                format!("{:.2} GB / {:.2} GB", memory_used_gb, total_memory_gb)
            ),
            "uptime" => println!(
                "{}{}{}",
                config.display.uptime.bold(),
                config.display.separator,
                format_uptime(uptime_seconds)
            ),
            "os_age" => println!(
                "{}{}{}",
                config.display.os_age.bold(),
                config.display.separator,
                os_age
            ),
            _ => continue,
        }
    }
}
