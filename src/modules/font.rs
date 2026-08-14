use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Font;

impl Module for Font {
    fn name(&self) -> &'static str {
        "font"
    }

    fn label(&self) -> &'static str {
        "Font"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::fontconfig::monospace_family().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
