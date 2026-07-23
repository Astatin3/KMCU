pub trait MCU {
    fn alive(&mut self) -> anyhow::Result<()>;
}
