use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Memory;

impl Module for Memory {
    fn name(&self) -> &'static str {
        "memory"
    }
    fn label(&self) -> &'static str {
        "Memory"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::procfs::meminfo()
            .map(|mem| {
                let used_kib = mem.total_kib.saturating_sub(mem.available_kib);
                format!(
                    "{} / {}",
                    crate::render::text::format_gib(used_kib),
                    crate::render::text::format_gib(mem.total_kib)
                )
            })
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
