//! Desktop environment and window manager detection

use std::{env, process::Command};

/// Detect the current window manager or desktop environment
pub fn detect_wm_or_de() -> String {
    if let Ok(env_var) = env::var("XDG_CURRENT_DESKTOP").or_else(|_| env::var("DESKTOP_SESSION")) {
        if !env_var.is_empty() {
            return capitalize_first_letter(&env_var);
        }
    }

    if env::var("WAYLAND_DISPLAY").is_ok() {
        return "Wayland".to_string();
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg("ps -e | grep -E 'sway|hyprland|kwin|mutter|xfwm4|openbox|i3|bspwm|awesome|weston|gnome-session'")
        .output()
        .ok();

    if let Some(output) = output {
        let result = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = result.lines().next() {
            if let Some(process) = line.split_whitespace().last() {
                return capitalize_first_letter(process);
            }
        }
    }

    "Unknown".to_string()
}

pub fn capitalize_first_letter(s: &str) -> String {
    if let Some(first) = s.chars().next() {
        format!("{}{}", first.to_uppercase(), &s[1..])
    } else {
        s.to_string()
    }
}