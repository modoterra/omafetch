use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Display;

impl Module for Display {
    fn name(&self) -> &'static str {
        "display"
    }

    fn label(&self) -> &'static str {
        "Display"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::env::wm()
            .filter(|wm| wm.eq_ignore_ascii_case("hyprland"))
            .and_then(|_| crate::probe::command::run_capture("hyprctl", &["monitors"]))
            .and_then(|output| display_from_hyprctl(&output))
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}

fn display_from_hyprctl(input: &str) -> Option<String> {
    let mut displays = Vec::new();
    let mut current = DisplayInfo::default();

    for line in input.lines() {
        let line = line.trim();

        if line.starts_with("Monitor ") {
            if let Some(display) = current.format() {
                displays.push(display);
            }
            current = DisplayInfo::default();
            continue;
        }

        if let Some((resolution, _position)) = line.split_once(" at ") {
            current.mode = Some(format_mode(resolution));
        } else if let Some(model) = line.strip_prefix("model: ") {
            current.model = Some(model.to_string());
        } else if let Some(scale) = line.strip_prefix("scale: ") {
            current.scale = Some(format_scale(scale));
        }
    }

    if let Some(display) = current.format() {
        displays.push(display);
    }

    if displays.is_empty() {
        None
    } else {
        Some(displays.join(", "))
    }
}

#[derive(Default)]
struct DisplayInfo {
    model: Option<String>,
    mode: Option<String>,
    scale: Option<String>,
}

impl DisplayInfo {
    fn format(self) -> Option<String> {
        let mode = self.mode?;
        let mut parts = Vec::new();
        if let Some(model) = self.model {
            parts.push(model);
        }
        parts.push(mode);
        if let Some(scale) = self.scale {
            parts.push(scale);
        }
        Some(parts.join(" "))
    }
}

fn format_mode(resolution: &str) -> String {
    let Some((size, refresh)) = resolution.split_once('@') else {
        return resolution.to_string();
    };

    let hz = refresh
        .parse::<f64>()
        .ok()
        .map(|value| format!("{} Hz", value.round() as u64))
        .unwrap_or_else(|| format!("{refresh} Hz"));
    format!("{size} {hz}")
}

fn format_scale(scale: &str) -> String {
    scale
        .parse::<f64>()
        .ok()
        .map(|value| format!("{}x", trim_float(value)))
        .unwrap_or_else(|| format!("{scale}x"))
}

fn trim_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as u64)
    } else {
        format!("{value:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hyprctl_display() {
        let input =
            "Monitor DP-1 (ID 0):\n\t3840x2160@120.00 at 0x0\n\tmodel: PA32QCV\n\tscale: 2.00\n";

        assert_eq!(
            display_from_hyprctl(input),
            Some("PA32QCV 3840x2160 120 Hz 2x".to_string())
        );
    }
}
