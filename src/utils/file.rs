//! File reading utilities

use std::fs;
use std::path::Path;
use crate::error::{Result, SwiftfetchError};

/// Safely read a file to string with error handling
pub fn read_file_safe<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path).map_err(SwiftfetchError::from)
}

/// Read first line of a file, trimmed
pub fn read_first_line<P: AsRef<Path>>(path: P) -> Result<String> {
    let content = read_file_safe(path)?;
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
