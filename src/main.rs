use KMCU::{MajorStateError, PrinterConfig, PrinterRuntime};
use log::info;

fn main() {
    #[cfg(feature = "log")]
    pretty_env_logger::init();

    // Catch errors
    if let Err(e) = run() {
        log::error!("{e}");
    }
}

fn run() -> Result<(), MajorStateError> {
    info!("Starting printer...");

    let config = PrinterConfig::parse(include_str!("../kmcu.toml"))?;

    let mut printer = PrinterRuntime::from_config(config)?;

    printer.alive()?;
    info!("Printer is alive!");

    printer.kinematics.test_x();

    Ok(())
}
