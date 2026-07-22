use std::{cell::RefCell, rc::Rc};

use crate::{config::axis::AxisConfig, traits::axis::Axis};

pub trait MCU {
    // type InitialConfig;

    // /// Creates a new self based off of a certain config type
    // fn new(printer_config: &PrinterConfig, config: &Self::InitialConfig) -> anyhow::Result<Self>
    // where
    //     Self: Sized;

    /// Returns an axis from a name
    fn new_axis(
        this: Rc<RefCell<dyn MCU>>,
        axis_config: AxisConfig,
    ) -> anyhow::Result<Box<dyn Axis>>
    where
        Self: Sized;
}
