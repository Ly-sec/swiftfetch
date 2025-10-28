mod error;
mod data;
mod collectors;
mod utils;
mod config;
mod display;

use crate::error::Result;
use crate::data::SystemInfo;
use config::{ensure_user_config_exists, load_config};
use display::{render_output, SystemData};

fn main() -> Result<()> {
    ensure_user_config_exists();
    let config = load_config();

    let system_info = collect_system_info()?;

    // Convert structured data to display format for backward compatibility
    let system_data = SystemData {
        os_name: system_info.os.name,
        kernel_version: system_info.os.kernel_version,
        cpu_brand: system_info.hardware.cpu.brand,
        gpu: system_info.hardware.gpu.primary,
        all_gpus: system_info.hardware.gpu.all_gpus,
        username: system_info.user.username,
        hostname: system_info.user.hostname,
        wm_de: system_info.status.desktop_environment,
        memory: system_info.hardware.memory.formatted,
        pkg_count: system_info.packages.system_packages,
        flatpak_pkg_count: system_info.packages.flatpak_packages,
        uptime_formatted: system_info.status.uptime_formatted,
        os_age: system_info.os.age,
        editor: system_info.status.editor,
        shell: system_info.status.shell,
        terminal: system_info.status.terminal,
        user_info: system_info.user.formatted_user_info,
        disk_usage: system_info.hardware.disk.usage,
        init_system: system_info.status.init_system,
        battery_info: system_info.status.battery_info,
    };

    render_output(&config, &system_data);
    Ok(())
}

/// Collect all system information
fn collect_system_info() -> Result<SystemInfo> {
    Ok(SystemInfo {
        os: collectors::system::collect_os_info()?,
        hardware: collectors::hardware::collect_hardware_info()?,
        packages: collectors::packages::collect_package_info()?,
        status: collectors::system::collect_system_status()?,
        user: collectors::system::collect_user_info()?,
    })
}