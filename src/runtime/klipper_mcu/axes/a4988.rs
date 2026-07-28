use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc};

use crate::{
    config::{A4988Config, GeneralAxisConfig},
    runtime::klipper_mcu::{KlipperMCURuntime, protocol::SendCommand},
    traits::Axis,
    utils::{error::IOError, pin::Pin},
};

pub struct KA4988 {
    step_pin: Pin<4>,
    dir_pin: Pin<4>,

    config: GeneralAxisConfig,
    oid: u8,
}

impl KA4988 {
    pub fn new(
        config: A4988Config,
        oid: u8,
        klipper: Rc<RefCell<KlipperMCURuntime>>,
    ) -> Result<Box<dyn Axis>, IOError> {
        let step_pin = Pin::from_str(&config.step_pin).ok_or(IOError::InvalidPin)?;
        let dir_pin = Pin::from_str(&config.dir_pin).ok_or(IOError::InvalidPin)?;

        let invert_step = if step_pin.invert > 0 && dir_pin.invert > 0 {
            2
        } else {
            step_pin.invert
        };

        klipper
            .borrow_mut()
            .send_command_expect_ack(SendCommand::config_stepper {
                oid,
                step_pin: step_pin.num,
                dir_pin: dir_pin.num,
                invert_step,
                step_pulse_ticks: 0,
            })?;

        Ok(Box::new(Self {
            step_pin,
            dir_pin,
            config: config.config,
            oid,
        }))
    }
}

impl Axis for KA4988 {
    fn simple_move(&mut self, velocity: crate::utils::units::Velocity)
    where
        Self: Sized,
    {
        todo!()
    }
}
