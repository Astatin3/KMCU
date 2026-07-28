//! A wrapper for the serial connection
//! using the protocol for elegoo printers
//! with the magic byte 0xA55A
//!
//! On the Elegoo CC2, the toolhead will sometimes
//! just not work if it gets stuck in a bad state.
//! This code is guaranteed to fix it

mod command;

use alloc::boxed::Box;

use crate::{
    config::Connection,
    os::{GPIO, sleep},
    runtime::{connection::init_connection, elegoo_0xA55A::command::BootloaderCmd},
    traits::{Binary, Read, Stream, Write},
    utils::{
        error::{IOError, MCUError},
        pin::Pin,
        units::{long_millisecond, long_time::LongTime},
    },
};

pub struct Elegoo0xA55A {
    gpio: GPIO,
    stream: Box<dyn Stream>,
}

// This is from github.com/elegooofficial/CentauriCarbon2/blob/main/elegoo/mcu.cpp

impl Elegoo0xA55A {
    pub fn new(pin: &str, connection: Connection) -> Result<Self, IOError> {
        debug!("Running Elegoo 0xA55A sequence");

        let pin = Pin::from_str(pin).ok_or(IOError::InvalidPin)?;

        let gpio = GPIO::new(pin, false)?;

        // Set the MCU to bootloader mode by toggling it for 200ms
        gpio.set(false);
        sleep(LongTime::new::<long_millisecond>(200));
        gpio.set(true);

        // Wait for the MCU to boot
        sleep(LongTime::new::<long_millisecond>(50));

        // Start the stream
        let mut stream = init_connection(connection)?;

        const N: usize = 5; // times to retry
        const C: u32 = 0x12345678; // Some data to check for

        // Try a few times to ping the MCU
        for i in 0..N {
            BootloaderCmd::Ping(C).encode(&mut *stream, &())?;
            match BootloaderCmd::decode(&mut *stream, &()) {
                Ok(BootloaderCmd::Pong(C)) => break, // got the packet
                Err(IOError::Timeout) if i == N - 1 => return Err(IOError::Timeout),
                Err(IOError::Timeout) => continue, // try again
                _ => return Err(IOError::UnexpectedCommand),
            }
        }

        // Jump to OS
        BootloaderCmd::Jump.encode(&mut *stream, &())?;

        // Wait for the thing to load again
        sleep(LongTime::new::<long_millisecond>(200));

        // Flush the stream because it probably contains nonsense
        stream.flush_input()?;

        // Everything should be good now
        Ok(Self { gpio, stream })
    }
}

impl Read for Elegoo0xA55A {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        self.stream.read(buf)
    }

    fn flush_input(&mut self) -> Result<(), IOError> {
        self.stream.flush_input()
    }
}

impl Write for Elegoo0xA55A {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        self.stream.write(buf)
    }

    fn flush_output(&mut self) -> Result<(), IOError> {
        self.stream.flush_output()
    }
}

impl Stream for Elegoo0xA55A {}
