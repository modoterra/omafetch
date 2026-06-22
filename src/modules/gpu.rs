use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Gpu;

impl Module for Gpu {
    fn name(&self) -> &'static str {
        "gpu"
    }

    fn label(&self) -> &'static str {
        "GPU"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::command::run_capture("lspci", &["-mm", "-d", "::03"])
            .and_then(|output| gpu_from_lspci(&output))
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}

fn gpu_from_lspci(input: &str) -> Option<String> {
    input
        .lines()
        .filter_map(|line| {
            gpu_from_machine_line(line).or_else(|| {
                line.split_once(": ")
                    .map(|(_, value)| clean_gpu(value))
                    .filter(|_| is_gpu_line(line))
            })
        })
        .next()
}

fn gpu_from_machine_line(line: &str) -> Option<String> {
    let fields = quoted_fields(line);
    let class = fields.first()?;
    if !is_gpu_class(class) {
        return None;
    }

    let vendor = fields.get(1)?;
    let model = fields.get(2)?;
    Some(clean_gpu(&format!("{vendor} {model}")))
}

fn quoted_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut chars = line.chars();

    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }

        let mut field = String::new();
        for ch in chars.by_ref() {
            if ch == '"' {
                break;
            }
            field.push(ch);
        }
        fields.push(field);
    }

    fields
}

fn is_gpu_line(line: &str) -> bool {
    line.contains("VGA compatible controller")
        || line.contains("3D controller")
        || line.contains("Display controller")
}

fn is_gpu_class(class: &str) -> bool {
    matches!(
        class,
        "VGA compatible controller" | "3D controller" | "Display controller"
    )
}

fn clean_gpu(value: &str) -> String {
    value
        .replace("Advanced Micro Devices, Inc. [AMD/ATI]", "AMD")
        .replace("Intel Corporation", "Intel")
        .replace("NVIDIA Corporation", "NVIDIA")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lspci_gpu() {
        let input = "c1:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon 8060S]\n";

        assert_eq!(
            gpu_from_lspci(input),
            Some("AMD Strix Halo [Radeon 8060S]".to_string())
        );
    }

    #[test]
    fn parses_machine_readable_lspci_gpu() {
        let input = "c3:00.0 \"Display controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics]\" -rc1 -p00 \"Framework Computer Inc.\" \"Device 000a\"\n";

        assert_eq!(
            gpu_from_lspci(input),
            Some(
                "AMD Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics]"
                    .to_string()
            )
        );
    }
}
