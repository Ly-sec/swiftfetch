//! File reading utilities

use std::fs;
use std::path::Path;
use crate::error::{Result, SwiftfetchError};

/// Safely read a file to string with error handling
pub fn read_file_safe<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path).map_err(SwiftfetchError::from)
}

/// Read first line of a file, trimmed
/// Optimized for single-line files like /proc/sys/kernel/hostname
pub fn read_first_line<P: AsRef<Path>>(path: P) -> Result<String> {
    use std::io::Read;
    use std::fs::File;
    
    // For small single-line files, read with a small buffer
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 256]; // Most proc files are tiny
    let bytes_read = file.read(&mut buffer)?;
    
    if bytes_read == 0 {
        return Ok(String::new());
    }
    
    // Convert to string and get first line
    let content = std::str::from_utf8(&buffer[..bytes_read])
        .map_err(|_| SwiftfetchError::Parse("Invalid UTF-8".to_string()))?;
    Ok(content.lines().next().unwrap_or("").trim().to_string())
}

/// Check if a file exists safely
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Read file and return default on error
#[allow(dead_code)]
pub fn read_file_or_default<P: AsRef<Path>>(path: P, default: &str) -> String {
    read_file_safe(path).unwrap_or_else(|_| default.to_string())
}
