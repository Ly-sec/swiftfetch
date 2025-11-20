use dirs::config_dir;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub display: DisplayConfig,
    pub colors: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct DisplayConfig {
    pub items: Vec<ConfigEntry>,
    pub separator: Option<String>,
    pub ascii_path: Option<String>,
    pub ascii_color: String,
    pub use_default_ascii: Option<bool>,
    pub show_all_gpus: Option<bool>,
    #[serde(alias = "artwork_mode")]
    pub display_mode: Option<String>,
    #[serde(alias = "kitty_image_path")]
    pub image_path: Option<String>,
    #[serde(alias = "kitty_image_width")]
    pub image_width: Option<u32>,
    #[serde(alias = "kitty_image_height")]
    pub image_height: Option<u32>,
    #[serde(alias = "artwork_padding_columns")]
    pub image_padding_columns: Option<usize>,
    #[serde(alias = "kitty_rows")]
    pub image_rows: Option<usize>,
    #[serde(alias = "kitty_offset_columns")]
    pub image_offset_columns: Option<i32>,
    #[serde(alias = "kitty_offset_rows")]
    pub image_offset_rows: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: String,
    pub color: Option<String>,
    pub value_color: Option<String>,
}

pub fn ensure_user_config_exists() {
    let user_config_path = config_dir()
        .map(|p| p.join("swiftfetch/config.toml"))
        .expect("Could not determine config dir");

    if !user_config_path.exists() {
        let default_config_path = "/usr/share/swiftfetch/config.toml";

        if let Some(parent) = user_config_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        if Path::new(default_config_path).exists() {
            fs::copy(default_config_path, &user_config_path).unwrap();
            println!("Created config at {}", user_config_path.display());
        } else {
            eprintln!("Missing default config at {}", default_config_path);
        }
    }
}

pub fn load_config() -> Config {
    let user_config_path = config_dir()
        .map(|p| p.join("swiftfetch/config.toml"))
        .unwrap_or_else(|| "config.toml".into());

    // Try user config first, then fallback to system config
    // Use read_to_string which will fail fast if file doesn't exist
    let config_data = fs::read_to_string(&user_config_path)
        .or_else(|_| fs::read_to_string("/usr/share/swiftfetch/config.toml"))
        .expect("Failed to read config file");

    toml::de::from_str(&config_data).expect("Failed to parse config file")
}
