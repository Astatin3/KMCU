use serde_json::json;

use crate::{
    runtime::klipper_mcu::KlipperMCU,
    traits::{connection::Connection, mcu::MCU},
    wire::types::command::CommandFilled,
};

impl<C: Connection> MCU for KlipperMCU<C> {
    fn alive(&mut self) -> anyhow::Result<bool> {
        self.write(CommandFilled::new("debug_ping", json!({"data": []})))?;

        let _ = self.read()?;

        Ok(false)
    }
}
