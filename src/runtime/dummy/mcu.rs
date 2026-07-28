use core::cell::RefCell;

use alloc::rc::Rc;

use crate::{
    config::SimMCUConfig,
    traits::MCU,
    utils::error::{IOError, MCUError, RuntimeError},
};

pub struct SimMCURuntime {
    // axes: HashMap<String, Box<dyn Axis>>,
}

impl MCU for SimMCURuntime {
    fn alive(&mut self) -> Result<(), IOError> {
        Ok(())
    }
}

impl SimMCURuntime {
    pub fn from_config(_config: SimMCUConfig) -> Result<Rc<RefCell<dyn MCU>>, MCUError>
    where
        Self: Sized,
    {
        Ok(Rc::new(RefCell::new(SimMCURuntime {})))
    }
}
