#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct OmarchyState {
    pub version: Option<String>,
    pub theme_name: Option<String>,
    pub variant: Option<String>,
    pub accent: Option<String>,
    pub wallpaper: Option<String>,
}

impl OmarchyState {
    pub fn discover() -> Self {
        let Some(paths) = crate::omarchy::paths::OmarchyPaths::discover() else {
            return Self::default();
        };
        let theme = crate::omarchy::theme::discover(&paths);

        Self {
            version: discover_version(&paths),
            theme_name: theme.name,
            variant: theme.variant,
            accent: theme.accent,
            wallpaper: theme.wallpaper,
        }
    }

    pub fn theme_label(&self) -> String {
        match (&self.theme_name, &self.variant) {
            (Some(theme), Some(variant)) => {
                format!("{} ({variant})", crate::omarchy::theme::display_name(theme))
            }
            (Some(theme), None) => crate::omarchy::theme::display_name(theme),
            _ => "unknown".to_string(),
        }
    }
}

fn discover_version(paths: &crate::omarchy::paths::OmarchyPaths) -> Option<String> {
    if !paths.is_packaged_install() {
        let path = paths.install_dir.to_string_lossy();
        let hash = crate::probe::command::run_capture(
            "git",
            &["-C", &path, "rev-parse", "--short", "HEAD"],
        );
        return Some(match hash {
            Some(hash) => format!("dev ({hash})"),
            None => "dev".to_string(),
        });
    }

    if let Some(version) = packaged_omarchy_version() {
        return Some(version);
    }

    crate::probe::filesystem::read_to_string(&paths.version)
}

fn packaged_omarchy_version() -> Option<String> {
    crate::probe::command::run_capture("pacman", &["-Q", "omarchy"])
        .and_then(|line| parse_pacman_q_version(&line))
}

fn parse_pacman_q_version(line: &str) -> Option<String> {
    line.split_whitespace().nth(1).map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pacman_q_omarchy_line() {
        assert_eq!(
            parse_pacman_q_version("omarchy 4.0.0.r1046.gd570d99-1"),
            Some("4.0.0.r1046.gd570d99-1".to_string())
        );
    }
}
