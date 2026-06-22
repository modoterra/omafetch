use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Swap;

impl Module for Swap {
    fn name(&self) -> &'static str {
        "swap"
    }

    fn label(&self) -> &'static str {
        "Swap"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::procfs::meminfo()
            .map(|mem| {
                let used = mem.swap_total_kib.saturating_sub(mem.swap_free_kib);
                format!(
                    "{} / {}",
                    crate::render::text::format_gib(used),
                    crate::render::text::format_gib(mem.swap_total_kib)
                )
            })
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
