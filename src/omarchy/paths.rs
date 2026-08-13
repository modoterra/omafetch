use std::path::{Path, PathBuf};

const PACKAGED_INSTALL: &str = "/usr/share/omarchy";

pub struct OmarchyPaths {
    pub install_dir: PathBuf,
    pub version: PathBuf,
    pub current_theme_name: PathBuf,
    pub current_theme_dir: PathBuf,
    pub current_colors: PathBuf,
    pub current_background: PathBuf,
}

impl OmarchyPaths {
    pub fn discover() -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        Some(Self::from_home(&home))
    }

    pub fn from_home(home: &Path) -> Self {
        let install_dir = install_dir();
        let current = current_dir(home);

        Self {
            version: install_dir.join("version"),
            install_dir,
            current_theme_name: current.join("theme.name"),
            current_theme_dir: current.join("theme"),
            current_colors: current.join("theme/colors.toml"),
            current_background: current.join("background"),
        }
    }

    pub fn is_packaged_install(&self) -> bool {
        self.install_dir == Path::new(PACKAGED_INSTALL)
    }
}

fn install_dir() -> PathBuf {
    std::env::var_os("OMARCHY_PATH")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from(PACKAGED_INSTALL))
}

fn current_dir(home: &Path) -> PathBuf {
    let state_current = home.join(".local/state/omarchy/current");
    if state_current.join("theme.name").is_file() {
        return state_current;
    }

    let config_current = home.join(".config/omarchy/current");
    if config_current.join("theme.name").is_file() {
        return config_current;
    }

    state_current
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

    fn temp_home() -> PathBuf {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "omafetch-omarchy-paths-{}-{id}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_theme_name(current: &Path) {
        fs::create_dir_all(current).unwrap();
        fs::write(current.join("theme.name"), "vantablack\n").unwrap();
    }

    #[test]
    fn prefers_state_current_when_theme_name_exists() {
        let home = temp_home();
        write_theme_name(&home.join(".local/state/omarchy/current"));
        write_theme_name(&home.join(".config/omarchy/current"));

        let paths = OmarchyPaths::from_home(&home);
        assert_eq!(
            paths.current_theme_name,
            home.join(".local/state/omarchy/current/theme.name")
        );
    }

    #[test]
    fn falls_back_to_config_current() {
        let home = temp_home();
        write_theme_name(&home.join(".config/omarchy/current"));

        let paths = OmarchyPaths::from_home(&home);
        assert_eq!(
            paths.current_theme_name,
            home.join(".config/omarchy/current/theme.name")
        );
    }

    #[test]
    fn defaults_to_state_current_when_neither_exists() {
        let home = temp_home();
        let paths = OmarchyPaths::from_home(&home);
        assert_eq!(
            paths.current_theme_name,
            home.join(".local/state/omarchy/current/theme.name")
        );
    }
}
