//! File reading utilities

use crate::error::{Result, SwiftfetchError};
use std::fs;
use std::path::Path;

/// Safely read a file to string with error handling
pub fn read_file_safe<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path).map_err(SwiftfetchError::from)
}

/// Read first line of a file, trimmed
/// Optimized for single-line files like /proc/sys/kernel/hostname
/// Uses direct syscalls for maximum performance
pub fn read_first_line<P: AsRef<Path>>(path: P) -> Result<String> {
    use libc;
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let path_cstr = CString::new(path.as_ref().as_os_str().as_bytes())
        .map_err(|_| SwiftfetchError::Parse("Invalid path".to_string()))?;

    // Use direct syscall for maximum performance
    unsafe {
        let fd = libc::open(path_cstr.as_ptr(), libc::O_RDONLY);
        if fd < 0 {
            return Err(SwiftfetchError::from(std::io::Error::last_os_error()));
        }

        let mut buffer = [0u8; 256];
        let bytes_read = libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len());
        libc::close(fd);

        if bytes_read < 0 {
            return Err(SwiftfetchError::from(std::io::Error::last_os_error()));
        }

        if bytes_read == 0 {
            return Ok(String::new());
        }

        // Convert to string and get first line
        let content = std::str::from_utf8(&buffer[..bytes_read as usize])
            .map_err(|_| SwiftfetchError::Parse("Invalid UTF-8".to_string()))?;
        Ok(content.lines().next().unwrap_or("").trim().to_string())
    }
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
