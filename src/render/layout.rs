use std::fmt::Write;

use owo_colors::OwoColorize;
use owo_colors::Style;
use unicode_width::UnicodeWidthStr;

use crate::omarchy::state::OmarchyState;
use crate::render::document::{RenderDocument, RenderRow};

pub fn render_document(state: &OmarchyState, document: &RenderDocument<'_>) -> String {
    let logo = crate::render::logo::logo_for(state);
    let logo_lines = logo.lines().collect::<Vec<_>>();
    let logo_width = logo_lines
        .iter()
        .map(|line| UnicodeWidthStr::width(*line))
        .max()
        .unwrap_or(0);
    let label_width = document.label_width();
    let logo_right_padding = 2;
    let terminal_width = terminal_width();
    let value_width = terminal_width
        .saturating_sub(logo_width)
        .saturating_sub(logo_right_padding)
        .saturating_sub(label_width)
        .saturating_sub(2)
        .max(12);
    let logo_top_offset = 1;
    let logo_end = logo_lines.len() + logo_top_offset;
    let height = logo_end.max(document.len());

    let mut rendered = String::with_capacity(height.saturating_mul(96));
    rendered.push('\n');

    for index in 0..height {
        let logo_line = index
            .checked_sub(logo_top_offset)
            .and_then(|index| logo_lines.get(index))
            .copied()
            .unwrap_or("");
        let logo_padding = logo_width.saturating_sub(UnicodeWidthStr::width(logo_line));

        if logo_line.is_empty() && matches!(document.rows.get(index), Some(RenderRow::Gap) | None) {
            rendered.push('\n');
            continue;
        } else if logo_line.is_empty() {
            write!(
                rendered,
                "{}{}",
                sparkle_line(document, index, logo_width, logo_end, height),
                " ".repeat(logo_right_padding)
            )
            .expect("writing to a String cannot fail");
        } else {
            write!(
                rendered,
                "{}{}{}",
                logo_line.style(logo_style(index)),
                " ".repeat(logo_padding),
                " ".repeat(logo_right_padding)
            )
            .expect("writing to a String cannot fail");
        }

        match document.rows.get(index) {
            Some(RenderRow::Output {
                output,
                value,
                show_label,
            }) => {
                let label = if *show_label {
                    format!("{:label_width$}", output.label)
                } else {
                    " ".repeat(label_width)
                };
                let value = crate::render::text::truncate_value(output.name, value, value_width);
                writeln!(
                    rendered,
                    "{}  {}",
                    label.dimmed(),
                    highlight_value(output.name, &value)
                )
                .expect("writing to a String cannot fail");
            }
            Some(RenderRow::Gap) | None => rendered.push('\n'),
        }
    }

    rendered
}

fn sparkle_line(
    document: &RenderDocument<'_>,
    index: usize,
    logo_width: usize,
    logo_end: usize,
    height: usize,
) -> String {
    if index <= logo_end || index + 1 >= height {
        return " ".repeat(logo_width);
    }

    match document.rows.get(index) {
        Some(RenderRow::Output { output, value, .. }) => {
            crate::render::sparkle::sparkle_for_row(output.name, &output.label, value, logo_width)
        }
        Some(RenderRow::Gap) | None => " ".repeat(logo_width),
    }
}

fn terminal_width() -> usize {
    ioctl_terminal_width().or_else(columns_width).unwrap_or(100)
}

fn columns_width() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|width| *width > 0)
}

fn ioctl_terminal_width() -> Option<usize> {
    let mut size = std::mem::MaybeUninit::<libc::winsize>::uninit();

    // SAFETY: ioctl writes a winsize to the provided valid pointer and does not retain it.
    let result = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, size.as_mut_ptr()) };
    if result != 0 {
        return None;
    }

    // SAFETY: ioctl returned success, so the structure has been initialized.
    let size = unsafe { size.assume_init() };
    (size.ws_col > 0).then_some(size.ws_col as usize)
}

fn logo_style(index: usize) -> Style {
    match index % 6 {
        0 => Style::new().cyan(),
        1 => Style::new().blue(),
        2 => Style::new().magenta(),
        3 => Style::new().red(),
        4 => Style::new().yellow(),
        _ => Style::new().green(),
    }
}

fn highlight_value(name: &str, value: &str) -> String {
    if value == "unknown" {
        return value.dimmed().to_string();
    }

    match name {
        "omarchy" | "omarchy-source" | "omarchy-channel" | "theme" => value.cyan().to_string(),
        "kernel-config" => highlight_key_value(value),
        "wm" | "terminal" | "shell" => value.blue().to_string(),
        "gpu" | "gtt-memory" | "display" => value.magenta().to_string(),
        "memory" | "swap" => highlight_usage(value),
        "disk" => highlight_disk(value),
        "packages" => highlight_packages(value),
        "rootfs" => highlight_root(value),
        "localip" => value.cyan().to_string(),
        _ => value.to_string(),
    }
}

fn highlight_key_value(value: &str) -> String {
    let Some((key, rest)) = value.split_once(' ') else {
        return value.green().to_string();
    };

    format!("{} {}", key.cyan(), rest.green())
}

fn highlight_usage(value: &str) -> String {
    let Some((used, total)) = value.split_once(" / ") else {
        return value.to_string();
    };

    format!("{} / {}", used.yellow(), total.dimmed())
}

fn highlight_disk(value: &str) -> String {
    let Some((mount, usage)) = value.split_once("  ") else {
        return value.to_string();
    };

    format!("{}  {}", mount.cyan(), highlight_usage(usage))
}

fn highlight_packages(value: &str) -> String {
    value
        .replace("Pacman", &"Pacman".cyan().to_string())
        .replace("AUR", &"AUR".magenta().to_string())
}

fn highlight_root(value: &str) -> String {
    value
        .split_whitespace()
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                part.cyan().to_string()
            } else if part == "encrypted" {
                part.green().to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
