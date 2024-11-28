use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the user's home directory
    let home_dir = match env::var("HOME") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Failed to get the home directory.");
            return;
        }
    };

    // Define the target directory for the config file
    let config_dir = format!("{}/.config/swiftfetch", home_dir);
    let config_file = format!("{}/config.toml", config_dir);

    // Check if the directory exists, and if not, create it
    if !Path::new(&config_dir).exists() {
        println!("Creating config directory: {}", config_dir);
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    // Check if the config file already exists; if not, copy the default config
    if !Path::new(&config_file).exists() {
        println!("Copying default config file to: {}", config_file);
        let source = Path::new("config/config.toml");
        fs::copy(source, config_file).expect("Failed to copy config file");
        println!("Config file copied successfully.");
    }
}