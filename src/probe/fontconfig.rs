use std::path::PathBuf;

use crate::probe::filesystem::read_to_string;

pub fn monospace_family() -> Option<String> {
    fonts_conf_path()
        .and_then(read_to_string)
        .and_then(|input| family_from_fonts_conf(&input))
        .or_else(fc_match_monospace)
}

fn fonts_conf_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    Some(home.join(".config/fontconfig/fonts.conf"))
}

fn fc_match_monospace() -> Option<String> {
    crate::probe::command::run_capture("fc-match", &["monospace", "-f", "%{family}\n"])
        .and_then(|output| family_from_fc_match(&output))
}

fn family_from_fonts_conf(input: &str) -> Option<String> {
    let mut rest = input;
    while let Some(start) = rest.find("<match") {
        let after_start = &rest[start..];
        let Some(end) = after_start.find("</match>") else {
            break;
        };
        let block = &after_start[..end];
        if let Some(family) = family_from_match_block(block) {
            return Some(family);
        }
        rest = &after_start[end + "</match>".len()..];
    }
    None
}

fn family_from_match_block(block: &str) -> Option<String> {
    if tests_family(block, "monospace") {
        edited_family(block)
    } else {
        None
    }
}

fn tests_family(block: &str, family: &str) -> bool {
    let mut rest = block;
    while let Some(start) = rest.find("<test") {
        let after_start = &rest[start..];
        let Some(end) = after_start.find("</test>") else {
            break;
        };
        let test = &after_start[..end];
        if has_name_attr(test, "family") && string_values(test).any(|value| value == family) {
            return true;
        }
        rest = &after_start[end + "</test>".len()..];
    }
    false
}

fn edited_family(block: &str) -> Option<String> {
    let mut rest = block;
    while let Some(start) = rest.find("<edit") {
        let after_start = &rest[start..];
        let Some(end) = after_start.find("</edit>") else {
            break;
        };
        let edit = &after_start[..end];
        if has_name_attr(edit, "family") {
            return string_values(edit).next().map(ToString::to_string);
        }
        rest = &after_start[end + "</edit>".len()..];
    }
    None
}

fn has_name_attr(tag: &str, name: &str) -> bool {
    let header = tag.split('>').next().unwrap_or(tag);
    header.contains(&format!("name=\"{name}\"")) || header.contains(&format!("name='{name}'"))
}

fn string_values(block: &str) -> impl Iterator<Item = &str> {
    block.split("<string>").skip(1).filter_map(|part| {
        part.split_once("</string>")
            .map(|(value, _)| value.trim())
            .filter(|value| !value.is_empty())
    })
}

fn family_from_fc_match(output: &str) -> Option<String> {
    output
        .lines()
        .next()
        .and_then(|line| line.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_omarchy_font_set_fonts_conf() {
        let input = r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
  <match target="pattern">
    <test name="family" qual="any">
      <string>monospace</string>
    </test>
    <edit name="family" mode="prepend_first" binding="strong">
      <string>JetBrainsMono Nerd Font</string>
    </edit>
  </match>
</fontconfig>
"#;

        assert_eq!(
            family_from_fonts_conf(input),
            Some("JetBrainsMono Nerd Font".to_string())
        );
    }

    #[test]
    fn parses_assign_monospace_among_other_families() {
        let input = r#"<fontconfig>
  <match target="pattern">
    <test name="family" qual="any">
      <string>sans-serif</string>
    </test>
    <edit name="family" mode="assign" binding="strong">
      <string>Liberation Sans</string>
    </edit>
  </match>
  <match target="pattern">
    <test qual="any" name="family">
      <string>monospace</string>
    </test>
    <edit name="family" mode="assign" binding="strong">
      <string>CaskaydiaMono Nerd Font</string>
    </edit>
  </match>
  <alias>
    <family>monospace</family>
    <accept>
      <family>Noto Color Emoji</family>
    </accept>
  </alias>
</fontconfig>
"#;

        assert_eq!(
            family_from_fonts_conf(input),
            Some("CaskaydiaMono Nerd Font".to_string())
        );
    }

    #[test]
    fn ignores_fonts_conf_without_monospace_match() {
        let input = r#"<fontconfig>
  <match target="pattern">
    <test name="family" qual="any">
      <string>sans-serif</string>
    </test>
    <edit name="family" mode="assign" binding="strong">
      <string>Liberation Sans</string>
    </edit>
  </match>
</fontconfig>
"#;

        assert_eq!(family_from_fonts_conf(input), None);
    }

    #[test]
    fn parses_fc_match_first_family() {
        assert_eq!(
            family_from_fc_match("JetBrainsMono Nerd Font,JetBrainsMono NF\n"),
            Some("JetBrainsMono Nerd Font".to_string())
        );
        assert_eq!(family_from_fc_match("\n"), None);
    }
}
