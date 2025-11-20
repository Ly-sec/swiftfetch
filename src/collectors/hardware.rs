//! Hardware information collection (CPU, GPU, Memory, Disk)

use crate::data::{CpuInfo, DiskInfo, GpuInfo, HardwareInfo, MemoryInfo};
use crate::error::Result;
use crate::utils::{command::*, parsing::*};
use std::fs;

/// Collect all hardware information (parallelized for speed)
pub fn collect_hardware_info() -> Result<HardwareInfo> {
    // Collect CPU, GPU, memory, and disk info in parallel
    let ((cpu, gpu), (memory, disk)) = rayon::join(
        || rayon::join(|| collect_cpu_info(), || collect_gpu_info()),
        || rayon::join(|| collect_memory_info(), || collect_disk_info()),
    );

    Ok(HardwareInfo {
        cpu: cpu?,
        gpu: gpu?,
        memory: memory?,
        disk: disk?,
    })
}

/// Collect CPU information
pub fn collect_cpu_info() -> Result<CpuInfo> {
    Ok(CpuInfo {
        brand: read_cpu_info()?,
    })
}

/// Collect GPU information
pub fn collect_gpu_info() -> Result<GpuInfo> {
    let all_gpus = detect_all_gpus().unwrap_or_else(|_| vec!["Unknown GPU".to_string()]);
    let primary = if all_gpus.is_empty() {
        "Unknown GPU".to_string()
    } else {
        all_gpus[0].clone()
    };

    Ok(GpuInfo { primary, all_gpus })
}

/// Collect memory information
pub fn collect_memory_info() -> Result<MemoryInfo> {
    let (total_gb, used_gb) = read_memory_info()?;
    let formatted = format!("{:.2} GB / {:.2} GB", used_gb, total_gb);

    Ok(MemoryInfo {
        used_gb,
        total_gb,
        formatted,
    })
}

/// Collect disk information
pub fn collect_disk_info() -> Result<DiskInfo> {
    Ok(DiskInfo {
        usage: get_disk_usage()?,
    })
}

// Hardware detection functions
fn read_cpu_info() -> Result<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // Read line by line to find model name early (stops after first CPU)
    let file = File::open("/proc/cpuinfo")?;
    let mut reader = BufReader::new(file);
    let mut line = String::with_capacity(128);

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        if line.starts_with("model name") {
            return Ok(extract_after_colon(&line).unwrap_or("Unknown CPU".to_string()));
        }
    }
    Err(crate::error::SwiftfetchError::Detection(
        "CPU info not found".to_string(),
    ))
}

fn read_memory_info() -> Result<(f64, f64)> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // Read line by line to extract memory values early (optimized with reuse)
    let file = File::open("/proc/meminfo")?;
    let mut reader = BufReader::new(file);
    let mut line = String::with_capacity(64);

    let mut total: f64 = 0.0;
    let mut available: f64 = 0.0;
    let mut found_total = false;
    let mut found_available = false;

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }

        if !found_total && line.starts_with("MemTotal") {
            total = extract_memory_from_line(&line);
            found_total = true;
        } else if !found_available && line.starts_with("MemAvailable") {
            available = extract_memory_from_line(&line);
            found_available = true;
        }

        // Early exit if we found both
        if found_total && found_available {
            break;
        }
    }

    // Calculate used memory
    let used = total - available;

    // Convert to GB
    let total_gb = total / 1024.0 / 1024.0;
    let used_gb = used / 1024.0 / 1024.0;

    Ok((total_gb, used_gb))
}

fn extract_memory_from_line(line: &str) -> f64 {
    if let Some(value_str) = line.split(':').nth(1) {
        let value_str = value_str.trim().replace(" kB", "");
        return value_str.parse().unwrap_or(0.0);
    }
    0.0
}

fn get_disk_usage() -> Result<String> {
    use libc;
    use std::ffi::CString;

    // Use statvfs syscall directly (faster than df command)
    unsafe {
        let path = CString::new("/").unwrap();
        let mut stat: libc::statvfs = std::mem::zeroed();

        if libc::statvfs(path.as_ptr(), &mut stat) == 0 {
            let total_bytes = stat.f_blocks.wrapping_mul(stat.f_frsize as u64);
            let available_bytes = stat.f_bavail.wrapping_mul(stat.f_frsize as u64);
            let used_bytes = total_bytes.saturating_sub(available_bytes);

            let format_size = |bytes: u64| {
                if bytes >= 1_000_000_000_000 {
                    format!("{:.1}T", bytes as f64 / 1_000_000_000_000.0)
                } else if bytes >= 1_000_000_000 {
                    format!("{:.1}G", bytes as f64 / 1_000_000_000.0)
                } else if bytes >= 1_000_000 {
                    format!("{:.1}M", bytes as f64 / 1_000_000.0)
                } else {
                    format!("{}K", bytes / 1024)
                }
            };

            let total = format_size(total_bytes);
            let used = format_size(used_bytes);
            let percent = if total_bytes > 0 {
                ((used_bytes as f64 / total_bytes as f64) * 100.0) as u64
            } else {
                0
            };

            return Ok(format!("{} / {} ({}%)", used, total, percent));
        }
    }

    // Fallback to df command if statvfs fails
    let output = run_command("df", &["-h", "/"])?;
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let used = parts[2];
            let total = parts[1];
            let percentage = parts[4];
            return Ok(format!("{} / {} ({})", used, total, percentage));
        }
    }

    Err(crate::error::SwiftfetchError::Detection(
        "Disk usage not found".to_string(),
    ))
}

fn detect_all_gpus() -> Result<Vec<String>> {
    // Use direct sysfs reading (faster than lspci subprocess)
    let mut gpus = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("card") && !name.contains("-") {
                        // Read device name directly from sysfs
                        if let Ok(device_name) = fs::read_to_string(path.join("device/name")) {
                            let name = device_name.trim();
                            if !name.is_empty() {
                                // Determine if integrated or discrete
                                let gpu_type = if is_integrated_gpu(&path) {
                                    " [Integrated]"
                                } else {
                                    " [Discrete]"
                                };
                                gpus.push(format!("{}{}", name, gpu_type));
                            }
                        }
                    }
                }
            }
        }
    }

    if !gpus.is_empty() {
        // Sort GPUs: discrete first, then integrated
        gpus.sort_by(|a, b| {
            let a_integrated = a.contains("Integrated");
            let b_integrated = b.contains("Integrated");
            a_integrated.cmp(&b_integrated)
        });
        return Ok(gpus);
    }

    // Fallback to lspci if sysfs fails
    if let Ok(output) = run_command("lspci", &[]) {
        let gpu_lines: Vec<&str> = output
            .lines()
            .filter(|line| {
                line.contains("VGA compatible controller")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
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
                gpus.sort_by(|a, b| {
                    let a_integrated = a.contains("Integrated");
                    let b_integrated = b.contains("Integrated");
                    a_integrated.cmp(&b_integrated)
                });
                return Ok(gpus);
            }
        }
    }

    Ok(vec![])
}

fn is_integrated_gpu(card_path: &std::path::Path) -> bool {
    // Check vendor - Intel GPUs are usually integrated
    if let Ok(vendor) = fs::read_to_string(card_path.join("device/vendor")) {
        if vendor.trim() == "0x8086" {
            return true;
        }
    }

    // Check if it's on a typical integrated GPU bus (00:02.0)
    if let Ok(pci_path) = fs::read_to_string(card_path.join("device")) {
        if pci_path.contains("0000:00:02") {
            return true;
        }
    }

    false
}

fn parse_gpu_from_lspci(line: &str) -> Option<String> {
    if let Some(colon_pos) = line.rfind(':') {
        let gpu_part = line[colon_pos + 1..].trim();

        // Clean up revision info
        let cleaned = gpu_part.split(" (rev ").next().unwrap_or(gpu_part).trim();

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
    // Parse AMD GPUs
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
                return Some(bracket_content.to_string());
            }
        }
    }

    Some(gpu_description.to_string())
}

fn parse_amd_gpu(description: &str) -> Option<String> {
    // Look for brackets with Radeon content
    if let Some(bracket_start) = description.rfind('[') {
        if let Some(bracket_end) = description[bracket_start..].find(']') {
            let bracket_content = &description[bracket_start + 1..bracket_start + bracket_end];
            if bracket_content.contains("Radeon") {
                // If it contains a range like "RX 7700 XT / 7800 XT", take the higher-end model
                if bracket_content.contains(" / ") {
                    let parts: Vec<&str> = bracket_content.split(" / ").collect();
                    if let Some(last_part) = parts.last() {
                        return Some(format!("AMD {}", last_part.trim()));
                    }
                } else {
                    return Some(format!("AMD {}", bracket_content));
                }
            }
        }
    }

    // Look for "Radeon" pattern in the main text
    if let Some(radeon_pos) = description.find("Radeon") {
        let after_radeon = &description[radeon_pos..];
        let radeon_part = after_radeon
            .split(" [")
            .next()
            .unwrap_or(after_radeon)
            .split(" (")
            .next()
            .unwrap_or(after_radeon);

        return Some(format!("AMD {}", radeon_part.trim()));
    }

    // Check for integrated AMD GPUs by codename (Raphael, Renoir, etc.)
    if description.contains("Raphael")
        || description.contains("Renoir")
        || description.contains("Cezanne")
        || description.contains("Barcelo")
    {
        return Some("AMD Raphael".to_string());
    }

    // Extract any codename from brackets if no Radeon found
    if let Some(bracket_start) = description.rfind('[') {
        if let Some(bracket_end) = description[bracket_start..].find(']') {
            let bracket_content = &description[bracket_start + 1..bracket_start + bracket_end];
            // Skip vendor brackets like [AMD/ATI]
            if !bracket_content.contains('/') && bracket_content.len() > 2 {
                return Some(format!("AMD {}", bracket_content));
            }
        }
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
        let cleaned = geforce_part.trim().trim_end_matches(']').to_string();
        return Some(cleaned);
    }

    // Look for bracket content
    if let Some(start) = description.rfind('[') {
        if let Some(end) = description[start..].find(']') {
            let bracket_content = &description[start + 1..start + end];
            if bracket_content.contains("GeForce")
                || bracket_content.contains("RTX")
                || bracket_content.contains("GTX")
            {
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
    if cleaned.contains("UHD Graphics")
        || cleaned.contains("HD Graphics")
        || cleaned.contains("Iris")
        || cleaned.contains("Arc")
    {
        return Some(format!(
            "Intel {}",
            cleaned.split(" [").next().unwrap_or(&cleaned).trim()
        ));
    }

    Some(format!("Intel {}", cleaned.trim()))
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
       (lspci_line.starts_with("00:02.0") && line_lower.contains("intel"))
    {
        " [Integrated]".to_string()
    } else {
        " [Discrete]".to_string()
    }
}
