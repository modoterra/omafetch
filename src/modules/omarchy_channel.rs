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
    channel_from(&input)
}

fn channel_from(input: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_stable_before_generic_mirror_host() {
        assert_eq!(
            channel_from("Server = https://stable-mirror.omarchy.org/$repo/os/$arch\n"),
            Some("stable".to_string())
        );
        assert_eq!(
            channel_from("Include = https://pkgs.omarchy.org/stable/$repo\n"),
            Some("stable".to_string())
        );
    }

    #[test]
    fn detects_rc_channel() {
        assert_eq!(
            channel_from("Server = https://rc-mirror.omarchy.org/$repo/os/$arch\n"),
            Some("rc".to_string())
        );
        assert_eq!(
            channel_from("Include = https://pkgs.omarchy.org/rc/$repo\n"),
            Some("rc".to_string())
        );
    }

    #[test]
    fn detects_edge_channel() {
        assert_eq!(
            channel_from("Server = https://mirror.omarchy.org/$repo/os/$arch\n"),
            Some("edge".to_string())
        );
        assert_eq!(
            channel_from("Include = https://pkgs.omarchy.org/edge/$repo\n"),
            Some("edge".to_string())
        );
    }

    #[test]
    fn unrelated_mirrorlist_is_unknown() {
        assert_eq!(
            channel_from("Server = https://geo.mirror.pkgbuild.com/$repo/os/$arch\n"),
            None
        );
    }
}
