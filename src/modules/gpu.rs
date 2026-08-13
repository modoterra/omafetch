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
        // lspci class IDs are 4 hex digits; -d ::03 matches 0003, not 03xx display devices.
        let value = crate::probe::command::run_capture("lspci", &["-mm"])
            .and_then(|output| gpu_from_lspci(&output))
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}

struct GpuDevice {
    class: String,
    name: String,
}

impl GpuDevice {
    fn is_discrete(&self) -> bool {
        is_discrete(&self.class, &self.name)
    }
}

fn gpu_from_lspci(input: &str) -> Option<String> {
    let devices: Vec<GpuDevice> = input.lines().filter_map(parse_gpu).collect();
    devices
        .iter()
        .find(|device| device.is_discrete())
        .or_else(|| devices.first())
        .map(|device| device.name.clone())
}

fn parse_gpu(line: &str) -> Option<GpuDevice> {
    gpu_from_machine_line(line).or_else(|| gpu_from_human_line(line))
}

fn gpu_from_machine_line(line: &str) -> Option<GpuDevice> {
    let fields = quoted_fields(line);
    let class = fields.first()?;
    if !is_gpu_class(class) {
        return None;
    }

    let vendor = fields.get(1)?;
    let model = fields.get(2)?;
    Some(GpuDevice {
        class: class.clone(),
        name: clean_gpu(&format!("{vendor} {model}")),
    })
}

fn gpu_from_human_line(line: &str) -> Option<GpuDevice> {
    if !is_gpu_line(line) {
        return None;
    }

    let (_, value) = line.split_once(": ")?;
    let class = if line.contains("3D controller") {
        "3D controller"
    } else if line.contains("VGA compatible controller") {
        "VGA compatible controller"
    } else {
        "Display controller"
    };

    Some(GpuDevice {
        class: class.to_string(),
        name: clean_gpu(value),
    })
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

fn is_discrete(class: &str, name: &str) -> bool {
    if class == "3D controller" {
        return true;
    }

    let name = name.to_ascii_lowercase();
    if name.contains("nvidia") && !name.contains("tegra") {
        return true;
    }

    is_amd_discrete_name(&name) || is_intel_discrete_name(&name)
}

fn is_amd_discrete_name(name: &str) -> bool {
    if let Some(sku) = name.split("radeon rx").nth(1) {
        let sku = sku.trim();
        if sku.starts_with("vega 56") || sku.starts_with("vega 64") {
            return true;
        }

        let token = sku.split([' ', '/']).next().unwrap_or("");
        return token.len() >= 3 && token.chars().all(|ch| ch.is_ascii_digit());
    }

    name.contains("radeon pro w") || name.contains("radeon vii")
}

fn is_intel_discrete_name(name: &str) -> bool {
    name.contains("arc a") || name.contains("arc b") || name.contains(" dg2")
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

    #[test]
    fn ignores_non_gpu_lspci_devices() {
        let input = concat!(
            "00:00.0 \"Host bridge\" \"Advanced Micro Devices, Inc. [AMD]\" \"Strix/Strix Halo Root Complex\" -r02 -p00 \"Framework Computer Inc.\" \"Device 000a\"\n",
            "c3:00.0 \"Display controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics]\" -rc1 -p00 \"Framework Computer Inc.\" \"Device 000a\"\n",
            "c3:00.1 \"Audio device\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Radeon High Definition Audio Controller\" -p00 \"Framework Computer Inc.\" \"Device 000a\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some(
                "AMD Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics]"
                    .to_string()
            )
        );
    }

    #[test]
    fn prefers_discrete_nvidia_3d_controller_over_intel_igpu() {
        let input = concat!(
            "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Raptor Lake-P [Iris Xe Graphics]\" -r04 -p00 \"Dell\" \"Device 0c0b\"\n",
            "01:00.0 \"3D controller\" \"NVIDIA Corporation\" \"AD107M [GeForce RTX 4060 Max-Q / Mobile]\" -ra1 -p00 \"Dell\" \"Device 0c0b\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("NVIDIA AD107M [GeForce RTX 4060 Max-Q / Mobile]".to_string())
        );
    }

    #[test]
    fn prefers_discrete_nvidia_vga_over_intel_igpu() {
        let input = concat!(
            "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Alder Lake-P GT2 [Iris Xe Graphics]\" -r0c -p00 \"Dell\" \"Device 0b1a\"\n",
            "01:00.0 \"VGA compatible controller\" \"NVIDIA Corporation\" \"GA104 [GeForce RTX 3070]\" -ra1 -p00 \"Dell\" \"Device 0b1a\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("NVIDIA GA104 [GeForce RTX 3070]".to_string())
        );
    }

    #[test]
    fn prefers_discrete_amd_rx_over_igpu() {
        let input = concat!(
            "c3:00.0 \"Display controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics]\" -rc1 -p00 \"Framework Computer Inc.\" \"Device 000a\"\n",
            "01:00.0 \"VGA compatible controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Navi 48 [Radeon RX 9070 XT]\" -p00 \"AMD\" \"Device 0001\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("AMD Navi 48 [Radeon RX 9070 XT]".to_string())
        );
    }

    #[test]
    fn prefers_discrete_intel_arc_over_igpu() {
        let input = concat!(
            "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Raptor Lake-S GT1 [UHD Graphics 770]\" -p00 \"ASUS\" \"Device 0001\"\n",
            "03:00.0 \"VGA compatible controller\" \"Intel Corporation\" \"DG2 [Arc A770]\" -p00 \"Intel Corporation\" \"Device 0000\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("Intel DG2 [Arc A770]".to_string())
        );
    }

    #[test]
    fn does_not_treat_intel_arc_igpu_as_discrete() {
        let input = concat!(
            "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Meteor Lake-P [Intel Arc Graphics]\" -p00 \"Dell\" \"Device 0001\"\n",
            "01:00.0 \"3D controller\" \"NVIDIA Corporation\" \"AD107M [GeForce RTX 4060 Max-Q / Mobile]\" -p00 \"Dell\" \"Device 0001\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("NVIDIA AD107M [GeForce RTX 4060 Max-Q / Mobile]".to_string())
        );
    }

    #[test]
    fn does_not_treat_amd_rx_vega_apu_as_discrete() {
        let input = concat!(
            "05:00.0 \"VGA compatible controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Picasso [Radeon RX Vega 11]\" -p00 \"Lenovo\" \"Device 1234\"\n",
            "06:00.0 \"VGA compatible controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Navi 48 [Radeon RX 9070 XT]\" -p00 \"AMD\" \"Device 0001\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("AMD Navi 48 [Radeon RX 9070 XT]".to_string())
        );
    }

    #[test]
    fn prefers_discrete_amd_rx_vega_56_over_igpu() {
        let input = concat!(
            "05:00.0 \"VGA compatible controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Raven Ridge [Radeon Vega Series / Radeon Vega Mobile Series]\" -p00 \"HP\" \"Device 0001\"\n",
            "01:00.0 \"VGA compatible controller\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Vega 10 XL/XT [Radeon RX Vega 56/64]\" -p00 \"AMD\" \"Device 0001\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("AMD Vega 10 XL/XT [Radeon RX Vega 56/64]".to_string())
        );
    }

    #[test]
    fn prefers_discrete_intel_arc_b_over_igpu() {
        let input = concat!(
            "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Raptor Lake-S GT1 [UHD Graphics 770]\" -p00 \"ASUS\" \"Device 0001\"\n",
            "03:00.0 \"VGA compatible controller\" \"Intel Corporation\" \"Battlemage G21 [Arc B580]\" -p00 \"Intel Corporation\" \"Device 0000\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("Intel Battlemage G21 [Arc B580]".to_string())
        );
    }

    #[test]
    fn prefers_human_readable_discrete_nvidia_over_intel_igpu() {
        let input = concat!(
            "00:02.0 VGA compatible controller: Intel Corporation Raptor Lake-P [Iris Xe Graphics]\n",
            "01:00.0 3D controller: NVIDIA Corporation AD107M [GeForce RTX 4060 Max-Q / Mobile]\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("NVIDIA AD107M [GeForce RTX 4060 Max-Q / Mobile]".to_string())
        );
    }

    #[test]
    fn does_not_treat_nvidia_tegra_as_discrete() {
        let input = concat!(
            "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Alder Lake-P GT2 [Iris Xe Graphics]\" -p00 \"Dell\" \"Device 0001\"\n",
            "01:00.0 \"VGA compatible controller\" \"NVIDIA Corporation\" \"Tegra X1 (nvgpu)\" -p00 \"NVIDIA Corporation\" \"Device 0001\"\n",
        );

        assert_eq!(
            gpu_from_lspci(input),
            Some("Intel Alder Lake-P GT2 [Iris Xe Graphics]".to_string())
        );
    }

    #[test]
    fn empty_lspci_yields_no_gpu() {
        assert_eq!(gpu_from_lspci(""), None);
        assert_eq!(gpu_from_lspci("\n"), None);
    }

    #[test]
    fn non_gpu_lspci_yields_no_gpu() {
        let input = concat!(
            "00:00.0 \"Host bridge\" \"Advanced Micro Devices, Inc. [AMD]\" \"Strix/Strix Halo Root Complex\" -r02 -p00 \"Framework Computer Inc.\" \"Device 000a\"\n",
            "c3:00.1 \"Audio device\" \"Advanced Micro Devices, Inc. [AMD/ATI]\" \"Radeon High Definition Audio Controller\" -p00 \"Framework Computer Inc.\" \"Device 000a\"\n",
        );

        assert_eq!(gpu_from_lspci(input), None);
    }
}
