//! Hardware information collection (CPU, GPU, Memory, Disk)

use crate::error::Result;
use crate::utils::{file::*, command::*, parsing::*};
use crate::data::{HardwareInfo, CpuInfo, GpuInfo, MemoryInfo, DiskInfo};
use std::fs;

/// Collect all hardware information
pub fn collect_hardware_info() -> Result<HardwareInfo> {
    Ok(HardwareInfo {
        cpu: collect_cpu_info()?,
        gpu: collect_gpu_info()?,
        memory: collect_memory_info()?,
        disk: collect_disk_info()?,
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

    Ok(GpuInfo {
        primary,
        all_gpus,
    })
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
    let cpuinfo = read_file_safe("/proc/cpuinfo")?;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            return Ok(extract_after_colon(line).unwrap_or("Unknown CPU".to_string()));
        }
    }
    Err(crate::error::SwiftfetchError::Detection("CPU info not found".to_string()))
}

fn read_memory_info() -> Result<(f64, f64)> {
    let meminfo = read_file_safe("/proc/meminfo")?;

    // Extract memory values
    let total = extract_memory(&meminfo, "MemTotal");
    let available = extract_memory(&meminfo, "MemAvailable");

    // Calculate used memory
    let used = total - available;

    // Convert to GB
    let total_gb = total / 1024.0 / 1024.0;
    let used_gb = used / 1024.0 / 1024.0;

    Ok((total_gb, used_gb))
}

fn extract_memory(meminfo: &str, field: &str) -> f64 {
    for line in meminfo.lines() {
        if line.starts_with(field) {
            if let Some(value_str) = line.split(':').nth(1) {
                let value_str = value_str.trim().replace(" kB", "");
                return value_str.parse().unwrap_or(0.0);
            }
        }
    }
    0.0
}

fn get_disk_usage() -> Result<String> {
    let output = run_command("df", &["-h", "/"])?;
    
    for line in output.lines().skip(1) { // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let used = parts[2];
            let total = parts[1];
            let percentage = parts[4];
            return Ok(format!("{} / {} ({})", used, total, percentage));
        }
    }
    
    Err(crate::error::SwiftfetchError::Detection("Disk usage not found".to_string()))
}

fn detect_all_gpus() -> Result<Vec<String>> {
    // Try lspci first (most reliable)
    if let Ok(output) = run_command("lspci", &[]) {
        // Look for VGA compatible controllers and 3D controllers
        let gpu_lines: Vec<&str> = output
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
    // First try to get exact model from device ID
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
        let radeon_part = after_radeon
            .split(" [")
            .next()
            .unwrap_or(after_radeon)
            .split(" (")
            .next()
            .unwrap_or(after_radeon);
        
        return Some(format!("AMD {}", radeon_part.trim()));
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

fn get_exact_amd_model_from_sysfs() -> Result<String> {
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
    Err(crate::error::SwiftfetchError::Detection("AMD GPU not found in sysfs".to_string()))
}

fn amd_device_id_to_name(device_id: &str) -> Option<String> {
    match device_id {
        // RX 7800 XT
        "0x7480" => Some("Radeon RX 7800 XT".to_string()),
        // RX 7700 XT  
        "0x7479" => Some("Radeon RX 7700 XT".to_string()),
        // RX 6800 XT
        "0x73bf" => Some("Radeon RX 6800 XT".to_string()),
        // RX 6700 XT
        "0x73df" => Some("Radeon RX 6700 XT".to_string()),
        _ => None,
    }
}

fn resolve_amd_gpu_range(bracket_content: &str) -> String {
    // For ranges like "Radeon RX 7700 XT / 7800 XT", default to the higher-end model
    if bracket_content.contains(" / ") {
        let parts: Vec<&str> = bracket_content.split(" / ").collect();
        if let Some(last_part) = parts.last() {
            return last_part.trim().to_string();
        }
    }
    bracket_content.to_string()
}

fn lookup_gpu_by_ids(vendor_id: &str, device_id: &str) -> Option<String> {
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
