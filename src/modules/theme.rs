use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Theme;

impl Module for Theme {
    fn name(&self) -> &'static str {
        "theme"
    }
    fn label(&self) -> &'static str {
        "Theme"
    }

    fn collect(&self, ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            ctx.omarchy.theme_label(),
        ))
    }
}
