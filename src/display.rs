use self::kitty_support::KittyArtworkInfo;
use crate::config::{Config, ConfigEntry};
use shellexpand;
use std::{
    fs,
    io::{self, BufRead},
    path::Path,
    process::Command,
};
use unicode_width::UnicodeWidthStr;

pub fn get_default_ascii(distro: &str) -> Option<&'static str> {
    match distro.to_lowercase().as_str() {
        "arch" => Some(include_str!("../ascii/arch.txt")),
        "ubuntu" => Some(include_str!("../ascii/ubuntu.txt")),
        "cachyos" => Some(include_str!("../ascii/cachyos.txt")),
        "debian" => Some(include_str!("../ascii/debian.txt")),
        "fedora" => Some(include_str!("../ascii/fedora.txt")),
        "gentoo" => Some(include_str!("../ascii/gentoo.txt")),
        "void" => Some(include_str!("../ascii/void.txt")),
        "nixos" => Some(include_str!("../ascii/nixos.txt")),
        "pika" => Some(include_str!("../ascii/pika.txt")),
        _ => None,
    }
}

pub fn hex_to_ansi(color: &str) -> String {
    // First try ANSI color names
    if let Some(ansi_code) = get_ansi_color_code(color) {
        return ansi_code;
    }

    // Fallback to hex color parsing for custom colors
    if color.starts_with('#') && color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color[1..3], 16),
            u8::from_str_radix(&color[3..5], 16),
            u8::from_str_radix(&color[5..7], 16),
        ) {
            return format!("\x1b[38;2;{};{};{}m", r, g, b);
        }
    }

    // Default fallback
    "\x1b[0m".to_string()
}

fn get_ansi_color_code(color_name: &str) -> Option<String> {
    match color_name.to_lowercase().as_str() {
        // Standard 8 colors (30-37)
        "black" => Some("\x1b[30m".to_string()),
        "red" => Some("\x1b[31m".to_string()),
        "green" => Some("\x1b[32m".to_string()),
        "yellow" => Some("\x1b[33m".to_string()),
        "blue" => Some("\x1b[34m".to_string()),
        "magenta" => Some("\x1b[35m".to_string()),
        "cyan" => Some("\x1b[36m".to_string()),
        "white" => Some("\x1b[37m".to_string()),

        // Bright colors (90-97)
        "bright_black" | "gray" | "grey" => Some("\x1b[90m".to_string()),
        "bright_red" => Some("\x1b[91m".to_string()),
        "bright_green" => Some("\x1b[92m".to_string()),
        "bright_yellow" => Some("\x1b[93m".to_string()),
        "bright_blue" => Some("\x1b[94m".to_string()),
        "bright_magenta" => Some("\x1b[95m".to_string()),
        "bright_cyan" => Some("\x1b[96m".to_string()),
        "bright_white" => Some("\x1b[97m".to_string()),

        // Additional aliases
        "orange" => Some("\x1b[91m".to_string()), // bright red
        "purple" => Some("\x1b[35m".to_string()), // magenta
        "violet" => Some("\x1b[95m".to_string()), // bright magenta

        // Reset
        "reset" | "default" => Some("\x1b[0m".to_string()),

        _ => {
            // Show available colors if an invalid color is used
            eprintln!("Error: Unknown color '{}'. Available colors:", color_name);
            eprintln!("  Standard: black, red, green, yellow, blue, magenta, cyan, white");
            eprintln!("  Bright:   bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white");
            eprintln!("  Aliases:  gray/grey, orange, purple, violet, reset/default");
            eprintln!("  Or use hex colors like #FF0000");
            None
        }
    }
}

pub fn load_ascii_lines(config: &Config, os_name: &str) -> Vec<String> {
    let use_default = config.display.use_default_ascii.unwrap_or(true);

    match config
        .display
        .ascii_path
        .as_ref()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
    {
        Some(ascii_path) => {
            let expanded_path = shellexpand::tilde(ascii_path).to_string();
            if Path::new(&expanded_path).exists() {
                if let Ok(file) = fs::File::open(&expanded_path) {
                    return io::BufReader::new(file)
                        .lines()
                        .filter_map(Result::ok)
                        .collect();
                }
            }
            // If file does not exist or failed to open
            if use_default {
                if let Some(default_ascii) = get_default_ascii(os_name) {
                    return default_ascii.lines().map(String::from).collect();
                }
            }
            vec![]
        }
        None if use_default => {
            if let Some(default_ascii) = get_default_ascii(os_name) {
                return default_ascii.lines().map(String::from).collect();
            }
            vec![]
        }
        _ => vec![],
    }
}

pub struct SystemData {
    pub os_name: String,
    pub kernel_version: String,
    pub cpu_brand: String,
    pub gpu: String,
    pub all_gpus: Vec<String>,
    pub username: String,
    pub hostname: String,
    pub wm_de: String,
    pub memory: String,
    pub pkg_count: usize,
    pub flatpak_pkg_count: usize,
    pub uptime_formatted: String,
    pub os_age: String,
    pub editor: String,
    pub shell: String,
    pub terminal: String,
    pub user_info: String,
    pub disk_usage: String,
    pub init_system: String,
    pub battery_info: String,
}

pub fn render_output(config: &Config, system_data: &SystemData) {
    let separator = config.display.separator.as_deref().unwrap_or(": ");
    let show_all_gpus = config.display.show_all_gpus.unwrap_or(false);

    let mut rendered_items = Vec::new();

    for entry in config.display.items.iter() {
        // Handle GPU entries specially if show_all_gpus is enabled
        if entry.value == "gpu" && show_all_gpus {
            // For GPU entries, add multiple entries if we have multiple GPUs
            for (gpu_idx, _gpu_name) in system_data.all_gpus.iter().enumerate() {
                let modified_entry = ConfigEntry {
                    key: entry.key.clone(),
                    r#type: entry.r#type.clone(),
                    value: format!("gpu{}", gpu_idx + 1),
                    color: entry.color.clone(),
                    value_color: entry.value_color.clone(),
                };
                rendered_items.push(modified_entry);
            }
        } else if entry.value != "gpu" || !show_all_gpus {
            // For non-GPU entries, or GPU entries when show_all_gpus is false, add as normal
            rendered_items.push(entry.clone());
        }
    }

    let ascii_color_code = config
        .colors
        .get(&config.display.ascii_color)
        .map(|color| hex_to_ansi(color))
        .unwrap_or_else(|| {
            get_ansi_color_code(&config.display.ascii_color)
                .unwrap_or_else(|| "\x1b[0m".to_string())
        });

    // Pre-compute and cache color codes to avoid repeated parsing
    let mut color_cache: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut get_cached_color = |color_key: &str| -> String {
        color_cache
            .entry(color_key.to_string())
            .or_insert_with(|| {
                config
                    .colors
                    .get(color_key)
                    .map(|c| hex_to_ansi(c))
                    .unwrap_or_else(|| {
                        get_ansi_color_code(color_key).unwrap_or_else(|| "\x1b[0m".to_string())
                    })
            })
            .clone()
    };

    let mut output = String::with_capacity(4096);

    let mut ascii_lines: Vec<String> = Vec::new();
    let display_mode = config
        .display
        .display_mode
        .as_deref()
        .unwrap_or("ascii")
        .to_lowercase();

    let use_image_mode = matches!(display_mode.as_str(), "image" | "kitty");

    if use_image_mode {
        if let Some(image_path) = config.display.image_path.as_ref() {
            if !kitty_support::terminal_supports_kitty() {
                eprintln!(
                    "Image mode requested but the terminal did not advertise Kitty-compatible graphics \
                     support. Attempting to transmit anyway (set SWIFTFETCH_FORCE_KITTY=1 to skip this warning)."
                );
            }

            let expanded_path = shellexpand::tilde(image_path).to_string();
            let mut kitty_buffer = String::new();
            match kitty_support::render_image(
                &expanded_path,
                config.display.image_width,
                config.display.image_height,
                config.display.image_padding_columns,
                config.display.image_rows,
                config.display.image_offset_columns.unwrap_or(0),
                config.display.image_offset_rows.unwrap_or(0),
                &mut kitty_buffer,
            ) {
                Ok(KittyArtworkInfo {
                    pad_columns,
                    pad_rows,
                }) => {
                    let padding = if pad_columns > 0 {
                        " ".repeat(pad_columns)
                    } else {
                        String::new()
                    };

                    let desired_lines = std::cmp::max(rendered_items.len(), pad_rows).max(1);
                    ascii_lines = vec![padding; desired_lines];

                    output.push_str("\x1b[s");
                    output.push_str(&kitty_buffer);
                    output.push_str("\x1b[u");
                }
                Err(err) => {
                    eprintln!(
                        "Kitty image rendering failed (falling back to ASCII): {}",
                        err
                    );
                }
            }
        } else {
            eprintln!(
                "Image display mode was requested but 'image_path' was not set. Falling back to ASCII output."
            );
        }
    }

    if ascii_lines.is_empty() {
        ascii_lines = load_ascii_lines(config, &system_data.os_name);
    }

    let max_ascii_length = ascii_lines
        .iter()
        .map(|line| UnicodeWidthStr::width(line.as_str()))
        .max()
        .unwrap_or(0);

    for (i, entry) in rendered_items.iter().enumerate() {
        let ascii_line = if i < ascii_lines.len() {
            &ascii_lines[i]
        } else {
            ""
        };

        // Pre-allocate space for padded ASCII
        let padded_ascii = if ascii_line.len() < max_ascii_length {
            format!("{:<width$}", ascii_line, width = max_ascii_length)
        } else {
            ascii_line.to_string()
        };

        output.push_str(&ascii_color_code);
        output.push_str(&padded_ascii);

        if entry.value.is_empty() {
            output.push('\n');
            continue;
        }

        output.push_str("  ");

        let key_color_code = entry
            .color
            .as_ref()
            .map(|c| get_cached_color(c))
            .unwrap_or_else(|| "\x1b[0m".to_string());

        let value_color_code = entry
            .value_color
            .as_ref()
            .map(|c| get_cached_color(c))
            .unwrap_or_else(|| "\x1b[0m".to_string());

        let output_value = get_output_value(entry, system_data);

        if entry.r#type == "text" {
            let text_color = entry
                .color
                .as_ref()
                .map(|c| get_cached_color(c))
                .unwrap_or_else(|| "\x1b[0m".to_string());
            output.push_str(&text_color);
            output.push_str(&output_value);
            output.push_str("\x1b[0m\n");
        } else if entry.key.is_empty() || entry.key == "user_info" {
            output.push_str(&value_color_code);
            output.push_str(&output_value);
            output.push_str("\x1b[0m\n");
        } else {
            output.push_str(&key_color_code);
            output.push_str(&entry.key);
            output.push_str(separator);
            output.push_str("\x1b[0m");
            output.push_str(&value_color_code);
            output.push_str(&output_value);
            output.push_str("\x1b[0m\n");
        }
    }

    // Print remaining ASCII lines
    for i in rendered_items.len()..ascii_lines.len() {
        let padded_ascii = if ascii_lines[i].len() < max_ascii_length {
            format!("{:<width$}", ascii_lines[i], width = max_ascii_length)
        } else {
            ascii_lines[i].clone()
        };
        output.push_str(&ascii_color_code);
        output.push_str(&padded_ascii);
        output.push('\n');
    }

    output.push_str("\x1b[0m");
    print!("{}", output);
}

fn get_output_value(entry: &ConfigEntry, system_data: &SystemData) -> String {
    match entry.r#type.as_str() {
        "default" => match entry.value.as_str() {
            "kernel" => system_data.kernel_version.clone(),
            "os" => system_data.os_name.clone(),
            "cpu" => system_data.cpu_brand.clone(),
            "gpu" => system_data.gpu.clone(),
            "gpu1" => system_data
                .all_gpus
                .get(0)
                .cloned()
                .unwrap_or_else(|| "No GPU".to_string()),
            "gpu2" => system_data
                .all_gpus
                .get(1)
                .cloned()
                .unwrap_or_else(|| "No secondary GPU".to_string()),
            "gpu3" => system_data
                .all_gpus
                .get(2)
                .cloned()
                .unwrap_or_else(|| "No third GPU".to_string()),
            "wm" => system_data.wm_de.clone(),
            "editor" => system_data.editor.clone(),
            "shell" => system_data.shell.clone(),
            "terminal" => system_data.terminal.clone(),
            "username" => system_data.username.clone(),
            "hostname" => system_data.hostname.trim().to_string(),
            "memory" => system_data.memory.clone(),
            "pkg_count" => system_data.pkg_count.to_string(),
            "flatpak_pkg_count" => system_data.flatpak_pkg_count.to_string(),
            "uptime_seconds" => system_data.uptime_formatted.clone(),
            "os_age" => system_data.os_age.clone(),
            "user_info" => system_data.user_info.clone(),
            "disk_usage" => system_data.disk_usage.clone(),
            "init_system" => system_data.init_system.clone(),
            "battery" => system_data.battery_info.clone(),
            _ => "Unknown default value".to_string(),
        },
        "text" => entry.value.clone(),
        "command" => Command::new("sh")
            .arg("-c")
            .arg(&entry.value)
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "Command failed".to_string()),
        _ => "Invalid type".to_string(),
    }
}

mod kitty_support {
    use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
    use base64::Engine;
    use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageOutputFormat};
    use std::{env, fmt::Write, io::Cursor, mem, path::Path};

    pub struct KittyArtworkInfo {
        pub pad_columns: usize,
        pub pad_rows: usize,
    }

    const DEFAULT_CHAR_WIDTH: f32 = 11.5;
    const DEFAULT_CHAR_HEIGHT: f32 = 18.0;
    const DEFAULT_GAP_COLUMNS: usize = 1;

    pub fn terminal_supports_kitty() -> bool {
        if matches!(
            env::var("SWIFTFETCH_FORCE_KITTY"),
            Ok(v) if v == "1" || v.eq_ignore_ascii_case("true")
        ) {
            return true;
        }

        env::var("KITTY_WINDOW_ID").is_ok()
            || env::var("WEZTERM_PANE").is_ok()
            || env::var("TERM_PROGRAM")
                .map(|prog| term_program_supports_kitty(&prog))
                .unwrap_or(false)
            || env::var("TERM")
                .map(|term| term_name_supports_kitty(&term))
                .unwrap_or(false)
    }

    fn term_program_supports_kitty(value: &str) -> bool {
        let value = value.to_lowercase();
        matches!(
            value.as_str(),
            "kitty" | "wezterm" | "ghostty" | "tabby" | "warp-terminal"
        )
    }

    fn term_name_supports_kitty(term: &str) -> bool {
        let term = term.to_lowercase();

        if term.contains("kitty") || term.contains("wezterm") || term.contains("foot") {
            return true;
        }

        if term.contains("ghostty") {
            return true;
        }

        // Ghostty users often override TERM to xterm-256color or tmux-256color.
        env::vars().any(|(key, _)| key.starts_with("GHOSTTY_"))
    }

    fn terminal_cell_metrics() -> Option<(f32, f32)> {
        #[cfg(unix)]
        {
            use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
            let mut ws: winsize = unsafe { mem::zeroed() };
            let result = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) };
            if result == 0 && ws.ws_col > 0 && ws.ws_row > 0 && ws.ws_xpixel > 0 && ws.ws_ypixel > 0
            {
                let char_width = ws.ws_xpixel as f32 / ws.ws_col as f32;
                let char_height = ws.ws_ypixel as f32 / ws.ws_row as f32;
                return Some((char_width, char_height));
            }
        }
        None
    }

    pub fn render_image(
        path: &str,
        target_width: Option<u32>,
        target_height: Option<u32>,
        configured_padding: Option<usize>,
        configured_rows: Option<usize>,
        offset_columns: i32,
        offset_rows: i32,
        output: &mut String,
    ) -> Result<KittyArtworkInfo, String> {
        let mut image = image::open(Path::new(path))
            .map_err(|err| format!("Failed to open image '{}': {}", path, err))?;

        image = resize_image(image, target_width, target_height);

        let (final_width, final_height) = image.dimensions();

        let mut png_bytes = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_bytes);
            image
                .write_to(&mut cursor, ImageOutputFormat::Png)
                .map_err(|err| format!("Failed to encode image '{}': {}", path, err))?;
        }

        let (char_width, char_height) =
            terminal_cell_metrics().unwrap_or((DEFAULT_CHAR_WIDTH, DEFAULT_CHAR_HEIGHT));
        let auto_columns = ((final_width as f32 / char_width).ceil() as usize).max(1);
        let auto_pad_columns = configured_padding.unwrap_or(auto_columns + DEFAULT_GAP_COLUMNS);
        let auto_pad_rows = configured_rows
            .unwrap_or_else(|| ((final_height as f32 / char_height).ceil() as usize).max(1));

        let pad_columns = adjust_with_offset(auto_pad_columns, offset_columns, 0);
        let pad_rows = adjust_with_offset(auto_pad_rows, offset_rows, 1);
        let vertical_offset_rows = compute_vertical_offset_rows(auto_pad_rows) + offset_rows;

        transmit_png(
            &png_bytes,
            final_width,
            final_height,
            offset_columns,
            vertical_offset_rows,
            output,
        )?;
        output.push('\n');

        Ok(KittyArtworkInfo {
            pad_columns,
            pad_rows,
        })
    }

    fn resize_image(image: DynamicImage, width: Option<u32>, height: Option<u32>) -> DynamicImage {
        match (width, height) {
            (Some(w), Some(h)) => image.resize_exact(w, h, FilterType::Lanczos3),
            (Some(w), None) => {
                let ratio = w as f32 / image.width() as f32;
                let h = ((image.height() as f32 * ratio).round().max(1.0)) as u32;
                image.resize_exact(w, h, FilterType::Lanczos3)
            }
            (None, Some(h)) => {
                let ratio = h as f32 / image.height() as f32;
                let w = ((image.width() as f32 * ratio).round().max(1.0)) as u32;
                image.resize_exact(w, h, FilterType::Lanczos3)
            }
            (None, None) => image,
        }
    }

    fn transmit_png(
        png_bytes: &[u8],
        width: u32,
        height: u32,
        horizontal_offset: i32,
        vertical_offset: i32,
        output: &mut String,
    ) -> Result<(), String> {
        let encoded = BASE64_ENGINE.encode(png_bytes);
        let mut start = 0usize;
        const CHUNK: usize = 4096;

        while start < encoded.len() {
            let end = (start + CHUNK).min(encoded.len());
            let chunk = &encoded[start..end];
            let more_flag = if end < encoded.len() { 1 } else { 0 };

            write!(
                output,
                "\x1b_Ga=T,f=100,s={},v={},x={},y={},m={};",
                width, height, horizontal_offset, vertical_offset, more_flag
            )
            .map_err(|_| "Failed to write Kitty image escape sequence".to_string())?;

            output.push_str(chunk);
            output.push_str("\x1b\\");

            start = end;
        }

        Ok(())
    }

    fn compute_vertical_offset_rows(pad_rows: usize) -> i32 {
        if pad_rows == 0 {
            return 0;
        }

        let offset = (pad_rows as f32 * 0.08).round() as i32;
        offset.max(1)
    }

    fn adjust_with_offset(base: usize, delta: i32, min_value: usize) -> usize {
        let adjusted = base as i32 + delta;
        if adjusted < min_value as i32 {
            min_value
        } else {
            adjusted as usize
        }
    }
}
