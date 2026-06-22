use anyhow::{Result, bail};

use crate::modules::types::Module;

pub struct ModuleRegistry {
    modules: Vec<ModuleEntry>,
}

struct ModuleEntry {
    key: ModuleKey,
    module: Box<dyn Module>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKey {
    Omarchy,
    OmarchySource,
    OmarchyChannel,
    OmarchyUpdated,
    Theme,
    Host,
    Os,
    OsAge,
    Kernel,
    KernelConfig,
    Wm,
    Terminal,
    Shell,
    Display,
    Cpu,
    Gpu,
    GttMemory,
    Memory,
    Swap,
    Disk,
    RootFs,
    Battery,
    LocalIp,
    Packages,
    Uptime,
}

impl ModuleKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Omarchy => "omarchy",
            Self::OmarchySource => "omarchy-source",
            Self::OmarchyChannel => "omarchy-channel",
            Self::OmarchyUpdated => "omarchy-updated",
            Self::Theme => "theme",
            Self::Host => "host",
            Self::Os => "os",
            Self::OsAge => "os-age",
            Self::Kernel => "kernel",
            Self::KernelConfig => "kernel-config",
            Self::Wm => "wm",
            Self::Terminal => "terminal",
            Self::Shell => "shell",
            Self::Display => "display",
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
            Self::GttMemory => "gtt-memory",
            Self::Memory => "memory",
            Self::Swap => "swap",
            Self::Disk => "disk",
            Self::RootFs => "rootfs",
            Self::Battery => "battery",
            Self::LocalIp => "localip",
            Self::Packages => "packages",
            Self::Uptime => "uptime",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "omarchy" => Self::Omarchy,
            "omarchy-source" => Self::OmarchySource,
            "omarchy-channel" => Self::OmarchyChannel,
            "omarchy-updated" => Self::OmarchyUpdated,
            "theme" => Self::Theme,
            "host" => Self::Host,
            "os" => Self::Os,
            "os-age" => Self::OsAge,
            "kernel" => Self::Kernel,
            "kernel-config" => Self::KernelConfig,
            "wm" => Self::Wm,
            "terminal" => Self::Terminal,
            "shell" => Self::Shell,
            "display" => Self::Display,
            "cpu" => Self::Cpu,
            "gpu" => Self::Gpu,
            "gtt-memory" => Self::GttMemory,
            "memory" => Self::Memory,
            "swap" => Self::Swap,
            "disk" => Self::Disk,
            "rootfs" => Self::RootFs,
            "battery" => Self::Battery,
            "localip" => Self::LocalIp,
            "packages" => Self::Packages,
            "uptime" => Self::Uptime,
            _ => return None,
        })
    }
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: vec![
                entry(ModuleKey::Omarchy, crate::modules::omarchy::Omarchy),
                entry(
                    ModuleKey::OmarchySource,
                    crate::modules::omarchy_source::OmarchySource,
                ),
                entry(
                    ModuleKey::OmarchyChannel,
                    crate::modules::omarchy_channel::OmarchyChannel,
                ),
                entry(
                    ModuleKey::OmarchyUpdated,
                    crate::modules::omarchy_updated::OmarchyUpdated,
                ),
                entry(ModuleKey::Theme, crate::modules::theme::Theme),
                entry(ModuleKey::Host, crate::modules::host::Host),
                entry(ModuleKey::Os, crate::modules::os::Os),
                entry(ModuleKey::OsAge, crate::modules::os_age::OsAge),
                entry(ModuleKey::Kernel, crate::modules::kernel::Kernel),
                entry(
                    ModuleKey::KernelConfig,
                    crate::modules::kernel_config::KernelConfig,
                ),
                entry(ModuleKey::Wm, crate::modules::wm::Wm),
                entry(ModuleKey::Terminal, crate::modules::terminal::Terminal),
                entry(ModuleKey::Shell, crate::modules::shell::Shell),
                entry(ModuleKey::Display, crate::modules::display::Display),
                entry(ModuleKey::Cpu, crate::modules::cpu::Cpu),
                entry(ModuleKey::Gpu, crate::modules::gpu::Gpu),
                entry(ModuleKey::GttMemory, crate::modules::gtt_memory::GttMemory),
                entry(ModuleKey::Memory, crate::modules::memory::Memory),
                entry(ModuleKey::Swap, crate::modules::swap::Swap),
                entry(ModuleKey::Disk, crate::modules::disk::Disk),
                entry(ModuleKey::RootFs, crate::modules::rootfs::RootFs),
                entry(ModuleKey::Battery, crate::modules::battery::Battery),
                entry(ModuleKey::LocalIp, crate::modules::localip::LocalIp),
                entry(ModuleKey::Packages, crate::modules::packages::Packages),
                entry(ModuleKey::Uptime, crate::modules::uptime::Uptime),
            ],
        }
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .map(|entry| entry.key.as_str())
            .collect()
    }

    pub fn defaults(&self) -> Vec<ModuleKey> {
        vec![
            ModuleKey::Omarchy,
            ModuleKey::OmarchySource,
            ModuleKey::OmarchyChannel,
            ModuleKey::Theme,
            ModuleKey::Host,
            ModuleKey::Os,
            ModuleKey::OsAge,
            ModuleKey::Kernel,
            ModuleKey::KernelConfig,
            ModuleKey::Wm,
            ModuleKey::Terminal,
            ModuleKey::Shell,
            ModuleKey::Display,
            ModuleKey::Cpu,
            ModuleKey::Gpu,
            ModuleKey::GttMemory,
            ModuleKey::Memory,
            ModuleKey::Swap,
            ModuleKey::Disk,
            ModuleKey::RootFs,
            ModuleKey::Packages,
            ModuleKey::OmarchyUpdated,
            ModuleKey::Uptime,
            ModuleKey::LocalIp,
            ModuleKey::Battery,
        ]
    }

    pub fn compact_defaults(&self) -> Vec<ModuleKey> {
        vec![
            ModuleKey::Omarchy,
            ModuleKey::Theme,
            ModuleKey::Host,
            ModuleKey::Os,
            ModuleKey::Kernel,
            ModuleKey::Wm,
        ]
    }

    pub fn resolve_or_defaults<'a>(&'a self, names: &[String]) -> Result<Vec<&'a dyn Module>> {
        if names.is_empty() {
            return self.resolve_static(&self.defaults());
        }

        let mut selected = self.compact_defaults();
        for name in names {
            let Some(key) = ModuleKey::parse(name) else {
                bail!(
                    "unknown module: {name}\navailable modules: {}",
                    self.names().join(", ")
                );
            };

            if !selected.contains(&key) {
                selected.push(key);
            }
        }

        self.resolve_static(&selected)
    }

    fn resolve_static<'a>(&'a self, keys: &[ModuleKey]) -> Result<Vec<&'a dyn Module>> {
        let mut resolved = Vec::new();

        for key in keys {
            let Some(entry) = self.modules.iter().find(|entry| entry.key == *key) else {
                bail!(
                    "unknown module: {}\navailable modules: {}",
                    key.as_str(),
                    self.names().join(", ")
                );
            };

            resolved.push(entry.module.as_ref());
        }

        Ok(resolved)
    }
}

fn entry(key: ModuleKey, module: impl Module + 'static) -> ModuleEntry {
    ModuleEntry {
        key,
        module: Box::new(module),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_order_is_curated() {
        assert_eq!(
            ModuleRegistry::new().defaults(),
            vec![
                ModuleKey::Omarchy,
                ModuleKey::OmarchySource,
                ModuleKey::OmarchyChannel,
                ModuleKey::Theme,
                ModuleKey::Host,
                ModuleKey::Os,
                ModuleKey::OsAge,
                ModuleKey::Kernel,
                ModuleKey::KernelConfig,
                ModuleKey::Wm,
                ModuleKey::Terminal,
                ModuleKey::Shell,
                ModuleKey::Display,
                ModuleKey::Cpu,
                ModuleKey::Gpu,
                ModuleKey::GttMemory,
                ModuleKey::Memory,
                ModuleKey::Swap,
                ModuleKey::Disk,
                ModuleKey::RootFs,
                ModuleKey::Packages,
                ModuleKey::OmarchyUpdated,
                ModuleKey::Uptime,
                ModuleKey::LocalIp,
                ModuleKey::Battery,
            ]
        );
    }

    #[test]
    fn explicit_modules_keep_compact_identity_context() {
        let registry = ModuleRegistry::new();
        let modules = registry
            .resolve_or_defaults(&["cpu".to_string(), "memory".to_string(), "theme".to_string()])
            .expect("modules should resolve");
        let names = modules
            .iter()
            .map(|module| module.name())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "omarchy", "theme", "host", "os", "kernel", "wm", "cpu", "memory"
            ]
        );
    }

    #[test]
    fn unknown_module_returns_clear_error() {
        let err = match ModuleRegistry::new().resolve_or_defaults(&["nope".to_string()]) {
            Ok(_) => panic!("expected unknown module error"),
            Err(err) => err.to_string(),
        };

        assert!(err.contains("unknown module: nope"));
        assert!(err.contains("available modules:"));
    }
}
