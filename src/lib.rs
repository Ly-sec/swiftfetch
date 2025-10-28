//! swiftfetch library
//! 
//! A fast and simple system information fetch tool written in Rust.

pub mod error;
pub mod data;
pub mod collectors;
pub mod utils;
pub mod config;
pub mod display;

pub use error::{SwiftfetchError, Result};
pub use data::{SystemInfo, HardwareInfo, OsInfo, UserInfo, PackageInfo, SystemStatus};

/// Collect all system information
pub fn collect_system_info() -> Result<SystemInfo> {
    Ok(SystemInfo {
        os: collectors::system::collect_os_info()?,
        hardware: collectors::hardware::collect_hardware_info()?,
        packages: collectors::packages::collect_package_info()?,
        status: collectors::system::collect_system_status()?,
        user: collectors::system::collect_user_info()?,
    })
}
