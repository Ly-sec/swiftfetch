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
    let wm_de = env::var("XDG_SESSION_DESKTOP")
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".to_string());

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

fn get_os_age() -> Result<String, std::io::Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("birth_install=$(stat -c %W /); current=$(date +%s); time_progression=$((current - birth_install)); days_difference=$((time_progression / 86400)); echo $days_difference day\\(s\\)")
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to get OS age"));
    }

    let os_age = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(os_age)
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
    for line in version_info.lines() {
        if line.starts_with("Linux version") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 2 {
                return Ok(parts[2].to_string()); // Kernel version is the 3rd element
            }
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Kernel version not found"))
}

fn read_cpu_info() -> Result<String, std::io::Error> {
    let cpuinfo = read_file("/proc/cpuinfo")?;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            return Ok(line.split(":").nth(1).unwrap_or("Unknown CPU").trim().to_string());
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "CPU info not found"))
}

fn read_memory_info() -> (f64, f64) {
    let meminfo = read_file("/proc/meminfo").unwrap_or_else(|_| "".to_string());

    let total_memory_kb = meminfo.lines()
        .find(|line| line.starts_with("MemTotal"))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    let free_memory_kb = meminfo.lines()
        .find(|line| line.starts_with("MemFree"))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    let buffers_kb = meminfo.lines()
        .find(|line| line.starts_with("Buffers"))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    let cached_kb = meminfo.lines()
        .find(|line| line.starts_with("Cached"))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    let total_memory_gb = total_memory_kb / 1024.0 / 1024.0; // Convert to GB
    let available_memory_kb = free_memory_kb + buffers_kb + cached_kb;
    let used_memory_gb = (total_memory_kb - available_memory_kb) / 1024.0 / 1024.0; // Convert to GB

    (total_memory_gb, used_memory_gb)
}


fn read_uptime() -> Result<u64, std::io::Error> {
    let uptime_str = read_file("/proc/uptime")?;
    let uptime_secs: f64 = uptime_str.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0.0);
    Ok(uptime_secs as u64)
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

fn get_pacman_package_count() -> usize {
    let pacman_db_path = "/var/lib/pacman/local";
    let count = fs::read_dir(pacman_db_path)
        .unwrap_or_else(|_| panic!("Failed to read pacman database"))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_dir()))
        .count();

    count
}

fn get_flatpak_package_count() -> usize {
    let flatpak_dir = "/var/lib/flatpak/app";
    let count = fs::read_dir(flatpak_dir)
        .unwrap_or_else(|_| panic!("Failed to read flatpak directory"))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_dir()))
        .count();

    count
}
