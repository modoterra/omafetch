use anyhow::{Context, Result};
use std::io::Write;
use std::mem::MaybeUninit;

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
        Some(Command::Public) | Some(Command::About) | None => {
            let state = OmarchyState::discover();
            let ctx = ModuleContext { omarchy: &state };
            let is_public = matches!(&cli.command, Some(Command::Public));
            let is_about = matches!(&cli.command, Some(Command::About));
            let is_default_output = cli.modules.is_empty() || is_public || is_about;
            let modules = if is_public || is_about {
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

            if is_about {
                wait_for_key()?;
            }
        }
    }

    Ok(())
}

fn write_stdout(output: &str) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes())?;
    handle.flush()?;
    Ok(())
}

fn stdin_is_tty() -> bool {
    // SAFETY: isatty only inspects the stdin file descriptor.
    unsafe { libc::isatty(libc::STDIN_FILENO) == 1 }
}

fn wait_for_key() -> Result<()> {
    if !stdin_is_tty() {
        return Ok(());
    }

    let fd = libc::STDIN_FILENO;
    let mut original = MaybeUninit::<libc::termios>::uninit();
    // SAFETY: tcgetattr writes a termios into the provided valid pointer.
    let result = unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) };
    if result != 0 {
        anyhow::bail!("could not read terminal attributes");
    }
    // SAFETY: tcgetattr returned success, so the structure is initialized.
    let original = unsafe { original.assume_init() };

    let mut raw = original;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;

    // SAFETY: raw is a fully initialized termios derived from the current settings.
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
        anyhow::bail!("could not set terminal to raw mode");
    }

    let _restore = TerminalRestore { fd, original };

    let mut byte = 0u8;
    // SAFETY: read writes at most one byte into the local buffer.
    let read_result = unsafe { libc::read(fd, (&raw mut byte).cast(), 1) };
    if read_result < 0 {
        return Err(std::io::Error::last_os_error()).context("could not read key");
    }

    Ok(())
}

struct TerminalRestore {
    fd: libc::c_int,
    original: libc::termios,
}

impl Drop for TerminalRestore {
    fn drop(&mut self) {
        // SAFETY: original came from a successful tcgetattr on this fd.
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSANOW, &self.original);
        }
    }
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
