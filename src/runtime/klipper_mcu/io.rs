use std::io::Cursor;

use crate::error::Res;
use crate::runtime::klipper_mcu::{
    KlipperMCURuntime,
    protocol::{Frame, FramePayload, RecvCommand, SendCommand},
};
use crate::traits::{Binary, Write};

impl KlipperMCURuntime {
    pub fn send_command(&mut self, command: SendCommand) -> Res<()> {
        trace!("Sent command '{command:?}'");

        let frame = Frame::send(self.seq, command);
        frame.encode(&mut *self.stream, &self.identity.commands)?;

        Ok(())
    }

    /// Receive a frame and decode its payload as a command, or `None` for an ACK/NAK.
    pub fn recv_frame_or_ack(&mut self) -> Res<Option<RecvCommand>> {
        let frame = Frame::decode(&mut *self.stream, &self.identity.responses)?;
        self.seq = frame.seq();

        match frame.payload {
            FramePayload::Empty => Ok(None),
            FramePayload::RecvCommand(cmd) => {
                trace!("Received command '{cmd:?}'");
                Ok(Some(cmd))
            }
            FramePayload::SendCommand(_) => unreachable!(),
        }
    }

    /// Receive a command but expect it to not be blank
    pub fn recv_command(&mut self) -> Res<RecvCommand> {
        match self.recv_frame_or_ack() {
            Ok(Some(cmd)) => Ok(cmd),
            Ok(None) => Err(err!("Unexpected blank command")),
            Err(e) => Err(e),
        }
    }
}
