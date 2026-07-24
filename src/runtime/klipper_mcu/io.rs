use crate::runtime::klipper_mcu::{
    KlipperMCURuntime,
    protocol::{
        command::{RecvCommand, SendCommand},
        message::Frame,
    },
};
use crate::traits::binary::Binary;

impl KlipperMCURuntime {
    pub fn send_command(&mut self, command: &SendCommand) -> anyhow::Result<()> {
        trace!("Sent command '{command:?}'");

        let mut payload = Vec::with_capacity(64);
        command.encode(&mut payload, &self.identity.commands)?;

        let seq = (self.seq % 16) as u8;
        let frame =
            Frame::new(&payload, seq).ok_or_else(|| anyhow::anyhow!("Message too large"))?;

        frame
            .write_to(&mut *self.stream)
            .map_err(|e| anyhow::anyhow!("Failed to send: {e}"))
    }

    /// Receive some frame data
    pub(crate) fn recv_frame(&mut self) -> anyhow::Result<Frame> {
        let frame = Frame::read_from(&mut *self.stream)?;
        self.seq = frame.seq() as usize;
        Ok(frame)
    }

    /// Receive a command that's potentially blank
    pub fn recv_frame_or_ack(&mut self) -> anyhow::Result<Option<RecvCommand>> {
        let frame = self.recv_frame()?;

        if frame.is_empty() {
            return Ok(None);
        }

        let mut cursor = frame.payload();
        let cmd = RecvCommand::decode(&mut cursor, &self.identity.responses)?;

        trace!("Received command '{cmd:?}'");

        Ok(Some(cmd))
    }

    /// Receive a command but expect it to not be blank
    pub fn recv_command(&mut self) -> anyhow::Result<RecvCommand> {
        match self.recv_frame_or_ack() {
            Ok(Some(cmd)) => Ok(cmd),
            Ok(None) => Err(anyhow::anyhow!("Unexpected blank command")),
            Err(e) => Err(e),
        }
    }
}
