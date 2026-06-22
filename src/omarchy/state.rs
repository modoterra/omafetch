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
            version: crate::probe::filesystem::read_to_string(&paths.version),
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
