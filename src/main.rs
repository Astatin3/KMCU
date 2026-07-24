use KMCU::{FromConfig, PrinterConfig, PrinterRuntime};
use log::info;

fn main() {
    pretty_env_logger::init();

    // Catch errors
    if let Err(e) = run() {
        log::error!("{}", e);
    }
}

fn run() -> anyhow::Result<()> {
    info!("Starting printer...");

    let config = PrinterConfig::parse(include_str!("../kmcu.toml"))?;

    let printer = PrinterRuntime::from_config(config)?;

    printer.alive()?;
    info!("Printer is alive!");

    Ok(())
}
