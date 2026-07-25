use crate::error::Res;

pub trait MCU {
    fn alive(&mut self) -> Res<()>;
}
