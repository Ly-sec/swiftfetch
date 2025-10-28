//! Hardware-specific information structures

/// Hardware information
#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub gpu: GpuInfo,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
}

/// CPU information
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub brand: String,
    // Future: temperature, cores, frequency
}

/// GPU information with support for multiple GPUs
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub primary: String,
    pub all_gpus: Vec<String>,
}

/// Memory information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    #[allow(dead_code)]
    pub used_gb: f64,
    #[allow(dead_code)]
    pub total_gb: f64,
    pub formatted: String,
}

/// Disk usage information
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub usage: String,
    // Future: individual partition info
}
