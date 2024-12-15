use std::fs;
use std::env;
use std::process::Command;

pub fn get_system_info() -> (
    String, // OS Name
    String, // Kernel Version
    String, // CPU Brand
    String, // Username
    String, // Hostname
    String, // WM/DE
    f64,    // Memory Used (GB)
    f64,    // Total Memory (GB)
    usize,  // Pacman Package Count
    usize,  // Flatpak Package Count
    u64,    // Uptime
    String, // OS Age (in days)
) {
    let os_name = read_os_name().unwrap_or_else(|_| "Unknown".to_string());
    let kernel_version = read_kernel_version().unwrap_or_else(|_| "Unknown".to_string());
    let cpu_brand = read_cpu_info().unwrap_or_else(|_| "Unknown CPU".to_string());
    let uptime = read_uptime().unwrap_or(0);
    let (total_memory_gb, memory_used_gb) = read_memory_info();

    let username = env::var("USER").unwrap_or_else(|_| "Unknown".to_string());
    let hostname = read_file("/proc/sys/kernel/hostname").unwrap_or_else(|_| "Unknown".to_string());
    let wm_de = detect_wm_or_de();

    let (pacman_pkg_count, flatpak_pkg_count) = rayon::join(
        get_pacman_package_count,
        get_flatpak_package_count,
    );

    let os_age = get_os_age().unwrap_or_else(|_| "Unknown".to_string());

    (
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
        uptime,
        os_age,
    )
}

fn detect_wm_or_de() -> String {
    // Check common environment variables first
    if let Ok(env_var) = env::var("XDG_CURRENT_DESKTOP").or_else(|_| env::var("DESKTOP_SESSION")) {
        if !env_var.is_empty() {
            return capitalize_first_letter(&env_var);
        }
    }

    // Check for Wayland
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return "Wayland".to_string();
    }

    // Fallback: check for running WM/DE processes
    let output = Command::new("sh")
        .arg("-c")
        .arg("ps -e | grep -E 'sway|hyprland|kwin|mutter|xfwm4|openbox|i3|bspwm|awesome|weston|gnome-session'")
        .output()
        .ok();

    if let Some(output) = output {
        let result = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = result.lines().next() {
            if let Some(process) = line.split_whitespace().last() {
                return capitalize_first_letter(process);
            }
        }
    }

    "Unknown".to_string()
}

fn capitalize_first_letter(s: &str) -> String {
    if let Some(first) = s.chars().next() {
        format!("{}{}", first.to_uppercase(), &s[1..])
    } else {
        s.to_string()
    }
}

fn get_os_age() -> Result<String, std::io::Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(
            "birth=$(stat -c %W / 2>/dev/null || echo 0); \
             if [ \"$birth\" -gt 0 ]; then \
               current=$(date +%s); \
               age=$(( (current - birth) / 86400 )); \
               echo \"$age day(s)\"; \
             else \
               echo \"Unsupported\"; \
             fi",
        )
        .output()?;

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(result)
}


fn read_os_name() -> Result<String, std::io::Error> {
    let os_release = read_file("/etc/os-release")?;
    for line in os_release.lines() {
        if line.starts_with("PRETTY_NAME") {
            return Ok(line.split('=').nth(1).unwrap_or("Unknown").trim_matches('"').to_string());
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "OS name not found"))
}

fn read_kernel_version() -> Result<String, std::io::Error> {
    let version_info = read_file("/proc/version")?;
    version_info.split_whitespace().nth(2)
        .map(|v| v.to_string())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Kernel version not found"))
}

fn read_cpu_info() -> Result<String, std::io::Error> {
    let cpuinfo = read_file("/proc/cpuinfo")?;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            return Ok(line.split(':').nth(1).unwrap_or("Unknown CPU").trim().to_string());
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "CPU info not found"))
}

fn read_memory_info() -> (f64, f64) {
    let meminfo = read_file("/proc/meminfo").unwrap_or_default();
    let total = extract_memory(&meminfo, "MemTotal");
    let free = extract_memory(&meminfo, "MemFree") 
             + extract_memory(&meminfo, "Buffers") 
             + extract_memory(&meminfo, "Cached");

    let total_gb = total / 1024.0 / 1024.0;
    let used_gb = (total - free) / 1024.0 / 1024.0;

    (total_gb, used_gb)
}

fn extract_memory(meminfo: &str, key: &str) -> f64 {
    meminfo.lines()
        .find(|line| line.starts_with(key))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn read_uptime() -> Result<u64, std::io::Error> {
    let uptime_str = read_file("/proc/uptime")?;
    let secs = uptime_str.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0.0);
    Ok(secs as u64)
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

fn get_pacman_package_count() -> usize {
    fs::read_dir("/var/lib/pacman/local")
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}

fn get_flatpak_package_count() -> usize {
    let flatpak_dir = "/var/lib/flatpak/app";
    
    if let Ok(entries) = fs::read_dir(flatpak_dir) {
        entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_dir()))
            .count()
    } else {
        0
    }
}

