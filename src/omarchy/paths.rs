use std::path::PathBuf;

pub struct OmarchyPaths {
    pub install_dir: PathBuf,
    pub version: PathBuf,
    pub current_theme_name: PathBuf,
    pub current_theme_dir: PathBuf,
    pub current_colors: PathBuf,
}

impl OmarchyPaths {
    pub fn discover() -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let install_dir = home.join(".local/share/omarchy");
        let current = home.join(".config/omarchy/current");

        Some(Self {
            install_dir: install_dir.clone(),
            version: install_dir.join("version"),
            current_theme_name: current.join("theme.name"),
            current_theme_dir: current.join("theme"),
            current_colors: current.join("theme/colors.toml"),
        })
    }
}
