use owo_colors::OwoColorize;

pub fn sparkle_for_row(module: &str, label: &str, value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let seed = format!("{module}\0{label}\0{value}");
    let hash = fnv1a(seed.as_bytes());
    let count = sparkle_count(hash);
    let mut chars = vec![' '; width];

    for index in 0..count {
        let shifted = hash.rotate_right((index * 13) as u32);
        let position = shifted as usize % width;
        let glyph = sparkle_glyph((shifted >> 8) as usize);
        chars[position] = glyph;
    }

    chars
        .into_iter()
        .enumerate()
        .map(|(index, ch)| {
            if ch == ' ' {
                " ".to_string()
            } else {
                color_sparkle(module, ch, hash.rotate_left(index as u32))
            }
        })
        .collect()
}

fn sparkle_glyph(index: usize) -> char {
    let glyphs: &[char] = if unicode_sparkles() {
        &['·', '✦', '✧', '✶']
    } else {
        &['.', '+', '*']
    };

    glyphs[index % glyphs.len()]
}

fn unicode_sparkles() -> bool {
    ["LC_ALL", "LC_CTYPE", "LANG"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .any(|value| {
            let value = value.to_ascii_uppercase();
            value.contains("UTF-8") || value.contains("UTF8")
        })
}

fn sparkle_count(hash: u64) -> usize {
    match hash % 24 {
        0..=1 => 0,
        2..=6 => 1,
        7..=11 => 2,
        12..=16 => 3,
        17..=20 => 4,
        21..=22 => 5,
        _ => 6,
    }
}

fn color_sparkle(module: &str, ch: char, hash: u64) -> String {
    match module {
        "kernel-config" | "rootfs" => ch.green().to_string(),
        "display" | "gpu" | "gtt-memory" => ch.magenta().to_string(),
        "memory" | "swap" | "disk" | "packages" => ch.yellow().to_string(),
        "localip" | "omarchy" | "omarchy-source" | "omarchy-channel" | "theme" => {
            ch.cyan().to_string()
        }
        _ => match hash % 6 {
            0 => ch.cyan().to_string(),
            1 => ch.blue().to_string(),
            2 => ch.magenta().to_string(),
            3 => ch.red().to_string(),
            4 => ch.yellow().to_string(),
            _ => ch.green().to_string(),
        },
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;

    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkle_is_stable() {
        assert_eq!(
            sparkle_for_row("disk", "Disk", "root 1.0 GiB / 2.0 GiB", 16),
            sparkle_for_row("disk", "Disk", "root 1.0 GiB / 2.0 GiB", 16)
        );
    }
}
