//! Command execution utilities

use crate::error::{Result, SwiftfetchError};
use std::process::Command;

/// Execute a command and return stdout as String
pub fn run_command(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(SwiftfetchError::Detection(format!(
            "Command '{}' failed with exit code: {:?}",
            program,
            output.status.code()
        )))
    }
}

/// Check if a command exists in PATH
pub fn command_exists(program: &str) -> bool {
    use std::env;

    if let Ok(path) = env::var("PATH") {
        for dir in path.split(':') {
            let full_path = std::path::Path::new(dir).join(program);
            if full_path.exists() && full_path.is_file() {
                return true;
            }
        }
    }
    false
}

/// Execute command and return success status only
pub fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
