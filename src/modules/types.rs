use crate::omarchy::state::OmarchyState;

pub struct ModuleContext<'a> {
    pub omarchy: &'a OmarchyState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleOutput {
    pub name: &'static str,
    pub label: String,
    pub value: String,
}

impl ModuleOutput {
    pub fn new(name: &'static str, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name,
            label: label.into(),
            value: value.into(),
        }
    }

    pub fn unknown(name: &'static str, label: &'static str) -> Self {
        Self::new(name, label, "unknown")
    }
}

pub trait Module: Sync {
    fn name(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn collect(&self, ctx: &ModuleContext<'_>) -> Option<ModuleOutput>;
}
