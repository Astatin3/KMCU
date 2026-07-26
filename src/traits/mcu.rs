use crate::Res;

pub trait MCU {
    fn alive(&mut self) -> Res<()>;
}
