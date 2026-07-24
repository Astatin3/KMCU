use serde_json::json;

use crate::{
    runtime::klipper_mcu::{KlipperMCURuntime, protocol::command::CommandFilled},
    traits::mcu::MCU,
};

impl MCU for KlipperMCURuntime {
    fn alive(&mut self) -> anyhow::Result<()> {
        self.send_command(&CommandFilled::new("debug_ping", json!({"data": []})))?;

        let _ = self.recv_command()?;

        Ok(())
    }
}
