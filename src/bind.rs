use anyhow::{Context, Result, bail};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub const WRAPPER_NAME: &str = "omarchy-launch-about";
pub const REQUIRE_LINE: &str = "require(\"hypr.omafetch\")";

pub const WRAPPER_SOURCE: &str = "\
#!/usr/bin/env bash
# Replace Omarchy's About launcher so the stock menu item opens omafetch.
exec omarchy-launch-or-focus-tui omafetch about
";

pub const HYPR_MODULE_SOURCE: &str = "\
-- Written by `omafetch bind`. Prefer ~/.local/bin so the stock
-- omarchy-launch-about command runs the omafetch wrapper.
do
  local home = os.getenv(\"HOME\") or \"\"
  local omarchy_bin = (os.getenv(\"OMARCHY_PATH\") or \"/usr/share/omarchy\") .. \"/bin\"
  local local_bin = home .. \"/.local/bin\"
  local kept = {}
  local seen = {}
  local function add(dir)
    if dir ~= \"\" and not seen[dir] then
      seen[dir] = true
      table.insert(kept, dir)
    end
  end
  add(local_bin)
  add(omarchy_bin)
  for entry in (os.getenv(\"PATH\") or \"/usr/local/bin:/usr/bin\"):gmatch(\"[^:]+\") do
    if entry ~= omarchy_bin then
      add(entry)
    end
  end
  hl.env(\"PATH\", table.concat(kept, \":\"))
end

-- About window: default fetch is 72x36 cells plus Ghostty pad and borders.
o.window(\"org.omarchy.omafetch\", { float = true })
o.window(\"org.omarchy.omafetch\", { center = true })
o.window(\"org.omarchy.omafetch\", { size = { 724, 714 } })
";

pub struct BindPaths {
    pub local_bin: PathBuf,
    pub hyprland_lua: PathBuf,
    pub omafetch_lua: PathBuf,
}

impl BindPaths {
    pub fn discover() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is unset")?;
        Ok(Self {
            local_bin: home.join(".local/bin"),
            hyprland_lua: home.join(".config/hypr/hyprland.lua"),
            omafetch_lua: home.join(".config/hypr/omafetch.lua"),
        })
    }

    pub fn wrapper(&self) -> PathBuf {
        self.local_bin.join(WRAPPER_NAME)
    }
}

pub fn bind(paths: &BindPaths) -> Result<String> {
    if !paths.hyprland_lua.is_file() {
        bail!(
            "missing {}; omafetch bind expects an Omarchy Hyprland config",
            paths.hyprland_lua.display()
        );
    }

    fs::create_dir_all(&paths.local_bin)
        .with_context(|| format!("cannot create {}", paths.local_bin.display()))?;
    write_executable(&paths.wrapper(), WRAPPER_SOURCE)?;

    if let Some(parent) = paths.omafetch_lua.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    fs::write(&paths.omafetch_lua, HYPR_MODULE_SOURCE)
        .with_context(|| format!("cannot write {}", paths.omafetch_lua.display()))?;

    let hyprland = fs::read_to_string(&paths.hyprland_lua)
        .with_context(|| format!("cannot read {}", paths.hyprland_lua.display()))?;
    let hyprland = strip_legacy_inline(&hyprland);
    let hyprland = ensure_require(&hyprland);
    fs::write(&paths.hyprland_lua, hyprland)
        .with_context(|| format!("cannot write {}", paths.hyprland_lua.display()))?;

    Ok(format!(
        "bound stock About to omafetch\n  {}\n  {}\n  {}\nrestart the Omarchy shell if About still opens fastfetch",
        paths.wrapper().display(),
        paths.omafetch_lua.display(),
        paths.hyprland_lua.display()
    ))
}

pub fn unbind(paths: &BindPaths) -> Result<String> {
    let wrapper = paths.wrapper();
    if wrapper.exists() {
        fs::remove_file(&wrapper)
            .with_context(|| format!("cannot remove {}", wrapper.display()))?;
    }
    if paths.omafetch_lua.exists() {
        fs::remove_file(&paths.omafetch_lua)
            .with_context(|| format!("cannot remove {}", paths.omafetch_lua.display()))?;
    }
    if paths.hyprland_lua.is_file() {
        let hyprland = fs::read_to_string(&paths.hyprland_lua)
            .with_context(|| format!("cannot read {}", paths.hyprland_lua.display()))?;
        let hyprland = remove_require(&hyprland);
        fs::write(&paths.hyprland_lua, hyprland)
            .with_context(|| format!("cannot write {}", paths.hyprland_lua.display()))?;
    }

    Ok("unbound stock About from omafetch".to_string())
}

pub fn ensure_require(source: &str) -> String {
    if source.lines().any(|line| line.trim() == REQUIRE_LINE) {
        return normalize_trailing_newline(source);
    }

    let require_with_newline = format!("{REQUIRE_LINE}\n");
    let autostart = "require(\"hypr.autostart\")";
    if let Some(index) = source.find(autostart) {
        let insert_at = index + autostart.len();
        let mut updated = String::with_capacity(source.len() + require_with_newline.len() + 1);
        updated.push_str(&source[..insert_at]);
        updated.push('\n');
        updated.push_str(&require_with_newline);
        updated.push_str(&source[insert_at..]);
        return normalize_trailing_newline(&updated);
    }

    let mut updated = source.to_string();
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&require_with_newline);
    updated
}

pub fn remove_require(source: &str) -> String {
    let filtered = source
        .lines()
        .filter(|line| line.trim() != REQUIRE_LINE)
        .collect::<Vec<_>>()
        .join("\n");
    normalize_trailing_newline(&filtered)
}

pub fn strip_legacy_inline(source: &str) -> String {
    let mut output = Vec::new();
    let mut skipping_do = false;
    let mut do_depth = 0;
    let mut skip_following_do = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if skipping_do {
            if trimmed == "do" || trimmed.starts_with("do ") {
                do_depth += 1;
            }
            // Only the unindented `end` closes the legacy PATH block.
            if trimmed == "end" && !line.starts_with(char::is_whitespace) {
                do_depth -= 1;
                if do_depth == 0 {
                    skipping_do = false;
                }
            }
            continue;
        }

        if trimmed.contains("omarchy-launch-about wrapper")
            || trimmed.contains("Omarchy's env setup puts its bin first")
            || trimmed.contains("omafetch About:")
            || trimmed.contains("org.omarchy.omafetch")
            || trimmed.contains("pad and Hyprland borders")
            || trimmed.contains("Measured on this machine at")
        {
            skip_following_do = true;
            continue;
        }

        if trimmed == "do" && skip_following_do {
            skipping_do = true;
            do_depth = 1;
            skip_following_do = false;
            continue;
        }

        if !trimmed.is_empty() && !trimmed.starts_with("--") {
            skip_following_do = false;
        }

        output.push(line.to_string());
    }

    normalize_trailing_newline(&output.join("\n"))
}

fn write_executable(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("cannot write {}", path.display()))?;
    let mut permissions = fs::metadata(path)
        .with_context(|| format!("cannot stat {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("cannot chmod {}", path.display()))?;
    Ok(())
}

fn normalize_trailing_newline(source: &str) -> String {
    let mut source = source.trim_end().to_string();
    source.push('\n');
    source
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

    fn temp_paths() -> BindPaths {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("omafetch-bind-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let paths = BindPaths {
            local_bin: root.join(".local/bin"),
            hyprland_lua: root.join(".config/hypr/hyprland.lua"),
            omafetch_lua: root.join(".config/hypr/omafetch.lua"),
        };
        fs::create_dir_all(paths.hyprland_lua.parent().expect("parent")).expect("mkdir hypr");
        paths
    }

    #[test]
    fn bind_writes_wrapper_and_hypr_module() {
        let paths = temp_paths();
        fs::write(
            &paths.hyprland_lua,
            "require(\"default.hypr.omarchy\")\nrequire(\"hypr.autostart\")\n",
        )
        .unwrap();

        bind(&paths).expect("bind");

        let wrapper = fs::read_to_string(paths.wrapper()).expect("wrapper");
        assert!(wrapper.contains("omafetch about"));
        let mode = fs::metadata(paths.wrapper()).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);

        let module = fs::read_to_string(&paths.omafetch_lua).expect("module");
        assert!(module.contains("org.omarchy.omafetch"));

        let hyprland = fs::read_to_string(&paths.hyprland_lua).expect("hyprland");
        assert!(hyprland.contains(REQUIRE_LINE));
        assert!(hyprland.contains("require(\"hypr.autostart\")"));
        assert_eq!(hyprland.matches(REQUIRE_LINE).count(), 1);

        bind(&paths).expect("bind again");
        let hyprland = fs::read_to_string(&paths.hyprland_lua).expect("hyprland");
        assert_eq!(hyprland.matches(REQUIRE_LINE).count(), 1);
    }

    #[test]
    fn bind_strips_legacy_inline_rules() {
        let paths = temp_paths();
        fs::write(
            &paths.hyprland_lua,
            r#"require("hypr.autostart")

-- Prefer ~/.local/bin so a user omarchy-launch-about wrapper wins over
-- /usr/share/omarchy/bin. Omarchy's env setup puts its bin first.
do
  local home = os.getenv("HOME") or ""
  local function add(dir)
    table.insert(kept, dir)
  end
  hl.env("PATH", home)
end

-- omafetch About: sized to the default fetch
o.window("org.omarchy.omafetch", { float = true })
o.window("org.omarchy.omafetch", { center = true })
o.window("org.omarchy.omafetch", { size = { 724, 714 } })
"#,
        )
        .unwrap();

        bind(&paths).expect("bind");
        let hyprland = fs::read_to_string(&paths.hyprland_lua).expect("hyprland");
        assert!(hyprland.contains(REQUIRE_LINE));
        assert!(!hyprland.contains("org.omarchy.omafetch"));
        assert!(!hyprland.contains("hl.env"));
    }

    #[test]
    fn unbind_removes_wrapper_and_require() {
        let paths = temp_paths();
        fs::write(&paths.hyprland_lua, "require(\"hypr.autostart\")\n").unwrap();
        bind(&paths).expect("bind");
        unbind(&paths).expect("unbind");

        assert!(!paths.wrapper().exists());
        assert!(!paths.omafetch_lua.exists());
        let hyprland = fs::read_to_string(&paths.hyprland_lua).expect("hyprland");
        assert!(!hyprland.contains(REQUIRE_LINE));
        assert!(hyprland.contains("require(\"hypr.autostart\")"));
    }

    #[test]
    fn bind_fails_without_hyprland_config() {
        let paths = temp_paths();
        let err = bind(&paths).expect_err("missing hyprland.lua");
        assert!(
            err.to_string()
                .contains("expects an Omarchy Hyprland config")
        );
    }
}
