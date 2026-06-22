use crate::probe::filesystem::read_to_string;

#[derive(Debug, PartialEq, Eq)]
pub struct BatteryInfo {
    pub capacity: Option<u8>,
    pub status: Option<String>,
}

pub fn host_model() -> Option<String> {
    let vendor = read_to_string("/sys/devices/virtual/dmi/id/sys_vendor");
    let product = read_to_string("/sys/devices/virtual/dmi/id/product_name");

    match (vendor, product) {
        (Some(vendor), Some(product)) if product.contains(&vendor) => Some(product),
        (Some(vendor), Some(product)) => Some(format!("{vendor} {product}")),
        (_, Some(product)) => Some(product),
        (Some(vendor), _) => Some(vendor),
        _ => read_to_string("/etc/hostname"),
    }
}

pub fn battery() -> Option<BatteryInfo> {
    let entries = std::fs::read_dir("/sys/class/power_supply").ok()?;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if read_to_string(path.join("type")).as_deref() != Some("Battery") {
            continue;
        }

        let capacity = read_to_string(path.join("capacity")).and_then(|value| value.parse().ok());
        let status = read_to_string(path.join("status")).map(|value| value.to_lowercase());

        return Some(BatteryInfo { capacity, status });
    }

    None
}

pub fn kernel_runtime_config() -> Vec<String> {
    let mut values = Vec::new();

    if let Some(preempt) = selected_bracket_value("/sys/kernel/debug/sched/preempt") {
        values.push(format!("preempt {preempt}"));
    }

    if let Some(pstate) = read_to_string("/sys/devices/system/cpu/amd_pstate/status") {
        values.push(format!("pstate {pstate}"));
    }

    if let Some(thp) = selected_bracket_value("/sys/kernel/mm/transparent_hugepage/enabled") {
        values.push(format!("THP {thp}"));
    }

    if let Some(zswap) = read_to_string("/sys/module/zswap/parameters/enabled") {
        values.push(format!("zswap {}", boolish(&zswap)));
    }

    if let Some(cmdline) = read_to_string("/proc/cmdline") {
        if let Some(iommu) = cmdline_value(&cmdline, "iommu") {
            values.push(format!("IOMMU {iommu}"));
        }

        if let Some(ttm_pages) =
            cmdline_value(&cmdline, "ttm.pages_limit").and_then(|value| value.parse::<u64>().ok())
        {
            let bytes = ttm_pages.saturating_mul(4096);
            values.push(format!(
                "TTM {:.0}G",
                bytes as f64 / 1024.0 / 1024.0 / 1024.0
            ));
        }
    }

    values
}

pub fn gtt_memory_bytes() -> Option<u64> {
    let entries = std::fs::read_dir("/sys/class/drm").ok()?;

    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let device = entry.path().join("device");
        let Some(vendor) = read_to_string(device.join("vendor")) else {
            continue;
        };
        if vendor != "0x1002" {
            continue;
        }

        if let Some(total) = read_to_string(device.join("mem_info_gtt_total"))
            .and_then(|value| value.parse::<u64>().ok())
        {
            return Some(total);
        }
    }

    None
}

fn selected_bracket_value(path: &str) -> Option<String> {
    let input = read_to_string(path)?;
    input
        .split_whitespace()
        .find(|value| value.starts_with('[') && value.ends_with(']'))
        .map(|value| value.trim_matches(['[', ']']).to_string())
}

fn boolish(value: &str) -> &str {
    match value {
        "Y" | "y" | "1" => "on",
        "N" | "n" | "0" => "off",
        value => value,
    }
}

fn cmdline_value(input: &str, key: &str) -> Option<String> {
    input.split_whitespace().find_map(|part| {
        let (candidate, value) = part.split_once('=')?;
        (candidate == key).then(|| value.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_boolish_values() {
        assert_eq!(boolish("Y"), "on");
        assert_eq!(boolish("N"), "off");
    }
}
