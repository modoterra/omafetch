use crate::omarchy::paths::OmarchyPaths;
use crate::probe::filesystem::read_to_string;

#[derive(Debug, Default)]
pub struct ThemeIdentity {
    pub name: Option<String>,
    pub variant: Option<String>,
    pub accent: Option<String>,
    pub wallpaper: Option<String>,
}

pub fn discover(paths: &OmarchyPaths) -> ThemeIdentity {
    let colors = read_to_string(&paths.current_colors);

    ThemeIdentity {
        name: read_to_string(&paths.current_theme_name),
        variant: colors
            .as_deref()
            .and_then(|input| read_toml_string(input, "mode")),
        accent: colors
            .as_deref()
            .and_then(|input| read_toml_string(input, "accent")),
        wallpaper: discover_wallpaper(paths),
    }
}

fn discover_wallpaper(paths: &OmarchyPaths) -> Option<String> {
    let backgrounds = paths.current_theme_dir.join("backgrounds");
    let entries = std::fs::read_dir(backgrounds).ok()?;

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str().map(ToString::to_string))
        .find(|name| name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg"))
}

fn read_toml_string(input: &str, key: &str) -> Option<String> {
    input.lines().find_map(|line| {
        let line = line.trim();
        if line.starts_with('#') {
            return None;
        }

        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key).then(|| value.trim().trim_matches('"').to_string())
    })
}

pub fn display_name(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_toml_string_values() {
        assert_eq!(
            read_toml_string("mode = \"dark\"\naccent = \"#5fafd7\"", "mode"),
            Some("dark".to_string())
        );
    }

    #[test]
    fn formats_theme_display_name() {
        assert_eq!(display_name("dazzle-dusk"), "Dazzle Dusk");
    }
}
