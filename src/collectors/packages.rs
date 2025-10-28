//! Package management information collection

use crate::error::Result;
use crate::utils::{file::*, command::*};
use crate::data::PackageInfo;
use std::fs;

/// Collect package information
pub fn collect_package_info() -> Result<PackageInfo> {
    Ok(PackageInfo {
        system_packages: get_package_count().unwrap_or(0),
        flatpak_packages: get_flatpak_package_count().unwrap_or(0),
    })
}

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

pub fn get_package_count() -> Result<usize> {
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

pub fn get_flatpak_package_count() -> Result<usize> {
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
    command_exists("nix-store")
}

fn is_arch_system() -> bool {
    file_exists("/var/lib/pacman/local")
}

fn is_void_system() -> bool {
    file_exists("/var/db/xbps") || command_exists("xbps-query")
}

fn is_debian_system() -> bool {
    file_exists("/var/lib/dpkg/status") || command_exists("apt")
}

fn is_fedora_system() -> bool {
    file_exists("/var/lib/rpm") || command_exists("dnf") || command_exists("yum")
}

fn is_gentoo_system() -> bool {
    file_exists("/var/db/pkg") || command_exists("emerge")
}

// Implementation functions for each package manager
fn get_nix_package_count() -> Result<usize> {
    let output = run_command("nix-store", &["--query", "--requisites", "/run/current-system/sw"])?;
    Ok(output.lines().count())
}

fn get_pacman_package_count() -> Result<usize> {
    Ok(fs::read_dir("/var/lib/pacman/local")?.count())
}

fn get_xbps_package_count() -> Result<usize> {
    let output = run_command("xbps-query", &["-l"])?;
    Ok(output.lines().count())
}

fn get_apt_package_count() -> Result<usize> {
    let output = run_command("dpkg-query", &["-f", "${binary:Package}\n", "-W"])?;
    Ok(output.lines().filter(|line| !line.is_empty()).count())
}

fn get_dnf_package_count() -> Result<usize> {
    // First try dnf, then fall back to yum, then rpm
    if command_exists("dnf") {
        let output = run_command("dnf", &["list", "installed", "--quiet"])?;
        return Ok(output.lines().filter(|line| !line.is_empty() && !line.starts_with("Installed Packages")).count());
    }
    
    if command_exists("yum") {
        let output = run_command("yum", &["list", "installed", "--quiet"])?;
        return Ok(output.lines().filter(|line| !line.is_empty() && !line.starts_with("Installed Packages")).count());
    }
    
    // Fallback to rpm
    let output = run_command("rpm", &["-qa"])?;
    Ok(output.lines().filter(|line| !line.is_empty()).count())
}

fn get_portage_package_count() -> Result<usize> {
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
