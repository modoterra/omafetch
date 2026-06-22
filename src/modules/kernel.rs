use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Kernel;

impl Module for Kernel {
    fn name(&self) -> &'static str {
        "kernel"
    }
    fn label(&self) -> &'static str {
        "Kernel"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::procfs::kernel().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
