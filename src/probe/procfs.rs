use std::time::Duration;

use crate::probe::filesystem::read_to_string;

#[derive(Debug, PartialEq, Eq)]
pub struct MemInfo {
    pub total_kib: u64,
    pub available_kib: u64,
    pub swap_total_kib: u64,
    pub swap_free_kib: u64,
}

pub fn os_name_from(input: &str) -> Option<String> {
    parse_key_value(input, "PRETTY_NAME").or_else(|| parse_key_value(input, "NAME"))
}

pub fn os_name() -> Option<String> {
    read_to_string("/etc/os-release").and_then(|input| os_name_from(&input))
}

pub fn kernel() -> Option<String> {
    read_to_string("/proc/sys/kernel/osrelease").map(|release| format!("Linux {release}"))
}

pub fn uptime() -> Option<Duration> {
    let input = read_to_string("/proc/uptime")?;
    let seconds = input.split_whitespace().next()?.parse::<f64>().ok()? as u64;
    Some(Duration::from_secs(seconds))
}

pub fn cpu_model_from(input: &str) -> Option<String> {
    input.lines().find_map(|line| {
        line.strip_prefix("model name")
            .and_then(|rest| rest.split_once(':'))
            .map(|(_, value)| clean_cpu(value.trim()))
    })
}

pub fn cpu_model() -> Option<String> {
    read_to_string("/proc/cpuinfo").and_then(|input| cpu_model_from(&input))
}

pub fn meminfo_from(input: &str) -> Option<MemInfo> {
    let mut total_kib = None;
    let mut available_kib = None;
    let mut swap_total_kib = None;
    let mut swap_free_kib = None;

    for line in input.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total_kib = parse_kib(value);
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available_kib = parse_kib(value);
        } else if let Some(value) = line.strip_prefix("SwapTotal:") {
            swap_total_kib = parse_kib(value);
        } else if let Some(value) = line.strip_prefix("SwapFree:") {
            swap_free_kib = parse_kib(value);
        }
    }

    Some(MemInfo {
        total_kib: total_kib?,
        available_kib: available_kib?,
        swap_total_kib: swap_total_kib.unwrap_or(0),
        swap_free_kib: swap_free_kib.unwrap_or(0),
    })
}

pub fn meminfo() -> Option<MemInfo> {
    read_to_string("/proc/meminfo").and_then(|input| meminfo_from(&input))
}

pub fn rootfs_from(input: &str) -> Option<String> {
    let (device, fs, options) = input.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let device = fields.next()?;
        let mount = fields.next()?;
        let fs = fields.next()?;
        let options = fields.next()?;
        (mount == "/").then_some((device, fs, options))
    })?;

    let mut parts = vec![fs.to_string()];

    if device.starts_with("/dev/mapper/") {
        parts.push("encrypted".to_string());
    }

    if let Some(subvol) = options
        .split(',')
        .find_map(|option| option.strip_prefix("subvol="))
    {
        parts.push(format!("subvol={subvol}"));
    }

    if let Some(compress) = options
        .split(',')
        .find_map(|option| option.strip_prefix("compress="))
    {
        parts.push(format!("compress={compress}"));
    }

    Some(parts.join(" "))
}

pub fn rootfs() -> Option<String> {
    read_to_string("/proc/mounts").and_then(|input| rootfs_from(&input))
}

fn parse_key_value(input: &str, key: &str) -> Option<String> {
    input.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate == key).then(|| value.trim_matches('"').to_string())
    })
}

fn parse_kib(value: &str) -> Option<u64> {
    value.split_whitespace().next()?.parse().ok()
}

fn clean_cpu(value: &str) -> String {
    value
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("  ", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_os_release() {
        assert_eq!(
            os_name_from("NAME=Arch Linux\nPRETTY_NAME=\"Arch Linux\""),
            Some("Arch Linux".to_string())
        );
    }

    #[test]
    fn parses_cpuinfo() {
        assert_eq!(
            cpu_model_from("processor: 0\nmodel name\t: AMD Ryzen AI Max+ 395\n"),
            Some("AMD Ryzen AI Max+ 395".to_string())
        );
    }

    #[test]
    fn parses_meminfo() {
        assert_eq!(
            meminfo_from(
                "MemTotal:       2048000 kB\nMemAvailable:   1024000 kB\nSwapTotal: 500 kB\nSwapFree: 200 kB\n"
            ),
            Some(MemInfo {
                total_kib: 2048000,
                available_kib: 1024000,
                swap_total_kib: 500,
                swap_free_kib: 200
            })
        );
    }

    #[test]
    fn parses_rootfs() {
        assert_eq!(
            rootfs_from("/dev/mapper/root / btrfs rw,subvol=/@,compress=zstd:3 0 0"),
            Some("btrfs encrypted subvol=/@ compress=zstd:3".to_string())
        );
    }
}
