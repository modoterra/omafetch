use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct OmarchyChannel;

impl Module for OmarchyChannel {
    fn name(&self) -> &'static str {
        "omarchy-channel"
    }

    fn label(&self) -> &'static str {
        "Channel"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let mirror = channel_from_file("/etc/pacman.d/mirrorlist");
        let packages = channel_from_file("/etc/pacman.conf");
        let value = match (mirror, packages) {
            (Some(mirror), Some(packages)) if mirror == packages => mirror,
            (Some(mirror), Some(packages)) => format!("mirror={mirror} packages={packages}"),
            (Some(mirror), None) => format!("mirror={mirror}"),
            (None, Some(packages)) => format!("packages={packages}"),
            _ => "unknown".to_string(),
        };

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}

fn channel_from_file(path: &str) -> Option<String> {
    let input = crate::probe::filesystem::read_to_string(path)?;

    if input.contains("stable-mirror.omarchy.org") || input.contains("pkgs.omarchy.org/stable") {
        Some("stable".to_string())
    } else if input.contains("rc-mirror.omarchy.org") || input.contains("pkgs.omarchy.org/rc") {
        Some("rc".to_string())
    } else if input.contains("mirror.omarchy.org") || input.contains("pkgs.omarchy.org/edge") {
        Some("edge".to_string())
    } else {
        None
    }
}
