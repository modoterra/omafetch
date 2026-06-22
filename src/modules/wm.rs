use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Wm;

impl Module for Wm {
    fn name(&self) -> &'static str {
        "wm"
    }
    fn label(&self) -> &'static str {
        "WM"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::env::wm_with_version().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
