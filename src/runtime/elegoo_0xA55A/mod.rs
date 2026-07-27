//! A wrapper for the serial connection
//! using the protocol for elegoo printers
//! with the magic byte 0xA55A

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

// 1. Power pulse (LOW → 200ms → HIGH)
// 2. Open serial
// 3. Send bootloader PING  (0xA5 0x5A ...)
// 4. Receive PONG
// 5. Send JUMP             (0xA5 0x5A ...)
// 6. Close serial          (or wait briefly)
// 7. Open serial again (or reuse)
// 8. Normal Klipper identify

impl Elegoo0xA55A {
    pub fn new(pin: &str, connection: Connection) -> Result<Self, IOError> {
        debug!("Running Elegoo 0xA55A sequence");

        let pin = Pin::from_str(pin).ok_or(IOError::InvalidPin)?;

        let gpio = GPIO::new(pin, false)?;

        gpio.set(false);
        sleep(LongTime::new::<long_millisecond>(200));
        gpio.set(true);

        // Wait for the MCU to boot
        sleep(LongTime::new::<long_millisecond>(50));

        let mut stream = init_connection(connection)?;

        const C: u32 = 0x12345678;

        // Try a few times to ping the MCU
        let sucsess = for _ in 0..5 {
            BootloaderCmd::Ping(C).encode(&mut *stream, &())?;
            match BootloaderCmd::decode(&mut *stream, &()) {
                Ok(BootloaderCmd::Pong(v)) if v == C => break, // got the packet
                Err(IOError::Timeout) => continue,             // try again
                _ => return Err(IOError::UnexpectedCommand),
            }
        };

        // Jump to OS
        BootloaderCmd::Jump.encode(&mut *stream, &())?;

        // Wait for the thing to load again
        sleep(LongTime::new::<long_millisecond>(200));

        // Flush the stream because it probably contains nonsense
        stream.flush_input()?;

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
