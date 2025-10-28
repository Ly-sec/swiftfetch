use std::{fs, process::Command, io};

/// Supported package managers for different Linux distributions
#[derive(Debug)]
pub enum PackageManager {
    Nix,      // NixOS
    Pacman,   // Arch Linux, Manjaro
    Xbps,     // Void Linux
    Apt,      // Debian, Ubuntu
    Dnf,      // Fedora, RHEL 8+
    Portage,  // Gentoo
}

pub fn get_package_count() -> io::Result<usize> {
    match detect_package_manager() {
        Some(PackageManager::Nix) => get_nix_package_count(),
        Some(PackageManager::Pacman) => get_pacman_package_count(),
        Some(PackageManager::Xbps) => get_xbps_package_count(),
        Some(PackageManager::Apt) => get_apt_package_count(),
        Some(PackageManager::Dnf) => get_dnf_package_count(),
        Some(PackageManager::Portage) => get_portage_package_count(),
        None => Ok(0),
    }
}

pub fn get_flatpak_package_count() -> io::Result<usize> {
    let flatpak_dir = "/var/lib/flatpak/app";
    Ok(fs::read_dir(flatpak_dir)?.count())
}

fn detect_package_manager() -> Option<PackageManager> {
    if is_nix_system() {
        Some(PackageManager::Nix)
    } else if is_arch_system() {
        Some(PackageManager::Pacman)
    } else if is_void_system() {
        Some(PackageManager::Xbps)
    } else if is_debian_system() {
        Some(PackageManager::Apt)
    } else if is_fedora_system() {
        Some(PackageManager::Dnf)
    } else if is_gentoo_system() {
        Some(PackageManager::Portage)
    } else {
        None
    }
}

fn is_nix_system() -> bool {
    Command::new("which")
        .arg("nix-store")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_arch_system() -> bool {
    fs::metadata("/var/lib/pacman/local").is_ok()
}

fn is_void_system() -> bool {
    fs::metadata("/var/db/xbps").is_ok() || 
    Command::new("which")
        .arg("xbps-query")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_debian_system() -> bool {
    fs::metadata("/var/lib/dpkg/status").is_ok() || 
    Command::new("which")
        .arg("apt")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_fedora_system() -> bool {
    fs::metadata("/var/lib/rpm").is_ok() || 
    Command::new("which")
        .arg("dnf")
        .output()
        .map(|output| output.status.success())
        .unwrap_or_else(|_| {
            Command::new("which")
                .arg("yum")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        })
}

fn is_gentoo_system() -> bool {
    fs::metadata("/var/db/pkg").is_ok() || 
    Command::new("which")
        .arg("emerge")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_nix_package_count() -> io::Result<usize> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("nix-store --query --requisites /run/current-system | cut -d- -f2- | sort | uniq | wc -l")
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.trim().parse().unwrap_or(0))
}

fn get_pacman_package_count() -> io::Result<usize> {
    Ok(fs::read_dir("/var/lib/pacman/local")?.count())
}

fn get_xbps_package_count() -> io::Result<usize> {
    let output = Command::new("xbps-query")
        .arg("-l")
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.lines().count())
}

fn get_apt_package_count() -> io::Result<usize> {
    let output = Command::new("dpkg-query")
        .arg("-f")
        .arg("${binary:Package}\n")
        .arg("-W")
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.lines().filter(|line| !line.is_empty()).count())
}

fn get_dnf_package_count() -> io::Result<usize> {
    // First try dnf, then fall back to yum, then rpm
    if let Ok(output) = Command::new("dnf")
        .arg("list")
        .arg("installed")
        .arg("--quiet")
        .output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        return Ok(output_str.lines().filter(|line| !line.is_empty() && !line.starts_with("Installed Packages")).count());
    }
    
    if let Ok(output) = Command::new("yum")
        .arg("list")
        .arg("installed")
        .arg("--quiet")
        .output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        return Ok(output_str.lines().filter(|line| !line.is_empty() && !line.starts_with("Installed Packages")).count());
    }
    
    // Fallback to rpm
    let output = Command::new("rpm")
        .arg("-qa")
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.lines().filter(|line| !line.is_empty()).count())
}

fn get_portage_package_count() -> io::Result<usize> {
    // Count directories in /var/db/pkg, excluding the category directories
    let mut count = 0;
    
    if let Ok(entries) = fs::read_dir("/var/db/pkg") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        count += sub_entries.count();
                    }
                }
            }
        }
    }
    
    Ok(count)
}
