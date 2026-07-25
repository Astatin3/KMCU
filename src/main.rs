use KMCU::{FromConfig, PrinterConfig, PrinterRuntime, Res};
use log::info;

fn main() {
    #[cfg(feature = "log")]
    pretty_env_logger::init();

    // Catch errors
    if let Err(e) = run() {
        log::error!("{e:?}");
    }
}

fn run() -> Res<()> {
    info!("Starting printer...");

    let config = PrinterConfig::parse(include_str!("../kmcu.toml"))?;

    let printer = PrinterRuntime::from_config(config)?;

    printer.alive()?;
    info!("Printer is alive!");

    Ok(())
}
