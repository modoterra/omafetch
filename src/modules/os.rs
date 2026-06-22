use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Os;

impl Module for Os {
    fn name(&self) -> &'static str {
        "os"
    }
    fn label(&self) -> &'static str {
        "OS"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::procfs::os_name().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
