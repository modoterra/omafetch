use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct OmarchySource;

impl Module for OmarchySource {
    fn name(&self) -> &'static str {
        "omarchy-source"
    }

    fn label(&self) -> &'static str {
        "Source"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::omarchy::paths::OmarchyPaths::discover()
            .and_then(|paths| {
                let path = paths.install_dir.to_string_lossy().to_string();
                let branch = crate::probe::command::run_capture(
                    "git",
                    &["-C", &path, "rev-parse", "--abbrev-ref", "HEAD"],
                );
                let commit = crate::probe::command::run_capture(
                    "git",
                    &["-C", &path, "rev-parse", "--short", "HEAD"],
                );

                match (branch, commit) {
                    (Some(branch), Some(commit)) => Some(format!("{branch} ({commit})")),
                    (Some(branch), None) => Some(branch),
                    _ => None,
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
