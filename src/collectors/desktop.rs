//! Desktop environment and window manager detection

use std::env;

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

    // Check for window manager processes by reading /proc
    // Limit scan to first 50 PIDs (most WMs start very early) and use early exit
    let wm_names = ["sway", "hyprland", "kwin_wayland", "kwin_x11", "niri", "mutter", 
                    "xfwm4", "openbox", "i3", "bspwm", "awesome", "weston", 
                    "gnome-session"];
    
    if let Ok(entries) = std::fs::read_dir("/proc") {
        let mut count = 0;
        for entry in entries {
            if count > 50 { break; } // Limit scan even more aggressively
            if let Ok(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip non-PID directories faster
                    if name.parse::<u32>().is_ok() {
                        count += 1;
                        // Try reading comm file (faster than cmdline)
                        if let Ok(cmdline) = std::fs::read_to_string(entry.path().join("comm")) {
                            let cmd = cmdline.trim();
                            // Early match check
                            for wm in &wm_names {
                                if cmd == *wm || cmd.starts_with(wm) {
                                    if cmd == "gnome-session" || cmd.starts_with("gnome-session-") {
                                        return "GNOME".to_string();
                                    }
                                    return capitalize_first_letter(cmd);
                                }
                            }
                        }
                    }
                }
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