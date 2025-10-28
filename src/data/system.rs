//! System-wide information structures

use super::hardware::HardwareInfo;

/// Complete system information gathered by swiftfetch
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: OsInfo,
    pub hardware: HardwareInfo,
    pub packages: PackageInfo,
    pub status: SystemStatus,
    pub user: UserInfo,
}

/// Operating system related information
#[derive(Debug, Clone)]
pub struct OsInfo {
    pub name: String,
    pub kernel_version: String,
    pub age: String,
}

/// User and session information
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub hostname: String,
    pub formatted_user_info: String,
}

/// Package management information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub system_packages: usize,
    pub flatpak_packages: usize,
}

/// System status and runtime information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    #[allow(dead_code)]
    pub uptime_seconds: u64,
    pub uptime_formatted: String,
    pub init_system: String,
    pub battery_info: String,
    pub desktop_environment: String,
    pub editor: String,
    pub shell: String,
    pub terminal: String,
}
