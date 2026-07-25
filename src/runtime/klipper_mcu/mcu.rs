use serde_json::json;

use crate::{
    error::Res,
    runtime::klipper_mcu::{KlipperMCURuntime, protocol::command::SendCommand},
    traits::MCU,
};

impl MCU for KlipperMCURuntime {
    fn alive(&mut self) -> Res<()> {
        self.send_command(SendCommand::identify {
            offset: 0,
            count: 0,
        });

        self.recv_frame_or_ack()?;

        Ok(())
    }
}
