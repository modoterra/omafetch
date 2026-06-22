use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Terminal;

impl Module for Terminal {
    fn name(&self) -> &'static str {
        "terminal"
    }
    fn label(&self) -> &'static str {
        "Term"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::env::terminal_with_version().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
