use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Cpu;

impl Module for Cpu {
    fn name(&self) -> &'static str {
        "cpu"
    }
    fn label(&self) -> &'static str {
        "CPU"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::procfs::cpu_model().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
