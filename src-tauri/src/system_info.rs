use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SystemInfo {
    pub os: OsInfo,
    pub motherboard: MoboInfo,
    pub cpu: CpuInfo,
    pub ram: Vec<RamStick>,
    pub gpus: Vec<GpuInfo>,
    pub drives: Vec<DriveInfo>,
    pub network_adapters: Vec<NetAdapter>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NetAdapter {
    pub name: Option<String>,
    pub mac: Option<String>,
    pub speed_mbps: Option<u64>,
    pub kind: Option<String>, // "Ethernet" | "Wireless" | "Bluetooth" | other
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OsInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub build: Option<String>,
    pub install_date: Option<String>,
    pub last_boot: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MoboInfo {
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub bios_vendor: Option<String>,
    pub bios_version: Option<String>,
    pub bios_date: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CpuInfo {
    pub name: Option<String>,
    pub cores: Option<u32>,
    pub threads: Option<u32>,
    pub max_clock_mhz: Option<u32>,
    pub l2_cache_kb: Option<u32>,
    pub l3_cache_kb: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RamStick {
    pub slot: Option<String>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub speed_mhz: Option<u32>,
    pub memory_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GpuInfo {
    pub name: Option<String>,
    pub vram_bytes: Option<u64>,
    pub driver_version: Option<String>,
    pub driver_date: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DriveInfo {
    pub model: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub interface_type: Option<String>,
    pub health: Option<String>,
}

#[derive(Default)]
pub struct SystemInfoCache {
    cached: std::sync::Mutex<Option<SystemInfo>>,
}

impl SystemInfoCache {
    pub fn get_or_collect(&self) -> SystemInfo {
        let mut guard = self.cached.lock().unwrap();
        if let Some(ref info) = *guard {
            return info.clone();
        }
        let info = collect();
        *guard = Some(info.clone());
        info
    }

    pub fn invalidate(&self) {
        *self.cached.lock().unwrap() = None;
    }
}

pub fn collect() -> SystemInfo {
    #[cfg(target_os = "windows")]
    {
        collect_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        SystemInfo::default()
    }
}

#[cfg(target_os = "windows")]
fn collect_windows() -> SystemInfo {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Os {
        caption: Option<String>,
        version: Option<String>,
        build_number: Option<String>,
        install_date: Option<String>,
        last_boot_up_time: Option<String>,
        locale: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Bb {
        manufacturer: Option<String>,
        product: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Bios {
        manufacturer: Option<String>,
        #[serde(rename = "SMBIOSBIOSVersion")]
        smbios_bios_version: Option<String>,
        release_date: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Cpu {
        name: Option<String>,
        number_of_cores: Option<u32>,
        number_of_logical_processors: Option<u32>,
        max_clock_speed: Option<u32>,
        l2_cache_size: Option<u32>,
        l3_cache_size: Option<u32>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Mem {
        device_locator: Option<String>,
        manufacturer: Option<String>,
        part_number: Option<String>,
        capacity: Option<u64>,
        speed: Option<u32>,
        memory_type: Option<u32>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Vc {
        name: Option<String>,
        #[serde(rename = "AdapterRAM")]
        adapter_ram: Option<u64>,
        driver_version: Option<String>,
        driver_date: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Dk {
        model: Option<String>,
        size: Option<u64>,
        interface_type: Option<String>,
        status: Option<String>,
    }

    let mut info = SystemInfo::default();
    let Ok(com) = COMLibrary::new() else { return info; };
    let Ok(conn) = WMIConnection::new(com) else { return info; };

    if let Ok(rows) = conn.raw_query::<Os>(
        "SELECT Caption, Version, BuildNumber, InstallDate, LastBootUpTime, Locale FROM Win32_OperatingSystem",
    ) {
        if let Some(o) = rows.into_iter().next() {
            info.os = OsInfo {
                name: o.caption,
                version: o.version,
                build: o.build_number,
                install_date: o.install_date,
                last_boot: o.last_boot_up_time,
                locale: o.locale,
            };
        }
    }
    if let Ok(rows) = conn.raw_query::<Bb>("SELECT Manufacturer, Product FROM Win32_BaseBoard") {
        if let Some(b) = rows.into_iter().next() {
            info.motherboard.manufacturer = b.manufacturer;
            info.motherboard.product = b.product;
        }
    }
    if let Ok(rows) =
        conn.raw_query::<Bios>("SELECT Manufacturer, SMBIOSBIOSVersion, ReleaseDate FROM Win32_BIOS")
    {
        if let Some(b) = rows.into_iter().next() {
            info.motherboard.bios_vendor = b.manufacturer;
            info.motherboard.bios_version = b.smbios_bios_version;
            info.motherboard.bios_date = b.release_date;
        }
    }
    if let Ok(rows) = conn.raw_query::<Cpu>(
        "SELECT Name, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed, L2CacheSize, L3CacheSize FROM Win32_Processor",
    ) {
        if let Some(c) = rows.into_iter().next() {
            info.cpu = CpuInfo {
                name: c.name,
                cores: c.number_of_cores,
                threads: c.number_of_logical_processors,
                max_clock_mhz: c.max_clock_speed,
                l2_cache_kb: c.l2_cache_size,
                l3_cache_kb: c.l3_cache_size,
            };
        }
    }
    if let Ok(rows) = conn.raw_query::<Mem>(
        "SELECT DeviceLocator, Manufacturer, PartNumber, Capacity, Speed, MemoryType FROM Win32_PhysicalMemory",
    ) {
        info.ram = rows
            .into_iter()
            .map(|m| RamStick {
                slot: m.device_locator,
                manufacturer: m
                    .manufacturer
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
                part_number: m
                    .part_number
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
                capacity_bytes: m.capacity,
                speed_mhz: m.speed,
                memory_type: m.memory_type.map(memory_type_name),
            })
            .collect();
    }
    if let Ok(rows) = conn.raw_query::<Vc>(
        "SELECT Name, AdapterRAM, DriverVersion, DriverDate FROM Win32_VideoController",
    ) {
        info.gpus = rows
            .into_iter()
            .map(|v| GpuInfo {
                name: v.name,
                vram_bytes: v.adapter_ram,
                driver_version: v.driver_version,
                driver_date: v.driver_date,
            })
            .collect();
    }
    if let Ok(rows) =
        conn.raw_query::<Dk>("SELECT Model, Size, InterfaceType, Status FROM Win32_DiskDrive")
    {
        info.drives = rows
            .into_iter()
            .map(|d| DriveInfo {
                model: d.model.map(|s| s.trim().to_string()),
                capacity_bytes: d.size,
                interface_type: d.interface_type,
                health: Some(map_drive_status(d.status.as_deref())),
            })
            .collect();
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Na {
        name: Option<String>,
        #[serde(rename = "MACAddress")]
        mac_address: Option<String>,
        speed: Option<u64>,
        adapter_type: Option<String>,
    }
    if let Ok(rows) = conn.raw_query::<Na>(
        "SELECT Name, MACAddress, Speed, AdapterType FROM Win32_NetworkAdapter",
    ) {
        info.network_adapters = rows
            .into_iter()
            .filter(|n| {
                n.mac_address
                    .as_ref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
            })
            .map(|n| NetAdapter {
                name: n.name,
                mac: n.mac_address,
                speed_mbps: n.speed.map(|s| s / 1_000_000),
                kind: n.adapter_type.map(|s| classify_adapter_type(&s)),
            })
            .collect();
    }
    info
}

pub fn classify_adapter_type(s: &str) -> String {
    let lc = s.to_lowercase();
    if lc.contains("802.11") || lc.contains("wireless") || lc.contains("wi-fi") || lc.contains("wifi") {
        "Wireless".to_string()
    } else if lc.contains("ethernet") || lc.contains("802.3") {
        "Ethernet".to_string()
    } else if lc.contains("bluetooth") {
        "Bluetooth".to_string()
    } else if lc.contains("tunnel") {
        "Tunnel".to_string()
    } else {
        s.to_string()
    }
}

pub fn memory_type_name(code: u32) -> String {
    match code {
        20 => "DDR".into(),
        21 => "DDR2".into(),
        22 => "DDR2 FB-DIMM".into(),
        24 => "DDR3".into(),
        26 => "DDR4".into(),
        30 => "LPDDR4".into(),
        34 => "DDR5".into(),
        35 => "LPDDR5".into(),
        _ => format!("Type {}", code),
    }
}

pub fn map_drive_status(status: Option<&str>) -> String {
    match status {
        Some("OK") => "Healthy".into(),
        Some("Degraded") => "Warning".into(),
        Some("Pred Fail") | Some("Error") => "Critical".into(),
        _ => "Unknown".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_type_known_codes() {
        assert_eq!(memory_type_name(26), "DDR4");
        assert_eq!(memory_type_name(34), "DDR5");
        assert_eq!(memory_type_name(24), "DDR3");
    }

    #[test]
    fn memory_type_unknown_code() {
        assert_eq!(memory_type_name(999), "Type 999");
    }

    #[test]
    fn classify_ethernet() {
        assert_eq!(classify_adapter_type("Ethernet 802.3"), "Ethernet");
        assert_eq!(classify_adapter_type("ethernet"), "Ethernet");
    }

    #[test]
    fn classify_wireless() {
        assert_eq!(classify_adapter_type("802.11 Wireless"), "Wireless");
        assert_eq!(classify_adapter_type("Wi-Fi"), "Wireless");
        assert_eq!(classify_adapter_type("WiFi"), "Wireless");
    }

    #[test]
    fn classify_bluetooth() {
        assert_eq!(classify_adapter_type("Bluetooth PAN"), "Bluetooth");
    }

    #[test]
    fn classify_tunnel() {
        assert_eq!(classify_adapter_type("Tunnel"), "Tunnel");
    }

    #[test]
    fn classify_unknown_passes_through() {
        assert_eq!(classify_adapter_type("Loopback"), "Loopback");
    }

    #[test]
    fn drive_status_mapping() {
        assert_eq!(map_drive_status(Some("OK")), "Healthy");
        assert_eq!(map_drive_status(Some("Pred Fail")), "Critical");
        assert_eq!(map_drive_status(Some("Degraded")), "Warning");
        assert_eq!(map_drive_status(None), "Unknown");
        assert_eq!(map_drive_status(Some("Foobar")), "Unknown");
    }

    #[test]
    fn cache_returns_same_instance() {
        let cache = SystemInfoCache::default();
        let a = cache.get_or_collect();
        let b = cache.get_or_collect();
        assert_eq!(a.os.name, b.os.name);
    }

    #[test]
    fn collect_returns_some_os_data_on_windows() {
        let info = collect();
        #[cfg(target_os = "windows")]
        {
            assert!(info.os.name.is_some(), "Windows must return OS name");
        }
        let _ = info;
    }
}
