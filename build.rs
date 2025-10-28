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

    // Define the target directories
    let config_dir = format!("{}/.config/swiftfetch", home_dir);
    let ascii_dir = format!("{}/ascii", config_dir);
    let config_file = format!("{}/config.toml", config_dir);

    // Check if the main config directory exists, and if not, create it
    if !Path::new(&config_dir).exists() {
        println!("Creating config directory: {}", config_dir);
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    // Check if the ascii directory exists, and if not, create it
    if !Path::new(&ascii_dir).exists() {
        println!("Creating ASCII directory: {}", ascii_dir);
        fs::create_dir_all(&ascii_dir).expect("Failed to create ASCII directory");
    }

    // Copy the config file if it doesn't exist
    if !Path::new(&config_file).exists() {
        println!("Copying default config file to: {}", config_file);
        let source = Path::new("config/config.toml");
        fs::copy(source, config_file).expect("Failed to copy config file");
        println!("Config file copied successfully.");
    }

    // Copy ASCII art files
    copy_ascii_files(&ascii_dir);

    // Tell cargo to rerun this build script if the ascii directory changes
    println!("cargo:rerun-if-changed=ascii/");
    println!("cargo:rerun-if-changed=config/config.toml");
}

fn copy_ascii_files(target_dir: &str) {
    let source_dir = Path::new("ascii");
    
    if !source_dir.exists() {
        println!("ASCII source directory not found, skipping ASCII files copy.");
        return;
    }

    match fs::read_dir(source_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                        if let Some(file_name) = path.file_name() {
                            let target_file = Path::new(target_dir).join(file_name);
                            
                            // Only copy if the file doesn't exist or is older than the source
                            let should_copy = if target_file.exists() {
                                if let (Ok(source_meta), Ok(target_meta)) = (fs::metadata(&path), fs::metadata(&target_file)) {
                                    if let (Ok(source_time), Ok(target_time)) = (source_meta.modified(), target_meta.modified()) {
                                        source_time > target_time
                                    } else {
                                        true // If we can't get timestamps, copy anyway
                                    }
                                } else {
                                    true
                                }
                            } else {
                                true
                            };

                            if should_copy {
                                match fs::copy(&path, &target_file) {
                                    Ok(_) => println!("Copied ASCII file: {:?}", file_name),
                                    Err(e) => eprintln!("Failed to copy ASCII file {:?}: {}", file_name, e),
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read ASCII directory: {}", e),
    }
}