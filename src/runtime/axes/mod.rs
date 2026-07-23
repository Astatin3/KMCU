use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{config::axis::DummyAxisConfig, traits::axis::Axis};

pub struct DummyAxis {
    config: DummyAxisConfig,
    position: f32,
}

impl DummyAxis {
    pub fn new(config: DummyAxisConfig) -> Box<dyn Axis> {
        Box::new(Self {
            config,
            position: 0.,
        })
    }
}

impl Axis for DummyAxis {
    fn step(&mut self, count: i32, _interval: Duration) {
        self.position += (count as f32) * self.config.step_amount_mm;
    }
}

// impl FromConfig for DummyAxis {
//     type ConfigType = DummyAxisConfig;

//     fn from_config(config: DummyAxisConfig) -> anyhow::Result<Self>
//     where
//         Self: Sized,
//     {
//         Ok(Self {
//             position: 0.,
//             config,
//         })
//     }
// }
