use crate::traits::{Binary, Read, Write};
use crate::{
    runtime::klipper_mcu::{
        KlipperMCURuntime,
        protocol::{Frame, FramePayload, RecvCommand, SendCommand},
    },
    utils::error::{IOError, MCUError},
};

impl KlipperMCURuntime {
    fn send_command(&mut self, command: SendCommand) -> Result<(), IOError> {
        trace!("Sent command seq:{} '{command:?}'", self.seq);

        let frame = Frame::send(self.seq, command);
        frame.encode(&mut *self.stream, &self.identity.commands)?;

        Ok(())
    }

    /// Receive a frame and decode its payload as a command, or `None` for an ACK/NAK.
    fn recv_command(&mut self) -> Result<Option<RecvCommand>, IOError> {
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

    pub fn send_command_expect_ack(&mut self, command: SendCommand) -> Result<(), IOError> {
        self.send_command(command)?;

        let frame = self.recv_command()?;

        match frame {
            None => Ok(()),
            Some(_) => Err(IOError::UnexpectedData),
        }
    }

    /// Receive a command but expect it to not be blank
    pub fn send_command_expect_reponse(
        &mut self,
        command: SendCommand,
    ) -> Result<RecvCommand, IOError> {
        self.send_command(command)?;

        let frame = self.recv_command()?;

        match frame {
            Some(cmd) => Ok(cmd),
            None => Err(IOError::UnexpectedNullData),
        }
    }
}
