use crate::error::Res;

pub trait FromConfig {
    type ConfigType;

    fn from_config(config: Self::ConfigType) -> Res<Self>
    where
        Self: Sized;
}
