//! Package management information collection

use crate::data::PackageInfo;
use crate::error::Result;
use crate::utils::{command::*, file::*};
use std::fs;

/// Collect package information (parallelized for speed)
pub fn collect_package_info() -> Result<PackageInfo> {
    // Collect system packages and flatpak packages in parallel
    let (system_packages, flatpak_packages) = rayon::join(
        || get_package_count().unwrap_or(0),
        || get_flatpak_package_count().unwrap_or(0),
    );

    Ok(PackageInfo {
        system_packages,
        flatpak_packages,
    })
}

/// Supported package managers for different Linux distributions
#[derive(Debug)]
pub enum PackageManager {
    Nix,     // NixOS
    Pacman,  // Arch Linux, Manjaro
    Xbps,    // Void Linux
    Apt,     // Debian, Ubuntu
    Dnf,     // Fedora, RHEL 8+
    Portage, // Gentoo
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

    // Only check if directory exists to avoid unnecessary work
    if !fs::metadata(flatpak_dir).is_ok() {
        return Ok(0);
    }

    // Count only directories (apps), not files
    let count = fs::read_dir(flatpak_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .count();
    Ok(count)
}

fn detect_package_manager() -> Option<PackageManager> {
    // Check file-based indicators first (faster than command_exists)
    // Order matters: check most common systems first

    if file_exists("/var/lib/pacman/local") {
        Some(PackageManager::Pacman)
    } else if file_exists("/var/lib/dpkg/status") {
        Some(PackageManager::Apt)
    } else if file_exists("/var/lib/rpm") {
        Some(PackageManager::Dnf)
    } else if file_exists("/var/db/xbps") {
        Some(PackageManager::Xbps)
    } else if file_exists("/var/db/pkg") {
        Some(PackageManager::Portage)
    } else if is_nix_system() {
        Some(PackageManager::Nix)
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
    let output = run_command(
        "nix-store",
        &["--query", "--requisites", "/run/current-system/sw"],
    )?;
    Ok(output.lines().count())
}

fn get_pacman_package_count() -> Result<usize> {
    Ok(fs::read_dir("/var/lib/pacman/local")?.count())
}

fn get_xbps_package_count() -> Result<usize> {
    // Count files in /var/db/xbps directly (faster than xbps-query subprocess)
    // XBPS stores package metadata as .plist files in /var/db/xbps
    let xbps_db = "/var/db/xbps";
    if file_exists(xbps_db) {
        // Count .plist files (each represents an installed package)
        if let Ok(entries) = fs::read_dir(xbps_db) {
            let count = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.ends_with(".plist"))
                        .unwrap_or(false)
                })
                .count();
            // Only return if we found plist files, otherwise fallback to command
            if count > 0 {
                return Ok(count);
            }
        }
    }

    // Fallback to xbps-query if directory counting fails
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
        return Ok(output
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("Installed Packages"))
            .count());
    }

    if command_exists("yum") {
        let output = run_command("yum", &["list", "installed", "--quiet"])?;
        return Ok(output
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("Installed Packages"))
            .count());
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
