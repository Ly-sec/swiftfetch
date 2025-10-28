//! String parsing utilities

use crate::error::{Result, SwiftfetchError};

/// Extract value after a colon and space
pub fn extract_after_colon(line: &str) -> Option<String> {
    line.split(':')
        .nth(1)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Parse memory value in kB to GB
#[allow(dead_code)]
pub fn kb_to_gb(kb_str: &str) -> Result<f64> {
    let kb: u64 = kb_str.trim()
        .parse()
        .map_err(|_| SwiftfetchError::Parse(format!("Invalid memory value: {}", kb_str)))?;
    Ok(kb as f64 / 1_048_576.0) // 1024^2
}

/// Format uptime from seconds
pub fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{}h {:02}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Clean and simplify GPU names
#[allow(dead_code)]
pub fn clean_gpu_name(raw_name: &str) -> String {
    raw_name
        .replace("Corporation", "")
        .replace("Advanced Micro Devices, Inc.", "AMD")
        .replace("Intel Corporation", "Intel")
        .split(" [")
        .next()
        .unwrap_or(raw_name)
        .trim()
        .to_string()
}
