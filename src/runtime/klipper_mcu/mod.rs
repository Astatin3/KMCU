use crate::{
    traits::{connection::Connection, mcu::MCU},
    wire::types::{
        command::CommandFilled,
        dictionary::{DEFAULT_DICT, Dictionary},
        message::Message,
    },
};

pub mod identify;
mod mcu;

pub struct KlipperMCU<C: Connection> {
    connection: C,
    seq: usize,

    commands: Dictionary,
    responses: Dictionary,

    #[allow(dead_code)]
    output: Dictionary, // Currently not used
}

impl<C: Connection> KlipperMCU<C> {
    pub fn new(connection: C) -> anyhow::Result<Self> {
        let mut this = Self {
            connection,
            seq: 0,
            commands: DEFAULT_DICT.clone(),
            responses: DEFAULT_DICT.clone(),
            output: DEFAULT_DICT.clone(),
        };

        let results = this.identify()?;

        debug!("Received identify results: {results:?}");

        let (commands, responses, output) = results.build_dictionaries()?;
        this.commands = commands;
        this.responses = responses;
        this.output = output;

        log::debug!("Got identity from MCU app={}", results.app);

        Ok(this)
    }

    fn write(&mut self, command: CommandFilled) -> anyhow::Result<()> {
        let message = Message::from_command(&command, (self.seq % 16) as u8, &self.commands)
            .ok_or(anyhow::anyhow!("Message too large"))?;
        self.connection.write(&message)
    }

    fn read(&mut self) -> anyhow::Result<Message> {
        let message = self.connection.read()?;
        if let Message::Serialized(ref raw) = message {
            self.seq = Message::wire_seq(raw) as usize;
        }
        Ok(message)
    }
}
