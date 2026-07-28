use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc};

use crate::{
    config::{A4988Config, AxisConfig, Connection, ConnectionWrapper, KlipperMCU},
    os::{GPIO, RpmsgEndpoint, Socket, sleep},
    runtime::{
        device_map::DeviceMap,
        dummy::DummyAxis,
        klipper_mcu::{
            KlipperMCURuntime,
            axes::{KA4988, KTMC2209},
            identify::IdentifyResults,
            protocol::SendCommand,
        },
    },
    traits::{Axis, MCU, Stream},
    utils::{
        error::{IOError, MCUError},
        pin::Pin,
    },
};

impl KlipperMCURuntime {
    pub fn from_config(
        stream: Box<dyn Stream>,
        config: KlipperMCU,
        device_map: &mut DeviceMap,
    ) -> Result<Rc<RefCell<dyn MCU>>, MCUError>
    where
        Self: Sized,
    {
        let mut this = Self {
            stream,
            seq: 0,
            identity: IdentifyResults::default(),
        };

        // Run the identify sequence
        this.identity = this.identify().map_err(MCUError::KlipperConnection)?;

        // Count the OIDs and allocate them
        this.send_command_expect_ack(SendCommand::allocate_oids {
            count: Self::count_oids(&config),
        })
        .map_err(|e| MCUError::KlipperInitOIDs(e))?;

        // Convert this into an Rc<RefCell<>> for easy access
        let this = Rc::new(RefCell::new(this));
        let mut current_oid_count = 0;

        // Have each device write their configs
        for axis in config.axis {
            match axis {
                (name, AxisConfig::Dummy(config)) => {
                    let axis = DummyAxis::new(config);
                    device_map.axes.insert(name, axis);
                }
                (name, AxisConfig::A4988(config)) => {
                    let axis = KA4988::new(config, current_oid_count, this.clone())
                        .map_err(|e| MCUError::Axis(e, (&name).into()))?;

                    device_map.axes.insert(name, axis);

                    current_oid_count += 1;
                }
                (name, AxisConfig::Tmc2209(config)) => {
                    let axis = KTMC2209::new(config, current_oid_count, this.clone())
                        .map_err(|e| MCUError::Axis(e, (&name).into()))?;

                    device_map.axes.insert(name, axis);

                    current_oid_count += 2;
                }
            };
        }

        // TODO: actually calculate a good CRC for finalize_config

        this.borrow_mut()
            .send_command_expect_ack(SendCommand::finalize_config { crc: 0 })
            .map_err(|e| MCUError::KlipperInitOIDs(e));

        Ok(this)
    }

    /// Register OID count
    pub fn count_oids(config: &KlipperMCU) -> u8 {
        let mut total_oids = 0;

        for axis in &config.axis {
            total_oids += match axis {
                (_, AxisConfig::Dummy(_)) => 0,
                (_, AxisConfig::A4988(_)) => 1,
                (_, AxisConfig::Tmc2209(_)) => 2,
            };
        }

        total_oids
    }
}
