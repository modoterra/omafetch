use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Uptime;

impl Module for Uptime {
    fn name(&self) -> &'static str {
        "uptime"
    }
    fn label(&self) -> &'static str {
        "Uptime"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::procfs::uptime()
            .map(|uptime| crate::render::text::format_duration(uptime.as_secs()))
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
