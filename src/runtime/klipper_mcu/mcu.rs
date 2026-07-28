use serde_json::json;

use crate::{
    runtime::klipper_mcu::{KlipperMCURuntime, protocol::SendCommand},
    traits::MCU,
    utils::error::{IOError, MCUError},
};

impl MCU for KlipperMCURuntime {
    fn alive(&mut self) -> Result<(), IOError> {
        // Send a ping to the MCU
        self.send_command_expect_reponse(SendCommand::identify {
            offset: 0,
            count: 0,
        })?;

        Ok(())
    }
}
