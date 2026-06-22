use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct LocalIp;

impl Module for LocalIp {
    fn name(&self) -> &'static str {
        "localip"
    }

    fn label(&self) -> &'static str {
        "IP"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::network::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
