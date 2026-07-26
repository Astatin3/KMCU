use alloc::boxed::Box;

use crate::{
    Res,
    config::{Connection, KlipperMCU},
    os::{GPIO, RpmsgEndpoint, Socket, sleep},
    runtime::klipper_mcu::{KlipperMCURuntime, identify::IdentifyResults},
    traits::Stream,
    utils::pin::Pin,
};

impl KlipperMCURuntime {
    pub fn from_config(config: KlipperMCU) -> Res<Self>
    where
        Self: Sized,
    {
        #[cfg(feature = "std")]
        if let Some(command) = config.exec_start {
            match std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&command)
                .output()
            {
                Ok(o) => {
                    if o.status.success() {
                        info!("Ran command '{command}'")
                    } else {
                        warn!("Command '{command}' resulted in error {}", o.status);
                    }
                }
                Err(_) => warn!("Command '{command}' failed to run!"),
            }
        }

        let power_pin = if let Some(pin_str) = config.power_pin {
            let gpio = GPIO::new(
                Pin::from_str(&pin_str).ok_or(err!("Invalid pin string '{pin_str}'"))?,
                false,
            )?;
            gpio.set(true);
            Some(gpio)
        } else {
            None
        };

        sleep(config.start_delay);

        let stream: Box<dyn Stream> = match config.connection {
            Connection::Serial(conn) => Box::new(
                Socket::new_serial(conn)
                    .map_err(|e| err!("Failed to create socket connection: {e}"))?,
            ),
            Connection::Socket(conn) => Box::new(
                Socket::new(conn).map_err(|e| err!("Failed to create socket connection: {e}"))?,
            ),
            Connection::Rpmsg(conn) => Box::new(
                RpmsgEndpoint::from_config(conn)
                    .map_err(|e| err!("Failed to create RPMSG connection: {e}"))?,
            ),
        };

        let mut this = Self {
            stream,
            seq: 0,
            power_pin,
            identity: IdentifyResults::empty(),
        };

        // Run the identify sequence
        this.identity = this
            .identify()
            .map_err(|e| err!("Failed identification: {e}"))?;

        Ok(this)
    }
}
