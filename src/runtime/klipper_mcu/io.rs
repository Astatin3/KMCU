use crate::traits::{Binary, Read, Write};
use crate::{
    runtime::klipper_mcu::{
        KlipperMCURuntime,
        protocol::{Frame, FramePayload, RecvCommand, SendCommand},
    },
    utils::error::{IOError, MCUError},
};

impl KlipperMCURuntime {
    pub fn send_command(&mut self, command: SendCommand) -> Result<(), MCUError> {
        trace!("Sent command '{command:?}'");

        let frame = Frame::send(self.seq, command);
        frame
            .encode(&mut *self.stream, &self.identity.commands)
            .map_err(MCUError::KlipperProtocol)?;

        Ok(())
    }

    /// Receive a frame and decode its payload as a command, or `None` for an ACK/NAK.
    pub fn recv_frame_or_ack(&mut self) -> Result<Option<RecvCommand>, MCUError> {
        let frame = Frame::decode(&mut *self.stream, &self.identity.responses)
            .map_err(MCUError::KlipperConnection)?;

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
    pub fn recv_command(&mut self) -> Result<RecvCommand, MCUError> {
        match self.recv_frame_or_ack() {
            Ok(Some(cmd)) => Ok(cmd),
            Ok(None) => Err(MCUError::KlipperConnection(IOError::UnexpectedNullData)),
            Err(e) => Err(e),
        }
    }
}
