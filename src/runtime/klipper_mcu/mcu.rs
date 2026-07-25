use serde_json::json;

use crate::{
    error::Res,
    runtime::klipper_mcu::{KlipperMCURuntime, protocol::SendCommand},
    traits::MCU,
};

impl MCU for KlipperMCURuntime {
    fn alive(&mut self) -> Res<()> {
        // Send a ping to the MCU
        self.send_command(SendCommand::identify {
            offset: 0,
            count: 0,
        })?;

        // Try to receive it
        self.recv_frame_or_ack()?;

        Ok(())
    }
}
