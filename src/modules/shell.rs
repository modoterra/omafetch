use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Shell;

impl Module for Shell {
    fn name(&self) -> &'static str {
        "shell"
    }
    fn label(&self) -> &'static str {
        "Shell"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::env::shell_with_version().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
