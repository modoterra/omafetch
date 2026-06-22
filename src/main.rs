mod app;
mod cli;
mod modules;
mod omarchy;
mod probe;
mod render;

fn main() {
    if let Err(err) = app::run() {
        eprintln!("omafetch: {err}");
        std::process::exit(1);
    }
}
