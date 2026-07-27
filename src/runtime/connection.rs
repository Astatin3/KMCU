use alloc::{boxed::Box, string::String};

use crate::{
    config::{Connection, ConnectionWrapper},
    os::{GPIO, RpmsgEndpoint, Socket, sleep},
    runtime::elegoo_0xA55A::Elegoo0xA55A,
    traits::{MCU, Read, Stream, Write},
    utils::{
        error::{IOError, MCUError},
        pin::Pin,
    },
};

pub fn init_connection_with_wrapper(
    connection: Connection,
    wrapper: ConnectionWrapper,
) -> Result<Box<dyn Stream>, MCUError> {
    match wrapper {
        ConnectionWrapper::None => {
            init_connection(connection).map_err(|e| MCUError::KlipperConnection(e))
        }
        ConnectionWrapper::SetPin { pin } => {
            let stream = init_connection(connection).map_err(|e| MCUError::KlipperConnection(e))?;

            let stream =
                ConnectionWithPin::new(&pin, stream).map_err(|e| MCUError::Pin(e, pin.into()))?;

            stream.set_pin(true);

            Ok(Box::new(stream) as Box<dyn Stream>)
        }
        ConnectionWrapper::Pulse { pin, pulse } => {
            let stream = init_connection(connection).map_err(|e| MCUError::KlipperConnection(e))?;

            let stream =
                ConnectionWithPin::new(&pin, stream).map_err(|e| MCUError::Pin(e, pin.into()))?;

            stream.set_pin(false);
            sleep(pulse);
            stream.set_pin(true);

            Ok(Box::new(stream) as Box<dyn Stream>)
        }
        ConnectionWrapper::ElegooA55A { pin } => {
            let stream =
                Elegoo0xA55A::new(&pin, connection).map_err(|e| MCUError::Elegoo0xA55A(e))?;

            Ok(Box::new(stream) as Box<dyn Stream>)
        }
    }
}

pub fn init_connection(connection: Connection) -> Result<Box<dyn Stream>, IOError> {
    Ok(match connection {
        Connection::Serial(conn) => Box::new(Socket::new_serial(conn)?) as Box<dyn Stream>,
        Connection::Socket(conn) => Box::new(Socket::new(conn)?) as Box<dyn Stream>,
        Connection::Rpmsg(conn) => Box::new(RpmsgEndpoint::from_config(conn)?) as Box<dyn Stream>,
    })
}

struct ConnectionWithPin {
    gpio: GPIO,
    stream: Box<dyn Stream>,
}

impl ConnectionWithPin {
    pub fn new(pin: &str, stream: Box<dyn Stream>) -> Result<Self, IOError> {
        let pin = Pin::from_str(pin).ok_or(IOError::InvalidPin)?;

        let gpio = GPIO::new(pin, false)?;

        Ok(Self { gpio, stream })
    }

    pub fn set_pin(&self, value: bool) -> Result<(), IOError> {
        self.gpio.set(value)?;
        Ok(())
    }
}

impl Read for ConnectionWithPin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        self.stream.read(buf)
    }

    fn flush_input(&mut self) -> Result<(), IOError> {
        self.stream.flush_input()
    }
}

impl Write for ConnectionWithPin {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        self.stream.write(buf)
    }

    fn flush_output(&mut self) -> Result<(), IOError> {
        self.stream.flush_output()
    }
}

impl Stream for ConnectionWithPin {}
