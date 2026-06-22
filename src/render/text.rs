#[allow(dead_code)]
pub fn fallback(value: Option<String>) -> String {
    value.unwrap_or_else(|| "unknown".to_string())
}

pub fn format_gib(kib: u64) -> String {
    format!("{:.1} GiB", kib as f64 / 1024.0 / 1024.0)
}

pub fn format_bytes(bytes: u64) -> String {
    format!("{:.1} GiB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
}

pub fn format_duration(total_seconds: u64) -> String {
    let days = total_seconds / 86_400;
    let hours = total_seconds % 86_400 / 3_600;
    let minutes = total_seconds % 3_600 / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

pub fn truncate_value(module: &str, value: &str, width: usize) -> String {
    let shortened = match module {
        "gpu" => shorten_gpu(value),
        "cpu" => shorten_cpu(value),
        "host" => shorten_host(value),
        _ => value.to_string(),
    };

    ellipsize(&shortened, width)
}

fn shorten_gpu(value: &str) -> String {
    let value = value.strip_suffix(" (rev c1)").unwrap_or(value);
    if let Some(radeon) = extract_radeon_model(value) {
        return radeon;
    }

    value.to_string()
}

fn shorten_cpu(value: &str) -> String {
    value
        .replace("AMD RYZEN", "AMD Ryzen")
        .replace(" w/ Radeon 8060S", "")
}

fn shorten_host(value: &str) -> String {
    value
        .split_once(" (")
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| value.to_string())
}

fn extract_radeon_model(value: &str) -> Option<String> {
    let start = value.rfind("Radeon ")?;
    let rest = &value[start..];
    let model = rest
        .trim_end_matches(']')
        .split(['/', ']'])
        .next()?
        .trim()
        .to_string();

    (!model.is_empty()).then_some(model)
}

fn ellipsize(value: &str, width: usize) -> String {
    if unicode_width::UnicodeWidthStr::width(value) <= width {
        return value.to_string();
    }

    if width <= 3 {
        return ".".repeat(width);
    }

    let mut result = String::new();
    let mut used = 0;
    let target = width - 3;

    for ch in value.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_width > target {
            break;
        }
        result.push(ch);
        used += ch_width;
    }

    result.push_str("...");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_gib() {
        assert_eq!(format_gib(1_048_576), "1.0 GiB");
    }

    #[test]
    fn formats_bytes() {
        assert_eq!(format_bytes(1_073_741_824), "1.0 GiB");
    }

    #[test]
    fn formats_duration() {
        assert_eq!(format_duration(13_320), "3h 42m");
    }

    #[test]
    fn shortens_gpu_before_truncating() {
        assert_eq!(
            truncate_value(
                "gpu",
                "AMD Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics] (rev c1)",
                30
            ),
            "Radeon 8060S Graphics"
        );
    }

    #[test]
    fn truncates_to_width() {
        assert_eq!(
            truncate_value("theme", "Dazzle Dusk (dark)", 10),
            "Dazzle ..."
        );
    }
}
