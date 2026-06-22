use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct GttMemory;

impl Module for GttMemory {
    fn name(&self) -> &'static str {
        "gtt-memory"
    }

    fn label(&self) -> &'static str {
        "GTT"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::sysfs::gtt_memory_bytes()
            .map(crate::render::text::format_bytes)
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
