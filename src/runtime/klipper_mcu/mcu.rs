use serde_json::json;

use crate::{
    runtime::klipper_mcu::{KlipperMCURuntime, protocol::command::CommandFilled},
    traits::mcu::MCU,
};

impl MCU for KlipperMCURuntime {
    fn alive(&mut self) -> anyhow::Result<()> {
        self.send_command(&CommandFilled::new(
            "identify",
            json!({ "offset": 0u32, "count": 0u8 }),
        ))?;

        self.recv_frame()?;

        Ok(())
    }
}
