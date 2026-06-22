use std::path::Path;

pub fn shell() -> Option<String> {
    std::env::var("SHELL")
        .ok()
        .and_then(|shell| basename(&shell))
}

pub fn shell_with_version() -> Option<String> {
    let shell = shell()?;
    let version = match shell.as_str() {
        "bash" => crate::probe::command::run_capture("bash", &["--version"])
            .and_then(|output| output.lines().next().map(parse_bash_version)),
        "zsh" => crate::probe::command::run_capture("zsh", &["--version"])
            .and_then(|output| output.split_whitespace().nth(1).map(ToString::to_string)),
        "fish" => crate::probe::command::run_capture("fish", &["--version"])
            .and_then(|output| output.split_whitespace().last().map(ToString::to_string)),
        _ => None,
    };

    Some(match version {
        Some(version) => format!("{shell} v{version}"),
        None => shell,
    })
}

pub fn terminal() -> Option<String> {
    if std::env::var_os("GHOSTTY_RESOURCES_DIR").is_some() {
        return Some("Ghostty".to_string());
    }
    if std::env::var_os("KITTY_WINDOW_ID").is_some() {
        return Some("Kitty".to_string());
    }
    if std::env::var_os("ALACRITTY_SOCKET").is_some() {
        return Some("Alacritty".to_string());
    }
    if std::env::var_os("WEZTERM_EXECUTABLE").is_some() {
        return Some("WezTerm".to_string());
    }

    std::env::var("TERM_PROGRAM")
        .ok()
        .or_else(|| std::env::var("TERMINAL").ok())
        .or_else(|| std::env::var("TERM").ok())
}

pub fn terminal_with_version() -> Option<String> {
    let terminal = terminal()?;
    let version = match terminal.as_str() {
        "Ghostty" => crate::probe::command::run_capture("ghostty", &["--version"])
            .and_then(|output| output.split_whitespace().nth(1).map(ToString::to_string)),
        "Kitty" => crate::probe::command::run_capture("kitty", &["--version"])
            .and_then(|output| output.split_whitespace().nth(1).map(ToString::to_string)),
        "WezTerm" => crate::probe::command::run_capture("wezterm", &["--version"])
            .and_then(|output| output.split_whitespace().nth(1).map(ToString::to_string)),
        _ => None,
    };

    Some(match version {
        Some(version) => format!("{terminal} v{version}"),
        None => terminal,
    })
}

pub fn wm() -> Option<String> {
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        return Some("Hyprland".to_string());
    }

    std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .or_else(|| std::env::var("DESKTOP_SESSION").ok())
}

pub fn wm_with_version() -> Option<String> {
    let wm = wm()?;
    let version = if wm.eq_ignore_ascii_case("hyprland") {
        crate::probe::command::run_capture("hyprctl", &["version"])
            .and_then(|output| output.split_whitespace().nth(1).map(ToString::to_string))
    } else {
        None
    };

    Some(match version {
        Some(version) if version.starts_with('v') => format!("{wm} {version}"),
        Some(version) => format!("{wm} v{version}"),
        None => wm,
    })
}

pub fn basename(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()?
        .to_str()
        .map(ToString::to_string)
        .filter(|value| !value.is_empty())
}

fn parse_bash_version(line: &str) -> String {
    line.split_once("version ")
        .and_then(|(_, rest)| rest.split_once('('))
        .map(|(version, _)| version.to_string())
        .unwrap_or_else(|| line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_basename() {
        assert_eq!(basename("/usr/bin/zsh"), Some("zsh".to_string()));
        assert_eq!(basename("fish"), Some("fish".to_string()));
    }
}
