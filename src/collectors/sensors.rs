//! Sensors and hardware monitoring (battery, temperature, etc.)

use crate::error::Result;
use std::fs;

/// Get battery information if available
#[allow(dead_code)]
pub fn get_battery_info() -> Result<String> {
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
    
    Err(crate::error::SwiftfetchError::Detection("No battery found".to_string()))
}

// Future: CPU/GPU temperature functions can go here
