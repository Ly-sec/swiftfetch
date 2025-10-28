use std::{fs, env, process::Command, io};

pub fn read_os_name() -> io::Result<String> {
    let os_release = read_file("/etc/os-release")?;
    for line in os_release.lines() {
        if line.starts_with("PRETTY_NAME") {
            return Ok(line.split('=').nth(1).unwrap_or("Unknown").trim_matches('"').to_string());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "OS name not found"))
}

pub fn read_kernel_version() -> io::Result<String> {
    let version_info = read_file("/proc/version")?;
    version_info.split_whitespace().nth(2)
        .map(|v| v.to_string())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Kernel version not found"))
}

pub fn read_cpu_info() -> io::Result<String> {
    let cpuinfo = read_file("/proc/cpuinfo")?;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            return Ok(line.split(':').nth(1).unwrap_or("Unknown CPU").trim().to_string());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "CPU info not found"))
}

pub fn read_memory_info() -> io::Result<(f64, f64)> {
    let meminfo = read_file("/proc/meminfo")?;

    // Total memory from MemTotal
    let total = extract_memory(&meminfo, "MemTotal");

    // Available memory from MemAvailable
    let available = extract_memory(&meminfo, "MemAvailable");

    // Calculate used memory
    let used = total - available;

    // Convert to GB
    let total_gb = total / 1024.0 / 1024.0;
    let used_gb = used / 1024.0 / 1024.0;

    Ok((total_gb, used_gb))
}

pub fn read_uptime() -> io::Result<u64> {
    let uptime_str = read_file("/proc/uptime")?;
    let secs = uptime_str.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0.0);
    Ok(secs as u64)
}

pub fn get_os_age() -> io::Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(
            "birth=$(stat -c %W / 2>/dev/null || echo 0); \
             if [ \"$birth\" -gt 0 ]; then \
               current=$(date +%s); \
               age=$(( (current - birth) / 86400 )); \
               if [ \"$age\" -eq 1 ]; then \
                 echo \"$age day\"; \
               else \
                 echo \"$age days\"; \
               fi; \
             else \
               echo \"Unsupported\"; \
             fi",
        )
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn get_hostname() -> io::Result<String> {
    read_file("/proc/sys/kernel/hostname")
}

pub fn get_username() -> String {
    env::var("USER").unwrap_or_else(|_| "Unknown".to_string())
}

pub fn get_editor() -> String {
    env::var("EDITOR").unwrap_or_else(|_| "nano".to_string())
}

pub fn get_shell() -> String {
    env::var("SHELL")
        .unwrap_or_else(|_| "Unknown".to_string())
        .replace("/usr/bin/", "")
        .replace("/bin/", "")
        .replace("/run/current-system/sw", "")
}

pub fn get_terminal() -> String {
    env::var("TERM")
        .unwrap_or_else(|_| "Unknown".to_string())
        .replace("xterm-", "")
}

pub fn get_disk_usage() -> io::Result<String> {
    // Get disk usage for root partition
    let output = Command::new("df")
        .arg("-h")
        .arg("/")
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines().skip(1) { // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let used = parts[2];
            let total = parts[1];
            let percentage = parts[4];
            return Ok(format!("{} / {} ({})", used, total, percentage));
        }
    }
    
    Err(io::Error::new(io::ErrorKind::NotFound, "Disk usage not found"))
}

pub fn detect_init_system() -> String {
    // Check for systemd first (most common)
    if Command::new("systemctl")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false) {
        return "systemd".to_string();
    }
    
    // Check for OpenRC
    if fs::metadata("/sbin/openrc").is_ok() || fs::metadata("/usr/sbin/openrc").is_ok() {
        return "OpenRC".to_string();
    }
    
    // Check for runit
    if fs::metadata("/etc/runit").is_ok() {
        return "runit".to_string();
    }
    
    // Check for SysV init
    if fs::metadata("/etc/init.d").is_ok() {
        return "SysV".to_string();
    }
    
    // Check for s6
    if fs::metadata("/etc/s6").is_ok() {
        return "s6".to_string();
    }
    
    "Unknown".to_string()
}

pub fn get_battery_info() -> io::Result<String> {
    // Try to find battery information in /sys/class/power_supply/
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                // Look for BAT* entries (typical battery naming)
                if name_str.starts_with("BAT") {
                    let battery_path = entry.path();
                    
                    // Read capacity percentage
                    let capacity_path = battery_path.join("capacity");
                    let status_path = battery_path.join("status");
                    
                    if let (Ok(capacity), Ok(status)) = (
                        fs::read_to_string(&capacity_path),
                        fs::read_to_string(&status_path)
                    ) {
                        let capacity = capacity.trim();
                        let status = status.trim();
                        
                        let status_icon = match status {
                            "Charging" => "⚡",
                            "Discharging" => "🔋",
                            "Full" => "🔋",
                            "Not charging" => "🔌",
                            _ => "🔋",
                        };
                        
                        return Ok(format!("{}% {}", capacity, status_icon));
                    }
                }
            }
        }
    }
    
    Err(io::Error::new(io::ErrorKind::NotFound, "No battery found"))
}

pub fn detect_gpu() -> io::Result<String> {
    let gpus = detect_all_gpus()?;
    
    if gpus.is_empty() {
        Ok("Unknown GPU".to_string())
    } else if gpus.len() == 1 {
        Ok(gpus[0].clone())
    } else {
        // For multiple GPUs, return the first (discrete) one
        // The display layer can handle showing multiple if configured
        Ok(gpus[0].clone())
    }
}

pub fn detect_all_gpus() -> io::Result<Vec<String>> {
    // Try lspci first (most reliable)
    if let Ok(output) = Command::new("lspci")
        .output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Look for VGA compatible controllers and 3D controllers
        let gpu_lines: Vec<&str> = output_str
            .lines()
            .filter(|line| {
                line.contains("VGA compatible controller") || 
                line.contains("3D controller") ||
                line.contains("Display controller")
            })
            .collect();
        
        if !gpu_lines.is_empty() {
            let mut gpus = Vec::new();
            
            for line in gpu_lines {
                if let Some(gpu_info) = parse_gpu_from_lspci(line) {
                    gpus.push(gpu_info);
                }
            }
            
            if !gpus.is_empty() {
                // Sort GPUs: discrete first, then integrated
                gpus.sort_by(|a, b| {
                    let a_integrated = a.to_lowercase().contains("integrated");
                    let b_integrated = b.to_lowercase().contains("integrated");
                    a_integrated.cmp(&b_integrated)
                });
                
                return Ok(gpus);
            }
        }
    }
    
    // Fallback: try to read from /sys/class/drm
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("card") && !name.contains("-") {
                        let device_path = path.join("device/vendor");
                        let product_path = path.join("device/device");
                        
                        if let (Ok(vendor), Ok(device)) = (
                            fs::read_to_string(&device_path),
                            fs::read_to_string(&product_path)
                        ) {
                            let vendor_id = vendor.trim();
                            let device_id = device.trim();
                            
                            if let Some(gpu_name) = lookup_gpu_by_ids(vendor_id, device_id) {
                                return Ok(vec![gpu_name]);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(vec![])
}

fn parse_gpu_from_lspci(line: &str) -> Option<String> {
    
    if let Some(colon_pos) = line.rfind(':') {
        let gpu_part = line[colon_pos + 1..].trim();
        
        // Clean up revision info
        let cleaned = gpu_part
            .split(" (rev ")
            .next()
            .unwrap_or(gpu_part)
            .trim();
        
        // Try to extract the actual GPU name
        if let Some(gpu_name) = extract_gpu_model_name(cleaned) {
            // Add [Discrete] or [Integrated] tag
            let gpu_type = detect_gpu_type(line, &gpu_name);
            return Some(format!("{}{}", gpu_name, gpu_type));
        }
    }
    None
}

fn extract_gpu_model_name(gpu_description: &str) -> Option<String> {
    // If no good bracket content, try to parse the description
    if gpu_description.contains("AMD") || gpu_description.contains("Advanced Micro Devices") {
        return parse_amd_gpu(gpu_description);
    } else if gpu_description.contains("NVIDIA") || gpu_description.contains("GeForce") {
        return parse_nvidia_gpu(gpu_description);
    } else if gpu_description.contains("Intel") {
        return parse_intel_gpu(gpu_description);
    }
    
    // Look for patterns in square brackets as fallback
    if let Some(start) = gpu_description.rfind('[') {
        if let Some(end) = gpu_description[start..].find(']') {
            let bracket_content = &gpu_description[start + 1..start + end];
            
            // Skip vendor-only brackets like [AMD/ATI]
            if !bracket_content.contains('/') && bracket_content.len() > 3 {
                return Some(simplify_gpu_name_v2(bracket_content));
            }
        }
    }
    
    Some(gpu_description.to_string())
}

fn parse_amd_gpu(description: &str) -> Option<String> {
    // First try to get the exact model from device ID by reading /sys/class/drm
    if let Ok(exact_model) = get_exact_amd_model_from_sysfs() {
        return Some(exact_model);
    }
    
    // Look for brackets with Radeon content
    if let Some(bracket_start) = description.rfind('[') {
        if let Some(bracket_end) = description[bracket_start..].find(']') {
            let bracket_content = &description[bracket_start + 1..bracket_start + bracket_end];
            if bracket_content.contains("Radeon") {
                // If it contains a range like "RX 7700 XT / 7800 XT", try to determine which one
                if bracket_content.contains(" / ") {
                    return Some(format!("AMD {}", resolve_amd_gpu_range(bracket_content)));
                } else {
                    return Some(format!("AMD {}", bracket_content));
                }
            }
        }
    }
    
    // Look for "Radeon" pattern in the main text
    if let Some(radeon_pos) = description.find("Radeon") {
        let after_radeon = &description[radeon_pos..];
        // Extract until we hit a bracket or other delimiter
        let radeon_part = after_radeon
            .split(" [")
            .next()
            .unwrap_or(after_radeon)
            .split(" (")
            .next()
            .unwrap_or(after_radeon);
        
        return Some(format!("AMD {}", radeon_part.trim()));
    }
    
    // Check for integrated AMD GPUs
    if description.contains("Raphael") || description.contains("Renoir") || 
       description.contains("Cezanne") || description.contains("Barcelo") {
        return Some("AMD Raphael".to_string());
    }
    
    Some("AMD GPU".to_string())
}

fn parse_nvidia_gpu(description: &str) -> Option<String> {
    // Look for GeForce pattern
    if let Some(geforce_pos) = description.find("GeForce") {
        let after_geforce = &description[geforce_pos..];
        let geforce_part = after_geforce
            .split(" [")
            .next()
            .unwrap_or(after_geforce)
            .split(" (")
            .next()
            .unwrap_or(after_geforce);
        
        return Some(geforce_part.trim().to_string());
    }
    
    // Look for bracket content
    if let Some(start) = description.rfind('[') {
        if let Some(end) = description[start..].find(']') {
            let bracket_content = &description[start + 1..start + end];
            if bracket_content.contains("GeForce") || bracket_content.contains("RTX") || bracket_content.contains("GTX") {
                return Some(bracket_content.to_string());
            }
        }
    }
    
    Some("NVIDIA GPU".to_string())
}

fn parse_intel_gpu(description: &str) -> Option<String> {
    // Remove "Intel Corporation" prefix
    let cleaned = description.replace("Intel Corporation ", "");
    
    // Look for common Intel GPU patterns
    if cleaned.contains("UHD Graphics") || cleaned.contains("HD Graphics") || 
       cleaned.contains("Iris") || cleaned.contains("Arc") {
        return Some(format!("Intel {}", 
            cleaned.split(" [").next().unwrap_or(&cleaned).trim()));
    }
    
    Some(format!("Intel {}", cleaned.trim()))
}

fn simplify_gpu_name_v2(name: &str) -> String {
    name.trim().to_string()
}

fn detect_gpu_type(lspci_line: &str, gpu_name: &str) -> String {
    // Check if it's likely integrated
    let line_lower = lspci_line.to_lowercase();
    let name_lower = gpu_name.to_lowercase();
    
    if line_lower.contains("integrated") || 
       name_lower.contains("integrated") ||
       name_lower.contains("raphael") || 
       name_lower.contains("renoir") ||
       name_lower.contains("cezanne") ||
       name_lower.contains("iris") ||
       name_lower.contains("uhd") ||
       name_lower.contains("hd graphics") ||
       // Intel integrated GPUs are usually on bus 00:02.0
       (lspci_line.starts_with("00:02.0") && line_lower.contains("intel")) {
        " [Integrated]".to_string()
    } else {
        " [Discrete]".to_string()
    }
}


fn get_exact_amd_model_from_sysfs() -> io::Result<String> {
    // Try to read the exact model from sysfs using device IDs
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("card") && !name.contains("-") {
                        let device_path = path.join("device/device");
                        let vendor_path = path.join("device/vendor");
                        
                        if let (Ok(device_id), Ok(vendor_id)) = (
                            fs::read_to_string(&device_path),
                            fs::read_to_string(&vendor_path)
                        ) {
                            let device_id = device_id.trim();
                            let vendor_id = vendor_id.trim();
                            
                            // Check if it's AMD (0x1002)
                            if vendor_id == "0x1002" {
                                if let Some(exact_model) = amd_device_id_to_name(device_id) {
                                    return Ok(format!("AMD {}", exact_model));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "AMD GPU not found in sysfs"))
}

fn amd_device_id_to_name(device_id: &str) -> Option<String> {
    // Common AMD GPU device IDs - this helps determine exact models
    match device_id {
        // RX 7800 XT
        "0x7480" => Some("Radeon RX 7800 XT".to_string()),
        // RX 7700 XT  
        "0x7479" => Some("Radeon RX 7700 XT".to_string()),
        // RX 6800 XT
        "0x73bf" => Some("Radeon RX 6800 XT".to_string()),
        // RX 6700 XT
        "0x73df" => Some("Radeon RX 6700 XT".to_string()),
        // Add more as needed
        _ => None,
    }
}

fn resolve_amd_gpu_range(bracket_content: &str) -> String {
    // For ranges like "Radeon RX 7700 XT / 7800 XT", default to the higher-end model
    // This is a reasonable assumption since the higher model is usually listed last
    if bracket_content.contains(" / ") {
        let parts: Vec<&str> = bracket_content.split(" / ").collect();
        if let Some(last_part) = parts.last() {
            // Take the last (usually higher-end) model
            return last_part.trim().to_string();
        }
    }
    bracket_content.to_string()
}

fn lookup_gpu_by_ids(vendor_id: &str, device_id: &str) -> Option<String> {
    // Use device ID mapping for better accuracy
    match vendor_id {
        "0x10de" => Some(format!("NVIDIA GPU ({})", device_id)),
        "0x1002" => {
            if let Some(exact_model) = amd_device_id_to_name(device_id) {
                Some(format!("AMD {}", exact_model))
            } else {
                Some(format!("AMD GPU ({})", device_id))
            }
        },
        "0x8086" => Some(format!("Intel GPU ({})", device_id)),
        _ => Some(format!("GPU ({}/{})", vendor_id, device_id)),
    }
}

fn extract_memory(meminfo: &str, key: &str) -> f64 {
    meminfo
        .lines()
        .find(|line| line.starts_with(key))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0)
}

pub fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}
