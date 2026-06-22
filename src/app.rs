use anyhow::Result;
use std::io::Write;

use crate::cli::{Cli, Command};
use crate::modules::registry::ModuleRegistry;
use crate::modules::types::{ModuleContext, ModuleOutput};
use crate::omarchy::state::OmarchyState;

pub fn run() -> Result<()> {
    let cli = Cli::parse(std::env::args().skip(1));
    let registry = ModuleRegistry::new();

    match &cli.command {
        Some(Command::List) => {
            let mut output = registry.names().join("\n");
            output.push('\n');
            write_stdout(&output)?;
        }
        Some(Command::Public) | None => {
            let state = OmarchyState::discover();
            let ctx = ModuleContext { omarchy: &state };
            let is_public = matches!(&cli.command, Some(Command::Public));
            let is_default_output = cli.modules.is_empty() || is_public;
            let modules = if is_public {
                registry.resolve_or_defaults(&[])?
            } else {
                registry.resolve_or_defaults(&cli.modules)?
            };
            let mut outputs = collect_outputs(&ctx, modules, is_default_output);

            if is_public {
                outputs = public_outputs(outputs);
            }

            let document =
                crate::render::document::RenderDocument::from_outputs(&outputs, is_default_output);
            let output = crate::render::layout::render_document(&state, &document);
            write_stdout(&output)?;
        }
    }

    Ok(())
}

fn write_stdout(output: &str) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes())?;
    Ok(())
}

fn collect_outputs(
    ctx: &ModuleContext<'_>,
    modules: Vec<&dyn crate::modules::types::Module>,
    is_default_output: bool,
) -> Vec<ModuleOutput> {
    std::thread::scope(|scope| {
        modules
            .into_iter()
            .map(|module| {
                scope.spawn(move || {
                    module
                        .collect(ctx)
                        .unwrap_or_else(|| ModuleOutput::unknown(module.name(), module.label()))
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("module collection panicked"))
            .filter(|output| {
                !(is_default_output && output.name == "battery" && output.value == "unknown")
            })
            .filter(|output| {
                !(is_default_output && output.name == "gtt-memory" && output.value == "unknown")
            })
            .collect()
    })
}

fn public_outputs(outputs: Vec<ModuleOutput>) -> Vec<ModuleOutput> {
    outputs
        .into_iter()
        .filter_map(|mut output| {
            match output.name {
                "localip" => return None,
                "omarchy-source" => output.value = public_source(&output.value),
                "rootfs" => output.value = public_rootfs(&output.value),
                "disk" => output.value = public_disk(&output.value),
                _ => {}
            }

            Some(output)
        })
        .collect()
}

fn public_source(value: &str) -> String {
    value
        .split_once(" (")
        .map(|(branch, _)| branch.to_string())
        .unwrap_or_else(|| value.to_string())
}

fn public_rootfs(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|part| *part != "@" && !part.starts_with("zstd") && !part.starts_with("compress="))
        .collect::<Vec<_>>()
        .join(" ")
}

fn public_disk(value: &str) -> String {
    value
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                return line.to_string();
            }

            let Some((_mount, usage)) = line.split_once("  ") else {
                return line.to_string();
            };
            format!("disk  {usage}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
