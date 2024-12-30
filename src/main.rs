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

fn format_output(label: &str, value: &str, separator: &str) -> String {
    format!("{}{}{}", label.bold(), separator, value)
}

fn main() {
    let config_path = dirs::config_dir()
        .map(|p| p.join("swiftfetch/config.toml"))
        .unwrap_or_else(|| "config.toml".into());

    let config_data = fs::read_to_string(&config_path).unwrap_or_else(|err| {
        eprintln!(
            "Failed to read configuration file at {}: {}",
            config_path.display(),
            err
        );
        process::exit(1);
    });

    let config: Config = toml::de::from_str(&config_data).unwrap_or_else(|err| {
        eprintln!(
            "Failed to parse configuration file at {}: {}",
            config_path.display(),
            err
        );
        process::exit(1);
    });

    let separator = config.display.separator.as_deref().unwrap_or(": ");
    let default_order = vec![
        "os", "kernel", "cpu", "wm", "packages", "flatpak", "ram", "uptime", "os_age", "editor",
        "shell", "terminal",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    let order = config.display.order.clone().unwrap_or(default_order);

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

    let user_host = config
        .display
        .user_host_format
        .as_deref()
        .unwrap_or("{username}@{hostname}")
        .replace("{username}", &username)
        .replace("{hostname}", &hostname);
    println!("\n{}", user_host.bold());
    

    let fields = vec![
        ("os", os_name),
        ("kernel", kernel_version),
        ("cpu", cpu_brand),
        ("wm", wm_de),
        ("packages", pacman_pkg_count.to_string()),
        ("flatpak", flatpak_pkg_count.to_string()),
        (
            "ram",
            format!("{:.2} GB / {:.2} GB", memory_used_gb, total_memory_gb),
        ),
        ("uptime", format_uptime(uptime_seconds)),
        ("os_age", os_age),
        ("editor", editor),
        ("shell", shell),
        ("terminal", terminal),
    
    ];

    let field_map: std::collections::HashMap<_, _> = fields.into_iter().collect();

    for field in order {
        if let Some(value) = field_map.get(field.as_str()) {
            let label = match field.as_str() {
                "os" => config.display.os.as_deref().unwrap_or("OS"),
                "kernel" => config.display.kernel.as_deref().unwrap_or("Kernel"),
                "cpu" => config.display.cpu.as_deref().unwrap_or("CPU"),
                "wm" => config.display.wm.as_deref().unwrap_or("WM"),
                "packages" => config.display.packages.as_deref().unwrap_or("PKGS"),
                "flatpak" => config.display.flatpak.as_deref().unwrap_or("FLAT"),
                "ram" => config.display.ram.as_deref().unwrap_or("RAM"),
                "uptime" => config.display.uptime.as_deref().unwrap_or("Uptime"),
                "os_age" => config.display.os_age.as_deref().unwrap_or("Age"),
                "editor" => config.display.editor.as_deref().unwrap_or("Editor"),
                "shell" => config.display.shell.as_deref().unwrap_or("Shell"),
                "terminal" => config.display.terminal.as_deref().unwrap_or("Terminal"),
                _ => continue,
            };
            println!("{}", format_output(label, value, separator));
        }
    }

    println!();
}