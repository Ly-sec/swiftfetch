//! swiftfetch library
//!
//! A fast and simple system information fetch tool written in Rust.

pub mod collectors;
pub mod config;
pub mod data;
pub mod display;
pub mod error;
pub mod utils;

pub use data::{HardwareInfo, OsInfo, PackageInfo, SystemInfo, SystemStatus, UserInfo};
pub use error::{Result, SwiftfetchError};

/// Collect all system information (parallelized for speed)
pub fn collect_system_info() -> Result<SystemInfo> {
    // Collect independent data in parallel using nested joins
    // Rayon::join only takes 2 closures, so we nest them
    let ((os, hardware), (packages, (status, user))) = rayon::join(
        || {
            rayon::join(
                || collectors::system::collect_os_info(),
                || collectors::hardware::collect_hardware_info(),
            )
        },
        || {
            rayon::join(
                || collectors::packages::collect_package_info(),
                || {
                    rayon::join(
                        || collectors::system::collect_system_status(),
                        || collectors::system::collect_user_info(),
                    )
                },
            )
        },
    );

    Ok(SystemInfo {
        os: os?,
        hardware: hardware?,
        packages: packages?,
        status: status?,
        user: user?,
    })
}
