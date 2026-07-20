use crate::wire::types::message::Message;

pub trait Connection {
    fn read(&mut self) -> anyhow::Result<Message>;
    fn write(&mut self, message: &Message) -> anyhow::Result<()>;
    fn alive_check(&mut self) -> anyhow::Result<()>;
}
